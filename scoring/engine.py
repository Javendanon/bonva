"""Pure, versioned policy evaluation. No model, IO, inference or clock."""
from decimal import Decimal, ROUND_HALF_UP
import math
from schemas.validate import validate

VERSION = "1.0"
DIMENSIONS = (
    "correctness", "test_strength", "algorithmic_efficiency", "runtime_efficiency",
    "memory_efficiency", "structural_complexity", "maintainability", "readability",
)
PROXIES = {
    "structural_complexity": "decision_max",
    "maintainability": "parameters_max",
    "readability": "function_lines_p95",
}


def number(value, minimum=0, maximum=None):
    if type(value) not in (int, float) or not math.isfinite(value):
        raise ValueError("Expected a finite number (not a boolean)")
    if value < minimum or (maximum is not None and value > maximum):
        raise ValueError("Number outside allowed range")
    return Decimal(str(value))


def rounded(value):
    return float(value.quantize(Decimal("0.000001"), rounding=ROUND_HALF_UP))


def validate_policy(policy):
    if policy["schema_version"] != VERSION or not isinstance(policy["profile"], str):
        raise ValueError("Unsupported policy version/profile")
    weights = policy["weights"]
    if set(weights) != set(DIMENSIONS):
        raise ValueError("Weights must name all eight dimensions")
    if sum(number(v, 0, 1) for v in weights.values()) != Decimal(1):
        raise ValueError("Weights must sum exactly to 1")
    number(policy["target_score"], 0, 10)
    if set(policy["normalization"]) != set(PROXIES):
        raise ValueError("Unsupported normalization dimensions")
    for dimension, metric in PROXIES.items():
        spec = policy["normalization"][dimension]
        if spec["metric"] != metric or number(spec["bad"]) <= number(spec["good"]):
            raise ValueError("Invalid normalization interval or metric")
    number(policy["gates"]["decision_max"], 1)
    if type(policy["gates"]["minimum_tests"]) is not int or policy["gates"]["minimum_tests"] < 1:
        raise ValueError("minimum_tests must be a positive integer")
    if type(policy["gates"]["allow_skipped_tests"]) is not bool:
        raise ValueError("allow_skipped_tests must be boolean")
    if type(policy["execution"]["seed"]) is not int or policy["execution"]["seed"] < 0:
        raise ValueError("seed must be a nonnegative integer")
    number(policy["execution"]["timeout_seconds"], 1)
    generated = policy["execution"]["generated_directories"]
    if not isinstance(generated, list) or any(not isinstance(p, str) or not p or "/" in p or p in (".", "..", "lib", "test", "config", "deps", "priv") for p in generated):
        raise ValueError("Generated directories must be non-source directory names")
    number(policy["comparison"]["minimum_delta"], 0, 10)
    number(policy["comparison"]["maximum_dimension_drop"], 0, 10)


def normalize_cost(value, good, bad):
    value, good, bad = number(value), number(good), number(bad)
    if bad <= good:
        raise ValueError("bad must exceed good")
    return max(Decimal(0), min(Decimal(10), 10 * (bad - value) / (bad - good)))


def validate_metrics(raw):
    validate("metrics", raw)
    if raw["schema_version"] != VERSION:
        raise ValueError("Unsupported metrics version")
    for key in ("build_success", "test_command_success", "analysis_success", "source_unchanged"):
        if raw[key] is not None and type(raw[key]) is not bool:
            raise ValueError(f"Invalid {key}")
    tests = raw["tests"]
    if tests is not None:
        if set(tests) != {"total", "passed", "failed", "skipped", "excluded"}:
            raise ValueError("Invalid test counters")
        for value in tests.values():
            if type(value) is not int or value < 0:
                raise ValueError("Invalid test count")
        if tests["total"] != sum(tests[k] for k in ("passed", "failed", "skipped", "excluded")):
            raise ValueError("Inconsistent test totals")
    for metric in (*PROXIES.values(), "function_count"):
        if raw[metric] is not None:
            number(raw[metric])


