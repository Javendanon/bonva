//! Requires Elixir, Mix and local sockets. Run explicitly with --ignored.
use agent_quality::{data, report, workspace};
use serde_json::Value;
use std::{fs, path::Path};
fn policy() -> Value {
    data::read(&data::root().join("scoring/policy.json")).unwrap()
}
fn profile() -> Value {
    data::read(&data::root().join("profiles/elixir-mvp.json")).unwrap()
}
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for (p, _) in workspace::inventory(from).unwrap() {
        let dest = to.join(&p);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(from.join(p), dest).unwrap();
    }
}
#[test]
#[ignore = "requires Mix execution"]
fn good_bad_cold_warm_match_and_do_not_modify_original() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let good = data::root().join("fixtures/good");
    let bad = data::root().join("fixtures/bad");
    let before = workspace::inventory(&good).unwrap();
    let cold = report::evaluate(&good, &policy(), &profile(), &cache).unwrap();
    let warm = report::evaluate(&good, &policy(), &profile(), &cache).unwrap();
    assert_eq!(cold["score"], 10.0);
    assert_eq!(warm["score"], 10.0);
    assert_eq!(warm["execution"]["cache"]["reused"], true);
    assert_eq!(warm["execution"]["cache"]["copied_files"], 0);
    for key in [
        "raw_metrics",
        "score",
        "gates",
        "quality_vector",
        "diagnosis",
        "source",
    ] {
        assert_eq!(cold[key], warm[key], "{key}");
    }
    assert!(
        !warm["evidence"]["build"]["stdout"]
            .as_str()
            .unwrap()
            .contains("Compiling")
    );
    assert_eq!(warm["raw_metrics"]["tests"]["passed"], 2);
    let bad_report = report::evaluate(&bad, &policy(), &profile(), &cache).unwrap();
    assert_eq!(bad_report["score"], 6.683333);
    assert_eq!(bad_report["accepted"], false);
    assert_eq!(before, workspace::inventory(&good).unwrap());
}
#[test]
#[ignore = "requires Mix execution"]
fn changed_source_and_tests_are_reexecuted_in_warm_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    copy(&data::root().join("fixtures/good"), &project);
    let cache = temp.path().join("cache");
    assert_eq!(
        report::evaluate(&project, &policy(), &profile(), &cache).unwrap()["accepted"],
        true
    );
    fs::write(
        project.join("lib/classifier.ex"),
        "defmodule Classifier do\n def label(_), do: :wrong\n def copy(xs), do: xs\nend\n",
    )
    .unwrap();
    let failed = report::evaluate(&project, &policy(), &profile(), &cache).unwrap();
    assert_eq!(failed["execution"]["cache"]["reused"], true);
    assert_eq!(failed["raw_metrics"]["tests"]["failed"], 1);
    assert_eq!(failed["accepted"], false);
    fs::write(
        project.join("test/classifier_test.exs"),
        "defmodule Empty do\n use ExUnit.Case\nend",
    )
    .unwrap();
    let empty = report::evaluate(&project, &policy(), &profile(), &cache).unwrap();
    assert_eq!(empty["accepted"], false);
    assert_eq!(empty["raw_metrics"]["tests"]["total"], 0);
    fs::write(project.join("lib/classifier.ex"), "defmodule Broken do def").unwrap();
    let broken = report::evaluate(&project, &policy(), &profile(), &cache).unwrap();
    assert_eq!(broken["raw_metrics"]["build_success"], false);
    assert!(broken["raw_metrics"]["tests"].is_null());
}
#[test]
#[ignore = "requires Mix execution"]
fn source_mutation_blocks_and_changed_contract_rejects_comparison() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    copy(&data::root().join("fixtures/good"), &project);
    let cache = temp.path().join("cache");
    fs::write(project.join("test/classifier_test.exs"),"defmodule OutputTest do\n use ExUnit.Case\n test \"write\" do\n File.write!(\"lib/generated.ex\", \"# change\")\n assert true\n end\nend").unwrap();
    let report = report::evaluate(&project, &policy(), &profile(), &cache).unwrap();
    assert_eq!(report["raw_metrics"]["source_unchanged"], false);
    assert_eq!(report["accepted"], false);
    assert!(!project.join("lib/generated.ex").exists());
}

#[test]
#[ignore = "requires Mix execution"]
fn generated_outputs_are_allowed_without_modifying_original() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    copy(&data::root().join("fixtures/good"), &project);
    fs::write(
        project.join("test/output_test.exs"),
        r#"defmodule OutputTest do
          use ExUnit.Case
          test "generated output" do
            File.mkdir_p!("tmp")
            File.write!("tmp/result.txt", "generated")
            assert true
          end
        end"#,
    )
    .unwrap();
    let before = workspace::inventory(&project).unwrap();
    let result =
        report::evaluate(&project, &policy(), &profile(), &temp.path().join("cache")).unwrap();
    assert_eq!(result["accepted"], true);
    assert_eq!(result["raw_metrics"]["source_unchanged"], true);
    assert_eq!(workspace::inventory(&project).unwrap(), before);
    assert!(!project.join("tmp/result.txt").exists());
}
