use super::{
    calc,
    config::{Configuration, Dimension, Normalization},
    decimal,
    expression::ContextValues,
    outcome::{Evaluation, EvaluationStatus, GateResult, Gates, Score},
};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::collections::BTreeMap;

type QualityVector = BTreeMap<String, Option<Score>>;

impl Dimension {
    fn evaluate(
        &self,
        name: &str,
        configuration: &Configuration,
        context: &ContextValues<'_>,
    ) -> Result<Option<Score>> {
        match self {
            Self::Unsupported => Ok(None),
            Self::Cost { value, when } => {
                let value = value.evaluate(context)?;
                if when.evaluate(context)? != Some(true) || value.is_null() {
                    return Ok(None);
                }
                let normalization = configuration
                    .policy
                    .normalization
                    .get(name)
                    .context("Missing normalization")?;
                Ok(Some(Score(normalization.normalize(decimal(&value)?)?)))
            }
            Self::Ratio {
                numerator,
                denominator,
                available,
            } => {
                let numerator = numerator.evaluate(context)?;
                let denominator = denominator.evaluate(context)?;
                let available = available.evaluate(context)?;
                if numerator.is_null() || denominator.is_null() || available.is_null() {
                    return Ok(None);
                }
                ratio(decimal(&numerator)?, decimal(&denominator)?)
            }
        }
    }
}

impl Normalization {
    fn normalize(&self, value: Decimal) -> Result<Decimal> {
        let remaining = calc(self.bad.checked_sub(value))?;
        let interval = calc(self.bad.checked_sub(self.good))?;
        let scaled = calc(Decimal::TEN.checked_mul(remaining))?;
        Ok(calc(scaled.checked_div(interval))?.clamp(Decimal::ZERO, Decimal::TEN))
    }
}

fn ratio(numerator: Decimal, denominator: Decimal) -> Result<Option<Score>> {
    if denominator == Decimal::ZERO {
        return Ok(None);
    }
    anyhow::ensure!(
        numerator >= Decimal::ZERO && numerator <= denominator,
        "Invalid ratio counters"
    );
    let scaled = calc(Decimal::TEN.checked_mul(numerator))?;
    Ok(Some(Score(calc(scaled.checked_div(denominator))?)))
}

pub(super) fn evaluate(
    configuration: &Configuration,
    context: &ContextValues<'_>,
    capabilities: &[String],
) -> Result<Evaluation> {
    let vector = quality_vector(configuration, context)?;
    let score = weighted_score(&vector, configuration)?;
    let gates = evaluate_gates(configuration, context, capabilities, score)?;
    let status = gates.status();
    let missing_dimensions = configuration
        .profile
        .dimension_order
        .iter()
        .filter(|name| vector.get(*name).is_some_and(Option::is_none))
        .cloned()
        .collect();
    Ok(Evaluation {
        status,
        accepted: status == EvaluationStatus::Accepted,
        score,
        target_score: context.policy["target_score"].clone(),
        quality_vector: vector,
        missing_dimensions,
        gates,
    })
}

fn quality_vector(
    configuration: &Configuration,
    context: &ContextValues<'_>,
) -> Result<QualityVector> {
    configuration
        .profile
        .dimensions
        .iter()
        .map(|(name, dimension)| {
            dimension
                .evaluate(name, configuration, context)
                .map(|score| (name.clone(), score))
        })
        .collect()
}

fn weighted_score(vector: &QualityVector, configuration: &Configuration) -> Result<Option<Score>> {
    let mut total = Decimal::ZERO;
    for (name, &weight) in &configuration.policy.weights {
        if weight == Decimal::ZERO {
            continue;
        }
        let Some(score) = vector.get(name).context("Missing dimension")? else {
            return Ok(None);
        };
        let contribution = calc(score.0.checked_mul(weight))?;
        total = calc(total.checked_add(contribution))?;
    }
    Ok(Some(Score(total)))
}

fn evaluate_gates(
    configuration: &Configuration,
    context: &ContextValues<'_>,
    capabilities: &[String],
    score: Option<Score>,
) -> Result<Gates> {
    let mut results = configuration
        .profile
        .gates
        .iter()
        .map(|gate| {
            gate.check
                .evaluate(context)
                .map(|outcome| GateResult::new(&gate.id, outcome))
        })
        .collect::<Result<Vec<_>>>()?;
    results.push(GateResult::new("required_metrics", Some(score.is_some())));
    results.push(GateResult::new(
        "target_score",
        score.map(|score| score.0 >= configuration.policy.target_score),
    ));
    results.extend(
        configuration
            .profile
            .required_capabilities
            .iter()
            .filter(|required| !capabilities.contains(required))
            .map(|required| GateResult::new(format!("capability:{required}"), None)),
    );
    Ok(Gates::new(results))
}
