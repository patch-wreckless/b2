use std::fmt::{self, Display};
use std::path::PathBuf;

use crate::xfs;
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

    fn symlink(path: impl Into<PathBuf>) -> Self {
        Self::new(path, "symlinks are not supported")
    }

    fn unknown_entry_type(path: impl Into<PathBuf>) -> Self {
        Self::new(path, "unknown entry type")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (path: {})", self.message, self.path.display())
    }
}

impl From<crate::xfs::Error> for Error {
    fn from(value: crate::xfs::Error) -> Self {
        Self::new(value.path, value.message)
    }
}

impl std::error::Error for Error {}

// The type of item...
pub type FilePathItem = Result<PathBuf, Error>;

pub trait IntoFilePaths: Iterator {
    fn into_file_paths(self) -> IntoFilePathsIter<Self>
    where
        Self: Sized;
}

impl<I> IntoFilePaths for I
where
    I: Iterator<Item = xfs::WalkItem>,
{
    fn into_file_paths(self) -> IntoFilePathsIter<Self> {
        IntoFilePathsIter { inner: self }
    }
}

pub struct IntoFilePathsIter<I> {
    inner: I,
}

impl<I> Iterator for IntoFilePathsIter<I>
where
    I: Iterator<Item = xfs::WalkItem>,
{
    type Item = FilePathItem;

    fn next(&mut self) -> Option<Self::Item> {
        for res in self.inner.by_ref() {
            match res {
                Ok(entry) => match entry {
                    xfs::Entry::File { path } => return Some(Ok(path)),
                    xfs::Entry::Directory { .. } => continue,
                    xfs::Entry::Symlink { path } => return Some(Err(Error::symlink(path))),
                    xfs::Entry::Unknown { path } => {
                        return Some(Err(Error::unknown_entry_type(path)));
                    }
                },
                Err(err) => return Some(Err(err.into())),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod walk {
        use super::*;

        fn sort_file_paths(entries: impl Iterator<Item = FilePathItem>) -> Vec<FilePathItem> {
            let mut entries: Vec<_> = entries.collect();
            entries.sort_by_key(|res| match res {
                Ok(path) => path.clone(),
                Err(err) => err.path.clone(),
            });
            entries
        }

        #[test]
        fn converts_file_entries_to_paths() {
            let expected = vec![Ok(PathBuf::from("/foo/bar")), Ok(PathBuf::from("/foo/baz"))];

            let input = vec![
                Ok(xfs::Entry::file(PathBuf::from("/foo/bar"))),
                Ok(xfs::Entry::file(PathBuf::from("/foo/baz"))),
            ];

            let actual = sort_file_paths(input.into_iter().into_file_paths());

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }

        #[test]
        fn skips_directories() {
            let expected = vec![Ok(PathBuf::from("/foo/bar/baz"))];

            let input = vec![
                Ok(xfs::Entry::directory(PathBuf::from("/foo"))),
                Ok(xfs::Entry::directory(PathBuf::from("/foo/bar"))),
                Ok(xfs::Entry::file(PathBuf::from("/foo/bar/baz"))),
            ];

            let actual = sort_file_paths(input.into_iter().into_file_paths());

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }

        #[test]
        fn emits_error_for_symlink() {
            let expected = vec![
                Ok(PathBuf::from("/bar")),
                Err(Error::symlink("/baz")),
                Ok(PathBuf::from("/foo")),
            ];

            let input = vec![
                Ok(xfs::Entry::file(PathBuf::from("/bar"))),
                Ok(xfs::Entry::symlink(PathBuf::from("/baz"))),
                Ok(xfs::Entry::file(PathBuf::from("/foo"))),
            ];

            let actual = sort_file_paths(input.into_iter().into_file_paths());

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }

        #[test]
        fn emits_error_for_unknown_entry_type() {
            let expected = vec![
                Ok(PathBuf::from("/bar")),
                Err(Error::unknown_entry_type("/baz")),
                Ok(PathBuf::from("/foo")),
            ];

            let input = vec![
                Ok(xfs::Entry::file(PathBuf::from("/bar"))),
                Ok(xfs::Entry::unknown(PathBuf::from("/baz"))),
                Ok(xfs::Entry::file(PathBuf::from("/foo"))),
            ];

            let actual = sort_file_paths(input.into_iter().into_file_paths());

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }

        #[test]
        fn emits_error_for_entry_error() {
            let expected = vec![
                Err(Error::new("/bar", "whoops")),
                Ok(PathBuf::from("/baz")),
                Ok(PathBuf::from("/foo")),
            ];

            let input = vec![
                Ok(xfs::Entry::file(PathBuf::from("/baz"))),
                Ok(xfs::Entry::file(PathBuf::from("/foo"))),
                Err(xfs::Error::new("/bar", "whoops")),
            ];

            let actual = sort_file_paths(input.into_iter().into_file_paths());

            assert_eq!(expected.len(), actual.len());
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(expected, &actual[i])
            }
        }
    }
}
