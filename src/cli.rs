//! Command dispatch; parsing, report output and domain evaluation stay separate.
mod command;
mod output;

use crate::{data, engine, list_analysis, report};
use anyhow::Result;
use command::{Command, ScoringFiles};
use output::{Report, ReportKind, validate_destination};
use serde_json::{Value, json};

// Retained for callers of the original public API. Domain code uses paths directly.
pub use crate::paths::resolve_destination;

const HELP: &str = "agent-quality evaluate PROJECT [--baseline PROJECT] [--policy FILE] [--profile FILE] [--cache-dir DIR] [--output NEW_FILE]\nagent-quality analyze-lists PROJECT [--cache-dir DIR] [--output NEW_FILE]\nagent-quality score METRICS --policy FILE --profile FILE\nagent-quality adapters\n\nscore only replays supplied metrics; it does not attest to a project evaluation.";

impl ScoringFiles {
    fn load(&self) -> Result<(Value, Value)> {
        let policy = data::read(&self.policy)?;
        let profile = data::read(&self.profile)?;
        engine::validate(&policy, &profile)?;
        Ok((policy, profile))
    }
}

impl Command {
    fn execute(self) -> Result<u8> {
        let report = match self {
            Self::Help => {
                println!("{HELP}");
                return Ok(0);
            }
            Self::Adapters => {
                println!(
                    "{}",
                    json!({"schema_version":"1.0","adapters":[{"id":"elixir","capabilities":["build","tests","source_ast"],"requirements":["Elixir >= 1.18","Mix","project dependencies and test services"]}]})
                );
                return Ok(0);
            }
            Self::Evaluate {
                project,
                scoring,
                cache,
                baseline,
                output,
            } => {
                let (policy, profile) = scoring.load()?;
                validate_destination(
                    output.as_deref(),
                    std::iter::once(project.as_path()).chain(baseline.as_deref()),
                )?;
                let mut value = report::evaluate(&project, &policy, &profile, &cache)?;
                if let Some(baseline) = baseline {
                    let before = report::evaluate(&baseline, &policy, &profile, &cache)?;
                    report::compare(&mut value, before, &policy, &profile)?;
                }
                Report {
                    value,
                    destination: output,
                    kind: ReportKind::Evaluation,
                }
            }
            Self::Score {
                metrics,
                scoring,
                output,
            } => {
                let (policy, profile) = scoring.load()?;
                let mut value = engine::evaluate(&data::read(&metrics)?, &policy, &profile, &[])?;
                value["mode"] = json!("unattested_replay");
                Report {
                    value,
                    destination: output,
                    kind: ReportKind::Evaluation,
                }
            }
            Self::AnalyzeLists {
                project,
                cache,
                output,
            } => {
                validate_destination(output.as_deref(), std::iter::once(project.as_path()))?;
                Report {
                    value: list_analysis::analyze(&project, &cache)?,
                    destination: output,
                    kind: ReportKind::ListAnalysis,
                }
            }
        };
        report.publish()
    }
}

/// Return an exit code instead of terminating the process, so callers retain cleanup control.
pub fn run(args: impl IntoIterator<Item = String>) -> Result<u8> {
    Command::parse(args)?.execute()
}

pub fn main() -> Result<u8> {
    run(std::env::args().skip(1))
}