def evaluate(raw, policy):
    validate_policy(policy)
    validate_metrics(raw)
    vector = dict.fromkeys(DIMENSIONS)
    tests = raw["tests"]
    if tests and tests["total"] > 0 and raw["test_command_success"] is not None:
        vector["correctness"] = Decimal(10) * tests["passed"] / tests["total"]
    for dimension, metric in PROXIES.items():
        if raw["analysis_success"] is True and raw[metric] is not None:
            spec = policy["normalization"][dimension]
            vector[dimension] = normalize_cost(raw[metric], spec["good"], spec["bad"])

    gates = []

    def gate(name, value):
        gates.append({"id": name, "status": "unknown" if value is None else "passed" if value else "failed"})

    gate("build_success", raw["build_success"])
    gate("source_unchanged", raw["source_unchanged"])
    gate("test_command_success", raw["test_command_success"])
    gate("analysis_success", raw["analysis_success"])
    gate("source_functions", None if raw["function_count"] is None else raw["function_count"] > 0)
    gate("tests.minimum", None if tests is None else tests["passed"] + tests["failed"] >= policy["gates"]["minimum_tests"])
    gate("tests.failures", None if tests is None else tests["failed"] == 0)
    gate("tests.skipped", None if tests is None else policy["gates"]["allow_skipped_tests"] or tests["skipped"] + tests["excluded"] == 0)
    gate("decision_max", None if raw["decision_max"] is None else raw["decision_max"] <= policy["gates"]["decision_max"])
    missing = [d for d in DIMENSIONS if policy["weights"][d] > 0 and vector[d] is None]
    score = None if missing else sum(vector[d] * number(policy["weights"][d]) for d in DIMENSIONS if policy["weights"][d] > 0)
    gate("required_metrics", not missing)
    gate("target_score", None if score is None else score >= number(policy["target_score"]))
    failed = [g["id"] for g in gates if g["status"] == "failed"]
    unknown = [g["id"] for g in gates if g["status"] == "unknown"]
    status = "rejected" if failed else "incomplete" if unknown else "accepted"
    return {
        "status": status, "accepted": status == "accepted",
        "score": None if score is None else rounded(score),
        "target_score": policy["target_score"],
        "quality_vector": {d: None if v is None else rounded(v) for d, v in vector.items()},
        "missing_dimensions": [d for d in DIMENSIONS if vector[d] is None],
        "gates": {"passed": not failed and not unknown, "failures": failed, "unknown": unknown, "results": gates},
    }


def compare(baseline_raw, candidate_raw, policy):
    before, after = evaluate(baseline_raw, policy), evaluate(candidate_raw, policy)
    dimensions = {}
    reasons = []
    for dimension in DIMENSIONS:
        a, b = before["quality_vector"][dimension], after["quality_vector"][dimension]
        delta = None if a is None or b is None else rounded(Decimal(str(b)) - Decimal(str(a)))
        dimensions[dimension] = {"before": a, "after": b, "delta": delta}
        if a is not None and b is None:
            reasons.append(f"lost_measurement:{dimension}")
        if delta is not None and delta < -policy["comparison"]["maximum_dimension_drop"]:
            reasons.append(f"dimension_regression:{dimension}")
    correctness_delta = dimensions["correctness"]["delta"]
    if correctness_delta is not None and correctness_delta < 0:
        reasons.append("correctness_regression")
    delta = None if before["score"] is None or after["score"] is None else rounded(Decimal(str(after["score"])) - Decimal(str(before["score"])))
    if delta is None or delta <= 0 or delta < policy["comparison"]["minimum_delta"]:
        reasons.append("insufficient_improvement")
    if not after["accepted"]:
        reasons.append("candidate_not_accepted")
    return {"baseline_score": before["score"], "candidate_score": after["score"], "delta": delta,
            "dimensions": dimensions, "improved": not reasons, "reasons": reasons}
