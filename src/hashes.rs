use crossbeam::channel::{Receiver, unbounded};
use sha1::{Digest, Sha1};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc;

use crate::ascii;
use crate::files;

#[derive(Copy, Clone)]
pub enum HashAlgo {
    SHA1,
    SHA256,
}

pub fn hashes<I>(algo: HashAlgo, file_paths: I) -> anyhow::Result<()>
where
    I: Iterator<Item = files::FilePathItem> + Send,
{
    let (file_sender, file_receiver) = unbounded::<files::FilePathItem>();
    let (hash_sender, hash_receiver) = mpsc::channel::<Result<FileHash, anyhow::Error>>();

    let hash_worker_count =
        std::thread::available_parallelism().map_or(1, |p| p.get().saturating_sub(2).max(1));

    std::thread::scope(|s| {
        for _ in 0..hash_worker_count {
            let receiver = file_receiver.clone();
            let sender = hash_sender.clone();
            s.spawn(move || {
                hash_files(receiver, sender, &algo);
            });
        }

        drop(hash_sender);
        drop(file_receiver);

        s.spawn(move || {
            for path in file_paths {
                file_sender.send(path).unwrap();
            }
        });

        for hash in hash_receiver.iter() {
            match hash {
                Ok(hash) => println!("{}", hash),
                Err(err) => eprintln!("{:?}", err),
            }
        }
    });

    Ok(())
}

fn hash_files(
    receiver: Receiver<files::FilePathItem>,
    sender: mpsc::Sender<Result<FileHash, anyhow::Error>>,
    algo: &HashAlgo,
) {
    for file in receiver.iter() {
        let file_hash = match file {
            Ok(path) => match get_file_hash(&path, algo) {
                Ok(hash) => Ok(FileHash { hash, path }),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        };
        sender.send(file_hash).unwrap();
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error hashing file '{path}': {message}")]
struct HashError {
    path: PathBuf,
    message: String,
}

impl HashError {
    fn new<E: Error>(path: &Path, source: E) -> Self {
        HashError {
            path: path.to_path_buf(),
            message: format!("{}", source),
        }
    }
}

struct Sha1Write<'a>(&'a mut Sha1);

impl<'a> Write for Sha1Write<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn get_file_hash(path: &Path, algo: &HashAlgo) -> Result<String, HashError> {
    match algo {
        HashAlgo::SHA1 => {
            let mut file = File::open(path).map_err(|e| HashError::new(path, e))?;
            let mut hasher = Sha1::new();
            let mut write = Sha1Write(&mut hasher);
            io::copy(&mut file, &mut write).map_err(|e| HashError::new(path, e))?;
            Ok(format!("sha1:{}", hex::encode(hasher.finalize())))
        }
        HashAlgo::SHA256 => {
            let hash = sha256::try_digest(path).map_err(|e| HashError::new(path, e))?;
            Ok(format!("sha256:{}", hash))
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error parsing file hash: {0}")]
pub struct ParseError(String);

impl ParseError {
    fn new(msg: &str) -> Self {
        ParseError(msg.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHash {
    pub hash: String,
    pub path: PathBuf,
}

impl fmt::Display for FileHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}",
            self.hash,
            ascii::escape(self.path.as_os_str().as_encoded_bytes().iter().copied())
        )
    }
}

impl From<&FileHash> for String {
    fn from(fh: &FileHash) -> Self {
        fh.to_string()
    }
}

impl FromStr for FileHash {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '\t');
        let hash = parts.next().ok_or(ParseError::new("missing hash"))?.trim();
        if hash.is_empty() {
            return Err(ParseError::new("empty hash"));
        }

        let path = parts.next().ok_or(ParseError::new("missing path"))?.trim();
        if path.is_empty() {
            return Err(ParseError::new("empty path"));
        }
        let path = ascii::unescape(&mut path.as_bytes().iter())
            .map_err(|e| ParseError::new(&format!("invalid escape sequence: {}", e)))?;

        Ok(FileHash {
            path: PathBuf::from(path),
            hash: hash.to_string(),
        })
    }
}
