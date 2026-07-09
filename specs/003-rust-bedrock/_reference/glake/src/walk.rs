//! Sitting B: one file → just it; a folder → every `*.jsonl` inside,
//! recursively (the lake nests as `dt=YYYY-MM-DD/…`). `Result` + `?` all
//! the way down — errors bubble to main, which reports and exits 2 (R6).

use std::io;
use std::path::{Path, PathBuf};

pub fn jsonl_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let meta = std::fs::metadata(path)?; // missing/unreadable → Err → R6
    let mut found = Vec::new();
    if meta.is_file() {
        found.push(path.to_path_buf()); // R3b: a file is taken as-is
    } else {
        collect(path, &mut found)?; // R3a: recurse
        found.sort(); // deterministic output order
    }
    Ok(found)
}

fn collect(dir: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, found)?;
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            found.push(path);
        }
    }
    Ok(())
}
