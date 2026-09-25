//! Decode and validate configuration once at the JSON boundary.
use super::{
    calc, decimal,
    expression::{Expression, Predicate},
};
use anyhow::{Context, Result, ensure};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, de};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn deserialize_decimal<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Decimal, D::Error> {
    decimal(&Value::deserialize(deserializer)?).map_err(de::Error::custom)
}

fn deserialize_weights<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, Decimal>, D::Error> {
    BTreeMap::<String, Value>::deserialize(deserializer)?
        .into_iter()
        .map(|(name, value)| {
            decimal(&value)
                .map(|weight| (name, weight))
                .map_err(de::Error::custom)
        })
        .collect()
}

#[derive(Deserialize)]
pub(super) struct Policy {
    schema_version: String,
    profile: String,
    #[serde(deserialize_with = "deserialize_weights")]
    pub weights: BTreeMap<String, Decimal>,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub target_score: Decimal,
    pub normalization: BTreeMap<String, Normalization>,
    pub comparison: ComparisonPolicy,
}

#[derive(Deserialize)]
pub(super) struct Normalization {
    pub metric: Option<String>,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub good: Decimal,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub bad: Decimal,
}

#[derive(Deserialize)]
pub(super) struct ComparisonPolicy {
    #[serde(deserialize_with = "deserialize_decimal")]
    pub minimum_delta: Decimal,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub maximum_dimension_drop: Decimal,
}

#[derive(Deserialize)]
pub(super) struct Profile {
    schema_version: String,
    id: String,
    pub dimensions: BTreeMap<String, Dimension>,
    pub dimension_order: Vec<String>,
    pub protected_dimensions: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub gates: Vec<Gate>,
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub(super) enum Dimension {
    Unsupported,
    Cost {
        value: Expression,
        when: Predicate,
    },
    Ratio {
        numerator: Expression,
        denominator: Expression,
        available: Expression,
    },
}

#[derive(Deserialize)]
pub(super) struct Gate {
    pub id: String,
    pub check: Predicate,
}

pub(super) struct Configuration {
    pub policy: Policy,
    pub profile: Profile,
}

impl Configuration {
    pub fn parse(policy: &Value, profile: &Value) -> Result<Self> {
        let configuration = Self {
            policy: serde_json::from_value(policy.clone()).context("Invalid scoring policy")?,
            profile: serde_json::from_value(profile.clone()).context("Invalid scoring profile")?,
        };
        configuration.validate()?;
        Ok(configuration)
    }

    fn validate(&self) -> Result<()> {
        let Self { policy, profile } = self;
        ensure!(
            policy.schema_version == "1.0" && profile.schema_version == "1.0",
            "Unsupported policy/profile version"
        );
        ensure!(policy.profile == profile.id, "Policy/profile mismatch");
        profile.validate_dimensions(&policy.weights)?;
        policy.validate_numbers()?;
        for (name, dimension) in &profile.dimensions {
            if let Dimension::Cost { value, .. } = dimension {
                let normalization = policy
                    .normalization
                    .get(name)
                    .context("Missing normalization")?;
                ensure!(
                    value.metric_name() == normalization.metric.as_deref(),
                    "Normalization metric mismatch"
                );
            }
        }
        profile.validate_gates()
    }
}

impl Policy {
    fn validate_numbers(&self) -> Result<()> {
        let mut total = Decimal::ZERO;
        for &weight in self.weights.values() {
            bounded(weight, Decimal::ZERO, Decimal::ONE)?;
            total = calc(total.checked_add(weight))?;
        }
        ensure!(total == Decimal::ONE, "Weights must sum exactly to 1");
        bounded(self.target_score, Decimal::ZERO, Decimal::TEN)?;
        for normalization in self.normalization.values() {
            let upper_bound = Decimal::from(1_000_000_000u64);
            bounded(normalization.good, Decimal::ZERO, upper_bound)?;
            bounded(normalization.bad, Decimal::ZERO, upper_bound)?;
            ensure!(
                normalization.bad > normalization.good,
                "bad must exceed good"
            );
        }
        bounded(self.comparison.minimum_delta, Decimal::ZERO, Decimal::TEN)?;
        bounded(
            self.comparison.maximum_dimension_drop,
            Decimal::ZERO,
            Decimal::TEN,
        )
    }
}

impl Profile {
    fn validate_dimensions(&self, weights: &BTreeMap<String, Decimal>) -> Result<()> {
        ensure!(
            weights.keys().eq(self.dimensions.keys()),
            "Weights must name exactly the profile dimensions"
        );
        let names: BTreeSet<_> = self.dimension_order.iter().collect();
        ensure!(
            self.dimension_order.len() == self.dimensions.len()
                && names.into_iter().eq(self.dimensions.keys()),
            "Dimension order must name each dimension once"
        );
        ensure!(
            self.protected_dimensions
                .iter()
                .all(|name| self.dimensions.contains_key(name)),
            "Unknown protected dimension"
        );
        Ok(())
    }

    fn validate_gates(&self) -> Result<()> {
        let mut ids = BTreeSet::new();
        for gate in &self.gates {
            ensure!(
                !matches!(gate.id.as_str(), "required_metrics" | "target_score")
                    && !gate.id.starts_with("capability:"),
                "Reserved gate ID"
            );
            ensure!(ids.insert(&gate.id), "Duplicate gate {}", gate.id);
        }
        Ok(())
    }
}

fn bounded(value: Decimal, minimum: Decimal, maximum: Decimal) -> Result<()> {
    ensure!(
        value >= minimum && value <= maximum,
        "Policy number outside range"
    );
    ensure!(
        value.normalize().scale() <= 6,
        "Policy supports at most six decimal places"
    );
    Ok(())
}
