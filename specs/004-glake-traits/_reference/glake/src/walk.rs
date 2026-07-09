//! Sitting B (003) → Sitting H (004): one file → just it; a folder → every
//! `*.jsonl` inside, recursively (the lake nests as `dt=YYYY-MM-DD/…`).
//!
//! v1 change (F5): the io layer now speaks [`GlakeError`], not bare
//! `io::Error`. Every `std::fs` call is wrapped with `.map_err(|e|
//! GlakeError::io(path, e))` so the error that reaches `main` carries the
//! path that actually failed — deep inside the recursion, not just the
//! root the user typed. `?` still bubbles everything up; `main` prints one
//! line and exits 2.

use crate::error::GlakeError;
use std::path::{Path, PathBuf};

/// Every `.jsonl` file under `path` (or `path` itself if it's a file,
/// taken as-is — 003 R3b), sorted for deterministic output.
pub fn jsonl_files(path: &Path) -> Result<Vec<PathBuf>, GlakeError> {
    // missing/unreadable → Err with path context → main → exit 2
    let meta = std::fs::metadata(path).map_err(|e| GlakeError::io(path, e))?;
    let mut found = Vec::new();
    if meta.is_file() {
        found.push(path.to_path_buf());
    } else {
        collect(path, &mut found)?; // 003 R3a: recurse
        found.sort();
    }
    Ok(found)
}

fn collect(dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), GlakeError> {
    for entry in std::fs::read_dir(dir).map_err(|e| GlakeError::io(dir, e))? {
        let path = entry.map_err(|e| GlakeError::io(dir, e))?.path();
        if path.is_dir() {
            collect(&path, found)?;
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            found.push(path);
        }
    }
    Ok(())
}

/// `std::fs::read_to_string` with the crate's error type — the same
/// path-context pattern, shared by both commands.
pub fn read_file(path: &Path) -> Result<String, GlakeError> {
    std::fs::read_to_string(path).map_err(|e| GlakeError::io(path, e))
}
