mod cli;
mod commands;

use anyhow::Result;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            std::process::ExitCode::from(2)
        }
    }
}

fn run() -> Result<std::process::ExitCode> {
    let cli = cli::parse();
    commands::dispatch(cli)
}
