use crate::paths::resolve_destination;
use anyhow::{Result, ensure};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub(super) enum ReportKind {
    Evaluation,
    ListAnalysis,
}

pub(super) struct Report {
    pub value: Value,
    pub destination: Option<PathBuf>,
    pub kind: ReportKind,
}

impl ReportKind {
    fn exit_code(&self, result: &Value) -> u8 {
        match self {
            Self::ListAnalysis => match result["status"].as_str() {
                Some("completed") => 0,
                Some("completed_with_findings") => 1,
                _ => 2,
            },
            Self::Evaluation => match result["status"].as_str() {
                Some("incomplete") => 2,
                _ if result["accepted"] == true
                    && result["baseline_comparison"]["improved"] != false =>
                {
                    0
                }
                _ => 1,
            },
        }
    }

    fn print_summary(&self, result: &Value) {
        match self {
            Self::ListAnalysis => eprintln!("list_analysis: {}", result["status"]),
            Self::Evaluation => eprintln!(
                "{}: score={}, target={}, failed_gates={}",
                result["status"],
                result["score"],
                result["target_score"],
                result["gates"]["failures"]
            ),
        }
    }
}

impl Report {
    pub fn publish(self) -> Result<u8> {
        let bytes = serde_json::to_vec_pretty(&self.value)?;
        match self.destination {
            Some(path) => {
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    fs::create_dir_all(parent)?;
                }
                let mut output = OpenOptions::new().create_new(true).write(true).open(path)?;
                output.write_all(&bytes)?;
                output.write_all(b"\n")?;
            }
            None => println!("{}", String::from_utf8(bytes)?),
        }
        self.kind.print_summary(&self.value);
        Ok(self.kind.exit_code(&self.value))
    }
}

pub(super) fn validate_destination<'a>(
    output: Option<&Path>,
    projects: impl Iterator<Item = &'a Path>,
) -> Result<()> {
    let Some(output) = output else {
        return Ok(());
    };
    let destination = resolve_destination(output)?;
    for project in projects {
        ensure!(
            !destination.starts_with(project.canonicalize()?),
            "Reports must be outside evaluated projects"
        );
    }
    ensure!(!destination.exists(), "Report already exists");
    Ok(())
}
