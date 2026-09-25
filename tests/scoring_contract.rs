//! Contracts retained when retiring the duplicate Python evaluator.
use agent_quality::{data, engine, report};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn policy() -> Value {
    data::read(&data::root().join("scoring/policy.json")).unwrap()
}
fn profile() -> Value {
    data::read(&data::root().join("profiles/elixir-mvp.json")).unwrap()
}
fn raw() -> Value {
    json!({
        "schema_version": "1.0", "build_success": true, "source_unchanged": true,
        "test_command_success": true, "analysis_success": true,
        "tests": {"total": 2, "passed": 2, "failed": 0, "skipped": 0, "excluded": 0},
        "function_count": 4, "decision_max": 1, "parameters_max": 1,
        "function_lines_p95": 1
    })
}
fn evaluate(raw: &Value, policy: &Value) -> Value {
    engine::evaluate(
        raw,
        policy,
        &profile(),
        &["build".into(), "tests".into(), "source_ast".into()],
    )
    .unwrap()
}

#[test]
fn unknown_evidence_and_failure_have_distinct_statuses() {
    let mut metrics = raw();
    metrics["build_success"] = Value::Null;
    let result = evaluate(&metrics, &policy());
    assert_eq!(result["status"], "incomplete");
    assert_eq!(result["score"], 10.0);
    assert_eq!(result["gates"]["unknown"], json!(["build_success"]));
    metrics["source_unchanged"] = json!(false);
    let result = evaluate(&metrics, &policy());
    assert_eq!(result["status"], "rejected");
    assert_eq!(result["gates"]["failures"], json!(["source_unchanged"]));
}

#[test]
fn absent_tests_keep_all_test_gates_unknown() {
    let mut metrics = raw();
    metrics["tests"] = Value::Null;
    let result = evaluate(&metrics, &policy());
    assert!(result["score"].is_null());
    assert_eq!(result["gates"]["failures"], json!(["required_metrics"]));
    assert_eq!(
        result["gates"]["unknown"],
        json!([
            "tests.minimum",
            "tests.failures",
            "tests.skipped",
            "target_score"
        ])
    );
}

#[test]
fn failed_command_keeps_counters_but_cannot_approve() {
    let mut metrics = raw();
    metrics["test_command_success"] = json!(false);
    let result = evaluate(&metrics, &policy());
    assert_eq!(result["quality_vector"]["correctness"], 10.0);
    assert_eq!(result["gates"]["failures"], json!(["test_command_success"]));
    metrics["test_command_success"] = Value::Null;
    assert!(evaluate(&metrics, &policy())["quality_vector"]["correctness"].is_null());
}

#[test]
fn target_gate_uses_unrounded_score() {
    let mut metrics = raw();
    metrics["decision_max"] = json!(4);
    let mut config = policy();
    config["target_score"] = json!(10);
    config["normalization"]["structural_complexity"]["good"] = json!(3.999999);
    let result = evaluate(&metrics, &config);
    assert_eq!(result["score"], 10.0);
    assert_eq!(result["gates"]["failures"], json!(["target_score"]));
    assert_eq!(result["accepted"], false);
}

#[test]
fn allowed_skips_still_require_executed_tests() {
    let mut metrics = raw();
    metrics["tests"]["passed"] = json!(0);
    metrics["tests"]["skipped"] = json!(2);
    let mut config = policy();
    config["gates"]["allow_skipped_tests"] = json!(true);
    let result = evaluate(&metrics, &config);
    let failures = result["gates"]["failures"].as_array().unwrap();
    assert!(failures.contains(&json!("tests.minimum")));
    assert!(!failures.contains(&json!("tests.skipped")));
}

#[test]
fn comparison_preserves_regression_reasons_and_thresholds() {
    let before = evaluate(&raw(), &policy());
    let mut metrics = raw();
    metrics["tests"]["passed"] = json!(1);
    metrics["tests"]["failed"] = json!(1);
    metrics["parameters_max"] = Value::Null;
    let after = evaluate(&metrics, &policy());
    let comparison = engine::compare(&before, &after, &policy(), &profile()).unwrap();
    assert_eq!(
        comparison["reasons"],
        json!([
            "dimension_regression:correctness",
            "lost_measurement:maintainability",
            "correctness_regression",
            "insufficient_improvement",
            "candidate_not_accepted"
        ])
    );
    let mut metrics = raw();
    metrics["decision_max"] = json!(4);
    let before = evaluate(&metrics, &policy());
    let after = evaluate(&raw(), &policy());
    for (minimum, improved) in [(0.25, true), (0.250001, false)] {
        let mut config = policy();
        config["comparison"]["minimum_delta"] = json!(minimum);
        let result = engine::compare(&before, &after, &config, &profile()).unwrap();
        assert_eq!(result["improved"], improved);
    }
}

