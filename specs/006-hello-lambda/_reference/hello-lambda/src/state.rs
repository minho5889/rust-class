//! Per-instance state (H3) — relay's counters, kept verbatim ON PURPOSE,
//! because their *meaning* changed underneath them (T4).
//!
//! In relay, `Counters` lived in an `Arc` shared by every handler task in
//! the one process: `/healthz` described the whole service. Here the same
//! three `AtomicU64`s live in a process-global `OnceLock` — and a Lambda
//! *process* is one **warm instance** that handles ONE request at a time.
//! So these counters are:
//!
//! - **per warm instance** — two concurrent curls can hit two instances and
//!   see two disjoint histories (`instance` + `started` make that visible);
//! - **reset on cold start** — the platform kills idle instances whenever
//!   it likes, and the counters die with the process;
//! - **never shared** — there is no cross-instance view. Any "just keep it
//!   in memory" plan on Lambda inherits all three properties, silently.
//!
//! The atomics are strictly *heavier than needed* on this platform (one
//! request at a time = no contention, ever) — kept because they're the same
//! code the learner wrote in 005, and because `Relaxed` atomic loads/adds
//! compile to plain loads/adds on an uncontended path anyway (zero-cost in
//! the way Rust likes: you pay only when contended, and here you never are).

use std::process;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Everything one warm instance knows about itself. Born on first touch
/// (or `main`'s eager touch at init — see main.rs), dies with the process.
#[derive(Debug)]
pub struct Instance {
    /// Instance id. Two sources, in order (H7 rev 2):
    ///
    /// 1. **On Lambda**: `$AWS_LAMBDA_LOG_STREAM_NAME`, verbatim — the
    ///    platform mints one log stream per instance, so it's a unique,
    ///    *free* identity that also lets you jump from a `/healthz` answer
    ///    straight to that instance's CloudWatch stream on deploy day.
    /// 2. **Locally** (tests, `cargo lambda watch`): `i-` + init-time
    ///    `SystemTime` nanos mixed with the process id through one round of
    ///    SplitMix64 (e.g. `i-9f3ac81b52d07e64`). Not cryptographic and
    ///    doesn't need to be — it only has to make two warm instances
    ///    *visibly* different in `/healthz` output (T4). No `rand` dep: the
    ///    entropy of "when did this process start, at nanosecond grain" is
    ///    plenty for a lab id.
    pub id: String,
    /// Cold-start time, RFC3339-ish UTC (`2026-07-10T22:04:31Z`), computed
    /// from `SystemTime` by hand (see [`rfc3339_utc`]) — a date crate is
    /// not worth its binary weight for one string formatted once.
    pub started: String,
    /// Every `POST /events` that reached the handler — on THIS instance.
    pub received: AtomicU64,
    /// Bodies that passed the door and were emitted to stdout (202).
    pub accepted: AtomicU64,
    /// Bodies turned away with a 400.
    pub rejected: AtomicU64,
}

static INSTANCE: OnceLock<Instance> = OnceLock::new();

/// The one instance record, created on first access.
///
/// teach: `OnceLock` is the standard-library answer to "a global that is
/// initialized exactly once, thread-safely, with no `unsafe` and no macro
/// crate" — the modern replacement for `lazy_static!`. In relay this job
/// was done by `Arc<Counters>` threaded through axum state; here there is
/// no router to carry state for us, and a process-global is an *honest*
/// shape: the state really is "whatever this one process remembers".
pub fn instance() -> &'static Instance {
    INSTANCE.get_or_init(Instance::new)
}

impl Instance {
    fn new() -> Instance {
        // teach: `duration_since` returns Err only if the clock reads
        // earlier than the epoch (pre-1970). `unwrap_or` gives that
        // impossible branch a value instead of a panic — H7 bans
        // unwrap()/expect() in this crate, and the habit costs one line.
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        let seed = (since_epoch.as_nanos() as u64) ^ u64::from(process::id());
        // The platform's per-instance identity when present, our own when
        // not — see the field docs for both shapes.
        let id = match std::env::var("AWS_LAMBDA_LOG_STREAM_NAME") {
            Ok(stream) if !stream.is_empty() => stream,
            _ => format!("i-{:016x}", splitmix64(seed)),
        };
        Instance {
            id,
            started: rfc3339_utc(since_epoch.as_secs()),
            received: AtomicU64::new(0),
            accepted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        }
    }
}

/// One round of SplitMix64 — a tiny, well-known bit mixer. The raw seed
/// (nanos ^ pid) has most of its entropy in the low bits; mixing spreads it
/// so ids don't share a long common prefix within one session.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Seconds-since-epoch → `YYYY-MM-DDTHH:MM:SSZ` (UTC, no leap seconds —
/// "rfc3339-ish", exactly as honest as the envelope `ts` the hooks emit).
fn rfc3339_utc(secs: u64) -> String {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Days-since-epoch → (year, month, day), Howard Hinnant's civil-from-days
/// algorithm (public domain, the one every date library uses underneath).
/// Restricted to `u64` days ≥ 0: this only ever formats "now", and now is
/// comfortably after 1970.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468; // shift epoch: 1970-01-01 → 0000-03-01
    let era = z / 146_097; // 400-year eras
    let doe = z % 146_097; // day-of-era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day-of-year, Mar-1-based
    let mp = (5 * doy + 2) / 153; // month, Mar=0
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    /// The hand-rolled clock math, pinned against known instants (the third
    /// one cross-checked with `date -u -d @1783641632`; it is also the
    /// second-grain prefix of a real event_id in the live lake).
    #[test]
    fn rfc3339_utc_known_instants() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_utc(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(rfc3339_utc(951_782_400), "2000-02-29T00:00:00Z"); // leap day
        assert_eq!(rfc3339_utc(1_783_641_632), "2026-07-10T00:00:32Z");
    }

    /// OnceLock semantics: every call sees the SAME instance — same id,
    /// same start time, same counters (bump through one handle, observe
    /// through another).
    #[test]
    fn instance_is_initialized_exactly_once() {
        let a = instance();
        let b = instance();
        assert!(std::ptr::eq(a, b), "OnceLock must hand out one value");
        // The id source depends on the environment this test runs in:
        // under `cargo test` the Lambda env var is normally absent (the
        // local fallback), but the assertion covers both honestly.
        match std::env::var("AWS_LAMBDA_LOG_STREAM_NAME") {
            Ok(stream) if !stream.is_empty() => assert_eq!(a.id, stream),
            _ => {
                assert!(a.id.starts_with("i-"), "id shape: {}", a.id);
                assert_eq!(a.id.len(), 18, "i- plus 16 hex digits");
            }
        }
        assert_eq!(a.started.len(), 20, "YYYY-MM-DDTHH:MM:SSZ");
        let before = a.received.load(Ordering::Relaxed);
        b.received.fetch_add(1, Ordering::Relaxed);
        assert_eq!(a.received.load(Ordering::Relaxed), before + 1);
    }
}
