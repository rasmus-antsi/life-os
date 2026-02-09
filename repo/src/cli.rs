use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "life-os",
    version,
    about = "Personal system checker and organizer (macOS). Safe by default."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check required folder layout exists (and key subfolders)
    Doctor {
        /// Print the resolved paths being checked
        #[arg(long)]
        verbose: bool,
    },

    Init {
        /// Print each folder created
        #[arg(long)]
        verbose: bool,
    },

    /// Tidy Desktop screenshots and Downloads inbox
    Tidy {
        /// Perform actions (move/delete). Without this, runs in dry-run mode.
        #[arg(long)]
        apply: bool,
        /// Delete all downloads (non-hidden), regardless of age
        #[arg(long)]
        all: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
