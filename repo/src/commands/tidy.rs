use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct TidyOptions {
    pub apply: bool,
    pub delete_all_downloads: bool,
    pub desktop: PathBuf,
    pub downloads: PathBuf,
    pub screenshots_dest: PathBuf,
}

#[derive(Debug, Default, Clone)]
pub struct TidyReport {
    pub desktop_screenshots: Vec<PathBuf>,
    pub desktop_other: Vec<PathBuf>,
    pub downloads_items: Vec<PathBuf>,
    pub downloads_total_bytes: u64,
    pub downloads_old_items: Vec<PathBuf>,
    pub downloads_old_bytes: u64,
    pub planned_downloads_deletions: Vec<PathBuf>,
    pub planned_moves: Vec<(PathBuf, PathBuf)>,
}

pub fn run(options: &TidyOptions) -> Result<TidyReport> {
    let mut report = TidyReport::default();

    let desktop_entries = read_dir_paths(&options.desktop)?;
    let downloads_entries = read_dir_paths(&options.downloads)?;

    for path in desktop_entries {
        if path.is_dir() {
            report.desktop_other.push(path);
            continue;
        }

        let file_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => {
                report.desktop_other.push(path);
                continue;
            }
        };

        if is_macos_screenshot(file_name) {
            report.desktop_screenshots.push(path.clone());
            let dest = unique_destination(&options.screenshots_dest, file_name);
            report.planned_moves.push((path, dest));
        } else {
            report.desktop_other.push(path);
        }
    }

    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(7 * 24 * 60 * 60))
        .context("failed to compute cutoff time")?;

    for path in downloads_entries {
        let file_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if file_name.starts_with('.') {
            continue;
        }

        let size = dir_or_file_size(&path);
        report.downloads_total_bytes = report.downloads_total_bytes.saturating_add(size);
        report.downloads_items.push(path.clone());

        if is_older_than(&path, cutoff) {
            report.downloads_old_items.push(path.clone());
            report.downloads_old_bytes = report.downloads_old_bytes.saturating_add(size);
        }

        if options.delete_all_downloads || is_older_than(&path, cutoff) {
            report.planned_downloads_deletions.push(path);
        }
    }

    if options.apply {
        if !report.planned_moves.is_empty() {
            fs::create_dir_all(&options.screenshots_dest).with_context(|| {
                format!(
                    "failed to create screenshots destination: {}",
                    options.screenshots_dest.display()
                )
            })?;
        }

        for (src, dest) in &report.planned_moves {
            fs::rename(src, dest).with_context(|| {
                format!(
                    "failed to move screenshot {} -> {}",
                    src.display(),
                    dest.display()
                )
            })?;
        }

        for path in &report.planned_downloads_deletions {
            if path.is_dir() {
                fs::remove_dir_all(path)
                    .with_context(|| format!("failed to delete dir: {}", path.display()))?;
            } else {
                fs::remove_file(path)
                    .with_context(|| format!("failed to delete file: {}", path.display()))?;
            }
        }
    }

    Ok(report)
}

fn read_dir_paths(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in
        fs::read_dir(dir).with_context(|| format!("failed to read directory: {}", dir.display()))?
    {
        let entry = entry.context("failed to read directory entry")?;
        out.push(entry.path());
    }
    Ok(out)
}

fn is_macos_screenshot(file_name: &str) -> bool {
    file_name.starts_with("Screenshot ") && file_name.ends_with(".png")
}

fn unique_destination(dest_dir: &Path, file_name: &str) -> PathBuf {
    let base_dest = dest_dir.join(file_name);
    if !base_dest.exists() {
        return base_dest;
    }

    let (stem, ext) = split_name_ext(file_name);
    for i in 1.. {
        let candidate = format!("{} ({}){}", stem, i, ext);
        let candidate_path = dest_dir.join(candidate);
        if !candidate_path.exists() {
            return candidate_path;
        }
    }

    base_dest
}

fn split_name_ext(file_name: &str) -> (String, String) {
    match file_name.rsplit_once('.') {
        Some((stem, ext)) => (stem.to_string(), format!(".{}", ext)),
        None => (file_name.to_string(), String::new()),
    }
}

pub fn dir_or_file_size(path: &Path) -> u64 {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(meta) if meta.is_dir() => dir_size(path),
        _ => 0,
    }
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return 0,
    };

    for entry in entries.flatten() {
        let p = entry.path();
        total = total.saturating_add(dir_or_file_size(&p));
    }
    total
}

fn is_older_than(path: &Path, cutoff: SystemTime) -> bool {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(_) => return false,
    };
    let modified = match meta.modified() {
        Ok(m) => m,
        Err(_) => return false,
    };
    modified < cutoff
}

pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", value, UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::{TidyOptions, run};
    use filetime::{FileTime, set_file_times};
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime};
    use tempfile::tempdir;

    fn write_file(path: &Path, bytes: usize) {
        let data = vec![b'a'; bytes];
        fs::write(path, data).expect("write file");
    }

    #[test]
    fn dry_run_reports_desktop_screenshots_and_other_files() {
        let dir = tempdir().expect("tempdir");
        let desktop = dir.path().join("Desktop");
        let downloads = dir.path().join("Downloads");
        let screenshots_dest = dir.path().join("Documents/screenshots");

        fs::create_dir_all(&desktop).expect("desktop");
        fs::create_dir_all(&downloads).expect("downloads");

        write_file(&desktop.join("Screenshot 2026-02-09 at 10.00.00.png"), 10);
        write_file(&desktop.join("notes.txt"), 5);

        let options = TidyOptions {
            apply: false,
            delete_all_downloads: false,
            desktop: desktop.clone(),
            downloads: downloads.clone(),
            screenshots_dest,
        };

        let report = run(&options).expect("tidy run");
        assert_eq!(report.desktop_screenshots.len(), 1);
        assert_eq!(report.desktop_other.len(), 1);
    }

    #[test]
    fn apply_moves_screenshots_and_renames_on_collision() {
        let dir = tempdir().expect("tempdir");
        let desktop = dir.path().join("Desktop");
        let downloads = dir.path().join("Downloads");
        let screenshots_dest = dir.path().join("Documents/screenshots");

        fs::create_dir_all(&desktop).expect("desktop");
        fs::create_dir_all(&downloads).expect("downloads");
        fs::create_dir_all(&screenshots_dest).expect("dest");

        let screenshot = desktop.join("Screenshot 2026-02-09 at 10.00.00.png");
        write_file(&screenshot, 10);
        write_file(
            &screenshots_dest.join("Screenshot 2026-02-09 at 10.00.00.png"),
            1,
        );

        let options = TidyOptions {
            apply: true,
            delete_all_downloads: false,
            desktop: desktop.clone(),
            downloads: downloads.clone(),
            screenshots_dest: screenshots_dest.clone(),
        };

        let _report = run(&options).expect("tidy run");

        assert!(!screenshot.exists());
        assert!(
            screenshots_dest
                .join("Screenshot 2026-02-09 at 10.00.00 (1).png")
                .exists()
        );
    }

    #[test]
    fn downloads_reports_total_size_and_excludes_hidden_items() {
        let dir = tempdir().expect("tempdir");
        let desktop = dir.path().join("Desktop");
        let downloads = dir.path().join("Downloads");
        let screenshots_dest = dir.path().join("Documents/screenshots");

        fs::create_dir_all(&desktop).expect("desktop");
        fs::create_dir_all(&downloads).expect("downloads");

        write_file(&downloads.join("a.txt"), 5);
        write_file(&downloads.join(".hidden"), 10);

        let options = TidyOptions {
            apply: false,
            delete_all_downloads: false,
            desktop,
            downloads: downloads.clone(),
            screenshots_dest,
        };

        let report = run(&options).expect("tidy run");
        assert_eq!(report.downloads_total_bytes, 5);
        assert_eq!(report.downloads_items.len(), 1);
    }

    #[test]
    fn apply_deletes_downloads_older_than_7_days_by_mtime() {
        let dir = tempdir().expect("tempdir");
        let desktop = dir.path().join("Desktop");
        let downloads = dir.path().join("Downloads");
        let screenshots_dest = dir.path().join("Documents/screenshots");

        fs::create_dir_all(&desktop).expect("desktop");
        fs::create_dir_all(&downloads).expect("downloads");

        let old_file = downloads.join("old.txt");
        let new_file = downloads.join("new.txt");
        let old_dir = downloads.join("old-dir");
        let hidden_old = downloads.join(".hidden-old");

        write_file(&old_file, 5);
        write_file(&new_file, 5);
        fs::create_dir_all(&old_dir).expect("old dir");
        write_file(&hidden_old, 5);

        let now = SystemTime::now();
        let eight_days = Duration::from_secs(8 * 24 * 60 * 60);
        let two_days = Duration::from_secs(2 * 24 * 60 * 60);
        let old_time = FileTime::from_system_time(now - eight_days);
        let new_time = FileTime::from_system_time(now - two_days);

        set_file_times(&old_file, old_time, old_time).expect("set old time");
        set_file_times(&old_dir, old_time, old_time).expect("set old dir time");
        set_file_times(&hidden_old, old_time, old_time).expect("set hidden time");
        set_file_times(&new_file, new_time, new_time).expect("set new time");

        let options = TidyOptions {
            apply: true,
            delete_all_downloads: false,
            desktop,
            downloads: downloads.clone(),
            screenshots_dest,
        };

        let _report = run(&options).expect("tidy run");

        assert!(!old_file.exists());
        assert!(!old_dir.exists());
        assert!(new_file.exists());
        assert!(hidden_old.exists());
    }

    #[test]
    fn human_bytes_formats_sizes() {
        assert_eq!(super::human_bytes(0), "0 B");
        assert_eq!(super::human_bytes(512), "512 B");
        assert_eq!(super::human_bytes(1024), "1.0 KB");
        assert_eq!(super::human_bytes(1536), "1.5 KB");
        assert_eq!(super::human_bytes(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn apply_deletes_all_downloads_when_flag_set() {
        let dir = tempdir().expect("tempdir");
        let desktop = dir.path().join("Desktop");
        let downloads = dir.path().join("Downloads");
        let screenshots_dest = dir.path().join("Documents/screenshots");

        fs::create_dir_all(&desktop).expect("desktop");
        fs::create_dir_all(&downloads).expect("downloads");

        let file = downloads.join("file.txt");
        let dir_item = downloads.join("dir");
        let hidden = downloads.join(".hidden");
        write_file(&file, 5);
        fs::create_dir_all(&dir_item).expect("dir");
        write_file(&hidden, 5);

        let options = TidyOptions {
            apply: true,
            delete_all_downloads: true,
            desktop,
            downloads: downloads.clone(),
            screenshots_dest,
        };

        let _report = run(&options).expect("tidy run");

        assert!(!file.exists());
        assert!(!dir_item.exists());
        assert!(hidden.exists());
    }
}
