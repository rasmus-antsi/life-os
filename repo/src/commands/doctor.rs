use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn run(verbose: bool) -> Result<std::process::ExitCode> {
    let home = dirs::home_dir().context("could not determine home directory")?;

    let workspace = home.join("Workspace");
    let system = home.join("System");
    let documents = home.join("Documents");

    // Roots to check
    let required_roots: [(&str, &Path); 3] = [
        ("Workspace", workspace.as_path()),
        ("System", system.as_path()),
        ("Documents", documents.as_path()),
    ];

    // System subfolders to check
    let required_system_subfolders: [&str; 10] = [
        "apps",
        "backups",
        "bootstrap",
        "configs",
        "dotfiles",
        "life-os",
        "logs",
        "scripts",
        "secrets",
        "temp",
    ];

    if verbose {
        println!("Resolved paths:");
        for (name, path) in &required_roots {
            println!("  - {name}: {}", path.display());
        }
        println!();
    }

    let mut missing: Vec<String> = Vec::new();

    // Check roots
    for (name, path) in &required_roots {
        if !path.exists() {
            missing.push(format!("Missing root: {name} ({})", path.display()));
        }
    }

    // If System exists, check its subfolders
    if system.exists() {
        for folder in required_system_subfolders {
            let p = system.join(folder);
            if !p.exists() {
                missing.push(format!(
                    "Missing System subfolder: {} ({})",
                    folder,
                    p.display()
                ));
            }
        }
    } else {
        // System missing already reported above; avoid noisy subfolder spam
    }

    if missing.is_empty() {
        println!("✓ life-os doctor: OK (roots and System subfolders exist)");
        Ok(std::process::ExitCode::from(0))
    } else {
        println!("✗ life-os doctor: problems found");
        for msg in &missing {
            println!("  - {msg}");
        }
        println!();

        println!("Fix suggestions:");
        println!("  Create roots if missing:");
        println!("    mkdir -p ~/Workspace ~/System ~/Documents");
        println!("  Create missing System subfolders:");
        print!("    mkdir -p ~/System");
        for f in required_system_subfolders {
            print!(" ~/System/{f}");
        }
        println!();

        Ok(std::process::ExitCode::from(1))
    }
}
