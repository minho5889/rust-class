//! The one owner (A4–A6): a single task holds every open lake file. No
//! other code in this crate can write to the lake — not "shouldn't": CAN'T,
//! because the `File` handles live in this function's local variables and
//! Rust has no way to reach another task's locals.
//!
//! That's the whole concurrency story. Handlers race each other freely;
//! the channel funnels their lines into one queue; this loop pops them one
//! at a time. Two writes can't interleave because there is never more than
//! one write in flight. Compare the alternative — `Mutex<File>` shared by
//! every handler — which also serializes, but couples every request's
//! latency to the disk and reintroduces the thing Rust keeps warning you
//! about: shared mutable state. Transferring ownership of the DATA to the
//! owner of the SINK is the idiomatic answer (memlens's writer-lock lesson
//! in async clothes).
//!
//! Drain-then-flush shutdown falls out of the same shape for free: when
//! every `Sender` is gone, `recv()` returns `None`, the loop ends, and we
//! flush — no shutdown flag, no cancellation token, no lost 202s (A6b).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// Run the writer task: receive compact JSONL lines until every sender is
/// dropped, append each to `<lake>/dt=<day>/events.jsonl`, flush after
/// every line, flush everything again on close. Returns how many lines hit
/// the disk (main prints it in the goodbye line).
///
/// Owns `rx` (the only receiving half) and, transitively, every open lake
/// `File` — both live and die inside this future.
pub async fn run(mut rx: mpsc::Receiver<String>, lake: PathBuf) -> u64 {
    // The per-day handle cache: open each day's file once, reuse it for
    // every later line of that day. Bounded in practice by distinct days
    // seen in one relay session — a handful, not a leak.
    let mut files: HashMap<String, File> = HashMap::new();
    let mut written = 0u64;

    // teach: THE loop of the spec. `recv().await` parks this task (costing
    // nothing) until a handler sends a line — or returns `None` forever
    // once the last `Sender` is dropped. Channel close IS the shutdown
    // signal; the `while let` ends and the drain is already done, because
    // `recv()` keeps yielding queued lines even after close.
    while let Some(line) = rx.recv().await {
        match append(&mut files, &lake, line).await {
            Ok(()) => written += 1,
            // A disk error here is past the point of no return — the 202
            // already went out. A lab tool logs it loudly and keeps
            // serving the rest of the queue; it must NOT panic (A9) and
            // must not stop draining (one bad day-dir shouldn't lose every
            // other day's events).
            Err(e) => tracing::error!("writer: dropping a line: {e}"),
        }
    }

    // Final flush on channel close (A6b). With flush-per-line this is
    // belt-and-braces, but it makes the guarantee local: *this* line, not
    // the history of the loop, is why shutdown can't strand bytes.
    for (day, file) in &mut files {
        if let Err(e) = file.flush().await {
            tracing::error!("writer: final flush of dt={day} failed: {e}");
        }
    }
    written
}

/// Append one line to its day's file, opening (and creating dirs) on first
/// encounter of that day.
async fn append(
    files: &mut HashMap<String, File>,
    lake: &Path,
    line: String,
) -> std::io::Result<()> {
    // teach: the sender deliberately dropped the day (the channel speaks
    // plain Strings — the smallest teaching shape), so the owner re-derives
    // it. The door guarantees every queued line has a comparable day; the
    // fallback below is for a world where that invariant broke. Routing
    // such a line to a visible dt=bad-ts quarantine keeps the writer total
    // (no unwrap — A9) and keeps conservation honest: better an event in a
    // wrong-looking folder you can see than one silently gone.
    let day = day_of_line(&line).unwrap_or_else(|| "bad-ts".to_owned());

    let file = match files.entry(day) {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
            let dir = lake.join(format!("dt={}", entry.key()));
            // First line of a new day: make the partition dir, then open
            // append-mode (create if missing — the shell hooks may already
            // have made today's file; O_APPEND coexists with them, see the
            // requirements' cross-process note).
            tokio::fs::create_dir_all(&dir).await?;
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("events.jsonl"))
                .await?;
            entry.insert(file)
        }
    };

    // teach: ONE write_all for line + newline, not two. The design's
    // strawman note names "two-write formatting" as exactly how a naive
    // handler tears lines even under O_APPEND; the single-owner design
    // doesn't NEED single-syscall writes to be correct, but building the
    // full record before writing is cheap hygiene — and we own the String,
    // so appending the '\n' in place allocates at most once.
    let mut record = line.into_bytes();
    record.push(b'\n');
    file.write_all(&record).await?;
    // Flush after every line (lab tool: durability over throughput —
    // tokio's File buffers internally and a sleepy handle may not have
    // pushed bytes to the OS yet). `flush` hands bytes to the kernel;
    // paranoid-durable would be `sync_all()` (fsync), which is more than
    // a localhost lab lake needs.
    file.flush().await
}

/// The partition day of a queued line: parse (it's compact JSON by
/// construction), read `ts`, apply glake's F2 day rule — the SAME
/// `day_of_ts` the door's classifier used, so door and writer can never
/// disagree about what day an event belongs to.
fn day_of_line(line: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let ts = value.get("ts")?.as_str()?;
    glake::classify::day_of_ts(ts).map(str::to_owned)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Door and writer share one day rule (glake's) — pin the derivation.
    #[test]
    fn day_of_line_matches_the_doors_day() {
        let line = r#"{"ts":"2026-07-09T01:02:03Z","event_id":"1"}"#;
        assert_eq!(day_of_line(line), Some("2026-07-09".to_owned()));
        assert_eq!(day_of_line(r#"{"ts":"nope"}"#), None);
        assert_eq!(day_of_line(r#"{"ts":1234}"#), None);
        assert_eq!(day_of_line("not json"), None);
        assert_eq!(day_of_line(r#"{"no_ts":true}"#), None);
    }
}
