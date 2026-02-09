use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::spec::SpecFile;

pub fn load_spec() -> Result<SpecFile> {
    let path = spec_path();
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read spec file: {}", path.display()))?;

    let spec: SpecFile = serde_json::from_str(&raw).context("failed to parse spec.json")?;

    Ok(spec)
}

fn spec_path() -> PathBuf {
    let home = dirs::home_dir().expect("home directory not found");
    home.join("System/life-os/config/spec.json")
}

pub fn expand_root(root: &str, home: &Path) -> PathBuf {
    if let Some(rest) = root.strip_prefix("~/") {
        home.join(rest)
    } else {
        PathBuf::from(root)
    }
}
