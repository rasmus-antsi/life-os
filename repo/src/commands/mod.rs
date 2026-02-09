use anyhow::{Context, Result};

use crate::cli::{Cli, Command};
use crate::commands::tidy::TidyOptions;

pub mod doctor;
pub mod init;
pub mod tidy;

pub fn dispatch(cli: Cli) -> Result<std::process::ExitCode> {
    match cli.command {
        Command::Doctor { verbose, plain } => {
            let report = doctor::run(verbose)?;
            print_doctor(&report, OutputStyle::new(plain, verbose));
            Ok(if report.missing.is_empty() {
                std::process::ExitCode::from(0)
            } else {
                std::process::ExitCode::from(1)
            })
        }
        Command::Init { verbose, plain } => {
            let report = init::run(verbose)?;
            print_init(&report, OutputStyle::new(plain, verbose));
            Ok(std::process::ExitCode::from(0))
        }
        Command::Tidy {
            apply,
            all,
            verbose,
            plain,
        } => {
            let home = dirs::home_dir().context("could not determine home directory")?;
            let options = TidyOptions {
                apply,
                delete_all_downloads: all,
                desktop: home.join("Desktop"),
                downloads: home.join("Downloads"),
                screenshots_dest: home.join("Documents/screenshots"),
            };
            let report = tidy::run(&options)?;
            print_tidy(&report, OutputStyle::new(plain, verbose), apply, all);
            Ok(std::process::ExitCode::from(0))
        }
    }
}

#[derive(Clone, Copy)]
struct OutputStyle {
    plain: bool,
    verbose: bool,
}

impl OutputStyle {
    fn new(plain: bool, verbose: bool) -> Self {
        Self { plain, verbose }
    }

    fn header(&self, text: &str) -> String {
        if self.plain {
            text.to_string()
        } else {
            color(text, Color::Accent)
        }
    }

    fn ok_symbol(&self) -> &'static str {
        if self.plain { "OK" } else { "✓" }
    }

    fn err_symbol(&self) -> &'static str {
        if self.plain { "ERROR" } else { "✗" }
    }

    fn section(&self, text: &str) -> String {
        if self.plain {
            text.to_string()
        } else {
            color(text, Color::Accent)
        }
    }

    fn dim(&self, text: &str) -> String {
        if self.plain {
            text.to_string()
        } else {
            color(text, Color::Dim)
        }
    }

    fn highlight(&self, text: &str) -> String {
        if self.plain {
            text.to_string()
        } else {
            color(text, Color::Accent)
        }
    }
}

fn print_doctor(report: &doctor::DoctorReport, style: OutputStyle) {
    println!("{}", style.header("life-os doctor"));
    if report.missing.is_empty() {
        let msg = format!(
            "{} Spec satisfied ({} areas, {} folders)",
            style.ok_symbol(),
            report.areas,
            report.required
        );
        println!("{}", color_if(style, &msg, Color::Success));
    } else {
        let msg = format!(
            "{} Missing folders ({})",
            style.err_symbol(),
            report.missing.len()
        );
        println!("{}", color_if(style, &msg, Color::Error));
        println!();
        println!("Missing");
        for path in &report.missing {
            println!("{} {}", bullet(style), path.display());
        }
    }

    if style.verbose {
        println!();
        println!("Roots");
        for root in &report.roots {
            println!("{} {}", bullet(style), root.display());
        }
    }
}

fn print_init(report: &init::InitReport, style: OutputStyle) {
    println!("{}", style.header("life-os init"));
    if report.created.is_empty() {
        let msg = format!(
            "{} Nothing to create (spec already satisfied)",
            style.ok_symbol()
        );
        println!("{}", color_if(style, &msg, Color::Success));
    } else {
        let msg = format!(
            "{} Created {} folder(s)",
            style.ok_symbol(),
            report.created.len()
        );
        println!("{}", color_if(style, &msg, Color::Success));
        if style.verbose {
            println!();
            println!("Created");
            for path in &report.created {
                println!("{} {}", bullet(style), path.display());
            }
        }
    }
}

