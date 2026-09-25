//! Domain outcomes and their stable JSON representation.
use super::config::deserialize_decimal;
use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};
use serde::{Deserialize, Serialize, Serializer, ser::Error};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Deserialize)]
pub(super) struct Score(#[serde(deserialize_with = "deserialize_decimal")] pub Decimal);

impl Score {
    pub fn rounded(value: Decimal) -> Self {
        Self(value.round_dp_with_strategy(6, RoundingStrategy::MidpointAwayFromZero))
    }
}

impl Serialize for Score {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = Self::rounded(self.0)
            .0
            .to_f64()
            .ok_or_else(|| S::Error::custom("Score outside JSON number range"))?;
        serializer.serialize_f64(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum GateStatus {
    Passed,
    Failed,
    Unknown,
}

impl From<Option<bool>> for GateStatus {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Passed,
            Some(false) => Self::Failed,
            None => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum EvaluationStatus {
    Accepted,
    Rejected,
    Incomplete,
}

#[derive(Serialize)]
pub(super) struct GateResult {
    pub id: String,
    pub status: GateStatus,
}

impl GateResult {
    pub fn new(id: impl Into<String>, outcome: Option<bool>) -> Self {
        Self {
            id: id.into(),
            status: outcome.into(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct Gates {
    pub passed: bool,
    pub failures: Vec<String>,
    pub unknown: Vec<String>,
    pub results: Vec<GateResult>,
}

impl Gates {
    pub fn new(results: Vec<GateResult>) -> Self {
        let with_status = |status| {
            results
                .iter()
                .filter(|gate| gate.status == status)
                .map(|gate| gate.id.clone())
                .collect::<Vec<_>>()
        };
        let failures = with_status(GateStatus::Failed);
        let unknown = with_status(GateStatus::Unknown);
        Self {
            passed: failures.is_empty() && unknown.is_empty(),
            failures,
            unknown,
            results,
        }
    }

    pub fn status(&self) -> EvaluationStatus {
        match (self.failures.is_empty(), self.unknown.is_empty()) {
            (false, _) => EvaluationStatus::Rejected,
            (true, false) => EvaluationStatus::Incomplete,
            (true, true) => EvaluationStatus::Accepted,
        }
    }
}

#[derive(Serialize)]
pub(super) struct Evaluation {
    pub status: EvaluationStatus,
    pub accepted: bool,
    pub score: Option<Score>,
    pub target_score: Value,
    pub quality_vector: BTreeMap<String, Option<Score>>,
    pub missing_dimensions: Vec<String>,
    pub gates: Gates,
}
