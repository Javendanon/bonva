use crate::{data, engine, list_analysis, report};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    if command == "help" || command == "--help" {
        println!(
            "agent-quality evaluate PROJECT [--baseline PROJECT] [--policy FILE] [--profile FILE] [--cache-dir DIR] [--output NEW_FILE]\nagent-quality analyze-lists PROJECT [--cache-dir DIR] [--output NEW_FILE]\nagent-quality score METRICS --policy FILE --profile FILE\nagent-quality adapters\n\nscore only replays supplied metrics; it does not attest to a project evaluation."
        );
        return Ok(());
    }
    if command == "adapters" {
        println!(
            "{}",
            json!({"schema_version":"1.0","adapters":[{"id":"elixir","capabilities":["build","tests","source_ast"],"requirements":["Elixir >= 1.18","Mix","project dependencies and test services"]}]})
        );
        return Ok(());
    }
    ensure!(
        command == "evaluate" || command == "score" || command == "analyze-lists",
        "Unknown command {command}"
    );
    let target = PathBuf::from(args.next().context("Project or metrics path required")?);
    let mut policy = data::root().join("scoring/policy.json");
    let mut profile = data::root().join("profiles/elixir-mvp.json");
    let mut cache = data::root().join(".quality-cache");
    let mut output = None;
    let mut baseline = None;
    while let Some(flag) = args.next() {
        ensure!(
            command != "analyze-lists" || ["--cache-dir", "--output"].contains(&flag.as_str()),
            "Unsupported list-analysis option {flag}"
        );
        let value = PathBuf::from(args.next().context("Flag requires a value")?);
        match flag.as_str() {
            "--policy" => policy = value,
            "--profile" => profile = value,
            "--cache-dir" => cache = value,
            "--output" => output = Some(value),
            "--baseline" => baseline = Some(value),
            _ => bail!("Unknown option {flag}"),
        }
    }
    let policy = if command == "analyze-lists" {
        Value::Null
    } else {
        data::read(&policy)?
    };
    let profile = if command == "analyze-lists" {
        Value::Null
    } else {
        data::read(&profile)?
    };
    if command != "analyze-lists" {
        engine::validate(&policy, &profile)?;
    }
    let mut result: Value;
    if command == "score" {
        ensure!(baseline.is_none(), "score does not accept --baseline");
        result = engine::evaluate(&data::read(&target)?, &policy, &profile, &[])?;
        result["mode"] = json!("unattested_replay");
    } else {
        if let Some(ref output) = output {
            let dest = resolve_destination(output)?;
            for p in std::iter::once(&target).chain(baseline.iter()) {
                ensure!(
                    !dest.starts_with(p.canonicalize()?),
                    "Reports must be outside evaluated projects"
                );
            }
            ensure!(!dest.exists(), "Report already exists");
        }
        result = if command == "analyze-lists" {
            list_analysis::analyze(&target, &cache)?
        } else {
            report::evaluate(&target, &policy, &profile, &cache)?
        };
        if let Some(base) = baseline {
            let before = report::evaluate(&base, &policy, &profile, &cache)?;
            report::compare(&mut result, before, &policy, &profile)?;
        }
    }
    let bytes = serde_json::to_vec_pretty(&result)?;
    if let Some(path) = output {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        let mut f = OpenOptions::new().create_new(true).write(true).open(path)?;
        f.write_all(&bytes)?;
        f.write_all(b"\n")?;
    } else {
        println!("{}", String::from_utf8(bytes)?);
    }
    if command == "analyze-lists" {
        eprintln!("list_analysis: {}", result["status"]);
        let code = match result["status"].as_str() {
            Some("completed") => 0,
            Some("completed_with_findings") => 1,
            _ => 2,
        };
        if code != 0 {
            std::process::exit(code);
        }
        return Ok(());
    }
    eprintln!(
        "{}: score={}, target={}, failed_gates={}",
        result["status"], result["score"], result["target_score"], result["gates"]["failures"]
    );
    let code = if result["status"] == "incomplete" {
        2
    } else if result["accepted"] == true
        && result["baseline_comparison"].get("improved") != Some(&json!(false))
    {
        0
    } else {
        1
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

pub fn resolve_destination(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut ancestor = absolute.as_path();
    let mut suffix = vec![];
    while !ancestor.exists() {
        suffix.push(ancestor.file_name().context("Invalid path")?.to_os_string());
        ancestor = ancestor.parent().context("Invalid path")?;
    }
    let mut canonical = ancestor.canonicalize()?;
    for p in suffix.into_iter().rev() {
        canonical.push(p);
    }
    Ok(canonical)
}
