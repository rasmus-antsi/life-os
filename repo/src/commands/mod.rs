use anyhow::{Context, Result};

use crate::cli::{Cli, Command};
use crate::commands::tidy::TidyOptions;

pub mod doctor;
pub mod init;
pub mod tidy;

pub fn dispatch(cli: Cli) -> Result<std::process::ExitCode> {
    match cli.command {
        Command::Doctor { verbose } => doctor::run(verbose),
        Command::Init { verbose } => init::run(verbose),
        Command::Tidy { apply, all } => {
            let home = dirs::home_dir().context("could not determine home directory")?;
            let options = TidyOptions {
                apply,
                delete_all_downloads: all,
                desktop: home.join("Desktop"),
                downloads: home.join("Downloads"),
                screenshots_dest: home.join("Documents/screenshots"),
            };
            let report = tidy::run(&options)?;
            print_report(&report, apply);
            Ok(std::process::ExitCode::from(0))
        }
    }
}

fn print_report(report: &tidy::TidyReport, apply: bool) {
    println!("Desktop:");
    println!("  Screenshots: {}", report.desktop_screenshots.len());
    for path in &report.desktop_screenshots {
        let size = tidy::dir_or_file_size(path);
        println!("    - {} ({})", path.display(), tidy::human_bytes(size));
    }
    println!("  Other files: {}", report.desktop_other.len());
    for path in &report.desktop_other {
        let size = tidy::dir_or_file_size(path);
        println!("    - {} ({})", path.display(), tidy::human_bytes(size));
    }

    println!("Downloads:");
    println!("  Items: {}", report.downloads_items.len());
    for path in &report.downloads_items {
        let size = tidy::dir_or_file_size(path);
        println!("    - {} ({})", path.display(), tidy::human_bytes(size));
    }
    println!(
        "  Total size: {} ({})",
        report.downloads_total_bytes,
        tidy::human_bytes(report.downloads_total_bytes)
    );

    println!("Actions:");
    println!("  Screenshots to move: {}", report.planned_moves.len());
    println!(
        "  Downloads to delete: {}",
        report.planned_downloads_deletions.len()
    );
    println!("  Apply enabled: {}", if apply { "yes" } else { "no" });
}
