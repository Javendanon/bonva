//! A versioned, deliberately narrow rule. Observations never participate in its decision.
use crate::{data, paths, runner, workspace};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn cost(pattern: &str, n: u64) -> Result<Value> {
    ensure!(n <= 8192, "Model size exceeds supported bound");
    let (visits, constructors) = match pattern {
        "append_singleton" => (n * n.saturating_sub(1) / 2, n * (n + 1) / 2),
        "prepend_reverse" => (n, 2 * n),
        _ => anyhow::bail!("Unsupported model pattern"),
    };
    Ok(
        json!({"result_matches":true,"input_visits":n,"extra_spine_visits":visits,"constructors":constructors}),
    )
}

fn cases() -> Vec<Value> {
    let mut all = vec![json!([])];
    let mut level = vec![Vec::<i64>::new()];
    for _ in 1..=5 {
        level = [-1, 0, 1]
            .iter()
            .flat_map(|x| {
                level.iter().map(move |tail| {
                    let mut v = vec![*x];
                    v.extend(tail);
                    v
                })
            })
            .collect();
        all.extend(level.iter().map(|v| json!(v)));
    }
    all.extend([json!([null, "x", [], [1, 1]]), json!([1, 1, 0, 1])]);
    all
}

pub fn statistics(values: &[u64]) -> Result<Value> {
    ensure!(!values.is_empty(), "Empty sample");
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    Ok(
        json!({"median":sorted[sorted.len()/2],"p95":sorted[(95*sorted.len()).div_ceil(100)-1],"count":sorted.len()}),
    )
}

fn knowledge(config: &Value) -> Result<Value> {
    let manifest = data::read(&data::root().join("knowledge/sources/manifest.json"))?;
    let mut sources = config["sources"]
        .as_array()
        .context("Rule sources")?
        .clone();
    for source in &mut sources {
        let record = manifest
            .as_array()
            .context("Manifest")?
            .iter()
            .find(|r| r["id"] == source["book"])
            .context("Unknown source")?;
        for (file, hash) in [("file", "sha256"), ("extracted", "extracted_sha256")] {
            ensure!(
                data::digest(&data::root().join(record[file].as_str().context("Source path")?))?
                    == record[hash].as_str().context("Source hash")?,
                "Knowledge integrity failure"
            );
        }
        let units = data::read(&data::root().join(record["extracted"].as_str().unwrap()))?;
        for unit in source["units"].as_array().context("Source units")? {
            let n = unit.as_u64().context("Unit number")?;
            ensure!(
                n > 0 && n <= units.as_array().context("Extracted units")?.len() as u64,
                "Invalid unit"
            );
        }
        source["source_sha256"] = record["sha256"].clone();
        source["extracted_sha256"] = record["extracted_sha256"].clone();
        source["unit_kind"] = record["unit_kind"].clone();
    }
    Ok(
        json!({"concept":config["concept"],"sources":sources,"language_references":config["language_references"]}),
    )
}

