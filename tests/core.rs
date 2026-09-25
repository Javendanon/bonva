use agent_quality::{data, engine, runner, workspace};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, time::Duration};

fn policy() -> Value {
    data::read(&data::root().join("scoring/policy.json")).unwrap()
}
fn profile() -> Value {
    data::read(&data::root().join("profiles/elixir-mvp.json")).unwrap()
}
fn capabilities() -> Vec<String> {
    vec!["build".into(), "tests".into(), "source_ast".into()]
}
fn raw() -> Value {
    json!({"schema_version":"1.0","build_success":true,"test_command_success":true,"analysis_success":true,"source_unchanged":true,"tests":{"total":2,"passed":2,"failed":0,"skipped":0,"excluded":0},"function_count":3,"decision_max":1,"parameters_max":1,"function_lines_p95":1})
}

#[test]
fn parity_with_64_python_reference_cases() {
    let oracle = data::read(&data::root().join("fixtures/scoring_oracle.json")).unwrap();
    for (i, case) in oracle["cases"].as_array().unwrap().iter().enumerate() {
        let actual = engine::evaluate(
            &case["raw_metrics"],
            &case["policy"],
            &profile(),
            &capabilities(),
        )
        .unwrap();
        assert_eq!(
            numeric_json(&actual),
            numeric_json(&case["expected"]),
            "case {i}"
        );
    }
}

