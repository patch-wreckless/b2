use crate::ascii;
use crate::xfs;
use std::collections::{BTreeMap, HashMap};

pub fn summarize(walk: impl Iterator<Item = Result<xfs::Entry, xfs::Error>>) -> anyhow::Result<()> {
    let mut files_by_extension: HashMap<String, Vec<String>> = HashMap::new();

    for file in walk {
        let entry = file.map_err(|err| anyhow::anyhow!(err))?;

        let file = match entry {
            xfs::Entry::Directory { .. } => continue,
            xfs::Entry::Symlink { path } => {
                anyhow::bail!(format!("symlinks are not supported ({})", path.display()))
            }
            xfs::Entry::Unknown { path } => {
                anyhow::bail!(format!("unknown entry type ({})", path.display()))
            }
            xfs::Entry::File { path } => path,
        };

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
