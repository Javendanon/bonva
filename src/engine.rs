//! Language-independent scoring with typed configuration and outcomes.
//! JSON remains the public boundary; calculation never executes processes.
mod comparison;
mod config;
mod evaluation;
mod expression;
mod outcome;

use anyhow::{Context, Result, ensure};
use config::Configuration;
use expression::ContextValues;
use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};
use serde_json::{Value, json};
use std::str::FromStr;

pub fn decimal(value: &Value) -> Result<Decimal> {
    ensure!(
        value.is_number(),
        "Expected finite decimal number, got {value}"
    );
    let text = value.to_string();
    Decimal::from_str(&text)
        .or_else(|_| Decimal::from_scientific(&text))
        .context("Decimal outside supported range")
}

fn calc(value: Option<Decimal>) -> Result<Decimal> {
    value.context("Decimal overflow or division by zero")
}

pub fn rounded(value: Decimal) -> Value {
    json!(
        value
            .round_dp_with_strategy(6, RoundingStrategy::MidpointAwayFromZero)
            .to_f64()
            .expect("Every Decimal score fits in the finite f64 range")
    )
}

pub fn validate(policy: &Value, profile: &Value) -> Result<()> {
    Configuration::parse(policy, profile).map(|_| ())
}

pub fn evaluate(
    raw: &Value,
    policy: &Value,
    profile: &Value,
    capabilities: &[String],
) -> Result<Value> {
    let configuration = Configuration::parse(policy, profile)?;
    ensure!(
        raw["schema_version"] == "1.0",
        "Unsupported metrics version"
    );
    let context = ContextValues { raw, policy };
    Ok(serde_json::to_value(evaluation::evaluate(
        &configuration,
        &context,
        capabilities,
    )?)?)
}

pub fn compare(before: &Value, after: &Value, policy: &Value, profile: &Value) -> Result<Value> {
    let configuration = Configuration::parse(policy, profile)?;
    Ok(serde_json::to_value(comparison::compare(
        before,
        after,
        &configuration,
    )?)?)
}
