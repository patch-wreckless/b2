use std::collections::{BTreeMap, HashMap};

use crate::ascii;
use crate::files;

pub fn summarize(file_paths: impl Iterator<Item = files::FilePathItem>) -> anyhow::Result<()> {
    let mut files_by_extension: HashMap<String, Vec<String>> = HashMap::new();

    for file in file_paths {
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
