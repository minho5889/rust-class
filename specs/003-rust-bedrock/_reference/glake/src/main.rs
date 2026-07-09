//! Thin binary: parse args, fan out to validate/stats, own the exit codes.
//! No unwrap/expect anywhere in the logic (L4): everything fallible returns
//! Result and is handled at this boundary.

use glake::classify::{Line, classify};
use glake::tally::tally;
use glake::walk::jsonl_files;
use std::path::Path;
use std::process::ExitCode;

// L6: the lens is opt-in (`cargo run --features lens`) and OPTIONAL in the
// dependency tree — with default features, `cargo tree` shows std only (R4).
#[cfg(feature = "lens")]
#[global_allocator]
static LENS: memlens::MemLens<std::alloc::System> = memlens::MemLens::system();

fn main() -> ExitCode {
    #[cfg(feature = "lens")]
    let _session = memlens::session("glake");

    let args: Vec<String> = std::env::args().collect();
    let (cmd, path) = match (args.get(1).map(String::as_str), args.get(2)) {
        (Some(c @ ("validate" | "stats")), Some(p)) => (c, p),
        _ => {
            eprintln!("usage: glake <validate|stats> <path>");
            return ExitCode::from(2);
        }
    };

    let files = match jsonl_files(Path::new(path)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("glake: {path}: {e}");
            return ExitCode::from(2);
        }
    };

    match cmd {
        "validate" => validate(&files),
        _ => stats(&files),
    }
}

fn validate(files: &[std::path::PathBuf]) -> ExitCode {
    let mut bad = 0u64;
    for file in files {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("glake: {}: {e}", file.display());
                return ExitCode::from(2);
            }
        };
        for (n, line) in content.lines().enumerate() {
            if let Line::Malformed { missing } = classify(line) {
                println!("{}:{}  missing key \"{missing}\"", file.display(), n + 1);
                bad += 1;
            }
        }
    }
    match bad {
        0 => {
            println!("all lines well-formed");
            ExitCode::SUCCESS
        }
        n => {
            println!("{n} malformed line(s)");
            ExitCode::from(1) // R1b: non-zero iff something was malformed
        }
    }
}

fn stats(files: &[std::path::PathBuf]) -> ExitCode {
    let mut contents = Vec::new();
    for file in files {
        match std::fs::read_to_string(file) {
            Ok(c) => contents.push(c),
            Err(e) => {
                eprintln!("glake: {}: {e}", file.display());
                return ExitCode::from(2);
            }
        }
    }
    let stats = tally(contents.iter().flat_map(|c| c.lines()));

    println!("{} files · {} events", files.len(), stats.events);
    if stats.malformed > 0 {
        println!("({} malformed line(s) — run validate)", stats.malformed);
    }
    println!("\nby type");
    for (kind, n) in sorted(&stats.by_kind) {
        println!("  {kind:<24} {n:>6}");
    }
    println!("\nby day");
    for (day, n) in sorted(&stats.by_day) {
        println!("  {day:<24} {n:>6}");
    }
    ExitCode::SUCCESS
}

/// Highest count first, then alphabetical — deterministic output.
fn sorted(map: &std::collections::HashMap<String, u64>) -> Vec<(&String, &u64)> {
    let mut v: Vec<_> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    v
}
