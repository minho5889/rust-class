//! [E] R7a/R7b — sink failures must never panic the traced program (R7a),
//! and dropped events must surface as a single loss marker once writing
//! resumes (R7b). Failure is injected via the crate's test hook; this test
//! binary owns its own trace file and runs as a single #[test] so the
//! global sink state is deterministic.

#![cfg(feature = "memlens")]

use memlens::MemLens;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::Ordering;

static LENS: MemLens<System> = MemLens::system();

#[test]
fn sink_failure_drops_events_without_panic_and_emits_loss_marker() {
    let path = std::env::temp_dir().join(format!("memlens-r7-{}.jsonl", std::process::id()));
    // SAFETY: single-threaded at this point; nothing else reads the
    // environment concurrently in this test binary.
    unsafe { std::env::set_var("MEMLENS_TRACE", &path) };
    let _session = memlens::session("r7-test");

    let layout = Layout::from_size_align(64, 8).expect("valid layout");

    // Healthy write.
    // SAFETY: non-zero-sized layout; matching dealloc below.
    let p1 = unsafe { LENS.alloc(layout) };
    assert!(!p1.is_null());

    // Break the sink: these operations must be DROPPED, not panic (R7a).
    memlens::record::FAIL_WRITES.store(true, Ordering::Relaxed);
    // SAFETY: as above; each alloc is paired with a dealloc.
    let p2 = unsafe { LENS.alloc(layout) };
    assert!(!p2.is_null());
    unsafe { LENS.dealloc(p2, layout) };

    // Heal the sink: the next write must emit ONE loss marker first (R7b).
    memlens::record::FAIL_WRITES.store(false, Ordering::Relaxed);
    unsafe { LENS.dealloc(p1, layout) };

    memlens::flush();
    let content = std::fs::read_to_string(&path).expect("trace file readable");
    let events: Vec<serde_json::Value> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("valid JSONL"))
        .collect();

    let loss: Vec<&serde_json::Value> = events
        .iter()
        .filter(|e| e["event_type"] == "memlens.loss")
        .collect();
    assert_eq!(loss.len(), 1, "exactly one loss marker, got {events:#?}");
    assert_eq!(
        loss[0]["payload"]["dropped"].as_u64(),
        Some(2),
        "the two failed writes (alloc p2, dealloc p2) must be counted"
    );

    // The loss marker precedes the healed dealloc in the file (seq order).
    let loss_idx = events
        .iter()
        .position(|e| e["event_type"] == "memlens.loss")
        .expect("loss present");
    let dealloc_after = events[loss_idx + 1..]
        .iter()
        .any(|e| e["event_type"] == "memlens.dealloc");
    assert!(dealloc_after, "healed dealloc must follow the loss marker");
}
