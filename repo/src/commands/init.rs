use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::spec::Node;
use crate::spec_loader::{expand_root, load_spec};

pub fn run(verbose: bool) -> Result<std::process::ExitCode> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let spec = load_spec()?;

    let mut created: Vec<PathBuf> = Vec::new();

    for area in &spec.areas {
        let root = expand_root(&area.root, &home);

        // Ensure root exists
        ensure_dir(&root, verbose, &mut created)
            .with_context(|| format!("failed ensuring root for area {}", area.name))?;

        // Ensure all required nodes exist
        ensure_tree(&root, &area.required, verbose, &mut created)?;
    }

    if created.is_empty() {
        println!("✓ life-os init: nothing to create (spec already satisfied)");
    } else {
        println!("✓ life-os init: created {} folder(s)", created.len());
        if !verbose {
            println!("Run with --verbose to see each created path.");
        }
    }

    Ok(std::process::ExitCode::from(0))
}

fn ensure_tree(
    base: &Path,
    nodes: &[Node],
    verbose: bool,
    created: &mut Vec<PathBuf>,
) -> Result<()> {
    for node in nodes {
        let path = base.join(&node.path);
        ensure_dir(&path, verbose, created)?;

        if !node.children.is_empty() {
            ensure_tree(&path, &node.children, verbose, created)?;
        }
    }
    Ok(())
}

fn ensure_dir(path: &Path, verbose: bool, created: &mut Vec<PathBuf>) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory: {}", path.display()))?;
    created.push(path.to_path_buf());
    if verbose {
        println!("created: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_dir, ensure_tree};
    use crate::spec::Node;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn ensure_tree_creates_nested_directories() {
        let dir = tempdir().expect("tempdir");
        let base = dir.path();

        let nodes = vec![Node {
            path: "a".to_string(),
            children: vec![
                Node {
                    path: "b".to_string(),
                    children: vec![],
                },
                Node {
                    path: "c".to_string(),
                    children: vec![Node {
                        path: "d".to_string(),
                        children: vec![],
                    }],
                },
            ],
        }];

        let mut created = Vec::new();
        ensure_tree(base, &nodes, false, &mut created).expect("ensure_tree");

        assert!(base.join("a").is_dir());
        assert!(base.join("a/b").is_dir());
        assert!(base.join("a/c").is_dir());
        assert!(base.join("a/c/d").is_dir());
    }

    #[test]
    fn ensure_dir_is_idempotent() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("exists");
        fs::create_dir_all(&path).expect("create dir");

        let mut created = Vec::new();
        ensure_dir(&path, false, &mut created).expect("ensure_dir");

        assert!(path.is_dir());
        assert!(created.is_empty());
    }
}
