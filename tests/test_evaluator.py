import copy
from decimal import Decimal
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scoring.engine import compare, evaluate, normalize_cost, validate_metrics, validate_policy
from analyzers.collector import collect, inventory, run
from scripts.evaluate import catalog, diagnosis, read_json
from schemas.validate import validate


def policy():
    return read_json(ROOT / "scoring/policy.json")


def metrics():
    return {"schema_version": "1.0", "build_success": True, "test_command_success": True, "source_unchanged": True,
            "analysis_success": True, "tests": {"total": 2, "passed": 2, "failed": 0, "skipped": 0, "excluded": 0},
            "function_count": 4, "decision_max": 1, "parameters_max": 1, "function_lines_p95": 1}


class ScoringTests(unittest.TestCase):
    def test_normalization_boundaries_monotonicity(self):
        self.assertEqual([normalize_cost(n, 3, 15) for n in (0, 3, 9, 15, 99)],
                         [Decimal(10), Decimal(10), Decimal(5), Decimal(0), Decimal(0)])
        values = [normalize_cost(n, 3, 15) for n in range(30)]
        self.assertEqual(values, sorted(values, reverse=True))

    def test_no_mutation_repeatability_and_unknown_dimensions(self):
        raw, config = metrics(), policy()
        saved = copy.deepcopy((raw, config))
        first = evaluate(raw, config)
        self.assertEqual(first, evaluate(raw, config))
        self.assertEqual((raw, config), saved)
        self.assertEqual(first["score"], 10)
        self.assertTrue(first["accepted"])
        self.assertIsNone(first["quality_vector"]["algorithmic_efficiency"])

    def test_missing_required_metric_blocks_and_does_not_reweight(self):
        raw = metrics()
        raw["decision_max"] = None
        result = evaluate(raw, policy())
        self.assertFalse(result["accepted"])
        self.assertIsNone(result["score"])
        self.assertIn("decision_max", result["gates"]["unknown"])

    def test_weighting_unsupported_metric_blocks(self):
        config = policy()
        config["weights"]["correctness"] = 0.3
        config["weights"]["test_strength"] = 0.1
        self.assertIsNone(evaluate(metrics(), config)["score"])

    def test_high_score_cannot_override_failed_build_or_command(self):
        for gate in ("build_success", "test_command_success", "analysis_success", "source_unchanged"):
            with self.subTest(gate=gate):
                raw = metrics()
                raw[gate] = False
                result = evaluate(raw, policy())
                self.assertFalse(result["accepted"])
                self.assertIn(gate, result["gates"]["failures"])

    def test_zero_skipped_excluded_failed_and_inconsistent_tests(self):
        for state in ("failed", "skipped", "excluded"):
            with self.subTest(state=state):
                raw = metrics()
                raw["tests"]["passed"] = 1
                raw["tests"][state] = 1
                self.assertFalse(evaluate(raw, policy())["accepted"])
        raw = metrics()
        raw["tests"] = dict.fromkeys(raw["tests"], 0)
        self.assertIsNone(evaluate(raw, policy())["score"])
        raw["tests"]["passed"] = 1
        with self.assertRaises(ValueError):
            evaluate(raw, policy())

    def test_invalid_numbers_weights_versions_and_config(self):
        for value in (True, float("nan"), float("inf"), -1, "10"):
            with self.subTest(value=value):
                raw = metrics()
                raw["decision_max"] = value
                with self.assertRaises(ValueError):
                    validate_metrics(raw)
        for value in (True, float("nan"), 0.3, -0.1, 1.1):
            config = policy()
            config["weights"]["correctness"] = value
            with self.assertRaises(ValueError):
                validate_policy(config)
        config = policy()
        config["normalization"]["readability"]["bad"] = 15
        with self.assertRaises(ValueError):
            validate_policy(config)
        raw = metrics()
        raw["schema_version"] = "2.0"
        with self.assertRaises(ValueError):
            evaluate(raw, policy())

    def test_gate_threshold_is_inclusive(self):
        config = policy()
        config["target_score"] = 0
        raw = metrics()
        raw["decision_max"] = 10
        self.assertTrue(evaluate(raw, config)["accepted"])
        raw["decision_max"] = 11
        self.assertFalse(evaluate(raw, config)["accepted"])

    def test_decimal_weighting_and_score_floor(self):
        raw = metrics()
        raw.update(decision_max=12, function_lines_p95=47)
        self.assertEqual(evaluate(raw, policy())["score"], 6.683333)
        raw.update(decision_max=999, parameters_max=999, function_lines_p95=999)
        raw["tests"].update(passed=0, failed=2)
        self.assertEqual(evaluate(raw, policy())["score"], 0)

    def test_comparison_requires_improvement_and_preserves_correctness(self):
        before, after = metrics(), metrics()
        self.assertFalse(compare(before, after, policy())["improved"])
        before["decision_max"] = 12
        self.assertTrue(compare(before, after, policy())["improved"])
        after["tests"].update(passed=1, failed=1)
        self.assertIn("correctness_regression", compare(before, after, policy())["reasons"])

    def test_comparison_checks_unweighted_dimensions(self):
        before, after = metrics(), metrics()
        before["decision_max"] = 12
        after["function_lines_p95"] = 30
        self.assertIn("dimension_regression:readability", compare(before, after, policy())["reasons"])
        after["parameters_max"] = None
        self.assertIn("lost_measurement:maintainability", compare(before, after, policy())["reasons"])


