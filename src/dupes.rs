use crate::hashes::FileHash;

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

pub fn dupes() -> anyhow::Result<()> {
    let mut dir = Dir(BTreeMap::new());

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let fh: FileHash = line
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        dir.insert(&fh.path, &fh.hash)?;
    }

    print_dir(OsStr::new(""), &dir, "");

    Ok(())
}

fn print_dir(name: &OsStr, dir: &Dir, indent: &str) {
    println!("{}{}/", indent, name.display());
    let indent = format!("{}  ", indent);
    for (name, node) in dir.0.iter() {
        match node {
            Node::File(file_hash) => println!("{}{} {}", indent, name.display(), file_hash.hash),
            Node::Dir(dir) => print_dir(name, dir, &indent),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum InsertError {
    #[error("path must contain a file name")]
    NoFileName,

    #[error("path \"{0}\" already exists and is not a directory")]
    NotADir(PathBuf),

    #[error("conflict at \"{path}\": new={new}, existing={existing}")]
    Conflict {
        path: PathBuf,
        new: String,
        existing: String,
    },
}

struct Dir(BTreeMap<OsString, Node>);

enum Node {
    File(FileHash),
    Dir(Dir),
}

fn strip_root(path: &Path) -> &Path {
    match path.has_root() {
        true => {
            let mut components = path.components();
            _ = components.next();
            components.as_path()
        }
        false => path,
    }
}

impl Dir {
    fn insert(&mut self, path: &Path, hash: &str) -> Result<(), InsertError> {
        let filename = path.file_name().ok_or(InsertError::NoFileName)?;

        let mut target = &mut self.0;

        if let Some(parent) = strip_root(path).parent() {
            for component in parent.components() {
                let name = component.as_os_str();

                let node = target
                    .entry(name.to_owned())
                    .or_insert(Node::Dir(Dir(BTreeMap::new())));

                match node {
                    Node::File(file_hash) => {
                        return Err(InsertError::NotADir(file_hash.path.clone()));
                    }
                    Node::Dir(dir) => {
                        target = &mut dir.0;
                    }
                }
            }
        }

        match target.entry(filename.to_owned()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Node::File(FileHash {
                    hash: hash.to_string(),
                    path: path.to_path_buf(),
                }));
            }
            std::collections::btree_map::Entry::Occupied(existing) => {
                return Err(InsertError::Conflict {
                    path: path.to_path_buf(),
                    new: hash.to_string(),
                    existing: match existing.get() {
                        Node::File(file_hash) => format!("FILE {}", file_hash.hash),
                        Node::Dir(_) => format!("DIR {}/", filename.display()),
                    },
                });
            }
        }

        Ok(())
    }
}
