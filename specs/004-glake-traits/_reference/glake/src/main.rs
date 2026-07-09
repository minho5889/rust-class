//! Thin binary: clap parses, `run` fans out to validate/stats, and THIS
//! boundary owns the exit codes (normative for v1, F5):
//!
//! - `2` — usage problems (clap's own default for bad args; our
//!   `GlakeError::Usage` for bad values) and io failures;
//! - `1` — `validate` found findings;
//! - `0` — clean.
//!
//! No unwrap/expect anywhere in the runtime paths: everything fallible
//! returns `Result<_, GlakeError>` and is mapped HERE to one clear stderr
//! line (`glake: <message>`), never a panic or a Debug dump.

use glake::classify::REQUIRED_KEYS;
use glake::cli::{Cli, Command};
use glake::error::GlakeError;
use glake::filter::Filter;
use glake::parser::{ClassifiedLine, EventParser};
use glake::tally::tally_filtered;
use glake::walk::{jsonl_files, read_file};
use std::path::PathBuf;
use std::process::ExitCode;

// The lens stays opt-in (`cargo run --features lens`) and OPTIONAL in the
// dependency tree — with default features, `cargo tree` shows no memlens
// (F11). F9's measurement runs stats under the lens once per parser.
#[cfg(feature = "lens")]
#[global_allocator]
static LENS: memlens::MemLens<std::alloc::System> = memlens::MemLens::system();

fn main() -> ExitCode {
    #[cfg(feature = "lens")]
    let _session = memlens::session("glake");

    // clap owns bad usage: it prints its own message + usage to stderr and
    // exits 2 (its default error code — pinned by tests/cli.rs).
    let cli = <Cli as clap::Parser>::parse();

    // F5: ONE place turns any library error into the user-visible line.
    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("glake: {e}");
            ExitCode::from(2)
        }
    }
}

/// Wire the parsed CLI to the library. The `Box<dyn EventParser>` built
/// from `--parser` is the crate's only dynamic-dispatch seam (F7/F13);
/// everything below takes `&dyn EventParser`.
fn run(cli: Cli) -> Result<ExitCode, GlakeError> {
    match cli.command {
        Command::Validate { path, parser } => {
            let parser = parser.build();
            validate(&jsonl_files(&path)?, parser.as_ref())
        }
        Command::Stats {
            path,
            kind,
            since,
            parser,
        } => {
            // Filter::new rejects a malformed --since value as a Usage
            // error → "glake: --since expects YYYY-MM-DD, …" + exit 2.
            let filter = Filter::new(kind, since)?;
            let parser = parser.build();
            stats(&jsonl_files(&path)?, parser.as_ref(), &filter)
        }
    }
}

/// `glake validate <path>`: report every non-event line with its location;
/// exit 1 iff anything was found (003 R1a/R1b, unchanged in v1).
fn validate(files: &[PathBuf], parser: &dyn EventParser) -> Result<ExitCode, GlakeError> {
    let mut bad = 0u64;
    for file in files {
        let content = read_file(file)?;
        for (n, line) in content.lines().enumerate() {
            match parser.classify(line, &REQUIRED_KEYS) {
                ClassifiedLine::Malformed { missing } => {
                    println!("{}:{}  missing key \"{missing}\"", file.display(), n + 1);
                    bad += 1;
                }
                // Only strict backends (--parser serde) produce this: the
                // hand scanner reports junk as a missing key instead.
                ClassifiedLine::Unparseable => {
                    println!("{}:{}  not a JSON object", file.display(), n + 1);
                    bad += 1;
                }
                ClassifiedLine::Blank | ClassifiedLine::Event { .. } => {}
            }
        }
    }
    Ok(match bad {
        0 => {
            println!("all lines well-formed");
            ExitCode::SUCCESS
        }
        n => {
            println!("{n} malformed line(s)");
            ExitCode::from(1)
        }
    })
}

/// `glake stats <path> [--type] [--since]`: the filtered tally, with F4's
/// two numbers and F2's bad-ts note when they apply.
fn stats(
    files: &[PathBuf],
    parser: &dyn EventParser,
    filter: &Filter,
) -> Result<ExitCode, GlakeError> {
    let mut contents = Vec::with_capacity(files.len());
    for file in files {
        contents.push(read_file(file)?);
    }
    let out = tally_filtered(
        parser,
        &REQUIRED_KEYS,
        contents.iter().flat_map(|c| c.lines()),
        filter,
    );

    // F4: both numbers when any filter is active; v0's exact header shape
    // when none is (an unfiltered v1 run is byte-identical to v0).
    if filter.is_active() {
        println!(
            "{} files · {} events (filtered from {})",
            files.len(),
            out.kept.events,
            out.total_events
        );
    } else {
        println!("{} files · {} events", files.len(), out.kept.events);
    }
    // F2: the one-line note, only when --since actually excluded something.
    if out.excluded_bad_ts > 0 {
        println!(
            "({} bad-ts event(s) excluded by --since)",
            out.excluded_bad_ts
        );
    }
    if out.kept.malformed > 0 {
        println!("({} malformed line(s) — run validate)", out.kept.malformed);
    }
    println!("\nby type");
    for (kind, n) in sorted(&out.kept.by_kind) {
        println!("  {kind:<24} {n:>6}");
    }
    println!("\nby day");
    for (day, n) in sorted(&out.kept.by_day) {
        println!("  {day:<24} {n:>6}");
    }
    Ok(ExitCode::SUCCESS)
}

/// Highest count first, then alphabetical — deterministic output.
fn sorted(map: &std::collections::HashMap<String, u64>) -> Vec<(&String, &u64)> {
    let mut v: Vec<_> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    v
}