@unittest.skipUnless(shutil.which("elixir"), "Elixir is required for parser tests")
class ParserTests(unittest.TestCase):
    def parse(self, text):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.ex"
            path.write_text(text)
            command = subprocess.run(["elixir", str(ROOT / "analyzers/elixir_ast.exs"), str(path)], capture_output=True, text=True)
            self.assertEqual(command.returncode, 0, command.stderr)
            return json.loads(command.stdout)[0]

    def test_real_ast_ignores_comments_strings_and_quotes(self):
        row = self.parse('''defmodule Sample do
          # if x, do: a ++ b
          def x, do: "if x, do: a ++ b"
          def q, do: quote(do: a ++ b)
        end''')
        self.assertEqual(row["advisory"], [])
        self.assertEqual([f["decision_indicator"] for f in row["functions"]], [1, 1])

    def test_append_syntax_advisory_and_not_temporal_complexity(self):
        row = self.parse("defmodule Sample do\n def join(a, b), do: a ++ b\nend")
        self.assertEqual(len(row["advisory"]), 1)
        self.assertEqual(row["advisory"][0]["line"], 2)
        self.assertEqual(row["functions"][0]["decision_indicator"], 1)

    def test_guard_default_arguments_and_pattern_matching(self):
        row = self.parse('''defmodule Sample do
          def x(a \\\\ 1)
          def x(a) when is_integer(a), do: a
          def x(_), do: 0
        end''')
        self.assertTrue(row["success"])
        self.assertEqual(len(row["functions"]), 2)
        self.assertEqual([f["arity"] for f in row["functions"]], [1, 1])

    def test_invalid_syntax_is_not_an_empty_success(self):
        self.assertFalse(self.parse("defmodule Broken do def")['success'])

    def test_case_decisions(self):
        row = self.parse("defmodule X do\n def f(x) do\n case x do\n  0 -> :a\n  1 -> :b\n  _ -> :c\n end\n end\nend")
        self.assertEqual(row["functions"][0]["decision_indicator"], 3)


