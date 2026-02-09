use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::check::check_tree;
use crate::spec_loader::{expand_root, load_spec};

pub fn run(verbose: bool) -> Result<std::process::ExitCode> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let spec = load_spec()?;

    let mut missing: Vec<PathBuf> = Vec::new();

    for area in &spec.areas {
        let root = expand_root(&area.root, &home);

        if verbose {
            println!("Checking {} at {}", area.name, root.display());
        }

        if !root.exists() {
            missing.push(root.clone());
            continue;
        }

        check_tree(&root, &area.required, &mut missing);
    }

    if missing.is_empty() {
        println!("✓ life-os doctor: OK (spec satisfied)");
        Ok(std::process::ExitCode::from(0))
    } else {
        println!("✗ life-os doctor: missing folders");
        for p in &missing {
            println!("  - {}", p.display());
        }
        Ok(std::process::ExitCode::from(1))
    }
}
