use crossbeam::channel::{Receiver, Sender, unbounded};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;
use std::{fmt, fs};

pub fn hashes(src: &Path) -> anyhow::Result<()> {
    let file_receiver = get_files(src);

    let (hash_sender, hash_receiver) = unbounded::<FileHash>();

    let hash_worker_count = std::thread::available_parallelism()
        .map(|n| {
            let p = n.get();
            if p > 2 { p - 2 } else { 1 }
        })
        .unwrap_or(1);

    let mut workers = Vec::new();

    for _ in 0..hash_worker_count {
        let receiver = file_receiver.clone();
        let sender = hash_sender.clone();
        workers.push(thread::spawn(move || {
            for file in receiver.iter() {
                let hash = sha256::try_digest(&file).unwrap();
                sender.send(FileHash { hash, path: file }).unwrap();
            }
            drop(sender);
            drop(receiver);
        }));
    }

    thread::spawn(|| {
        for worker in workers {
            worker.join().unwrap();
        }
        drop(hash_sender);
        drop(file_receiver);
    });

    for file in hash_receiver.iter() {
        println!("{} {}", file.path.display(), file.hash);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHash {
    pub hash: String,
    pub path: PathBuf,
}

impl fmt::Display for FileHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.path.display(), self.hash)
    }
}

impl From<&FileHash> for String {
    fn from(fh: &FileHash) -> Self {
        fh.to_string()
    }
}

impl FromStr for FileHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.rsplitn(2, char::is_whitespace);
        let hash = parts.next().ok_or("missing hash")?.trim();
        let path = parts.next().ok_or("missing path")?.trim();
        if path.is_empty() {
            return Err("empty path".into());
        }
        if hash.is_empty() {
            return Err("empty hash".into());
        }

        Ok(FileHash {
            path: PathBuf::from(path),
            hash: hash.to_string(),
        })
    }
}