fn print_tidy(report: &tidy::TidyReport, style: OutputStyle, apply: bool, delete_all: bool) {
    println!("{}", style.header("life-os tidy"));

    let desktop_clean = report.desktop_screenshots.len() <= 10 && report.desktop_other.len() <= 2;
    let mut downloads_level = downloads_level(report.downloads_total_bytes);
    if report.downloads_items.len() > 100 {
        downloads_level = downloads_level.bump();
    }

    let summary = format!(
        "{} Desktop {}, Downloads {}",
        style.ok_symbol(),
        if desktop_clean { "clean" } else { "busy" },
        downloads_level.as_str()
    );
    println!("{}", color_if(style, &summary, Color::Success));

    let show_full = style.verbose || !desktop_clean || !downloads_level.is_light();
    if !show_full {
        println!();
        println!("{}", style.section("Desktop"));
        println!(
            "{} Screenshots: {} ({})",
            bullet(style),
            style.highlight(&report.desktop_screenshots.len().to_string()),
            style.dim(&tidy::human_bytes(total_size(&report.desktop_screenshots)))
        );
        println!(
            "{} Other files: {} ({})",
            bullet(style),
            style.highlight(&report.desktop_other.len().to_string()),
            style.dim(&tidy::human_bytes(total_size(&report.desktop_other)))
        );
        println!();
        println!("{}", style.section("Downloads"));
        println!(
            "{} Items: {} ({})",
            bullet(style),
            style.highlight(&report.downloads_items.len().to_string()),
            style.dim(&tidy::human_bytes(report.downloads_total_bytes))
        );
        println!(
            "{} Old (>7 days): {} ({})",
            bullet(style),
            style.highlight(&report.downloads_old_items.len().to_string()),
            style.dim(&tidy::human_bytes(report.downloads_old_bytes))
        );
        if apply {
            println!();
            println!("{}", style.section("Actions"));
            println!(
                "{} Moved screenshots: {}",
                bullet(style),
                style.highlight(&report.planned_moves.len().to_string())
            );
            if delete_all {
                println!(
                    "{} Deleted downloads (all): {}",
                    bullet(style),
                    style.highlight(&report.planned_downloads_deletions.len().to_string())
                );
            } else {
                println!(
                    "{} Deleted downloads (>7 days): {}",
                    bullet(style),
                    style.highlight(&report.planned_downloads_deletions.len().to_string())
                );
            }
        }
        return;
    }

    println!();
    println!("{}", style.section("Desktop"));
    println!(
        "{} Screenshots: {} ({})",
        bullet(style),
        style.highlight(&report.desktop_screenshots.len().to_string()),
        style.dim(&tidy::human_bytes(total_size(&report.desktop_screenshots)))
    );
    for path in &report.desktop_screenshots {
        let size = tidy::dir_or_file_size(path);
        println!(
            "{} {} ({})",
            bullet(style),
            path.display(),
            style.dim(&tidy::human_bytes(size))
        );
    }
    println!(
        "{} Other files: {} ({})",
        bullet(style),
        style.highlight(&report.desktop_other.len().to_string()),
        style.dim(&tidy::human_bytes(total_size(&report.desktop_other)))
    );
    for path in &report.desktop_other {
        let size = tidy::dir_or_file_size(path);
        println!(
            "{} {} ({})",
            bullet(style),
            path.display(),
            style.dim(&tidy::human_bytes(size))
        );
    }

    println!();
    println!("{}", style.section("Downloads"));
    println!(
        "{} Items: {} ({})",
        bullet(style),
        style.highlight(&report.downloads_items.len().to_string()),
        style.dim(&tidy::human_bytes(report.downloads_total_bytes))
    );
    for path in &report.downloads_items {
        let size = tidy::dir_or_file_size(path);
        println!(
            "{} {} ({})",
            bullet(style),
            path.display(),
            style.dim(&tidy::human_bytes(size))
        );
    }
    println!(
        "{} Old (>7 days): {} ({})",
        bullet(style),
        style.highlight(&report.downloads_old_items.len().to_string()),
        style.dim(&tidy::human_bytes(report.downloads_old_bytes))
    );

    if apply {
        println!();
        println!("{}", style.section("Actions"));
        println!(
            "{} Moved screenshots: {}",
            bullet(style),
            style.highlight(&report.planned_moves.len().to_string())
        );
        if delete_all {
            println!(
                "{} Deleted downloads (all): {}",
                bullet(style),
                style.highlight(&report.planned_downloads_deletions.len().to_string())
            );
        } else {
            println!(
                "{} Deleted downloads (>7 days): {}",
                bullet(style),
                style.highlight(&report.planned_downloads_deletions.len().to_string())
            );
        }
    }
}

fn total_size(paths: &[std::path::PathBuf]) -> u64 {
    paths.iter().map(|p| tidy::dir_or_file_size(p)).sum()
}

#[derive(Clone, Copy)]
enum Color {
    Accent,
    Success,
    Error,
    Dim,
}

fn color_if(style: OutputStyle, text: &str, color_kind: Color) -> String {
    if style.plain {
        text.to_string()
    } else {
        color(text, color_kind)
    }
}

fn color(text: &str, color_kind: Color) -> String {
    let code = match color_kind {
        Color::Accent => "36",
        Color::Success => "32",
        Color::Error => "31",
        Color::Dim => "2",
    };
    format!("\u{1b}[{}m{}\u{1b}[0m", code, text)
}

fn bullet(style: OutputStyle) -> &'static str {
    if style.plain { "-" } else { "•" }
}

#[derive(Clone, Copy)]
enum DownloadsLevel {
    Light,
    Moderate,
    Heavy,
}

impl DownloadsLevel {
    fn as_str(self) -> &'static str {
        match self {
            DownloadsLevel::Light => "light",
            DownloadsLevel::Moderate => "moderate",
            DownloadsLevel::Heavy => "heavy",
        }
    }

    fn bump(self) -> Self {
        match self {
            DownloadsLevel::Light => DownloadsLevel::Moderate,
            DownloadsLevel::Moderate => DownloadsLevel::Heavy,
            DownloadsLevel::Heavy => DownloadsLevel::Heavy,
        }
    }

    fn is_light(self) -> bool {
        matches!(self, DownloadsLevel::Light)
    }
}

fn downloads_level(total_bytes: u64) -> DownloadsLevel {
    const GB: u64 = 1024 * 1024 * 1024;
    if total_bytes <= 1 * GB {
        DownloadsLevel::Light
    } else if total_bytes <= 5 * GB {
        DownloadsLevel::Moderate
    } else {
        DownloadsLevel::Heavy
    }
}
