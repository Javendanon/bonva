"""Elixir/Mix adapters. Execute a temporary snapshot; preserve tool evidence."""
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
IGNORED = {"_build", ".elixir_ls", ".DS_Store"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def inventory(project):
    files = {}
    for directory, dirs, names in os.walk(project, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in IGNORED and not (d == ".git" and Path(directory) == project))
        for name in dirs + sorted(names):
            path = Path(directory) / name
            if path.is_symlink():
                raise ValueError(f"Unsupported symlink in project: {path.relative_to(project)}")
        for name in sorted(names):
            if name in IGNORED or (name == ".git" and Path(directory) == project):
                continue
            path = Path(directory) / name
            if not path.is_file():
                raise ValueError(f"Unsupported special file: {path}")
            files[path.relative_to(project).as_posix()] = sha(path.read_bytes())
    return dict(sorted(files.items()))


def run(command, cwd, env, timeout):
    try:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=True)
    except OSError as exc:
        return {"command": command, "exit_code": None, "status": "unavailable", "stdout": "", "stderr": str(exc)}
    status = "finished"
    try:
        out, err = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        out, err = process.communicate()
        status = "timeout"
    return {"command": command, "exit_code": process.returncode, "status": status,
            "stdout": out.decode("utf-8", errors="replace"), "stderr": err.decode("utf-8", errors="replace")}


def success(result):
    if result["status"] == "unavailable":
        return None
    return result["status"] == "finished" and result["exit_code"] == 0


def collect(project, policy):
    project = Path(project).resolve()
    if not (project / "mix.exs").is_file():
        raise ValueError("Expected a Mix project with mix.exs")
    before = inventory(project)
    timeout = policy["execution"]["timeout_seconds"]
    evidence = {}
    raw = {"schema_version": "1.0", "build_success": None, "test_command_success": None, "source_unchanged": True,
           "analysis_success": None, "tests": None, "function_count": None,
           "decision_max": None, "parameters_max": None, "function_lines_p95": None}
    functions, advisory = [], []
    with tempfile.TemporaryDirectory(prefix="quality-evaluator-") as tmp:
        sandbox = Path(tmp) / "project"
        def ignore(directory, names):
            return [n for n in names if n in IGNORED or (n == ".git" and Path(directory) == project)]
        shutil.copytree(project, sandbox, ignore=ignore)
        if inventory(sandbox) != before:
            raise ValueError("Project changed while taking snapshot")
        env = dict(os.environ)
        env.update({"MIX_ENV": "test", "NO_COLOR": "1", "TERM": "dumb",
                    "QUALITY_TEST_RESULT": str(Path(tmp) / "exunit.json")})
        for name in ("MIX_BUILD_PATH", "MIX_DEPS_PATH", "MIX_EXS"):
            env.pop(name, None)
        evidence["runtime"] = run(["elixir", "--version"], sandbox, env, timeout)
        # Snapshot bytes are analyzed before build scripts can modify them.
        source_paths = [p for p in before if p.startswith("lib/") and p.endswith((".ex", ".exs"))]
        ast = run(["elixir", str(ROOT / "analyzers/elixir_ast.exs"), *source_paths], sandbox, env, timeout)
        evidence["ast"] = ast
        raw["analysis_success"] = success(ast)
        if raw["analysis_success"]:
            try:
                rows = json.loads(ast["stdout"])
                if [r["file"] for r in rows] != source_paths:
                    raise ValueError("AST file inventory mismatch")
                evidence["ast_results"] = rows
                raw["analysis_success"] = all(r["success"] for r in rows)
                if raw["analysis_success"]:
                    functions = [f for row in rows for f in row["functions"]]
                    advisory = [f for row in rows for f in row["advisory"]]
                    raw["function_count"] = len(functions)
                    if functions:
                        raw["decision_max"] = max(f["decision_indicator"] for f in functions)
                        raw["parameters_max"] = max(f["arity"] for f in functions)
                        lengths = sorted(f["lines"] for f in functions)
                        raw["function_lines_p95"] = lengths[math.ceil(0.95 * len(lengths)) - 1]
            except (ValueError, KeyError, TypeError) as exc:
                raw["analysis_success"] = False
                evidence["ast_adapter_error"] = str(exc)
        evidence["build"] = run(["mix", "compile", "--force", "--warnings-as-errors"], sandbox, env, timeout)
        raw["build_success"] = success(evidence["build"])
        if raw["build_success"]:
            evidence["tests"] = run([
                "elixir", "-r", str(ROOT / "analyzers/exunit_formatter.exs"), "-S", "mix", "test",
                "--seed", str(policy["execution"]["seed"]), "--max-cases", "1",
                "--formatter", "ExUnit.CLIFormatter", "--formatter", "QualityExUnitFormatter",
            ], sandbox, env, timeout)
            raw["test_command_success"] = success(evidence["tests"])
            result_path = Path(env["QUALITY_TEST_RESULT"])
            if result_path.is_file():
                try:
                    raw["tests"] = json.loads(result_path.read_text())
                except ValueError as exc:
                    evidence["test_adapter_error"] = str(exc)
                    raw["test_command_success"] = False
        # Build/test hooks that rewrite input invalidate all acceptance.
        after = inventory(sandbox)
        changes = [p for p in sorted(before.keys() | after.keys()) if before.get(p) != after.get(p)]
        if changes:
            evidence["snapshot_changes"] = changes
            generated = set(policy["execution"]["generated_directories"])
            source_changes = [p for p in changes if p.split("/", 1)[0] not in generated]
            evidence["source_changes"] = source_changes
            raw["source_unchanged"] = not source_changes
    if inventory(project) != before:
        raise ValueError("Original project changed during evaluation")
    identity = sha(json.dumps(before, sort_keys=True, separators=(",", ":")).encode())
    return {"raw_metrics": raw, "evidence": evidence, "functions": functions, "advisory": advisory,
            "source": {"sha256": identity, "files": before}}
