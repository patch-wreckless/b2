use crate::hashes::FileHash;

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

pub fn dupes() -> anyhow::Result<()> {
    let lines = io::stdin().lock().lines().filter_map(|res| match res {
        Ok(line) => {
            let line = line.trim();
            match line.is_empty() {
                true => None,
                false => Some(Ok(line.to_string())),
            }
        }
        err @ Err(_) => Some(err),
    });

    let hashes = lines.map(|res| -> anyhow::Result<FileHash> {
        match res {
            Ok(line) => line.parse::<FileHash>().map_err(|e| e.into()),
            Err(err) => Err(err.into()),
        }
    });

    let mut dir = Dir(BTreeMap::new());
    for hash in hashes {
        let hash = hash?;
        dir.insert(&hash.path, &hash.hash)?;
    }

    print_dir(&mut io::stdout(), OsStr::new(""), &dir, "")?;

    Ok(())
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

/// Removes the root from the given path. If the given path has no root it's returned unmodified.
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

#[cfg(test)]
mod tests {
    use super::*;

    mod strip_root {
        use super::*;
        use std::path::Path;

        #[test]
        fn removes_root() {
            let input = Path::new("/foo/bar");
            let expected = Path::new("foo/bar");
            let actual = strip_root(input);
            assert_eq!(expected, actual);
        }

        #[test]
        fn returns_rootless_path_unmodified() {
            let input = Path::new("foo/bar");
            let expected = input;
            let actual = strip_root(input);
            assert_eq!(expected, actual);
        }
    }
}

fn print_dir<W: Write>(w: &mut W, name: &OsStr, dir: &Dir, indent: &str) -> io::Result<()> {
    writeln!(w, "{}{}/", indent, name.to_string_lossy())?;
    let indent = format!("{}  ", indent);

    for (name, node) in dir.0.iter() {
        match node {
            Node::File(file_hash) => {
                writeln!(w, "{}{} {}", indent, name.to_string_lossy(), file_hash.hash)?;
            }
            Node::Dir(dir) => {
                print_dir(w, name, dir, &indent)?;
            }
        }
    }

    Ok(())
}
