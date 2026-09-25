//! Resolve output paths without requiring their final components to exist.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn resolve_destination(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut ancestor = absolute.as_path();
    let mut suffix = vec![];
    while !ancestor.exists() {
        suffix.push(ancestor.file_name().context("Invalid path")?.to_os_string());
        ancestor = ancestor.parent().context("Invalid path")?;
    }
    let mut canonical = ancestor.canonicalize()?;
    for p in suffix.into_iter().rev() {
        canonical.push(p);
    }
    Ok(canonical)
}