/// Validate detector evidence against inputs and independently computed expectations.
pub fn validate(
    raw: &Value,
    paths: &[String],
    inventory: &workspace::Inventory,
) -> Result<Vec<Value>> {
    ensure!(
        raw["schema_version"] == "1.0" && raw["detector"] == "list_construction_v1",
        "Unknown detector"
    );
    let files = raw["files"].as_array().context("File evidence")?;
    ensure!(files.len() == paths.len(), "Missing file evidence");
    let mut detected = vec![];
    for (file, path) in files.iter().zip(paths) {
        ensure!(
            file["file"] == *path && file["source_sha256"] == inventory[path].sha256,
            "Source evidence mismatch"
        );
        ensure!(
            file["success"].is_boolean() && file["unsupported"].is_array(),
            "Invalid coverage evidence"
        );
        for site in file["sites"].as_array().context("Sites")? {
            ensure!(
                file["success"] == true
                    && site["file"] == *path
                    && site["source_sha256"] == inventory[path].sha256,
                "Site source mismatch"
            );
            ensure!(
                site["line"].as_u64().is_some_and(|n| n > 0)
                    && site["arity"] == 1
                    && site["domain"] == "proper_lists_only",
                "Invalid site scope"
            );
            for key in [
                "single_unguarded_clause",
                "empty_accumulator",
                "input_is_parameter",
                "identity_element",
                "fixed_singleton_append_or_prepend",
                "default_bindings",
            ] {
                ensure!(site["premises"][key] == true, "Unverified premise {key}");
            }
            detected.push(site);
        }
    }
    let experiments = raw["experiments"].as_array().context("Experiments")?;
    ensure!(
        experiments.len() == detected.len().min(8)
            && raw["omitted_sites"].as_u64() == Some(detected.len().saturating_sub(8) as u64),
        "Coverage mismatch"
    );
    let expected_cases = cases();
    let mut findings = vec![];
    for (experiment, site) in experiments.iter().zip(detected) {
        for (key, value) in site.as_object().context("Site object")? {
            ensure!(
                experiment.get(key) == Some(value),
                "Experiment/site mismatch"
            );
        }
        let checks = experiment["checks"].as_array().context("Behavior checks")?;
        ensure!(
            checks.len() == expected_cases.len(),
            "Missing behavior checks"
        );
        for (check, input) in checks.iter().zip(&expected_cases) {
            ensure!(
                &check["input"] == input
                    && &check["actual"] == input
                    && &check["alternative"] == input
                    && check["passed"] == true,
                "Identity check failed"
            );
        }
        let series = experiment["series"].as_array().context("Series")?;
        ensure!(series.len() == 4, "Missing size series");
        let pattern = site["pattern"].as_str().context("Pattern")?;
        let mut modeled = vec![];
        for (row, n) in series.iter().zip([128, 256, 512, 1024]) {
            ensure!(
                row["n"] == n && row["input"] == json!((1..=n).collect::<Vec<_>>()),
                "Wrong size/input"
            );
            let actual = cost(pattern, n)?;
            let alternative = cost("prepend_reverse", n)?;
            ensure!(
                row["model"]["actual"] == actual && row["model"]["alternative"] == alternative,
                "Cost model mismatch"
            );
            let samples = row["samples"].as_array().context("Samples")?;
            ensure!(samples.len() == 9, "Missing samples");
            for (i, sample) in samples.iter().enumerate() {
                let order = if i % 2 == 0 {
                    json!(["actual", "alternative"])
                } else {
                    json!(["alternative", "actual"])
                };
                ensure!(sample["order"] == order, "Sample order mismatch");
                for variant in ["actual", "alternative"] {
                    ensure!(
                        sample[variant]["matches_identity"] == true
                            && sample[variant]["elapsed_ns"].as_u64().is_some()
                            && sample[variant]["reductions"].as_u64().is_some(),
                        "Invalid sample"
                    );
                }
            }
            modeled.push(json!({"n":n,"actual":actual,"alternative":alternative}));
        }
        if pattern == "append_singleton" {
            ensure!(
                modeled.iter().all(|m| m["actual"]["constructors"].as_u64()
                    > m["alternative"]["constructors"].as_u64()),
                "Decision premise failed"
            );
            findings.push(json!({"rule":"LIST_CONSTRUCTION_V1","file":site["file"],"line":site["line"],"function":site["name"],
                "finite_identity_checks":checks.len(),"premises":site["premises"],"modeled_work":modeled,
                "recommendation":"For this verified identity-copy expression on proper lists, prepend then reverse reduces modeled constructor work at the evaluated sizes.",
                "model_growth":{"actual":"quadratic","alternative":"linear"}}));
        }
    }
    Ok(findings)
}

