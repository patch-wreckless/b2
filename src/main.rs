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

fn run() -> anyhow::Result<()> {
    match Cli::try_parse()?.command {
        Command::Summarize { src } => summarize(&src),
        Command::Hashes { src } => hashes(&src),
        Command::Dupes => dupes(),
    }
}
