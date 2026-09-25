use crate::{adapters, data, engine};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn catalog() -> Result<(BTreeMap<String, Value>, Vec<Value>)> {
    let knowledge = data::read(&data::root().join("knowledge/evaluator_concepts.json"))?;
    let rules = data::read(&data::root().join("rules/evaluator.json"))?;
    data::schema("knowledge", &knowledge)?;
    data::schema("rules", &rules)?;
    let manifest = data::read(&data::root().join("knowledge/sources/manifest.json"))?;
    let mut concepts = BTreeMap::new();
    for c in knowledge["concepts"].as_array().context("Concept array")? {
        let mut c = c.clone();
        for source in c["sources"].as_array_mut().context("Sources array")? {
            if let Some(book) = source["book"].as_str() {
                let record = manifest
                    .as_array()
                    .context("Manifest array")?
                    .iter()
                    .find(|r| r["id"] == book)
                    .context("Unknown book")?;
                for (field, hash) in [("file", "sha256"), ("extracted", "extracted_sha256")] {
                    ensure!(
                        data::digest(
                            &data::root().join(record[field].as_str().context("Source file")?)
                        )? == record[hash].as_str().context("Source hash")?,
                        "Knowledge integrity failure"
                    );
                }
                let units = data::read(&data::root().join(record["extracted"].as_str().unwrap()))?;
                for unit in source["units"].as_array().context("Units")? {
                    ensure!(
                        unit.as_u64()
                            .is_some_and(|u| u > 0 && u <= units.as_array().unwrap().len() as u64),
                        "Invalid source locator"
                    );
                }
                source["source_sha256"] = record["sha256"].clone();
                source["unit_kind"] = record["unit_kind"].clone();
            } else {
                source["sha256"] = json!(data::digest(
                    &data::root().join(source["file"].as_str().context("Policy source")?)
                )?);
            }
        }
        ensure!(
            concepts
                .insert(c["id"].as_str().unwrap().to_string(), c)
                .is_none(),
            "Duplicate concept"
        );
    }
    let rules = rules["rules"].as_array().unwrap().clone();
    let mut ids = BTreeSet::new();
    for r in &rules {
        ensure!(ids.insert(r["id"].as_str().unwrap()), "Duplicate rule");
        for c in r["knowledge"].as_array().unwrap() {
            ensure!(
                concepts.contains_key(c.as_str().unwrap()),
                "Unknown concept"
            );
        }
        if r["mode"] != "deterministic" {
            ensure!(
                r.get("gate").is_none() && r.get("dimension").is_none(),
                "Advisory cannot score"
            );
        }
    }
    Ok((concepts, rules))
}
pub fn diagnosis(
    measured: &Value,
    result: &Value,
    policy: &Value,
    concepts: &BTreeMap<String, Value>,
    rules: &[Value],
) -> Result<Value> {
    let unresolved: BTreeSet<_> = result["gates"]["failures"]
        .as_array()
        .unwrap()
        .iter()
        .chain(result["gates"]["unknown"].as_array().unwrap())
        .filter_map(Value::as_str)
        .collect();
    let functions = measured["functions"]
        .as_array()
        .context("Function evidence")?;
    let mut findings = vec![];
    for rule in rules {
        let mut locations = vec![];
        if rule["gate"]
            .as_str()
            .is_some_and(|g| unresolved.contains(g))
        {
            locations.push(json!({"evidence":"raw_metrics","gate":rule["gate"]}));
            if rule["id"] == "DECISION_LIMIT" {
                locations = functions
                    .iter()
                    .filter(|f| {
                        f["decision_indicator"].as_u64() > policy["gates"]["decision_max"].as_u64()
                    })
                    .cloned()
                    .collect();
            }
        }
        if let Some(d) = rule["dimension"].as_str() {
            let field = rule["metric"].as_str().context("Rule metric")?;
            let threshold = engine::decimal(&policy["normalization"][d]["good"])?;
            locations = functions
                .iter()
                .filter(|f| engine::decimal(&f[field]).is_ok_and(|n| n > threshold))
                .cloned()
                .collect();
        }
        if rule["mode"] == "advisory" {
            locations = measured["advisory"]
                .as_array()
                .context("Advisory evidence")?
                .iter()
                .filter(|f| f["rule"] == rule["id"])
                .cloned()
                .collect();
        }
        let sources: Vec<_> = rule["knowledge"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| {
                concepts[c.as_str().unwrap()]["sources"]
                    .as_array()
                    .unwrap()
                    .clone()
            })
            .collect();
        for location in locations {
            findings.push(json!({"rule":rule["id"],"mode":rule["mode"],"location":location,"instruction":rule["instruction"],"knowledge":rule["knowledge"],"sources":sources}));
        }
    }
    let ids: BTreeSet<_> = findings
        .iter()
        .flat_map(|f| f["knowledge"].as_array().unwrap())
        .filter_map(Value::as_str)
        .collect();
    let mut weakest: Vec<_> = result["quality_vector"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(d, v)| json!({"dimension":d,"score":v}))
        .collect();
    weakest.sort_by(|a, b| {
        a["score"]
            .as_f64()
            .partial_cmp(&b["score"].as_f64())
            .unwrap()
            .then_with(|| a["dimension"].as_str().cmp(&b["dimension"].as_str()))
    });
    let mut instructions = vec![];
    for f in &findings {
        if !instructions.contains(&f["instruction"]) {
            instructions.push(f["instruction"].clone());
        }
    }
    Ok(
        json!({"findings":findings,"retrieved_knowledge":ids.iter().map(|id|concepts[*id].clone()).collect::<Vec<_>>(),"weakest_measured_dimensions":weakest,"instructions":instructions}),
    )
}
pub fn evaluate(project: &Path, policy: &Value, profile: &Value, cache: &Path) -> Result<Value> {
    engine::validate(policy, profile)?;
    let adapter = adapters::resolve(profile["adapter"].as_str().context("Adapter ID")?)?;
    let (concepts, rules) = catalog()?;
    let measured = adapter.collect(project, policy, cache)?;
    let mut result = engine::evaluate(
        &measured["raw_metrics"],
        policy,
        profile,
        &adapter.capabilities(),
    )?;
    let diagnosis = diagnosis(&measured, &result, policy, &concepts, &rules)?;
    for (k, v) in measured.as_object().unwrap() {
        result[k] = v.clone();
    }
    let mut files = BTreeMap::new();
    for directory in ["src", "analyzers"] {
        implementation_files(&data::root().join(directory), &mut files)?;
    }
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "profiles/elixir-mvp.json",
        "rules/evaluator.json",
        "knowledge/evaluator_concepts.json",
        "schemas/evaluator.json",
    ] {
        files.insert(path.into(), data::digest(&data::root().join(path))?);
    }
    result["schema_version"] = json!("1.0");
    result["profile"] = policy["profile"].clone();
    result["policy"] = policy.clone();
    result["evaluation_profile"] = profile.clone();
    result["implementation"] = json!(files);
    result["diagnosis"] = diagnosis;
    result["limitations"] = json!([
        "Acceptance applies only to the configured indicator profile and executed tests.",
        "Coverage, mutation, property-test classification, algorithmic/runtime/memory efficiency and concurrency scalability are not measured.",
        "AST indicators describe source clauses, not expanded macros, control-flow proofs or human readability.",
        "Fixed seed and serial test execution do not make arbitrary tests or BEAM scheduling deterministic.",
        "Reusable workspaces isolate files, not external databases or effects. Tests run on every evaluation."
    ]);
    data::schema("report", &result)?;
    Ok(result)
}
fn implementation_files(dir: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let p = entry?.path();
        if p.is_dir() {
            if p.file_name().unwrap() != "__pycache__" {
                implementation_files(&p, out)?;
            }
        } else if p.extension().is_some_and(|x| x == "rs" || x == "exs") {
            out.insert(
                p.strip_prefix(data::root())?.to_string_lossy().into(),
                data::digest(&p)?,
            );
        }
    }
    Ok(())
}
pub fn compare(
    candidate: &mut Value,
    baseline: Value,
    policy: &Value,
    profile: &Value,
) -> Result<()> {
    let mut comparison = engine::compare(&baseline, candidate, policy, profile)?;
    let contracts = |r: &Value| -> BTreeMap<String, Value> {
        r["source"]["files"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(p, _)| p.starts_with("test/"))
            .map(|(p, h)| (p.clone(), h.clone()))
            .collect()
    };
    for (changed, reason) in [
        (
            contracts(&baseline) != contracts(candidate),
            "test_contract_changed",
        ),
        (
            baseline["evidence"]["runtime"]["stdout"] != candidate["evidence"]["runtime"]["stdout"],
            "runtime_changed",
        ),
    ] {
        if changed {
            comparison["improved"] = json!(false);
            comparison["reasons"]
                .as_array_mut()
                .unwrap()
                .push(json!(reason));
        }
    }
    candidate["baseline_comparison"] = comparison;
    candidate["baseline_report"] = baseline;
    Ok(())
}
