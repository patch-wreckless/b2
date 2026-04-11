use crate::ascii;
use crossbeam::channel::Receiver;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Display};
use std::path::PathBuf;

/// An error encountered while enumerating file system entries.
#[derive(Debug)]
pub struct EntryError {
    msg: String,
}

impl EntryError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "entry error: {}", self.msg)
    }
}

impl std::error::Error for EntryError {}

pub fn summarize2(files: Receiver<std::result::Result<PathBuf, EntryError>>) -> anyhow::Result<()> {
    let mut files_by_extension: HashMap<String, Vec<String>> = HashMap::new();

    for file in files.iter() {
        let file = file.map_err(|err| anyhow::anyhow!(err))?;

        let extension = match file.extension() {
            Some(ext) => ext.to_string_lossy(),
            None => "".into(),
        };
        files_by_extension
            .entry(extension.to_string())
            .or_default()
            .push(ascii::escape(
                file.as_os_str().as_encoded_bytes().iter().copied(),
            ));
    }

    let mut sorted_entries: Vec<_> = files_by_extension.iter().collect();
    sorted_entries.sort_by_key(|&(key, _)| key);

    let mut sorted_files_by_extension = BTreeMap::new();

    for (key, mut value) in files_by_extension {
        value.sort();
        sorted_files_by_extension.insert(key, value);
    }

    serde_yaml::to_writer(std::io::stdout(), &sorted_files_by_extension)?;

    Ok(())
}
