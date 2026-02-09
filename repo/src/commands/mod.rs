use anyhow::Result;

use crate::cli::{Cli, Command};

pub mod doctor;

pub fn dispatch(cli: Cli) -> Result<std::process::ExitCode> {
    match cli.command {
        Command::Doctor { verbose } => doctor::run(verbose),
    }
}
