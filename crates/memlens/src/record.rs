//! The trace recorder: where every heap event becomes a JSONL line.
//!
//! ## The three tricks (teaching notes)
//!
//! 1. **The tracer must not trace itself.** Writing an event *allocates*
//!    (JSON strings, buffer growth) — and those allocations re-enter the
//!    allocator. A thread-local `IN_LENS` flag makes the lens transparent
//!    while the recorder runs. The flag is const-initialized and accessed
//!    via `try_with`, falling back to "forward, don't record" during thread
//!    init/teardown when TLS is unavailable — lost events, never a panic
//!    (R7a).
//! 2. **`seq` is assigned inside the writer lock**, so file order ≡ seq
//!    order by construction. (The "obvious" lock-free `AtomicU64` outside
//!    the lock lets two threads write out of order — caught by the design
//!    audit before a line of this file existed.)
//! 3. **The allocator path is infallible.** Every failure — poisoned lock,
//!    unopenable sink, full disk — degrades to "count a dropped event and
//!    move on". The next successful write emits one `memlens.loss` marker
//!    carrying the count (R7b), so the trace is honest about its own gaps.

use crate::event::{AllocPayload, LossPayload, MetaPayload, ReallocPayload, TraceEvent};
use std::cell::Cell;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Test hook (R7 failure-injection): when set, every sink write fails as if
/// the disk were full. Not part of the public API.
#[doc(hidden)]
pub static FAIL_WRITES: AtomicBool = AtomicBool::new(false);

/// Events dropped since the last successful loss-marker write (R7b).
static DROPPED: AtomicU64 = AtomicU64::new(0);

static SINK: Mutex<Option<Sink>> = Mutex::new(None);

struct Sink {
    out: BufWriter<File>,
    /// The trace's clock: monotonic, assigned ONLY while holding the lock.
    seq: u64,
    /// Total envelope lines written (drives periodic flush + event_id).
    lines: u64,
    session_id: String,
    pid: u32,
}

thread_local! {
    static IN_LENS: Cell<bool> = const { Cell::new(false) };
}

/// Try to enter the recorder on this thread. `false` means either we are
/// already inside it (reentrant allocation — must not be recorded) or TLS is
/// unavailable (thread init/teardown) — in both cases the caller just
/// forwards without recording.
fn enter_lens() -> bool {
    IN_LENS
        .try_with(|c| {
            if c.get() {
                false
            } else {
                c.set(true);
                true
            }
        })
        .unwrap_or(false)
}

fn exit_lens() {
    let _ = IN_LENS.try_with(|c| c.set(false));
}

pub(crate) fn record_alloc(addr: usize, size: usize, align: usize) {
    if !enter_lens() {
        return;
    }
    write_op(|seq| {
        TraceEvent::Alloc(AllocPayload {
            addr: fmt_addr(addr),
            size,
            align,
            seq,
            label: None,
        })
    });
    exit_lens();
}

pub(crate) fn record_dealloc(addr: usize, size: usize, align: usize) {
    if !enter_lens() {
        return;
    }
    write_op(|seq| {
        TraceEvent::Dealloc(AllocPayload {
            addr: fmt_addr(addr),
            size,
            align,
            seq,
            label: None,
        })
    });
    exit_lens();
}

pub(crate) fn record_realloc(
    old_addr: usize,
    new_addr: usize,
    old_size: usize,
    new_size: usize,
    align: usize,
) {
    if !enter_lens() {
        return;
    }
    write_op(|seq| {
        TraceEvent::Realloc(ReallocPayload {
            old_addr: fmt_addr(old_addr),
            new_addr: fmt_addr(new_addr),
            old_size,
            new_size,
            align,
            seq,
        })
    });
    exit_lens();
}

/// Record a scope/marker event (used by the teaching macros, bolt 1.4).
#[expect(
    dead_code,
    reason = "wired up by the lens_scope!/marker macros in bolt 1.4"
)]
pub(crate) fn record_scoped(make: impl FnOnce(u64) -> TraceEvent) {
    if !enter_lens() {
        return;
    }
    write_op(make);
    exit_lens();
}

