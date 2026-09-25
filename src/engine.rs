//! Language-independent scoring. No process execution or source-language knowledge.
use anyhow::{Context, Result, bail, ensure};
use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};
use serde_json::{Value, json};
use std::{collections::BTreeMap, str::FromStr};

pub fn decimal(v: &Value) -> Result<Decimal> {
    ensure!(v.is_number(), "Expected finite decimal number, got {v}");
    Decimal::from_str(&v.to_string())
        .or_else(|_| Decimal::from_scientific(&v.to_string()))
        .context("Decimal outside supported range")
}
fn calc(v: Option<Decimal>) -> Result<Decimal> {
    v.context("Decimal overflow or division by zero")
}
pub fn rounded(v: Decimal) -> Value {
    json!(
        v.round_dp_with_strategy(6, RoundingStrategy::MidpointAwayFromZero)
            .to_f64()
            .expect("Bounded score")
    )
}
fn bounded(v: &Value, min: Decimal, max: Decimal) -> Result<Decimal> {
    let d = decimal(v)?;
    ensure!(d >= min && d <= max, "Policy number outside range");
    ensure!(
        d.normalize().scale() <= 6,
        "Policy supports at most six decimal places"
    );
    Ok(d)
}
pub fn validate(policy: &Value, profile: &Value) -> Result<()> {
    ensure!(
        policy["schema_version"] == "1.0" && profile["schema_version"] == "1.0",
        "Unsupported policy/profile version"
    );
    ensure!(
        policy["profile"] == profile["id"],
        "Policy/profile mismatch"
    );
    let weights = policy["weights"]
        .as_object()
        .context("Weights object required")?;
    let dims = profile["dimensions"]
        .as_object()
        .context("Dimension definitions required")?;
    ensure!(
        weights.keys().eq(dims.keys()),
        "Weights must name exactly the profile dimensions"
    );
    let order = profile["dimension_order"]
        .as_array()
        .context("Dimension order required")?;
    let names = order
        .iter()
        .map(|d| d.as_str().context("Dimension name must be string"))
        .collect::<Result<std::collections::BTreeSet<_>>>()?;
    ensure!(
        order.len() == dims.len() && names.iter().copied().eq(dims.keys().map(String::as_str)),
        "Dimension order must name each dimension once"
    );
    for d in profile["protected_dimensions"]
        .as_array()
        .context("Protected dimensions required")?
    {
        ensure!(
            d.as_str().is_some_and(|d| dims.contains_key(d)),
            "Unknown protected dimension"
        );
    }
    let mut total = Decimal::ZERO;
    for w in weights.values() {
        total = calc(total.checked_add(bounded(w, Decimal::ZERO, Decimal::ONE)?))?;
    }
    ensure!(total == Decimal::ONE, "Weights must sum exactly to 1");
    bounded(&policy["target_score"], Decimal::ZERO, Decimal::TEN)?;
    for s in policy["normalization"]
        .as_object()
        .context("Normalizations required")?
        .values()
    {
        let good = bounded(&s["good"], Decimal::ZERO, Decimal::from(1_000_000_000u64))?;
        let bad = bounded(&s["bad"], Decimal::ZERO, Decimal::from(1_000_000_000u64))?;
        ensure!(bad > good, "bad must exceed good");
    }
    for key in ["minimum_delta", "maximum_dimension_drop"] {
        bounded(&policy["comparison"][key], Decimal::ZERO, Decimal::TEN)?;
    }
    let required = profile["required_capabilities"]
        .as_array()
        .context("Capabilities required")?;
    ensure!(
        required.iter().all(Value::is_string),
        "Capability must be a string"
    );
    let gates = profile["gates"].as_array().context("Gates required")?;
    let mut ids = std::collections::BTreeSet::new();
    for g in gates {
        let id = g["id"].as_str().context("Gate ID")?;
        ensure!(
            !matches!(id, "required_metrics" | "target_score") && !id.starts_with("capability:"),
            "Reserved gate ID"
        );
        ensure!(ids.insert(id), "Duplicate gate {id}");
    }
    Ok(())
}
fn value(expr: &Value, raw: &Value, policy: &Value) -> Result<Value> {
    if let Some(p) = expr["metric"].as_str() {
        return Ok(raw.pointer(p).cloned().unwrap_or(Value::Null));
    }
    if let Some(p) = expr["policy"].as_str() {
        return policy
            .pointer(p)
            .cloned()
            .context("Missing policy reference");
    }
    if let Some(c) = expr.get("constant") {
        return Ok(c.clone());
    }
    if let Some(a) = expr["sum"].as_array() {
        let mut s = Decimal::ZERO;
        for e in a {
            let v = value(e, raw, policy)?;
            if v.is_null() {
                return Ok(Value::Null);
            }
            s = calc(s.checked_add(decimal(&v)?))?;
        }
        return Ok(serde_json::from_str(&s.to_string())?);
    }
    bail!("Unknown value expression: {expr}")
}
fn predicate(expr: &Value, raw: &Value, policy: &Value) -> Result<Option<bool>> {
    if let Some(path) = expr["when_present"].as_str()
        && raw.pointer(path).is_none_or(Value::is_null)
    {
        return Ok(None);
    }
    if let Some(a) = expr["or"].as_array() {
        let mut unknown = false;
        for e in a {
            match predicate(e, raw, policy)? {
                Some(true) => return Ok(Some(true)),
                None => unknown = true,
                _ => (),
            }
        }
        return Ok(if unknown { None } else { Some(false) });
    }
    if let Some(e) = expr.get("truth") {
        let v = value(e, raw, policy)?;
        return if v.is_null() {
            Ok(None)
        } else {
            Ok(Some(v.as_bool().context("Expected boolean metric")?))
        };
    }
    let a = value(&expr["left"], raw, policy)?;
    let b = value(&expr["right"], raw, policy)?;
    if a.is_null() || b.is_null() {
        return Ok(None);
    }
    let (a, b) = (decimal(&a)?, decimal(&b)?);
    Ok(Some(match expr["op"].as_str() {
        Some("eq") => a == b,
        Some("gt") => a > b,
        Some("gte") => a >= b,
        Some("lte") => a <= b,
        _ => bail!("Unknown comparison operator"),
    }))
}
pub fn evaluate(
    raw: &Value,
    policy: &Value,
    profile: &Value,
    capabilities: &[String],
) -> Result<Value> {
    validate(policy, profile)?;
    ensure!(
        raw["schema_version"] == "1.0",
        "Unsupported metrics version"
    );
    let mut vector: BTreeMap<String, Option<Decimal>> = BTreeMap::new();
    for (name, spec) in profile["dimensions"].as_object().unwrap() {
        let v = match spec["mode"].as_str() {
            Some("unsupported") => None,
            Some("cost") => {
                let normalization = &policy["normalization"][name];
                let good = decimal(&normalization["good"])?;
                let bad = decimal(&normalization["bad"])?;
                let v = value(&spec["value"], raw, policy)?;
                ensure!(
                    spec["value"]["metric"]
                        .as_str()
                        .map(|p| p.trim_start_matches('/'))
                        == normalization["metric"].as_str(),
                    "Normalization metric mismatch"
                );
                if predicate(&spec["when"], raw, policy)? != Some(true) || v.is_null() {
                    None
                } else {
                    Some(
                        calc(
                            calc(Decimal::TEN.checked_mul(calc(bad.checked_sub(decimal(&v)?))?))?
                                .checked_div(calc(bad.checked_sub(good))?),
                        )?
                        .clamp(Decimal::ZERO, Decimal::TEN),
                    )
                }
            }
            Some("ratio") => {
                let n = value(&spec["numerator"], raw, policy)?;
                let d = value(&spec["denominator"], raw, policy)?;
                let available = value(&spec["available"], raw, policy)?;
                if n.is_null()
                    || d.is_null()
                    || available.is_null()
                    || decimal(&d)? == Decimal::ZERO
                {
                    None
                } else {
                    let n = decimal(&n)?;
                    let d = decimal(&d)?;
                    ensure!(n >= Decimal::ZERO && n <= d, "Invalid ratio counters");
                    Some(calc(calc(Decimal::TEN.checked_mul(n))?.checked_div(d))?)
                }
            }
            _ => bail!("Unsupported normalization mode"),
        };
        vector.insert(name.clone(), v);
    }
    let mut pairs = vec![];
    for gate in profile["gates"].as_array().unwrap() {
        pairs.push((
            gate["id"].as_str().unwrap().to_string(),
            predicate(&gate["check"], raw, policy)?,
        ));
    }
    let missing: Vec<_> = profile["dimension_order"]
        .as_array()
        .context("Dimension order")?
        .iter()
        .filter_map(|d| {
            let name = d.as_str()?;
            if vector.get(name) == Some(&None) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();
    let required_missing = missing
        .iter()
        .any(|d| decimal(&policy["weights"][d]).is_ok_and(|w| w > Decimal::ZERO));
    let mut score = Some(Decimal::ZERO);
    if required_missing {
        score = None;
    } else {
        for (d, v) in &vector {
            let w = decimal(&policy["weights"][d])?;
            if w > Decimal::ZERO {
                score = Some(calc(score.unwrap().checked_add(calc(
                    v.context("Missing weighted metric")?.checked_mul(w),
                )?))?);
            }
        }
    }
    pairs.push(("required_metrics".into(), Some(!required_missing)));
    pairs.push((
        "target_score".into(),
        score.map(|s| s >= decimal(&policy["target_score"]).unwrap()),
    ));
    for c in profile["required_capabilities"].as_array().unwrap() {
        let c = c.as_str().unwrap();
        if !capabilities.iter().any(|x| x == c) {
            pairs.push((format!("capability:{c}"), None));
        }
    }
    let failures: Vec<_> = pairs
        .iter()
        .filter(|(_, v)| *v == Some(false))
        .map(|(id, _)| id.clone())
        .collect();
    let unknown: Vec<_> = pairs
        .iter()
        .filter(|(_, v)| v.is_none())
        .map(|(id, _)| id.clone())
        .collect();
    let status = if !failures.is_empty() {
        "rejected"
    } else if !unknown.is_empty() {
        "incomplete"
    } else {
        "accepted"
    };
    let gates:Vec<_>=pairs.iter().map(|(id,v)|json!({"id":id,"status":match v{Some(true)=>"passed",Some(false)=>"failed",None=>"unknown"}})).collect();
    let vector: serde_json::Map<String, Value> = vector
        .into_iter()
        .map(|(d, v)| (d, v.map(rounded).unwrap_or(Value::Null)))
        .collect();
    Ok(
        json!({"status":status,"accepted":status=="accepted","score":score.map(rounded),"target_score":policy["target_score"],"quality_vector":vector,"missing_dimensions":missing,"gates":{"passed":status=="accepted","failures":failures,"unknown":unknown,"results":gates}}),
    )
}

pub fn compare(before: &Value, after: &Value, policy: &Value, profile: &Value) -> Result<Value> {
    let mut dims = serde_json::Map::new();
    let mut reasons = vec![];
    for d in profile["dimension_order"]
        .as_array()
        .context("Dimension order")?
    {
        let name = d.as_str().context("Dimension name")?;
        let a = &before["quality_vector"][name];
        let b = &after["quality_vector"][name];
        let delta = diff(a, b)?;
        if !a.is_null() && b.is_null() {
            reasons.push(format!("lost_measurement:{name}"));
        }
        if !delta.is_null()
            && decimal(&delta)? < -decimal(&policy["comparison"]["maximum_dimension_drop"])?
        {
            reasons.push(format!("dimension_regression:{name}"));
        }
        dims.insert(name.into(), json!({"before":a,"after":b,"delta":delta}));
    }
    // Protected dimensions are profile policy, not source-language assumptions.
    for d in profile["protected_dimensions"]
        .as_array()
        .context("Protected dimensions")?
    {
        let name = d.as_str().context("Dimension")?;
        let delta = &dims[name]["delta"];
        if !delta.is_null() && decimal(delta)? < Decimal::ZERO {
            reasons.push(format!("{name}_regression"));
        }
    }
    let delta = diff(&before["score"], &after["score"])?;
    if delta.is_null()
        || decimal(&delta)? <= Decimal::ZERO
        || decimal(&delta)? < decimal(&policy["comparison"]["minimum_delta"])?
    {
        reasons.push("insufficient_improvement".into());
    }
    if after["accepted"] != true {
        reasons.push("candidate_not_accepted".into());
    }
    Ok(
        json!({"baseline_score":before["score"],"candidate_score":after["score"],"delta":delta,"dimensions":dims,"improved":reasons.is_empty(),"reasons":reasons}),
    )
}
fn diff(a: &Value, b: &Value) -> Result<Value> {
    if a.is_null() || b.is_null() {
        Ok(Value::Null)
    } else {
        Ok(rounded(calc(decimal(b)?.checked_sub(decimal(a)?))?))
    }
}
