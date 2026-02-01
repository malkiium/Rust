use rayon::prelude::*;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const SIZE_LIMIT: u64 = 2 * 1024 * 1024 * 1024; // 2 GB
const START_PATH: &str = "C:\\Users\\eliha"; // Change this as needed

fn get_size(path: &Path) -> u64 {
    if path.is_file() {
        match fs::metadata(path) {
            Ok(meta) => meta.len(),
            Err(_) => 0,
        }
    } else if path.is_dir() {
        let mut total_size = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                total_size += get_size(&entry.path());
            }
        }
        total_size
    } else {
        0
    }
}

fn process_entry(entry: DirEntry, pending_dirs: &Arc<Mutex<Vec<PathBuf>>>, output: &Arc<Mutex<()>>) {
    let path = entry.path();
    let size = get_size(&path);

    if size > SIZE_LIMIT {
        let item_type = if path.is_file() { "File" } else { "Folder" };
        let gb_size = size as f64 / (1024.0 * 1024.0 * 1024.0);
        let _lock = output.lock().unwrap();
        println!("Large {}: {} ({:.2} GB)", item_type, path.display(), gb_size);
    }

    if path.is_dir() {
        if let Ok(children) = fs::read_dir(&path) {
            let mut dirs = pending_dirs.lock().unwrap();
            for child in children.flatten() {
                dirs.push(child.path());
            }
        }
    }
}

fn scan_directory(start_path: &Path) -> io::Result<()> {
    if !start_path.is_dir() {
        println!("Invalid directory: {}", start_path.display());
        return Ok(());
    }

    let pending_dirs = Arc::new(Mutex::new(vec![start_path.to_path_buf()]));
    let print_lock = Arc::new(Mutex::new(()));

    while !pending_dirs.lock().unwrap().is_empty() {
        let current_batch: Vec<PathBuf> = pending_dirs.lock().unwrap().drain(..).collect();

        current_batch.par_iter().for_each(|path| {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    process_entry(entry, &pending_dirs, &print_lock);
                }
            }
        });
    }

    Ok(())
}

fn main() {
    let start_path = Path::new(START_PATH);
    if let Err(e) = scan_directory(start_path) {
        eprintln!("Error scanning directory: {}", e);
    }
}
