use crate::{
    adapters::Adapter,
    data, runner,
    workspace::{self, Workspace},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub struct Elixir;
impl Adapter for Elixir {
    fn id(&self) -> &str {
        "elixir"
    }
    fn capabilities(&self) -> Vec<String> {
        ["build", "tests", "source_ast"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    fn collect(&self, project: &Path, policy: &Value, cache: &Path) -> Result<Value> {
        let started = Instant::now();
        let original = project.canonicalize()?;
        ensure!(
            original.join("mix.exs").is_file(),
            "Expected individual Mix project with mix.exs"
        );
        let timeout = policy["execution"]["timeout_seconds"]
            .as_f64()
            .context("Timeout required")?;
        ensure!(
            (1.0..=86400.0).contains(&timeout),
            "Timeout must be between 1 and 86400 seconds"
        );
        let timeout = Duration::from_secs_f64(timeout);
        let seed = policy["execution"]["seed"]
            .as_u64()
            .context("Nonnegative integer seed required")?;
        let generated: Vec<String> = policy["execution"]["generated_directories"]
            .as_array()
            .context("Generated directories required")?
            .iter()
            .map(|v| {
                let name = v.as_str().context("Directory name")?;
                ensure!(
                    !name.is_empty()
                        && !name.contains(['/', '\\'])
                        && !matches!(
                            name,
                            "." | ".."
                                | "lib"
                                | "deps"
                                | "test"
                                | "config"
                                | "priv"
                                | "mix.exs"
                                | "mix.lock"
                        ),
                    "Invalid generated directory"
                );
                Ok(name.to_string())
            })
            .collect::<Result<_>>()?;
        ensure!(
            policy["gates"]["minimum_tests"]
                .as_u64()
                .is_some_and(|n| n > 0),
            "Positive minimum_tests required"
        );
        ensure!(
            policy["gates"]["allow_skipped_tests"].is_boolean(),
            "allow_skipped_tests must be boolean"
        );
        ensure!(
            policy["gates"]["decision_max"]
                .as_u64()
                .is_some_and(|n| n > 0),
            "Positive decision limit required"
        );
        // Resolve cache before writing so it can never be located inside the target.
        let absolute_cache = crate::cli::resolve_destination(cache)?;
        ensure!(
            !absolute_cache.starts_with(&original) && !original.starts_with(&absolute_cache),
            "Cache and project must be disjoint"
        );
        let before = workspace::inventory(&original)?;
        let inputs = workspace::input_inventory(&before, &generated);
        let mut env = workspace::environment();
        // Keep preflight logs out of the read-only input tree.
        fs::create_dir_all(&absolute_cache)?;
        let preflight = absolute_cache.join(format!("preflight-{}", std::process::id()));
        let version = runner::run(
            &strings(&["elixir", "--version"]),
            &original,
            &env,
            timeout,
            &preflight,
            "version",
        )?;
        let descriptor = json!({"id":self.id(),"protocol_version":"1.0","ast":data::digest(&data::root().join("analyzers/elixir_ast.exs"))?,"formatter":data::digest(&data::root().join("analyzers/exunit_formatter.exs"))?});
        let fingerprint = workspace::fingerprint(
            &inputs,
            &env,
            version["stdout"].as_str().unwrap_or("unavailable"),
            &descriptor,
            &policy["execution"],
        );
        let work = Workspace::open(&original, &absolute_cache, &inputs, &fingerprint)?;
        let mut raw = json!({"schema_version":"1.0","build_success":null,"test_command_success":null,"analysis_success":null,"source_unchanged":true,"tests":null,"function_count":null,"decision_max":null,"parameters_max":null,"function_lines_p95":null});
        let mut evidence = json!({"runtime":version});
        let source_paths: Vec<_> = before
            .keys()
            .filter(|p| p.starts_with("lib/") && (p.ends_with(".ex") || p.ends_with(".exs")))
            .cloned()
            .collect();
        let mut command = strings(&["elixir"]);
        command.push(
            data::root()
                .join("analyzers/elixir_ast.exs")
                .to_string_lossy()
                .into(),
        );
        command.extend(source_paths.clone());
        // This is parse-only. It does not load mix.exs, compile, or expand target macros.
        let ast = runner::run(&command, &original, &env, timeout, &work.logs, "ast")?;
        raw["analysis_success"] = json!(runner::success(&ast));
        let mut functions = vec![];
        let mut advisory = vec![];
        if runner::success(&ast) == Some(true) {
            match parse_ast(ast["stdout"].as_str().unwrap_or(""), &source_paths, &before) {
                Ok(rows) => {
                    let ok = rows.iter().all(|r| r["success"] == true);
                    raw["analysis_success"] = json!(ok);
                    evidence["ast_results"] = json!(rows);
                    if ok {
                        for row in rows {
                            functions.extend(row["functions"].as_array().unwrap().clone());
                            advisory.extend(row["advisory"].as_array().unwrap().clone());
                        }
                        raw["function_count"] = json!(functions.len());
                        if !functions.is_empty() {
                            raw["decision_max"] = json!(
                                functions
                                    .iter()
                                    .map(|f| f["decision_indicator"].as_u64().unwrap())
                                    .max()
                            );
                            raw["parameters_max"] =
                                json!(functions.iter().map(|f| f["arity"].as_u64().unwrap()).max());
                            let mut lengths: Vec<_> = functions
                                .iter()
                                .map(|f| f["lines"].as_u64().unwrap())
                                .collect();
                            lengths.sort();
                            raw["function_lines_p95"] =
                                json!(lengths[(95 * lengths.len()).div_ceil(100) - 1]);
                        }
                    }
                }
                Err(e) => {
                    raw["analysis_success"] = json!(false);
                    evidence["ast_adapter_error"] = json!(e.to_string());
                }
            }
        }
        evidence["ast"] = ast;
        let build = runner::run(
            &strings(&["mix", "compile", "--warnings-as-errors"]),
            &work.project,
            &env,
            timeout,
            &work.logs,
            "build",
        )?;
        raw["build_success"] = json!(runner::success(&build));
        evidence["build"] = build;
        if raw["build_success"] == true {
            let result_file = work.logs.join("exunit.json");
            env.insert(
                "QUALITY_TEST_RESULT".into(),
                result_file.to_string_lossy().into(),
            );
            let command = vec![
                "elixir".into(),
                "-r".into(),
                data::root()
                    .join("analyzers/exunit_formatter.exs")
                    .to_string_lossy()
                    .into(),
                "-S".into(),
                "mix".into(),
                "test".into(),
                "--seed".into(),
                seed.to_string(),
                "--max-cases".into(),
                "1".into(),
                "--formatter".into(),
                "ExUnit.CLIFormatter".into(),
                "--formatter".into(),
                "QualityExUnitFormatter".into(),
            ];
            let tests = runner::run(&command, &work.project, &env, timeout, &work.logs, "tests")?;
            raw["test_command_success"] = json!(runner::success(&tests));
            evidence["tests"] = tests;
            if result_file.exists() {
                match data::read(&result_file).and_then(validate_tests) {
                    Ok(t) => raw["tests"] = t,
                    Err(e) => {
                        raw["test_command_success"] = json!(false);
                        evidence["test_adapter_error"] = json!(e.to_string());
                    }
                }
            } else {
                raw["test_command_success"] = json!(false);
                evidence["test_adapter_error"] = json!("Missing ExUnit completion event");
            }
        }
        let after = workspace::inventory(&work.project)?;
        let input_after = workspace::input_inventory(&after, &generated);
        let changed: Vec<_> = inputs
            .keys()
            .chain(input_after.keys())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|p| inputs.get(*p) != input_after.get(*p))
            .cloned()
            .collect();
        raw["source_unchanged"] = json!(changed.is_empty());
        evidence["source_changes"] = json!(changed);
        ensure!(
            workspace::inventory(&original)? == before,
            "Original project changed during evaluation"
        );
        work.complete(
            raw["source_unchanged"] == true
                && raw["build_success"] == true
                && evidence["tests"]["status"] == "finished",
        )?;
        data::schema("metrics", &raw)?;
        let hashes = workspace::hashes(&before);
        Ok(
            json!({"raw_metrics":raw,"evidence":evidence,"functions":functions,"advisory":advisory,"source":{"sha256":data::identity(&hashes),"files":hashes},"adapter":descriptor,"capabilities":self.capabilities(),"execution":{"workspace":work.project,"cache":work.info,"environment_sha256":workspace::environment_hash(&env),"elapsed_ms":started.elapsed().as_secs_f64()*1000.0}}),
        )
    }
}
fn strings(s: &[&str]) -> Vec<String> {
    s.iter().map(|s| s.to_string()).collect()
}
fn parse_ast(text: &str, files: &[String], inventory: &workspace::Inventory) -> Result<Vec<Value>> {
    let rows = data::decode(text.as_bytes())?
        .as_array()
        .context("AST must be array")?
        .clone();
    ensure!(rows.len() == files.len(), "AST file count mismatch");
    for (row, path) in rows.iter().zip(files) {
        ensure!(
            row["source_sha256"] == inventory[path].sha256,
            "Source changed during direct AST analysis: {path}"
        );
        ensure!(
            row["file"] == *path && row["success"].is_boolean(),
            "Invalid AST file response"
        );
        for field in ["functions", "advisory"] {
            ensure!(row[field].is_array(), "Missing AST collection");
        }
        for f in row["functions"].as_array().unwrap() {
            ensure!(
                f["file"] == *path && f["name"].is_string() && f["kind"].is_string(),
                "Invalid function evidence"
            );
            for field in ["arity", "line", "lines", "decision_indicator"] {
                ensure!(f[field].as_u64().is_some(), "Invalid AST count {field}");
            }
        }
        for f in row["advisory"].as_array().unwrap() {
            ensure!(
                f["file"] == *path
                    && f["line"].as_u64().is_some()
                    && f["rule"] == "LIST_APPEND_REVIEW",
                "Invalid advisory evidence"
            );
        }
    }
    Ok(rows)
}
fn validate_tests(t: Value) -> Result<Value> {
    let m = t.as_object().context("Test counters must be object")?;
    ensure!(m.len() == 5, "Unknown test counters");
    let mut sum = 0u64;
    for k in ["passed", "failed", "skipped", "excluded"] {
        sum = sum
            .checked_add(t[k].as_u64().context("Invalid test counter")?)
            .context("Test count overflow")?;
    }
    ensure!(t["total"].as_u64() == Some(sum), "Inconsistent test total");
    Ok(t)
}
