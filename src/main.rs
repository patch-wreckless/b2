use anyhow::{Error, Result, bail};

use std::env;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        bail!("usage: {} <src> <dst>", args[0]);
    }

    let src = args[1]
        .parse::<PathArg>()
        .map_err(|e| anyhow::anyhow!("<src>: {}: {}", args[1], e))?;

    let dst = args[2]
        .parse::<PathArg>()
        .map_err(|e| anyhow::anyhow!("<dst>: {}: {}", args[2], e))?;

    validate_directories(&src, &dst)?;

    let total = count_files(&src);

    println!("Total files: {}", total);

    Ok(())
}

fn validate_directories(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        bail!("{} is not a directory", src.display());
    }

    if !dst.is_dir() {
        bail!("{} is not a directory", dst.display());
    }

    if dst.starts_with(src) {
        bail!(
            "<src> must not contain <dst> ({} contains {})",
            dst.display(),
            src.display()
        );
    }

    Ok(())
}

struct PathArg(PathBuf);

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

fn count_files(path: &Path) -> u64 {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_files(&path);
            }
        }
    }

    count
}