class ProvenanceAndAdapterTests(unittest.TestCase):
    def test_contract_validation_rejects_unknown_fields_and_forged_types(self):
        raw = metrics()
        raw["score"] = 10
        with self.assertRaises(ValueError):
            validate("metrics", raw)
        raw = metrics()
        raw["tests"]["passed"] = True
        with self.assertRaises(ValueError):
            validate("metrics", raw)
        with self.assertRaises(ValueError):
            validate("report", {"schema_version": "1.0", "score": 10})

    def test_every_engine_gate_has_a_deterministic_rule(self):
        _, rules = catalog()
        engine_gates = {g["id"] for g in evaluate(metrics(), policy())["gates"]["results"]}
        rule_gates = {r["gate"] for r in rules if "gate" in r and r["mode"] == "deterministic"}
        self.assertEqual(engine_gates, rule_gates)

    def test_every_rule_has_provenance_and_advisory_cannot_score(self):
        concepts, rules = catalog()
        self.assertTrue(all(r["knowledge"] for r in rules))
        result = evaluate(metrics(), policy())
        measured = {"functions": [], "advisory": [{"rule": "LIST_APPEND_REVIEW", "file": "lib/a.ex", "line": 1}]}
        diagnosed = diagnosis(measured, result, policy(), concepts, rules)
        self.assertEqual(result["score"], 10)
        self.assertEqual(diagnosed["findings"][0]["mode"], "advisory")
        self.assertEqual(diagnosed["retrieved_knowledge"][0]["id"], "PERSISTENT_APPEND_COST")
        self.assertTrue(diagnosed["findings"][0]["sources"][0]["source_sha256"])

    def test_unavailable_runtime_cannot_approve(self):
        unavailable = {"status": "unavailable", "command": [], "exit_code": None, "stdout": "", "stderr": "missing"}
        with patch("analyzers.collector.run", return_value=unavailable):
            measured = collect(ROOT / "fixtures/good", policy())
        result = evaluate(measured["raw_metrics"], policy())
        self.assertFalse(result["accepted"])
        self.assertIsNone(result["score"])

    def test_timeout_is_evidence_not_success(self):
        result = run([sys.executable, "-c", "import time; time.sleep(10)"], ROOT, {}, 0.05)
        self.assertEqual(result["status"], "timeout")
        self.assertNotEqual(result["exit_code"], 0)

    def test_reject_symlink_before_executing_target(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            (path / "outside").symlink_to(ROOT)
            with self.assertRaises(ValueError):
                inventory(path)

    def test_duplicate_policy_keys_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "duplicate.json"
            path.write_text('{"score": 10, "score": 0}')
            with self.assertRaises(ValueError):
                read_json(path)


@unittest.skipUnless(shutil.which("mix"), "Mix is required for integration tests")
class EndToEndTests(unittest.TestCase):
    def cli(self, *args):
        return subprocess.run([sys.executable, str(ROOT / "scripts/evaluate.py"), *map(str, args)],
                              capture_output=True, text=True, timeout=120)

    def test_bad_good_comparison_matches_independent_expected_values(self):
        expected = read_json(ROOT / "fixtures/expected.json")
        before_good = inventory(ROOT / "fixtures/good")
        result = self.cli(ROOT / "fixtures/good", "--baseline", ROOT / "fixtures/bad")
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        for kind, value in (("good", report), ("bad", report["baseline_report"])):
            self.assertEqual(value["status"], expected[kind]["status"])
            self.assertEqual(value["score"], expected[kind]["score"])
            self.assertEqual(value["raw_metrics"]["decision_max"], expected[kind]["decision_max"])
            found = {f["rule"] for f in value["diagnosis"]["findings"]}
            self.assertTrue(set(expected[kind]["rules"]) <= found)
            self.assertEqual(value["raw_metrics"]["tests"]["passed"], 2)
        self.assertTrue(report["baseline_comparison"]["improved"])
        self.assertEqual(inventory(ROOT / "fixtures/good"), before_good)
        repeated = json.loads(self.cli(ROOT / "fixtures/good").stdout)
        for key in ("raw_metrics", "quality_vector", "gates", "score", "source", "diagnosis"):
            self.assertEqual(report[key], repeated[key])

    def test_real_failed_test_and_real_failed_build_block_acceptance(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample"
            shutil.copytree(ROOT / "fixtures/good", project)
            (project / "test/classifier_test.exs").write_text('defmodule FailingTest do\n use ExUnit.Case\n test "bad" do\n assert 1 == 2\n end\nend')
            first = self.cli(project)
            self.assertEqual(first.returncode, 1, first.stderr)
            report = json.loads(first.stdout)
            self.assertEqual(report["raw_metrics"]["tests"]["failed"], 1)
            self.assertFalse(report["accepted"])
            (project / "lib/classifier.ex").write_text("defmodule Broken do def")
            second = self.cli(project)
            self.assertEqual(second.returncode, 1)
            report = json.loads(second.stdout)
            self.assertFalse(report["raw_metrics"]["build_success"])
            self.assertIsNone(report["raw_metrics"]["tests"])

    def test_empty_and_skipped_suites_cannot_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample"
            shutil.copytree(ROOT / "fixtures/good", project)
            for body in ('', '@tag :skip\n test "skip", do: assert(true)'):
                (project / "test/classifier_test.exs").write_text(f'defmodule EmptyTest do\n use ExUnit.Case\n {body}\nend')
                result = self.cli(project)
                self.assertEqual(result.returncode, 1, result.stderr)
                report = json.loads(result.stdout)
                self.assertFalse(report["accepted"])
                self.assertEqual(report["raw_metrics"]["tests"]["passed"], 0)

    def test_source_mutation_has_its_own_gate_and_output_files_are_allowed(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample"
            shutil.copytree(ROOT / "fixtures/good", project)
            test = project / "test/classifier_test.exs"
            for target, expected in (("tmp/generated", True), ("lib/generated.ex", False)):
                test.write_text(f'''defmodule OutputTest do
  use ExUnit.Case
  test "writes output" do
    File.mkdir_p!("tmp")
    File.write!("{target}", "# generated")
    assert true
  end
end''')
                result = self.cli(project)
                report = json.loads(result.stdout)
                self.assertEqual(report["accepted"], expected)
                self.assertTrue(report["raw_metrics"]["build_success"])
                self.assertEqual(report["raw_metrics"]["source_unchanged"], expected)
                self.assertFalse((project / target).exists())

    def test_changed_test_contract_blocks_comparison(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "candidate"
            shutil.copytree(ROOT / "fixtures/good", project)
            helper = project / "test/test_helper.exs"
            helper.write_text("# Changed contract file\nExUnit.start()\n")
            result = self.cli(project, "--baseline", ROOT / "fixtures/bad")
            self.assertEqual(result.returncode, 1, result.stderr)
            report = json.loads(result.stdout)
            self.assertTrue(report["accepted"])
            self.assertFalse(report["baseline_comparison"]["improved"])
            self.assertIn("test_contract_changed", report["baseline_comparison"]["reasons"])


if __name__ == "__main__":
    unittest.main()
