use agent_quality::{data, list_analysis, workspace};
use serde_json::json;

#[test]
#[ignore = "requires Elixir >= 1.18 on PATH"]
fn actual_list_adapter_validates_and_rejects_tampered_evidence() {
    let cache = tempfile::tempdir().unwrap();
    for (fixture, expected) in [("bad", "completed_with_findings"), ("good", "completed")] {
        let project = data::root().join("fixtures").join(fixture);
        let report = list_analysis::analyze(&project, cache.path()).unwrap();
        assert_eq!(report["status"], expected);
        assert_eq!(report["source_unchanged"], true);
        assert_eq!(
            report["evidence"]["experiments"][0]["checks"]
                .as_array()
                .unwrap()
                .len(),
            366
        );
        let inventory = workspace::inventory(&project).unwrap();
        let paths: Vec<_> = report["evidence"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["file"].as_str().unwrap().to_owned())
            .collect();
        let mut different_observations = report["evidence"].clone();
        for experiment in different_observations["experiments"]
            .as_array_mut()
            .unwrap()
        {
            for row in experiment["series"].as_array_mut().unwrap() {
                for sample in row["samples"].as_array_mut().unwrap() {
                    sample["actual"]["elapsed_ns"] = json!(0);
                    sample["alternative"]["elapsed_ns"] = json!(u64::MAX);
                    sample["actual"]["reductions"] = json!(0);
                }
            }
        }
        assert_eq!(
            json!(list_analysis::validate(&different_observations, &paths, &inventory).unwrap()),
            report["findings"]
        );
        for pointer in [
            "/experiments/0/checks/0/actual",
            "/experiments/0/series/0/model/actual/constructors",
            "/experiments/0/series/0/samples/0/actual/matches_identity",
            "/experiments/0/premises/default_bindings",
            "/files/0/source_sha256",
            "/omitted_sites",
        ] {
            let mut raw = report["evidence"].clone();
            *raw.pointer_mut(pointer).unwrap() = json!("tampered");
            assert!(
                list_analysis::validate(&raw, &paths, &inventory).is_err(),
                "{pointer}"
            );
        }
    }
}

#[test]
#[ignore = "requires Elixir >= 1.18 on PATH"]
fn coverage_failure_is_not_a_success() {
    let cache = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("lib")).unwrap();
    for content in [
        "defmodule Empty do end",
        "defmodule Invalid do",
        "defmodule X do\ndef f(xs), do: xs ++ [1]\nend",
    ] {
        std::fs::write(project.path().join("lib/input.ex"), content).unwrap();
        let report = list_analysis::analyze(project.path(), cache.path()).unwrap();
        assert_eq!(report["status"], "incomplete");
        assert!(report["findings"].as_array().unwrap().is_empty());
    }
    let functions: String = (0..9)
        .map(|i| format!("def f{i}(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end)\n"))
        .collect();
    std::fs::write(
        project.path().join("lib/input.ex"),
        format!("defmodule Nine do\n{functions}end"),
    )
    .unwrap();
    let report = list_analysis::analyze(project.path(), cache.path()).unwrap();
    assert_eq!(report["status"], "incomplete");
    assert_eq!(report["evidence"]["omitted_sites"], 1);
    assert_eq!(report["findings"].as_array().unwrap().len(), 8);
}
