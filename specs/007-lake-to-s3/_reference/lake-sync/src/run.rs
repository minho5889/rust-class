//! The impure shell: execute a [`Plan`] against any store, with bounded
//! fan-out (S9, T4) and partial-progress honesty (S5).
//!
//! The concurrency shape is 005's bounded-channel lesson re-cast for
//! fan-out: instead of N producers feeding one writer through a bounded
//! channel, one producer (the read loop) feeds N uploads through a
//! bounded *permit pool*. Same law — backpressure at the edge, never
//! unbounded buffering in the middle.
//!
//! ```text
//!   for each Upload ──read body──► acquire permit (≤4) ──spawn──► put
//!        (sequential, one reused        │                    (JoinSet)
//!         line buffer — T4)             └── waits HERE when 4 in flight
//! ```
//!
//! Why the permit is acquired in the LOOP and moved into the task, not
//! acquired inside the task: both spellings bound the puts, but only this
//! one also bounds MEMORY — the loop can't read body #6 until a permit
//! frees, so at most `limit` bodies + 1 are ever resident. Acquire-inside-
//! the-task would happily buffer the whole lake into spawned-but-waiting
//! tasks. (The fake's gauge can only prove the put bound, S9; the memory
//! bound is this paragraph and the code's shape.)

use std::sync::Arc;

use lake_store::ObjectStore;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::SyncError;
use crate::plan::{Plan, Upload};

/// S9's bound: at most this many puts in flight (design decision row —
/// `JoinSet` + `Semaphore(4)`).
pub const MAX_IN_FLIGHT: usize = 4;

/// What actually happened — successes AND failures, because S5's rule is
/// that partial progress is *stated, not hidden*.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Keys whose put succeeded, sorted (completion order is scheduler
    /// noise; sorted output is diffable output).
    pub uploaded: Vec<String>,
    /// (key, error message) per failed put, sorted by key — the object is
    /// NAMED, per S5.
    pub failed: Vec<(String, String)>,
    /// Keys the plan skipped (carried through so one value can render the
    /// whole story).
    pub skipped: Vec<String>,
}

impl Outcome {
    /// stdout's story. Quiet on success (`uploaded N, skipped M` — the
    /// requirements transcript's exact shape); on failure it also NAMES
    /// every uploaded key first, because partial progress must be
    /// reported, not implied (S5).
    pub fn stdout_report(&self) -> String {
        let mut out = String::new();
        if !self.failed.is_empty() {
            for key in &self.uploaded {
                out.push_str(&format!("uploaded {key}\n"));
            }
        }
        out.push_str(&format!(
            "uploaded {}, skipped {}{}\n",
            self.uploaded.len(),
            self.skipped.len(),
            match self.failed.len() {
                0 => String::new(),
                n => format!(", FAILED {n}"),
            }
        ));
        out
    }

    /// stderr's story: one line per failed put, object named (S5).
    pub fn stderr_report(&self) -> String {
        self.failed
            .iter()
            .map(|(_, message)| format!("lake-sync: {message}\n"))
            .collect()
    }

    /// The exit-code half of glake's discipline: 0 clean, 1 if any put
    /// failed (2 — usage/io — is decided in main, before an Outcome
    /// exists).
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.failed.is_empty())
    }
}

/// Upload everything `plan.upload` names, ≤ `MAX_IN_FLIGHT` at a time.
///
/// `Arc<S>`, not `&S` — the ownership shape `JoinSet::spawn` forces
/// (design.md pins it): a spawned task may outlive any borrow the
/// compiler can see, so every task gets its own owned handle (`Arc`
/// clone), its own owned key and body (moved in), and its own permit
/// (`acquire_owned`, the Arc-flavored acquire that can move into a
/// `'static` task). This function is generic over `S: ObjectStore` — the
/// F13 monomorphization pattern; the fake and the real store both
/// compile their own copy, no `dyn` (S7).
pub async fn run<S: ObjectStore>(store: Arc<S>, plan: Plan) -> Result<Outcome, SyncError> {
    let semaphore = Arc::new(Semaphore::new(MAX_IN_FLIGHT));
    let mut tasks: JoinSet<(String, Result<(), lake_store::StoreError>)> = JoinSet::new();
    let mut outcome = Outcome {
        skipped: plan.skip,
        ..Outcome::default()
    };

    // ONE line buffer for every body read in the whole run (T4): the
    // reads are sequential (this loop), so one String's capacity gets
    // reused file after file — see plan.rs's honesty box for what is
    // streamed (lines through this buffer) vs accumulated (each PUT body).
    let mut line_buffer = String::new();

    for Upload { key, path, .. } in plan.upload {
        let body = crate::plan::read_body(&path, &mut line_buffer)?;

        // Backpressure lives HERE: with MAX_IN_FLIGHT puts pending, this
        // await parks the loop before the next read. acquire_owned needs
        // the Arc'd semaphore (a plain acquire borrows, and borrows can't
        // enter spawned tasks). The Err arm is unreachable-by-
        // construction — acquire fails only on a closed semaphore and
        // nothing here closes it — but the ban on unwrap is absolute, so
        // it degrades into a named failure instead of a panic.
        let permit = match Arc::clone(&semaphore).acquire_owned().await {
            Ok(permit) => permit,
            Err(closed) => {
                // Message spelled to name the object (S5), like every
                // other failure line.
                let message = format!("cannot put object {key:?}: semaphore closed ({closed})");
                outcome.failed.push((key, message));
                continue;
            }
        };

        let store = Arc::clone(&store);
        tasks.spawn(async move {
            // The permit rides inside the task and drops — releasing its
            // slot — only when the put resolves; `_permit = permit` (not
            // `_ = permit`) because a bare `_` would drop it immediately
            // and unbound the whole thing.
            let _permit = permit;
            let result = store.put(key.clone(), body).await;
            (key, result)
        });
    }

    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok((key, Ok(()))) => outcome.uploaded.push(key),
            // The error already names the key (StoreError::Put carries
            // it); keep the pair anyway so callers can match keys without
            // parsing messages.
            Ok((key, Err(e))) => outcome.failed.push((key, e.to_string())),
            // A JoinError means a put task panicked or was aborted —
            // nothing here aborts, and puts don't panic; reported, not
            // swallowed, if it ever happens.
            Err(join_error) => outcome.failed.push((
                "<unknown>".to_owned(),
                format!("upload task died: {join_error}"),
            )),
        }
    }

    outcome.uploaded.sort();
    outcome.failed.sort();
    Ok(outcome)
}
