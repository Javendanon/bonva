//! Parse CLI input into commands whose fields describe their valid states.
use crate::data;
use anyhow::{Context, Result, bail, ensure};
use std::path::PathBuf;

pub(super) enum Command {
    Help,
    Adapters,
    Evaluate {
        project: PathBuf,
        scoring: ScoringFiles,
        cache: PathBuf,
        baseline: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    Score {
        metrics: PathBuf,
        scoring: ScoringFiles,
        output: Option<PathBuf>,
    },
    AnalyzeLists {
        project: PathBuf,
        cache: PathBuf,
        output: Option<PathBuf>,
    },
}

pub(super) struct ScoringFiles {
    pub policy: PathBuf,
    pub profile: PathBuf,
}

impl Default for ScoringFiles {
    fn default() -> Self {
        Self {
            policy: data::root().join("scoring/policy.json"),
            profile: data::root().join("profiles/elixir-mvp.json"),
        }
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Evaluate,
    Score,
    AnalyzeLists,
}

struct Options {
    scoring: ScoringFiles,
    cache: PathBuf,
    baseline: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl Options {
    fn parse(args: impl Iterator<Item = String>, mode: Mode) -> Result<Self> {
        let mut options = Self {
            scoring: ScoringFiles::default(),
            cache: data::root().join(".quality-cache"),
            baseline: None,
            output: None,
        };
        let mut args = args;
        while let Some(flag) = args.next() {
            ensure!(
                !matches!(mode, Mode::AnalyzeLists)
                    || matches!(flag.as_str(), "--cache-dir" | "--output"),
                "Unsupported list-analysis option {flag}"
            );
            let value = PathBuf::from(args.next().context("Flag requires a value")?);
            match flag.as_str() {
                "--policy" => options.scoring.policy = value,
                "--profile" => options.scoring.profile = value,
                "--cache-dir" => options.cache = value,
                "--output" => options.output = Some(value),
                "--baseline" => options.baseline = Some(value),
                _ => bail!("Unknown option {flag}"),
            }
        }
        ensure!(
            !matches!(mode, Mode::Score) || options.baseline.is_none(),
            "score does not accept --baseline"
        );
        Ok(options)
    }
}

impl Command {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter();
        let name = args.next().unwrap_or_else(|| "help".into());
        let mode = match name.as_str() {
            "help" | "--help" => return Ok(Self::Help),
            "adapters" => return Ok(Self::Adapters),
            "evaluate" => Mode::Evaluate,
            "score" => Mode::Score,
            "analyze-lists" => Mode::AnalyzeLists,
            _ => bail!("Unknown command {name}"),
        };
        let target = PathBuf::from(args.next().context("Project or metrics path required")?);
        let Options {
            scoring,
            cache,
            baseline,
            output,
        } = Options::parse(args, mode)?;
        Ok(match mode {
            Mode::Evaluate => Self::Evaluate {
                project: target,
                scoring,
                cache,
                baseline,
                output,
            },
            Mode::Score => Self::Score {
                metrics: target,
                scoring,
                output,
            },
            Mode::AnalyzeLists => Self::AnalyzeLists {
                project: target,
                cache,
                output,
            },
        })
    }
}
