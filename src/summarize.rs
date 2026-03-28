use crossbeam::channel::{Receiver, Sender, unbounded};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

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
            .push(file.to_string_lossy().to_string());
    }

    let mut sorted_entries: Vec<_> = files_by_extension.iter().collect();
    sorted_entries.sort_by_key(|&(key, _)| key);

    for (extension, values) in sorted_entries {
        let extension = match extension.len() {
            0 => "''".to_string(),
            _ => extension.to_string(),
        };
        println!("{}:", extension);
        let mut values = values.iter().collect::<Vec<_>>();
        values.sort();
        for value in values {
            println!("  - {}", value);
        }
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
