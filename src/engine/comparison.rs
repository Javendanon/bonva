use super::{calc, config::Configuration, outcome::Score};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Snapshot {
    score: Option<Score>,
    quality_vector: BTreeMap<String, Option<Score>>,
    accepted: bool,
}

#[derive(Serialize)]
struct DimensionChange {
    before: Option<Score>,
    after: Option<Score>,
    delta: Option<Score>,
}

impl DimensionChange {
    fn new(before: Option<Score>, after: Option<Score>) -> Result<Self> {
        Ok(Self {
            before,
            after,
            delta: difference(before, after)?,
        })
    }

    fn regression_reasons(&self, name: &str, maximum_drop: Decimal) -> Vec<String> {
        let mut reasons = Vec::new();
        if self.before.is_some() && self.after.is_none() {
            reasons.push(format!("lost_measurement:{name}"));
        }
        if self.delta.is_some_and(|delta| delta.0 < -maximum_drop) {
            reasons.push(format!("dimension_regression:{name}"));
        }
        reasons
    }
}

#[derive(Serialize)]
pub(super) struct Comparison {
    baseline_score: Option<Score>,
    candidate_score: Option<Score>,
    delta: Option<Score>,
    dimensions: BTreeMap<String, DimensionChange>,
    improved: bool,
    reasons: Vec<String>,
}

pub(super) fn compare(
    before: &Value,
    after: &Value,
    configuration: &Configuration,
) -> Result<Comparison> {
    let before: Snapshot =
        serde_json::from_value(before.clone()).context("Invalid baseline score report")?;
    let after: Snapshot =
        serde_json::from_value(after.clone()).context("Invalid candidate score report")?;
    let mut dimensions = BTreeMap::new();
    let mut reasons = Vec::new();
    for name in &configuration.profile.dimension_order {
        let change = DimensionChange::new(
            before.quality_vector.get(name).copied().flatten(),
            after.quality_vector.get(name).copied().flatten(),
        )?;
        reasons.extend(
            change.regression_reasons(name, configuration.policy.comparison.maximum_dimension_drop),
        );
        dimensions.insert(name.clone(), change);
    }
    for name in &configuration.profile.protected_dimensions {
        let change = dimensions
            .get(name)
            .context("Missing protected dimension")?;
        if change.delta.is_some_and(|delta| delta.0 < Decimal::ZERO) {
            reasons.push(format!("{name}_regression"));
        }
    }
    let delta = difference(before.score, after.score)?;
    if delta.is_none_or(|delta| {
        delta.0 <= Decimal::ZERO || delta.0 < configuration.policy.comparison.minimum_delta
    }) {
        reasons.push("insufficient_improvement".into());
    }
    if !after.accepted {
        reasons.push("candidate_not_accepted".into());
    }
    Ok(Comparison {
        baseline_score: before.score,
        candidate_score: after.score,
        delta,
        dimensions,
        improved: reasons.is_empty(),
        reasons,
    })
}

fn difference(before: Option<Score>, after: Option<Score>) -> Result<Option<Score>> {
    match (before, after) {
        (Some(before), Some(after)) => {
            Ok(Some(Score::rounded(calc(after.0.checked_sub(before.0))?)))
        }
        _ => Ok(None),
    }
}
