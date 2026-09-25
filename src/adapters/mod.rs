//! Adapter boundary: language tooling lives here, never in the scoring engine.
pub mod elixir;
use anyhow::{Result, bail};
use serde_json::Value;
use std::path::Path;

pub trait Adapter {
    fn id(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;
    fn collect(&self, project: &Path, policy: &Value, cache: &Path) -> Result<Value>;
}
pub fn resolve(id: &str) -> Result<Box<dyn Adapter>> {
    match id {
        "elixir" => Ok(Box::new(elixir::Elixir)),
        _ => bail!("Adapter {id} is not installed"),
    }
}