/// Begin a traced session: emits the `memlens.meta` header and returns a
/// guard whose `Drop` writes any pending loss marker and flushes the sink.
#[must_use]
pub fn session(program: &str) -> LensSession {
    if enter_lens() {
        let meta = TraceEvent::Meta(MetaPayload {
            program: program.to_owned(),
            pid: std::process::id(),
            started_at: rfc3339_now(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        });
        with_sink(|sink| {
            let _ = write_line(sink, &meta);
        });
        exit_lens();
    }
    LensSession { _priv: () }
}

/// Flush buffered trace lines to disk. Harness/test utility — the allocator
/// path itself never needs it (periodic + drop + atexit flushes cover
/// normal operation), but anything reading the trace file while the traced
/// program is still alive must flush first or risk torn lines.
pub fn flush() {
    if !enter_lens() {
        return;
    }
    with_sink(|sink| {
        let _ = sink.out.flush();
    });
    exit_lens();
}

/// Session guard. Dropping it finalizes the trace (loss marker + flush) —
/// a live demonstration of deterministic `Drop`-based cleanup.
pub struct LensSession {
    _priv: (),
}

impl Drop for LensSession {
    fn drop(&mut self) {
        if !enter_lens() {
            return;
        }
        with_sink(|sink| {
            write_pending_loss(sink);
            let _ = sink.out.flush();
        });
        exit_lens();
    }
}

/// Run `f` with the sink, initializing it on first use. All failures are
/// absorbed (the allocator path must be infallible).
fn with_sink(f: impl FnOnce(&mut Sink)) {
    let Ok(mut guard) = SINK.lock() else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    };
    if guard.is_none() {
        *guard = init_sink();
    }
    match guard.as_mut() {
        Some(sink) => f(sink),
        None => {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Assign the next seq under the lock and write the event; count a drop on
/// any failure. A failed write consumes its seq — gaps are fine, reuse is
/// not (R1 wants strictly increasing, not dense).
fn write_op(make: impl FnOnce(u64) -> TraceEvent) {
    with_sink(|sink| {
        write_pending_loss(sink);
        sink.seq += 1;
        let event = make(sink.seq);
        if write_line(sink, &event).is_err() {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// If events were dropped, spend one seq on a loss marker (R7b). On failure
/// the count is restored — the marker will be retried on the next write.
fn write_pending_loss(sink: &mut Sink) {
    let pending = DROPPED.swap(0, Ordering::Relaxed);
    if pending == 0 {
        return;
    }
    sink.seq += 1;
    let loss = TraceEvent::Loss(LossPayload {
        dropped: pending,
        seq: sink.seq,
    });
    if write_line(sink, &loss).is_err() {
        DROPPED.fetch_add(pending, Ordering::Relaxed);
    }
}

fn write_line(sink: &mut Sink, event: &TraceEvent) -> std::io::Result<()> {
    if FAIL_WRITES.load(Ordering::Relaxed) {
        return Err(std::io::Error::other("injected write failure (test hook)"));
    }
    let payload = event
        .payload_json()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    sink.lines += 1;
    writeln!(
        sink.out,
        r#"{{"event_id":"{pid}-{n}","ts":"{ts}","session_id":"{sid}","spec_id":null,"actor":"memlens","event_type":"{ty}","schema_version":1,"payload":{payload}}}"#,
        pid = sink.pid,
        n = sink.lines,
        ts = rfc3339_now(),
        sid = sink.session_id,
        ty = event.event_type(),
    )?;
    // Bounded staleness without per-event syscalls: flush every 256 lines
    // (plus on session drop and at process exit).
    if sink.lines.is_multiple_of(256) {
        sink.out.flush()?;
    }
    Ok(())
}

/// Open the sink. Path: `MEMLENS_TRACE` env var, else
/// `datalake/raw-local/traces/dt=YYYY-MM-DD/<MEMLENS_PROGRAM|unnamed>-<pid>.jsonl`
/// under the current directory (the lake's partition convention).
fn init_sink() -> Option<Sink> {
    let pid = std::process::id();
    let path = std::env::var_os("MEMLENS_TRACE").map_or_else(
        || {
            let program = std::env::var("MEMLENS_PROGRAM").unwrap_or_else(|_| "unnamed".into());
            let date = &rfc3339_now()[..10];
            std::path::PathBuf::from(format!(
                "datalake/raw-local/traces/dt={date}/{program}-{pid}.jsonl"
            ))
        },
        std::path::PathBuf::from,
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;

    // Backup flush when the program exits without dropping its LensSession.
    static ATEXIT_REGISTERED: AtomicBool = AtomicBool::new(false);
    if !ATEXIT_REGISTERED.swap(true, Ordering::Relaxed) {
        extern "C" fn flush_at_exit() {
            if let Ok(mut guard) = SINK.lock()
                && let Some(sink) = guard.as_mut()
            {
                let _ = sink.out.flush();
            }
        }
        // SAFETY: `flush_at_exit` is a valid `extern "C" fn()` with no
        // preconditions; registering it with the C runtime is sound, and it
        // only takes a std Mutex and flushes a BufWriter — both fine after
        // main returns.
        unsafe {
            libc::atexit(flush_at_exit);
        }
    }

    Some(Sink {
        out: BufWriter::new(file),
        seq: 0,
        lines: 0,
        session_id: std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| "memlens".into()),
        pid,
    })
}

fn fmt_addr(addr: usize) -> String {
    format!("0x{addr:x}")
}

/// RFC3339 UTC, whole seconds — no chrono/time dependency; the classic
/// civil-from-days algorithm (Howard Hinnant), unit-tested below.
fn rfc3339_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    rfc3339_from_epoch(secs)
}

fn rfc3339_from_epoch(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (
        if m <= 2 { y + 1 } else { y },
        u32::try_from(m).unwrap_or(1),
        u32::try_from(d).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_known_values() {
        assert_eq!(rfc3339_from_epoch(0), "1970-01-01T00:00:00Z");
        // 2026-07-05T00:00:00Z
        assert_eq!(rfc3339_from_epoch(1_783_209_600), "2026-07-05T00:00:00Z");
        // Leap-year check: 2024-02-29T12:00:00Z
        assert_eq!(rfc3339_from_epoch(1_709_208_000), "2024-02-29T12:00:00Z");
    }
}
