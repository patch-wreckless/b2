mod ascii;
mod cli;
mod dupes;
mod hashes;
mod summarize;

use clap::Parser;
use cli::{Cli, Command};
use crossbeam::channel::{Receiver, Sender, unbounded};
use dupes::dupes;
use hashes::hashes;
use std::fs;
use std::process::exit;
use std::{
    path::{Path, PathBuf},
    thread,
};
use summarize::{EntryError, summarize2};

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
    let files = get_files(src);
    summarize2(files)
}

fn get_files(path: &Path) -> Receiver<std::result::Result<PathBuf, EntryError>> {
    let (sender, receiver) = unbounded::<std::result::Result<PathBuf, EntryError>>();
    let path = path.to_path_buf();
    thread::spawn(|| {
        send_files(path, sender.clone());
        drop(sender);
    });
    receiver
}

fn send_files(path: PathBuf, sender: Sender<std::result::Result<PathBuf, EntryError>>) {
    match fs::read_dir(&path) {
        Err(err) => {
            sender.send(Err(EntryError::new(&err.to_string()))).unwrap();
        }
        Ok(read_dir) => {
            for res in read_dir.into_iter() {
                match res {
                    Err(err) => {
                        sender.send(Err(EntryError::new(&err.to_string()))).unwrap();
                        return;
                    }
                    Ok(entry) => match entry.file_type() {
                        Err(err) => {
                            sender.send(Err(EntryError::new(&err.to_string()))).unwrap();
                            return;
                        }
                        Ok(ft) => {
                            if ft.is_file() {
                                sender.send(Ok(entry.path())).unwrap();
                                continue;
                            }
                            if ft.is_dir() {
                                send_files(entry.path(), sender.clone());
                                continue;
                            }
                            if ft.is_symlink() {
                                sender
                                    .send(Err(EntryError::new("symlinks are not supported")))
                                    .unwrap();
                                return;
                            }
                            sender
                                .send(Err(EntryError::new("unknown file type")))
                                .unwrap();
                            return;
                        }
                    },
                }
            }
        }
    }
}
