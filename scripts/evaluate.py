#!/usr/bin/env python3
"""Evaluate an Elixir Mix project; scores and diagnosis come only from scripts."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from analyzers.collector import collect
from scoring.engine import compare, evaluate, validate_policy
from schemas.validate import validate


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"Duplicate JSON key: {key}")
            result[key] = value
        return result
    return json.loads(Path(path).read_text(encoding="utf-8"), object_pairs_hook=unique)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def catalog():
    concepts_doc = read_json(ROOT / "knowledge/evaluator_concepts.json")
    rules_doc = read_json(ROOT / "rules/evaluator.json")
    validate("knowledge", concepts_doc)
    validate("rules", rules_doc)
    if concepts_doc["schema_version"] != "1.0" or rules_doc["schema_version"] != "1.0":
        raise ValueError("Unsupported knowledge/rule schema")
    concepts = {c["id"]: c for c in concepts_doc["concepts"]}
    rules = rules_doc["rules"]
    if len(concepts) != len(concepts_doc["concepts"]) or len({r["id"] for r in rules}) != len(rules):
        raise ValueError("Duplicate concept/rule ID")
    manifest = {r["id"]: r for r in read_json(ROOT / "knowledge/sources/manifest.json")}
    for concept in concepts.values():
        for source in concept["sources"]:
            if "book" in source:
                record = manifest[source["book"]]
                for path, key in ((record["file"], "sha256"), (record["extracted"], "extracted_sha256")):
                    if digest(ROOT / path) != record[key]:
                        raise ValueError(f"Knowledge integrity failure: {path}")
                units = read_json(ROOT / record["extracted"])
                if any(type(u) is not int or not 1 <= u <= len(units) for u in source["units"]):
                    raise ValueError("Invalid source locator")
                source["source_sha256"] = record["sha256"]
                source["unit_kind"] = record["unit_kind"]
            else:
                source["sha256"] = digest(ROOT / source["file"])
    for rule in rules:
        if rule["mode"] not in ("deterministic", "advisory", "unsupported"):
            raise ValueError("Invalid rule mode")
        if not rule["knowledge"] or any(c not in concepts for c in rule["knowledge"]):
            raise ValueError("Rule without known provenance")
        if rule["mode"] != "deterministic" and ("gate" in rule or "dimension" in rule):
            raise ValueError("Non-deterministic rules cannot score")
    return concepts, rules


def diagnosis(measured, result, policy, concepts, rules):
    findings = []
    unresolved = set(result["gates"]["failures"] + result["gates"]["unknown"])
    for rule in rules:
        locations = []
        if rule.get("gate") in unresolved:
            locations = [{"evidence": "raw_metrics", "gate": rule["gate"]}]
            if rule["id"] == "DECISION_LIMIT":
                locations = [f for f in measured["functions"] if f["decision_indicator"] > policy["gates"]["decision_max"]]
        if rule.get("dimension"):
            spec = policy["normalization"][rule["dimension"]]
            locations = [f for f in measured["functions"] if f[rule["metric"]] > spec["good"]]
        if rule["mode"] == "advisory":
            locations = [f for f in measured["advisory"] if f["rule"] == rule["id"]]
        for location in locations:
            findings.append({"rule": rule["id"], "mode": rule["mode"], "location": location,
                             "instruction": rule["instruction"], "knowledge": rule["knowledge"],
                             "sources": [s for c in rule["knowledge"] for s in concepts[c]["sources"]]})
    ids = sorted({c for f in findings for c in f["knowledge"]})
    weakest = sorted(
        ({"dimension": d, "score": v} for d, v in result["quality_vector"].items() if v is not None),
        key=lambda row: (row["score"], row["dimension"]),
    )
    return {"findings": findings, "retrieved_knowledge": [concepts[c] for c in ids],
            "weakest_measured_dimensions": weakest,
            "instructions": [f["instruction"] for i, f in enumerate(findings)
                             if f["instruction"] not in [p["instruction"] for p in findings[:i]]]}


def report(project, policy, concepts, rules):
    measured = collect(project, policy)
    result = evaluate(measured["raw_metrics"], policy)
    implementation = ["scoring/engine.py", "analyzers/collector.py", "analyzers/elixir_ast.exs",
                      "analyzers/exunit_formatter.exs", "scripts/evaluate.py", "rules/evaluator.json",
                      "knowledge/evaluator_concepts.json", "protocols/evaluation.json",
                      "schemas/evaluator.json", "schemas/validate.py"]
    return {"schema_version": "1.0", "profile": policy["profile"], "policy": policy,
            "implementation": {p: digest(ROOT / p) for p in implementation},
            **result, **measured, "diagnosis": diagnosis(measured, result, policy, concepts, rules),
            "limitations": [
                "Acceptance applies only to the configured indicator profile and executed tests.",
                "Coverage, mutation, property-test classification, algorithmic/runtime/memory efficiency and concurrency scalability are not measured.",
                "AST indicators describe source clauses, not expanded macros, control-flow proofs or human readability.",
                "Fixed seed and serial test execution do not make arbitrary tests or BEAM scheduling deterministic.",
            ]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project", type=Path)
    parser.add_argument("--policy", type=Path, default=ROOT / "scoring/policy.json")
    parser.add_argument("--baseline", type=Path, help="Mix project to independently re-evaluate")
    parser.add_argument("--output", type=Path, help="New JSON report file; refuses overwrite")
    args = parser.parse_args()
    policy = read_json(args.policy)
    validate_policy(policy)
    concepts, rules = catalog()
    if args.output:
        destination = args.output.resolve()
        for project in (args.project, args.baseline):
            if project and destination.is_relative_to(project.resolve()):
                raise ValueError("Report output must be outside evaluated projects")
        if destination.exists():
            raise ValueError("Report already exists; choose a new output file")
    result = report(args.project, policy, concepts, rules)
    if args.baseline:
        baseline = report(args.baseline, policy, concepts, rules)
        comparison = compare(baseline["raw_metrics"], result["raw_metrics"], policy)
        # Test removal or changing expectations cannot silently count as optimization.
        tests_before = {p: h for p, h in baseline["source"]["files"].items() if p.startswith("test/")}
        tests_after = {p: h for p, h in result["source"]["files"].items() if p.startswith("test/")}
        if tests_before != tests_after:
            comparison["improved"] = False
            comparison["reasons"].append("test_contract_changed")
        if baseline["evidence"]["runtime"]["stdout"] != result["evidence"]["runtime"]["stdout"]:
            comparison["improved"] = False
            comparison["reasons"].append("runtime_changed")
        result["baseline_comparison"] = comparison
        result["baseline_report"] = baseline
    validate("report", result)
    serialized = json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("x", encoding="utf-8") as stream:
            stream.write(serialized)
    else:
        print(serialized, end="")
    print(f"{result['status']}: score={result['score']}, target={result['target_score']}, "
          f"failed_gates={','.join(result['gates']['failures']) or 'none'}", file=sys.stderr)
    if args.baseline and not result["baseline_comparison"]["improved"]:
        return 1
    return 0 if result["accepted"] else 2 if result["status"] == "incomplete" else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, TypeError) as exc:
        print(json.dumps({"status": "error", "error": str(exc)}, ensure_ascii=False), file=sys.stderr)
        sys.exit(2)
