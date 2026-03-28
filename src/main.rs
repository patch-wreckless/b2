mod cli;
mod dupes;
mod hashes;
mod summarize;

use dupes::dupes;
use hashes::hashes;
use summarize::summarize;

fn main() {
    use std::process::exit;

    if let Err(e) = run() {
        eprintln!("fatal: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    use cli::{Cli, Commands};

    let cli = Cli::try_parse()?;

    match cli.command {
        Commands::Summarize { src } => summarize(&src).map_err(|e| e.into()),
        Commands::Hashes { src } => hashes(&src).map_err(|e| e.into()),
        Commands::Dupes => dupes().map_err(|e| e.into()),
    }
}
