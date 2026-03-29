mod cli;
mod dupes;
mod hashes;
mod summarize;

use clap::Parser;
use cli::{Cli, Command};
use dupes::dupes;
use hashes::hashes;
use std::process::exit;
use summarize::summarize;

fn main() {
    if let Err(e) = run() {
        eprintln!("fatal: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::try_parse()?;

    match cli.command {
        Command::Summarize { src } => summarize(&src).map_err(|e| e.into()),
        Command::Hashes { src } => hashes(&src).map_err(|e| e.into()),
        Command::Dupes => dupes().map_err(|e| e.into()),
    }
}
