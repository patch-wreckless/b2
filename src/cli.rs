use anyhow::{Error, Result};
use clap::{Parser, Subcommand};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PathArg(PathBuf);

impl Deref for PathArg {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for PathArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        PathBuf::from(s)
            .canonicalize()
            .map_err(|e| e.into())
            .map(PathArg)
    }
}

#[derive(Parser)]
#[command(name = "b2", version = "0.1")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Write a summary of the source directory organized by file extension to stdout.
    Summarize {
        /// The source directory to scan
        src: PathArg,
    },
}
