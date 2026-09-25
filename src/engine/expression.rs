//! Typed interpreter for the profile's small expression language.
use super::{calc, decimal};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;

pub(super) struct ContextValues<'a> {
    pub raw: &'a Value,
    pub policy: &'a Value,
}

#[derive(Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub(super) enum Expression {
    Metric { metric: String },
    Policy { policy: String },
    Constant { constant: Value },
    Sum { sum: Vec<Expression> },
}

impl Expression {
    pub fn metric_name(&self) -> Option<&str> {
        match self {
            Self::Metric { metric } => Some(metric.trim_start_matches('/')),
            _ => None,
        }
    }

    pub fn evaluate(&self, context: &ContextValues<'_>) -> Result<Value> {
        match self {
            Self::Metric { metric } => {
                Ok(context.raw.pointer(metric).cloned().unwrap_or(Value::Null))
            }
            Self::Policy { policy } => context
                .policy
                .pointer(policy)
                .cloned()
                .context("Missing policy reference"),
            Self::Constant { constant } => Ok(constant.clone()),
            Self::Sum { sum } => {
                let mut total = Decimal::ZERO;
                for expression in sum {
                    let value = expression.evaluate(context)?;
                    if value.is_null() {
                        return Ok(Value::Null);
                    }
                    total = calc(total.checked_add(decimal(&value)?))?;
                }
                Ok(serde_json::from_str(&total.to_string())?)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Predicate {
    #[serde(default)]
    when_present: Option<String>,
    #[serde(flatten)]
    condition: Condition,
}

#[derive(Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum Condition {
    Or {
        or: Vec<Predicate>,
    },
    Truth {
        truth: Expression,
    },
    Compare {
        left: Expression,
        right: Expression,
        op: Comparison,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Comparison {
    Eq,
    Gt,
    Gte,
    Lte,
}

impl Comparison {
    fn evaluate(&self, left: Decimal, right: Decimal) -> bool {
        match self {
            Self::Eq => left == right,
            Self::Gt => left > right,
            Self::Gte => left >= right,
            Self::Lte => left <= right,
        }
    }
}

impl Predicate {
    pub fn evaluate(&self, context: &ContextValues<'_>) -> Result<Option<bool>> {
        if self
            .when_present
            .as_ref()
            .is_some_and(|path| context.raw.pointer(path).is_none_or(Value::is_null))
        {
            return Ok(None);
        }
        match &self.condition {
            Condition::Or { or } => evaluate_or(or, context),
            Condition::Truth { truth } => {
                let value = truth.evaluate(context)?;
                match value {
                    Value::Null => Ok(None),
                    Value::Bool(value) => Ok(Some(value)),
                    _ => anyhow::bail!("Expected boolean metric"),
                }
            }
            Condition::Compare { left, right, op } => {
                let left = left.evaluate(context)?;
                let right = right.evaluate(context)?;
                if left.is_null() || right.is_null() {
                    return Ok(None);
                }
                Ok(Some(op.evaluate(decimal(&left)?, decimal(&right)?)))
            }
        }
    }
}

fn evaluate_or(predicates: &[Predicate], context: &ContextValues<'_>) -> Result<Option<bool>> {
    let mut result = Some(false);
    for predicate in predicates {
        match predicate.evaluate(context)? {
            Some(true) => return Ok(Some(true)),
            None => result = None,
            Some(false) => (),
        }
    }
    Ok(result)
}