pub fn analyze(project: &Path, cache: &Path) -> Result<Value> {
    let project = project.canonicalize()?;
    ensure!(project.is_dir(), "Project directory required");
    let cache = paths::resolve_destination(cache)?;
    ensure!(
        !cache.starts_with(&project) && !project.starts_with(&cache),
        "Cache and project must be disjoint"
    );
    let config_path = data::root().join("rules/list-construction.json");
    let config = data::read(&config_path)?;
    // Configuration is versioned with this validator: never silently change the contract.
    ensure!(
        config == serde_json::from_str::<Value>(include_str!("../rules/list-construction.json"))?,
        "Rebuild after changing rule configuration"
    );
    ensure!(
        config["domain"] == json!({"alphabet":[-1,0,1],"max_length":5})
            && config["additional_cases"] == json!([[null, "x", [], [1, 1]], [1, 1, 0, 1]])
            && config["benchmark"] == json!({"sizes":[128,256,512,1024],"warmup":2,"samples":9})
            && config["limits"] == json!({"max_sites":8,"timeout_seconds":90})
            && config["contract"] == "identity_copy_of_proper_list"
            && config["cost_model"] == "linked_list_spine_v1",
        "Unsupported rule configuration"
    );
    let knowledge = knowledge(&config)?;
    let before = workspace::inventory(&project)?;
    let paths: Vec<String> = before
        .keys()
        .filter(|p| p.starts_with("lib/") && (p.ends_with(".ex") || p.ends_with(".exs")))
        .cloned()
        .collect();
    let analyzer = data::root().join("analyzers/list_construction.exs");
    let analyzer_hash = data::digest(&analyzer)?;
    let config_hash = data::digest(&config_path)?;
    let logs = cache.join(format!(
        "lists-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(&cache)?;
    fs::create_dir(&logs)?;
    let mut command = vec![
        "elixir".into(),
        analyzer.to_string_lossy().into_owned(),
        config_path.to_string_lossy().into_owned(),
    ];
    command.extend(paths.iter().cloned());
    let mut env = workspace::environment();
    env.remove("QUALITY_LIST_LIBRARY");
    for key in [
        "ELIXIR_ERL_OPTIONS",
        "ERL_AFLAGS",
        "ERL_FLAGS",
        "ERL_ZFLAGS",
        "ERL_LIBS",
    ] {
        env.remove(key);
    }
    let run = runner::run(
        &command,
        &project,
        &env,
        Duration::from_secs(90),
        &logs,
        "list-analysis",
    )?;
    ensure!(
        before == workspace::inventory(&project)?,
        "Original project changed during analysis"
    );
    ensure!(
        analyzer_hash == data::digest(&analyzer)? && config_hash == data::digest(&config_path)?,
        "Evaluator changed during run"
    );
    let mut result = json!({"schema_version":"1.0","mode":"list_analysis","status":"incomplete","score":null,"affects_mvp_score":false,
        "project":project,"scope":"Recognized extracted identity-copy expressions in lib/**/*.ex and lib/**/*.exs; proper lists only",
        "knowledge":knowledge,"rule":config,"provenance":{"rule_sha256":config_hash,"analyzer_sha256":analyzer_hash,"engine_sha256":data::digest(&std::env::current_exe()?)?},
        "source_unchanged":true,"execution":{"status":run["status"],"exit_code":run["exit_code"],"elapsed_ms":run["elapsed_ms"],"logs":logs},
        "limitations":["Finite behavior checks are not a universal equivalence proof.","Model constructors are not measured BEAM allocations or memory bytes.","Timings and reductions are environment-sensitive extracted-expression microbenchmarks, not application performance or score inputs.","Unsupported syntax and unmatched functions are not certified efficient; project code is not compiled or loaded."],"findings":[]});
    if runner::success(&run) != Some(true) {
        return Ok(result);
    }
    let raw = data::read(&logs.join("list-analysis.stdout"))?;
    let findings = validate(&raw, &paths, &before)?;
    let complete = !raw["experiments"].as_array().unwrap().is_empty()
        && raw["omitted_sites"] == 0
        && raw["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["success"] == true && f["unsupported"].as_array().unwrap().is_empty());
    let mut observations = vec![];
    for experiment in raw["experiments"].as_array().unwrap() {
        for row in experiment["series"].as_array().unwrap() {
            let mut summary =
                json!({"file":experiment["file"],"line":experiment["line"],"n":row["n"]});
            for variant in ["actual", "alternative"] {
                for metric in ["elapsed_ns", "reductions"] {
                    let values: Vec<u64> = row["samples"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| s[variant][metric].as_u64().unwrap())
                        .collect();
                    if summary.get(variant).is_none() {
                        summary[variant] = json!({});
                    }
                    summary[variant][metric] = statistics(&values)?;
                }
            }
            observations.push(summary);
        }
    }
    result["status"] = json!(if !complete {
        "incomplete"
    } else if findings.is_empty() {
        "completed"
    } else {
        "completed_with_findings"
    });
    result["findings"] = json!(findings);
    result["observations"] = json!(observations);
    result["evidence"] = raw;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_model_and_domain() {
        assert_eq!(cases().len(), 366);
        assert_eq!(
            cost("append_singleton", 1024).unwrap()["constructors"],
            524800
        );
        assert_eq!(cost("prepend_reverse", 1024).unwrap()["constructors"], 2048);
        assert_eq!(cost("append_singleton", 0).unwrap()["constructors"], 0);
        assert!(cost("unknown", 1).is_err());
        assert!(cost("append_singleton", u64::MAX).is_err());
        assert_eq!(
            statistics(&[9, 1, 5, 3, 7]).unwrap(),
            json!({"count":5,"median":5,"p95":9})
        );
        assert!(statistics(&[]).is_err());
    }
}
