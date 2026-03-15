use anyhow::{Error, Result, anyhow, bail};
use crossbeam::channel::{Receiver, Sender, unbounded};

use std::collections::HashMap;
use std::env;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        bail!("usage: {} <src> <dst>", args[0]);
    }

    let src = args[1]
        .parse::<PathArg>()
        .map_err(|e| anyhow!("<src>: {}: {}", args[1], e))?;

    let dst = args[2]
        .parse::<PathArg>()
        .map_err(|e| anyhow!("<dst>: {}: {}", args[2], e))?;

    validate_directories(&src, &dst)?;

    let receiver = get_files(&src);

    let mut files_by_extension: HashMap<String, Vec<String>> = HashMap::new();

    for file in receiver.iter() {
        let extension = match file.extension() {
            Some(ext) => ext.to_string_lossy(),
            None => "".into(),
        };
        files_by_extension
            .entry(extension.to_string())
            .or_insert_with(Vec::new)
            .push(file.to_string_lossy().to_string());
    }

    let mut sorted_entries: Vec<_> = files_by_extension.iter().collect();
    sorted_entries.sort_by_key(|&(key, _)| key);

    for (extension, values) in sorted_entries {
        let extension = match extension.len() {
            0 => "''".to_string(),
            _ => extension.to_string(),
        };
        println!("{}:", extension);
        let mut values = values.iter().collect::<Vec<_>>();
        values.sort();
        for value in values {
            println!("  - {}", value);
        }
    }

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

fn get_files(path: &Path) -> Receiver<PathBuf> {
    let (sender, receiver) = unbounded::<PathBuf>();
    let path = path.to_path_buf();
    thread::spawn(|| {
        walk_dir(path, sender.clone());
        drop(sender);
    });
    receiver
}

fn walk_dir(path: PathBuf, sender: Sender<PathBuf>) {
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            match entry.file_type() {
                Ok(ft) if ft.is_file() => {
                    sender.send(entry.path()).unwrap();
                }
                Ok(ft) if ft.is_dir() => {
                    walk_dir(entry.path(), sender.clone());
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
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
