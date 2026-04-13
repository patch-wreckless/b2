mod ascii;
mod cli;
mod dupes;
mod files;
mod hashes;
mod summarize;
mod xfs;

use clap::Parser;
use cli::{Cli, Command};
use dupes::dupes;
use hashes::hashes;
use std::path::Path;
use std::process::exit;

use crate::cli::HashAlgo;
use crate::files::IntoFilePaths;
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
        Command::Hashes { algo, src } => run_hashes(algo, &src),
        Command::Dupes => dupes(),
    }
}

fn run_summarize(src: &Path) -> anyhow::Result<()> {
    summarize(walk_files(src))
}

fn run_hashes(algo: HashAlgo, src: &Path) -> anyhow::Result<()> {
    hashes(algo.into(), walk_files(src))
}

fn walk_files(src: &Path) -> impl Iterator<Item = files::FilePathItem> {
    xfs::walk(src).into_file_paths()
}