fn numeric_json(v: &Value) -> Value {
    match v {
        Value::Number(n) => json!(n.as_f64().unwrap()),
        Value::Array(a) => Value::Array(a.iter().map(numeric_json).collect()),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), numeric_json(v)))
                .collect(),
        ),
        _ => v.clone(),
    }
}
#[test]
fn missing_capability_prevents_approval() {
    let r = engine::evaluate(&raw(), &policy(), &profile(), &[]).unwrap();
    assert_eq!(r["accepted"], false);
    assert_eq!(r["status"], "incomplete");
    assert!(
        r["gates"]["unknown"]
            .as_array()
            .unwrap()
            .contains(&json!("capability:tests"))
    );
}
#[test]
fn missing_weighted_metric_never_reweights() {
    let mut r = raw();
    r["decision_max"] = Value::Null;
    let result = engine::evaluate(&r, &policy(), &profile(), &capabilities()).unwrap();
    assert!(result["score"].is_null());
    assert_eq!(result["accepted"], false);
}
#[test]
fn language_independent_profile() {
    let p = json!({"schema_version":"1.0","profile":"toy","weights":{"size":1},"target_score":8,"normalization":{"size":{"metric":"nodes","good":10,"bad":100}},"comparison":{"minimum_delta":0.1,"maximum_dimension_drop":0.5}});
    let spec = json!({"schema_version":"1.0","id":"toy","required_capabilities":["count"],"dimension_order":["size"],"protected_dimensions":[],"dimensions":{"size":{"mode":"cost","value":{"metric":"/nodes"},"when":{"truth":{"metric":"/parsed"}}}},"gates":[{"id":"parse","check":{"truth":{"metric":"/parsed"}}}]});
    let result = engine::evaluate(
        &json!({"schema_version":"1.0","nodes":10,"parsed":true}),
        &p,
        &spec,
        &["count".into()],
    )
    .unwrap();
    assert_eq!(result["score"], 10.0);
    assert_eq!(result["accepted"], true);
}
#[test]
fn invalid_weights_intervals_and_types_fail() {
    for value in [json!(true), json!(-1), json!(0.3), json!("0.4")] {
        let mut p = policy();
        p["weights"]["correctness"] = value;
        assert!(engine::validate(&p, &profile()).is_err());
    }
    let mut p = policy();
    p["normalization"]["readability"]["bad"] = json!(15);
    assert!(engine::validate(&p, &profile()).is_err());
    let mut r = raw();
    r["function_count"] = json!(true);
    assert!(data::schema("metrics", &r).is_err());
    assert!(data::decode(br#"{"a":1,"a":2}"#).is_err());
    assert!(data::decode(b"NaN").is_err());
}
#[test]
fn gate_cannot_be_overridden_by_high_score() {
    let mut r = raw();
    r["build_success"] = json!(false);
    let result = engine::evaluate(&r, &policy(), &profile(), &capabilities()).unwrap();
    assert_eq!(result["score"], 10.0);
    assert_eq!(result["accepted"], false);
}
#[test]
fn unknown_adapter_is_explicit() {
    assert!(agent_quality::adapters::resolve("not-installed").is_err());
}
#[test]
fn score_boundaries_and_monotonicity() {
    let mut previous = 11.0;
    for n in 0..30 {
        let mut r = raw();
        r["decision_max"] = json!(n);
        let result = engine::evaluate(&r, &policy(), &profile(), &capabilities()).unwrap();
        let score = result["score"].as_f64().unwrap();
        assert!(score <= previous);
        previous = score;
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
#[test]
fn workspace_sync_reuses_and_tracks_changes_deletions_and_permissions() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original");
    write(&original.join("lib/a.ex"), "first");
    write(&original.join("lib/b.ex"), "remove");
    let original = original.canonicalize().unwrap();
    let cache = temp.path().join("cache");
    let before = workspace::inventory(&original).unwrap();
    let first = workspace::Workspace::open(&original, &cache, &before, "test").unwrap();
    assert_eq!(first.info["copied_files"], 2);
    write(&first.project.join("_build/artifact"), "compiled");
    first.complete(true).unwrap();
    drop(first);
    let second = workspace::Workspace::open(&original, &cache, &before, "test").unwrap();
    assert_eq!(second.info["reused"], true);
    assert_eq!(second.info["copied_files"], 0);
    assert!(second.project.join("_build/artifact").exists());
    second.complete(true).unwrap();
    drop(second);
    write(&original.join("lib/a.ex"), "changed");
    fs::remove_file(original.join("lib/b.ex")).unwrap();
    let after = workspace::inventory(&original).unwrap();
    let third = workspace::Workspace::open(&original, &cache, &after, "test").unwrap();
    assert_eq!(third.info["copied_files"], 1);
    assert_eq!(third.info["removed_files"], 1);
    assert_eq!(
        fs::read_to_string(third.project.join("lib/a.ex")).unwrap(),
        "changed"
    );
    assert!(!third.project.join("lib/b.ex").exists());
}
#[test]
fn cache_invalidation_and_locking() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original");
    write(&original.join("mix.lock"), "lock1");
    let original = original.canonicalize().unwrap();
    let cache = temp.path().join("cache");
    let inv = workspace::inventory(&original).unwrap();
    let env = BTreeMap::new();
    let a = workspace::fingerprint(&inv, &env, "runtime1", &json!({}), &json!({}));
    let b = workspace::fingerprint(&inv, &env, "runtime2", &json!({}), &json!({}));
    assert_ne!(a, b);
    let work = workspace::Workspace::open(&original, &cache, &inv, &a).unwrap();
    assert!(workspace::Workspace::open(&original, &cache, &inv, &a).is_err());
    write(&work.project.join("_build/partial"), "unfinished");
    drop(work);
    let work = workspace::Workspace::open(&original, &cache, &inv, &a).unwrap();
    assert!(!work.project.join("_build/partial").exists());
    drop(work);
    write(&original.join("mix.lock"), "lock2");
    let c = workspace::fingerprint(
        &workspace::inventory(&original).unwrap(),
        &env,
        "runtime1",
        &json!({}),
        &json!({}),
    );
    assert_ne!(a, c);
}
#[test]
#[cfg(unix)]
fn symlinks_rejected_before_copying() {
    let temp = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink("/tmp", temp.path().join("link")).unwrap();
    assert!(workspace::inventory(temp.path()).is_err());
}
#[test]
fn timeout_and_missing_tool_are_not_success() {
    let temp = tempfile::tempdir().unwrap();
    let output = runner::run(
        &["/bin/sh".into(), "-c".into(), "sleep 10".into()],
        temp.path(),
        &workspace::environment(),
        Duration::from_millis(50),
        temp.path(),
        "timeout",
    )
    .unwrap();
    assert_eq!(output["status"], "timeout");
    assert_eq!(runner::success(&output), Some(false));
    let missing = runner::run(
        &["/missing/tool".into()],
        temp.path(),
        &BTreeMap::new(),
        Duration::from_secs(1),
        temp.path(),
        "missing",
    )
    .unwrap();
    assert_eq!(runner::success(&missing), None);
}