#[test]
fn runtime_and_test_contract_changes_block_improvement() {
    let mut metrics = raw();
    metrics["decision_max"] = json!(12);
    let mut baseline = evaluate(&metrics, &policy());
    let mut candidate = evaluate(&raw(), &policy());
    baseline["source"] = json!({"files": {"test/sample.exs": "before"}});
    candidate["source"] = json!({"files": {"test/sample.exs": "after"}});
    baseline["evidence"] = json!({"runtime": {"stdout": "runtime A"}});
    candidate["evidence"] = json!({"runtime": {"stdout": "runtime B"}});
    assert_eq!(
        engine::compare(&baseline, &candidate, &policy(), &profile()).unwrap()["improved"],
        true
    );
    report::compare(&mut candidate, baseline, &policy(), &profile()).unwrap();
    assert_eq!(candidate["baseline_comparison"]["improved"], false);
    assert_eq!(
        candidate["baseline_comparison"]["reasons"],
        json!(["test_contract_changed", "runtime_changed"])
    );
}

#[test]
fn all_gates_have_rules_and_advisories_keep_provenance_without_scoring() {
    let (concepts, rules) = report::catalog().unwrap();
    let result = evaluate(&raw(), &policy());
    let gates: BTreeSet<_> = result["gates"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|gate| gate["id"].as_str().unwrap())
        .collect();
    let rule_gates: BTreeSet<_> = rules
        .iter()
        .filter(|rule| rule["mode"] == "deterministic")
        .filter_map(|rule| rule["gate"].as_str())
        .collect();
    assert_eq!(gates, rule_gates);
    let measured = json!({"functions": [], "advisory": [
        {"rule": "LIST_APPEND_REVIEW", "file": "lib/a.ex", "line": 5},
        {"rule": "LIST_APPEND_REVIEW", "file": "lib/a.ex", "line": 2}
    ]});
    let diagnosis = report::diagnosis(&measured, &result, &policy(), &concepts, &rules).unwrap();
    assert_eq!(result["score"], 10.0);
    assert_eq!(diagnosis["findings"].as_array().unwrap().len(), 2);
    assert_eq!(diagnosis["findings"][0]["location"]["line"], 5);
    assert_eq!(diagnosis["findings"][1]["location"]["line"], 2);
    assert_eq!(diagnosis["findings"][0]["mode"], "advisory");
    assert_eq!(diagnosis["instructions"].as_array().unwrap().len(), 1);
    assert_eq!(
        diagnosis["retrieved_knowledge"][0]["id"],
        "PERSISTENT_APPEND_COST"
    );
    assert!(diagnosis["findings"][0]["sources"][0]["source_sha256"].is_string());
}

#[test]
fn malformed_profiles_are_rejected_at_the_configuration_boundary() {
    let mut profiles = Vec::new();
    let mut malformed = profile();
    malformed["dimensions"]["correctness"]["mode"] = json!("unknown");
    profiles.push(malformed);
    let mut malformed = profile();
    malformed["gates"][0]["check"] =
        json!({"op":"unknown", "left":{"constant":1}, "right":{"constant":2}});
    profiles.push(malformed);
    let mut malformed = profile();
    malformed["dimensions"]["correctness"]["numerator"] =
        json!({"metric":"/tests/passed", "constant":10});
    profiles.push(malformed);
    let mut malformed = profile();
    malformed["gates"][0]["check"] = json!({"truth":{"constant":true}, "or":[]});
    profiles.push(malformed);
    let mut malformed = profile();
    malformed["gates"][0]["check"] = json!({"truth":{"constant":true}, "when_present":42});
    profiles.push(malformed);
    for malformed in profiles {
        assert!(
            engine::validate(&policy(), &malformed).is_err(),
            "{malformed}"
        );
        assert!(engine::evaluate(&raw(), &policy(), &malformed, &[]).is_err());
    }
}

#[test]
fn predicate_or_keeps_unknowns_and_short_circuits_on_true() {
    let mut specification = profile();
    specification["gates"].as_array_mut().unwrap().push(json!({
        "id":"custom", "check":{"or":[
            {"truth":{"metric":"/absent"}},
            {"truth":{"constant":false}}
        ]}
    }));
    let caps = ["build".into(), "tests".into(), "source_ast".into()];
    let result = engine::evaluate(&raw(), &policy(), &specification, &caps).unwrap();
    assert_eq!(result["gates"]["unknown"], json!(["custom"]));
    let gate = specification["gates"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    gate["check"]["or"][1]["truth"]["constant"] = json!(true);
    gate["check"]["or"]
        .as_array_mut()
        .unwrap()
        .push(json!({"truth":{"policy":"/missing"}}));
    let result = engine::evaluate(&raw(), &policy(), &specification, &caps).unwrap();
    assert_eq!(result["accepted"], true);
}
