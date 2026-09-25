//! Exercise the executable boundary: JSON, exit codes and output protection.
use agent_quality::data;
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Output},
};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-quality"))
        .args(args)
        .output()
        .unwrap()
}

fn metrics_file(directory: &std::path::Path) -> std::path::PathBuf {
    let path = directory.join("metrics.json");
    fs::write(
        &path,
        json!({
            "schema_version":"1.0", "build_success":true, "test_command_success":true,
            "analysis_success":true, "source_unchanged":true,
            "tests":{"total":2,"passed":2,"failed":0,"skipped":0,"excluded":0},
            "function_count":3,"decision_max":1,"parameters_max":1,"function_lines_p95":1
        })
        .to_string(),
    )
    .unwrap();
    path
}

#[test]
fn help_and_adapter_discovery_do_not_require_a_project() {
    for args in [&[][..], &["help"], &["--help"]] {
        let output = cli(args);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("does not attest"));
    }
    let output = cli(&["adapters"]);
    assert!(output.status.success());
    assert_eq!(
        data::decode(&output.stdout).unwrap()["adapters"][0]["id"],
        "elixir"
    );
}

#[test]
fn invalid_commands_and_options_return_structured_errors() {
    for args in [
        vec!["unknown"],
        vec!["evaluate"],
        vec!["score", "missing", "--output"],
        vec!["score", "missing", "--baseline", "other"],
        vec!["analyze-lists", "missing", "--policy", "other"],
        vec!["evaluate", "missing", "--unknown", "value"],
    ] {
        let output = cli(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let error = data::decode(&output.stderr).unwrap();
        assert_eq!(error["status"], "error");
        assert!(error["error"].is_string());
    }
}

#[test]
fn score_replay_is_incomplete_and_does_not_execute_language_tools() {
    let temp = tempfile::tempdir().unwrap();
    let metrics = metrics_file(temp.path());
    let output = Command::new(env!("CARGO_BIN_EXE_agent-quality"))
        .args(["score", metrics.to_str().unwrap()])
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let result = data::decode(&output.stdout).unwrap();
    assert_eq!(result["mode"], "unattested_replay");
    assert_eq!(result["score"], 10.0);
    assert_eq!(result["status"], "incomplete");
    assert_eq!(result["accepted"], false);
}

#[test]
fn rejection_and_output_file_behavior_remain_stable() {
    let temp = tempfile::tempdir().unwrap();
    let metrics = metrics_file(temp.path());
    let mut raw = data::read(&metrics).unwrap();
    raw["build_success"] = json!(false);
    fs::write(&metrics, raw.to_string()).unwrap();
    let destination = temp.path().join("nested/report.json");
    let args = [
        "score",
        metrics.to_str().unwrap(),
        "--output",
        destination.to_str().unwrap(),
    ];
    let output = cli(&args);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let bytes = fs::read(&destination).unwrap();
    assert_eq!(data::decode(&bytes).unwrap()["status"], "rejected");
    assert!(bytes.ends_with(b"\n"));
    assert_eq!(cli(&args).status.code(), Some(2));
    assert_eq!(fs::read(&destination).unwrap(), bytes);
}

#[test]
fn project_output_is_rejected_before_creating_a_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir(&project).unwrap();
    let destination = project.join("report.json");
    let cache = temp.path().join("cache");
    for command in ["evaluate", "analyze-lists"] {
        let output = cli(&[
            command,
            project.to_str().unwrap(),
            "--output",
            destination.to_str().unwrap(),
            "--cache-dir",
            cache.to_str().unwrap(),
        ]);
        assert_eq!(output.status.code(), Some(2));
        let error: Value = data::decode(&output.stderr).unwrap();
        assert!(
            error["error"]
                .as_str()
                .unwrap()
                .contains("outside evaluated projects")
        );
        assert!(!cache.exists());
        assert!(!destination.exists());
    }
}

#[test]
#[ignore = "requires Elixir, Mix and local sockets"]
fn real_commands_preserve_success_rejection_and_list_exit_codes() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let good = data::root().join("fixtures/good");
    let bad = data::root().join("fixtures/bad");
    for (project, code, status) in [(&good, 0, "accepted"), (&bad, 1, "rejected")] {
        let output = cli(&[
            "evaluate",
            project.to_str().unwrap(),
            "--cache-dir",
            cache.to_str().unwrap(),
        ]);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(data::decode(&output.stdout).unwrap()["status"], status);
    }
    for (project, code, status) in [
        (&good, 0, "completed"),
        (&bad, 1, "completed_with_findings"),
    ] {
        let output = cli(&[
            "analyze-lists",
            project.to_str().unwrap(),
            "--cache-dir",
            cache.to_str().unwrap(),
        ]);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(data::decode(&output.stdout).unwrap()["status"], status);
    }
    let output = cli(&[
        "evaluate",
        good.to_str().unwrap(),
        "--baseline",
        good.to_str().unwrap(),
        "--cache-dir",
        cache.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let result = data::decode(&output.stdout).unwrap();
    assert_eq!(result["accepted"], true);
    assert_eq!(result["baseline_comparison"]["improved"], false);
}
