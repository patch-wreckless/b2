use crossbeam::channel::{Receiver, Sender, unbounded};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use crate::ascii;

pub fn summarize(src: &Path) -> anyhow::Result<()> {
    let receiver = get_files(src);

    let mut files_by_extension: HashMap<String, Vec<String>> = HashMap::new();

    for file in receiver.iter() {
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
