use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::fs::Metadata;
use std::path::{Path, PathBuf};

/// An error encountered while walking the file system.
#[derive(Debug, PartialEq)]
pub struct Error {
    pub path: PathBuf,
    pub message: String,
}

impl Error {
    fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (path: {})",
            self.message,
            self.path.display()
        )
    }
}

// A file system entry.
#[derive(Debug, PartialEq)]
pub enum Entry {
    File { path: PathBuf },
    Directory { path: PathBuf },
    Symlink { path: PathBuf },
    Unknown { path: PathBuf },
}

impl Entry {
    fn file(path: impl Into<PathBuf>) -> Self {
        Self::File { path: path.into() }
    }

    fn directory(path: impl Into<PathBuf>) -> Self {
        Self::Directory { path: path.into() }
    }

    fn symlink(path: impl Into<PathBuf>) -> Self {
        Self::Symlink { path: path.into() }
    }

    fn unknown(path: impl Into<PathBuf>) -> Self {
        Self::Unknown { path: path.into() }
    }
}

impl Entry {
    fn from(path: impl Into<PathBuf>, meta: &Metadata) -> Self {
        let path = path.into();
        if meta.is_dir() {
            return Self::directory(&path);
        }
        if meta.is_file() {
            return Self::file(&path);
        }
        if meta.is_symlink() {
            return Self::symlink(&path);
        }
        Self::unknown(&path)
    }
}

pub fn walk(path: impl Into<PathBuf>) -> impl Iterator<Item = Result<Entry, Error>> {
    WalkIter {
        active_read_dir: None,
        dir_queue: VecDeque::from([path.into()]),
    }
}

struct WalkIter {
    active_read_dir: Option<ReadDir>,
    dir_queue: VecDeque<PathBuf>,
}

impl Iterator for WalkIter {
    type Item = Result<Entry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(read_dir) = self.active_read_dir.as_mut() {
            for res in read_dir.inner.by_ref() {
                let entry = match res {
                    Err(err) => return Some(Err(Error::new(&read_dir.path, err.to_string()))),
                    Ok(entry) => entry,
                };

                let path = entry.path();

                let meta = match entry.metadata() {
                    Err(err) => return Some(Err(Error::new(&path, err.to_string()))),
                    Ok(meta) => meta,
                };

                match Entry::from(&path, &meta) {
                    e @ Entry::File { .. } => return Some(Ok(e)),
                    e @ Entry::Symlink { .. } => return Some(Ok(e)),
                    e @ Entry::Unknown { .. } => return Some(Ok(e)),
                    Entry::Directory { path } => self.dir_queue.push_back(path),
                }
            }
        }

        if let Some(path) = self.dir_queue.pop_front() {
            let dir = match read_dir(&path) {
                Err(err) => return Some(Err(err)),
                Ok(dir) => dir,
            };
            self.active_read_dir = Some(dir);
            return Some(Ok(Entry::directory(&path)));
        }

        None
    }
}

struct ReadDir {
    path: PathBuf,
    inner: std::fs::ReadDir,
}

impl ReadDir {
    fn new(path: impl Into<PathBuf>, inner: std::fs::ReadDir) -> Self {
        Self {
            path: path.into(),
            inner,
        }
    }
}

fn read_dir(path: &Path) -> Result<ReadDir, Error> {
    match std::fs::read_dir(path) {
        Ok(inner) => Ok(ReadDir::new(path, inner)),
        Err(err) => Err(Error::new(path, err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/xfs/test_data");

    mod walk {
        use super::*;
        use std::path::Path;

        fn sort_entries(
            entries: impl Iterator<Item = Result<Entry, Error>>,
        ) -> Vec<Result<Entry, Error>> {
            let mut entries: Vec<_> = entries.collect();
            entries.sort_by_key(|res| match res {
                Ok(entry) => match entry {
                    Entry::File { path } => path.clone(),
                    Entry::Directory { path } => path.clone(),
                    Entry::Symlink { path } => path.clone(),
                    Entry::Unknown { path } => path.clone(),
                },
                Err(err) => err.path.clone(),
            });
            entries
        }

        #[test]
        fn returns_dir_entry_for_empty_directory() {
            let test_dir = Path::new(TEST_DATA_DIR).join("empty_dir");

            // Git won't let us commit an empty directory.
            std::fs::create_dir_all(&test_dir).unwrap();

            let expected = vec![Ok(Entry::directory(&test_dir))];

            let actual = sort_entries(walk(&test_dir));

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }

        #[test]
        fn returns_files_in_directory() {
            let test_dir = Path::new(TEST_DATA_DIR).join("files_only");

            let expected = vec![
                Ok(Entry::directory(&test_dir)),
                Ok(Entry::file(&test_dir.join("file1.txt"))),
                Ok(Entry::file(&test_dir.join("file2.txt"))),
            ];

            let actual = sort_entries(walk(&test_dir));

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }

        #[test]
        fn returns_nested_files_and_directories() {
            let test_dir = Path::new(TEST_DATA_DIR).join("files_and_directories");

            let expected = vec![
                Ok(Entry::directory(&test_dir)),
                Ok(Entry::directory(&test_dir.join("bar"))),
                Ok(Entry::directory(&test_dir.join("bar").join("baz"))),
                Ok(Entry::file(
                    &test_dir.join("bar").join("baz").join("one.txt"),
                )),
                Ok(Entry::file(
                    &test_dir.join("bar").join("baz").join("two.txt"),
                )),
                Ok(Entry::file(&test_dir.join("bar").join("one.txt"))),
                Ok(Entry::file(&test_dir.join("bar").join("two.txt"))),
                Ok(Entry::directory(&test_dir.join("foo"))),
                Ok(Entry::file(&test_dir.join("foo").join("one.txt"))),
                Ok(Entry::file(&test_dir.join("foo").join("two.txt"))),
                Ok(Entry::file(&test_dir.join("one.txt"))),
                Ok(Entry::file(&test_dir.join("two.txt"))),
            ];

            let actual = sort_entries(walk(&test_dir));

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }
    }
}
