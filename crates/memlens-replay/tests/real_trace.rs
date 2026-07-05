//! Cross-crate integration: a REAL memlens trace (generated in-process via
//! the dev-dependency) parses, validates, and replays to the expected live
//! state. The trace file is the contract — this is the test that keeps the
//! two crates honest with each other.

use memlens::MemLens;
use std::alloc::{GlobalAlloc, Layout, System};

static LENS: MemLens<System> = MemLens::system();

#[test]
fn real_memlens_trace_parses_validates_replays() {
    let path =
        std::env::temp_dir().join(format!("memlens-replay-int-{}.jsonl", std::process::id()));
    // SAFETY: single-threaded at this point; the lens is NOT the global
    // allocator here, so nothing recorded before this line.
    unsafe { std::env::set_var("MEMLENS_TRACE", &path) };
    let _session = memlens::session("replay-integration");

    let layout_a = Layout::from_size_align(100, 8).expect("layout");
    let layout_b = Layout::from_size_align(200, 16).expect("layout");
    // SAFETY: non-zero layouts; every pointer freed or intentionally live.
    let a = unsafe { LENS.alloc(layout_a) };
    let b = unsafe { LENS.alloc(layout_b) };
    let a2 = unsafe { LENS.realloc(a, layout_a, 300) };
    unsafe { LENS.dealloc(b, layout_b) };
    assert!(!a2.is_null());

    memlens::flush();
    let content = std::fs::read_to_string(&path).expect("trace readable");
    let trace = memlens_replay::parse(&content).expect("parses");

    assert_eq!(
        trace.meta.as_ref().map(|m| m.program.as_str()),
        Some("replay-integration")
    );
    memlens_replay::validate(&trace.events).expect("real trace is balanced");

    let max = trace.events.last().expect("events").seq;
    let live = memlens_replay::replay(&trace.events, max);
    // Only a2's block (300 bytes) is still live; its lineage root is a's
    // original address.
    assert_eq!(live.len(), 1);
    let (addr, alloc) = live.iter().next().expect("one live");
    assert_eq!(*addr, a2 as usize as u64);
    assert_eq!(alloc.size, 300);
    assert_eq!(alloc.lineage_root, a as usize as u64);
    assert_eq!(memlens_replay::live_bytes(&trace.events, max), 300);

    // Cleanup (after the reads — this dealloc is past our parsed snapshot).
    let layout_a2 = Layout::from_size_align(300, 8).expect("layout");
    // SAFETY: a2 is live with this layout.
    unsafe { LENS.dealloc(a2, layout_a2) };
}
