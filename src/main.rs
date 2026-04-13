mod ascii;
mod cli;
mod dupes;
mod hashes;
mod summarize;
mod xfs;

use clap::Parser;
use cli::{Cli, Command};
use dupes::dupes;
use hashes::hashes;
use std::process::exit;
use std::path::Path;

use crate::summarize::summarize;

fn main() {
    if let Err(e) = run() {
        eprintln!("fatal: {}", e);
        exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    match Cli::try_parse()?.command {
        Command::Summarize { src } => run_summarize(&src),
        Command::Hashes { algo, src } => hashes(algo.into(), &src),
        Command::Dupes => dupes(),
    }
}

fn run_summarize(src: &Path) -> anyhow::Result<()> {
    summarize(xfs::walk(src))
}
