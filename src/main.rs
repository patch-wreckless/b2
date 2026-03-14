use std::env;
use std::fs;
use std::path::Path;

fn count_files(path: &Path) -> u64 {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_files(&path);
            }
        }
    }

    count
}

fn main() {
    let dir = env::args().nth(1).unwrap_or_else(|| ".".to_string());

    let path = Path::new(&dir);
    let total = count_files(path);

    println!("Total files: {}", total);
}
