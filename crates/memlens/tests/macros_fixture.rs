//! [E] R4 + R12-capture-side — the teaching macros record labeled events
//! ordered consistently with the allocations they surround, on ALL exit
//! paths (normal, early return, panic), and var/move/borrow markers are
//! emitted. One #[test] so sink state is deterministic; the lens is the
//! real global allocator here, so sentinel sizes identify our allocations
//! among harness noise.

#![cfg(feature = "memlens")]

use memlens::{MemLens, lens_borrow, lens_drop, lens_move, lens_scope, lens_var};
use std::alloc::System;

#[global_allocator]
static LENS: MemLens<System> = MemLens::system();

const SENTINEL: usize = 12_345;

fn early_return_scope() -> u32 {
    lens_scope!("early", {
        let v = vec![0u8; SENTINEL + 1];
        if v.len() > 10 {
            return 7; // scope_exit must still be recorded (Drop guard)
        }
        0
    })
}

#[test]
fn scopes_markers_and_exit_paths() {
    let path = std::env::temp_dir().join(format!("memlens-r4-{}.jsonl", std::process::id()));
    // The harness already initialized the sink (this binary's global
    // allocator is the lens) — retarget it to this test's own file.
    memlens::record::__retarget(&path);
    let _session = memlens::session("r4-fixture");

    // Normal path: alloc inside a labeled scope.
    lens_scope!("normal", {
        let buf = vec![0u8; SENTINEL];
        lens_var!(buf);
        let moved = lens_move!(buf);
        let borrowed = lens_borrow!(&moved);
        assert_eq!(borrowed.len(), SENTINEL);
        lens_drop!(moved);
    });

    // Early-return path.
    assert_eq!(early_return_scope(), 7);

    // Panic path: the guard must record scope_exit during unwind.
    let result = std::panic::catch_unwind(|| {
        lens_scope!("panicky", {
            let _v = vec![0u8; SENTINEL + 2];
            panic!("boom");
        })
    });
    assert!(result.is_err());

    memlens::flush();
    let content = std::fs::read_to_string(&path).expect("trace readable");
    let events: Vec<serde_json::Value> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("valid JSONL"))
        .collect();

    let idx = |ty: &str, label: &str| -> usize {
        events
            .iter()
            .position(|e| e["event_type"] == ty && e["payload"]["label"] == label)
            .unwrap_or_else(|| panic!("missing {ty} {label}"))
    };
    let alloc_of = |size: usize| -> usize {
        events
            .iter()
            .position(|e| {
                e["event_type"] == "memlens.alloc"
                    && e["payload"]["size"].as_u64() == Some(size as u64)
            })
            .unwrap_or_else(|| panic!("missing alloc of {size}"))
    };

    // R4 ordering: enter < alloc < exit, for every exit path.
    for (label, size) in [
        ("normal", SENTINEL),
        ("early", SENTINEL + 1),
        ("panicky", SENTINEL + 2),
    ] {
        let enter = idx("memlens.scope_enter", label);
        let exit = idx("memlens.scope_exit", label);
        let alloc = alloc_of(size);
        assert!(
            enter < alloc && alloc < exit,
            "{label}: enter({enter}) < alloc({alloc}) < exit({exit}) violated"
        );
    }

    // Markers: var label, move, borrow, drop — all present; drop marker
    // precedes the sentinel dealloc (ownership visibly freeing).
    idx("memlens.marker", "var:buf");
    idx("memlens.marker", "move:buf");
    idx("memlens.marker", "borrow:&moved");
    let drop_marker = idx("memlens.marker", "drop:moved");
    let dealloc = events
        .iter()
        .position(|e| {
            e["event_type"] == "memlens.dealloc"
                && e["payload"]["size"].as_u64() == Some(SENTINEL as u64)
        })
        .expect("sentinel dealloc");
    assert!(
        drop_marker < dealloc,
        "drop marker must precede the dealloc it causes"
    );
}
