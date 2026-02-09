use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::check::check_tree;
use crate::spec::Node;
use crate::spec_loader::{expand_root, load_spec};

#[derive(Debug)]
pub struct DoctorReport {
    pub missing: Vec<PathBuf>,
    pub areas: usize,
    pub required: usize,
    pub roots: Vec<PathBuf>,
}

pub fn run(_verbose: bool) -> Result<DoctorReport> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let spec = load_spec()?;

    let mut missing: Vec<PathBuf> = Vec::new();
    let mut roots: Vec<PathBuf> = Vec::new();

    for area in &spec.areas {
        let root = expand_root(&area.root, &home);
        roots.push(root.clone());

        if !root.exists() {
            missing.push(root.clone());
            continue;
        }

        check_tree(&root, &area.required, &mut missing);
    }

    let required = spec
        .areas
        .iter()
        .map(|area| count_nodes(&area.required))
        .sum();

    Ok(DoctorReport {
        missing,
        areas: spec.areas.len(),
        required,
        roots,
    })
}

fn count_nodes(nodes: &[Node]) -> usize {
    let mut total = 0;
    for node in nodes {
        total += 1;
        if !node.children.is_empty() {
            total += count_nodes(&node.children);
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn write_spec(home: &Path, content: &str) {
        let spec_path = home.join("System/life-os/config/spec.json");
        let parent = spec_path.parent().expect("spec parent");
        fs::create_dir_all(parent).expect("create spec dir");
        fs::write(spec_path, content).expect("write spec");
    }

    fn with_temp_home<F: FnOnce(&Path)>(f: F) {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let original_home = std::env::var("HOME").ok();

        let dir = tempdir().expect("tempdir");
        unsafe {
            std::env::set_var("HOME", dir.path());
        }

        f(dir.path());

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn doctor_returns_ok_when_spec_is_satisfied() {
        with_temp_home(|home| {
            write_spec(
                home,
                r#"{
  "version": 1,
  "areas": [
    {
      "name": "System",
      "root": "~/System",
      "required": [
        { "path": "apps" },
        { "path": "backups" },
        { "path": "bootstrap" },
        { "path": "configs" },
        { "path": "dotfiles" },
        {
          "path": "life-os",
          "children": [
            { "path": "repo" },
            { "path": "bin" },
            { "path": "config" },
            { "path": "state" },
            { "path": "logs" },
            { "path": "quarantine" }
          ]
        },
        { "path": "logs" },
        { "path": "scripts" },
        { "path": "secrets" },
        { "path": "temp" }
      ]
    },
    {
      "name": "Documents",
      "root": "~/Documents",
      "required": [
        { "path": "Image-Line" },
        { "path": "archive" },
        { "path": "audio" },
        { "path": "files" },
        { "path": "finance" },
        { "path": "images" },
        { "path": "legal" },
        { "path": "personal" },
        {
          "path": "school",
          "children": [
            { "path": "admin" },
            { "path": "assignments" },
            { "path": "archive" },
            { "path": "classes" },
            { "path": "files" },
            { "path": "img" },
            { "path": "notes" },
            { "path": "projects" },
            { "path": "resources" },
            { "path": "submissions" }
          ]
        },
        { "path": "videos" },
        { "path": "work" },
        { "path": "writing" },
        { "path": "screenshots" }
      ]
    },
    {
      "name": "Workspace",
      "root": "~/Workspace",
      "required": [
        { "path": "archive" },
        { "path": "assets" },
        { "path": "clients" },
        { "path": "code" },
        { "path": "hardware" },
        { "path": "learning" },
        { "path": "saas" },
        { "path": "sandbox" },
        { "path": "school" }
      ]
    }
  ]
}"#,
            );

            let required = [
                "System/apps",
                "System/backups",
                "System/bootstrap",
                "System/configs",
                "System/dotfiles",
                "System/life-os/repo",
                "System/life-os/bin",
                "System/life-os/config",
                "System/life-os/state",
                "System/life-os/logs",
                "System/life-os/quarantine",
                "System/logs",
                "System/scripts",
                "System/secrets",
                "System/temp",
                "Documents/Image-Line",
                "Documents/archive",
                "Documents/audio",
                "Documents/files",
                "Documents/finance",
                "Documents/images",
                "Documents/legal",
                "Documents/personal",
                "Documents/school/admin",
                "Documents/school/assignments",
                "Documents/school/archive",
                "Documents/school/classes",
                "Documents/school/files",
                "Documents/school/img",
                "Documents/school/notes",
                "Documents/school/projects",
                "Documents/school/resources",
                "Documents/school/submissions",
                "Documents/videos",
                "Documents/work",
                "Documents/writing",
                "Documents/screenshots",
                "Workspace/archive",
                "Workspace/assets",
                "Workspace/clients",
                "Workspace/code",
                "Workspace/hardware",
                "Workspace/learning",
                "Workspace/saas",
                "Workspace/sandbox",
                "Workspace/school",
            ];

            for path in required {
                fs::create_dir_all(home.join(path)).expect("create dir");
            }

            let report = run(false).expect("doctor run");
            assert!(report.missing.is_empty());
        });
    }

    #[test]
    fn doctor_returns_failure_when_missing_folders() {
        with_temp_home(|home| {
            write_spec(
                home,
                r#"{
  "version": 1,
  "areas": [
    {
      "name": "Documents",
      "root": "~/Documents",
      "required": [
        { "path": "archive" },
        { "path": "screenshots" }
      ]
    }
  ]
}"#,
            );

            fs::create_dir_all(home.join("Documents/archive")).expect("create archive");

            let report = run(false).expect("doctor run");
            assert_eq!(report.missing.len(), 1);
        });
    }
}
