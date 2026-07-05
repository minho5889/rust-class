//! [P] R1 — written BEFORE the recorder exists (test-first, per the
//! constitution): for any generated sequence of harness operations
//! (allocate / grow / free), every operation appears in the trace exactly
//! once, in execution order, with strictly increasing `seq`.
//!
//! Harness design notes:
//! - Operations go through a **local** `MemLens<System>` instance, calling
//!   the `GlobalAlloc` methods directly. The lens still records globally,
//!   but proptest's own allocations use the ordinary global allocator — so
//!   the trace contains exactly our operations, no noise. (Real
//!   `#[global_allocator]` installation is covered by integration tests.)
//! - proptest runs many cases in one process; a static byte offset tracks
//!   where each case's trace segment starts.

#![cfg(feature = "memlens")]

use memlens::MemLens;
use proptest::prelude::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;

static LENS: MemLens<System> = MemLens::system();

/// Where this test's trace goes. Set once, before any event is recorded.
fn trace_path() -> &'static std::path::PathBuf {
    static PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        let p = std::env::temp_dir().join(format!("memlens-prop-r1-{}.jsonl", std::process::id()));
        // SAFETY: called via OnceLock before any other thread reads the
        // environment in this test binary (proptest hasn't spawned work yet).
        unsafe { std::env::set_var("MEMLENS_TRACE", &p) };
        // Leak the session guard: the sink must stay open (and unflushed-by-
        // drop) across all proptest cases in this process.
        Box::leak(Box::new(memlens::session("prop-r1")));
        p
    })
}

/// Byte offset where the current case's segment begins.
static OFFSET: Mutex<u64> = Mutex::new(0);

#[derive(Debug, Clone)]
enum Op {
    Alloc { size: usize, align_pow: u8 },
    Grow { idx: usize, factor: usize },
    Free { idx: usize },
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (1usize..=65536, 0u8..=4).prop_map(|(size, align_pow)| Op::Alloc { size, align_pow }),
        1 => (any::<usize>(), 1usize..=4).prop_map(|(idx, factor)| Op::Grow { idx, factor }),
        1 => any::<usize>().prop_map(|idx| Op::Free { idx }),
    ]
}

/// What the trace must contain for one executed operation.
#[derive(Debug, PartialEq)]
enum Expected {
    Alloc {
        addr: usize,
        size: usize,
        align: usize,
    },
    Realloc {
        old: usize,
        new: usize,
        old_size: usize,
        new_size: usize,
        align: usize,
    },
    Dealloc {
        addr: usize,
        size: usize,
        align: usize,
    },
}

fn parse_addr(v: &serde_json::Value) -> usize {
    let s = v.as_str().expect("addr must be a string");
    usize::from_str_radix(s.trim_start_matches("0x"), 16).expect("addr must be 0x hex")
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]
    #[test]
    fn r1_every_op_recorded_exactly_once_in_order(ops in prop::collection::vec(op_strategy(), 1..40)) {
        let path = trace_path().clone();
        let mut start = OFFSET.lock().expect("offset lock");

        // Execute ops through the lens, building the expected event list.
        let mut live: Vec<(usize, Layout)> = Vec::new();
        let mut expected: Vec<Expected> = Vec::new();
        for op in &ops {
            match *op {
                Op::Alloc { size, align_pow } => {
                    let layout = Layout::from_size_align(size, 1 << align_pow).expect("valid layout");
                    // SAFETY: layout has non-zero size (strategy starts at 1).
                    let ptr = unsafe { LENS.alloc(layout) };
                    prop_assume!(!ptr.is_null());
                    live.push((ptr as usize, layout));
                    expected.push(Expected::Alloc { addr: ptr as usize, size, align: layout.align() });
                }
                Op::Grow { idx, factor } => {
                    if live.is_empty() { continue; }
                    let i = idx % live.len();
                    let (old_addr, old_layout) = live[i];
                    let new_size = (old_layout.size() * factor).clamp(1, 262_144);
                    // SAFETY: (old_addr, old_layout) is a live allocation from
                    // this allocator; new_size is non-zero and bounded.
                    let new_ptr = unsafe { LENS.realloc(old_addr as *mut u8, old_layout, new_size) };
                    prop_assume!(!new_ptr.is_null());
                    let new_layout = Layout::from_size_align(new_size, old_layout.align()).expect("valid layout");
                    live[i] = (new_ptr as usize, new_layout);
                    expected.push(Expected::Realloc {
                        old: old_addr, new: new_ptr as usize,
                        old_size: old_layout.size(), new_size, align: old_layout.align(),
                    });
                }
                Op::Free { idx } => {
                    if live.is_empty() { continue; }
                    let i = idx % live.len();
                    let (addr, layout) = live.swap_remove(i);
                    // SAFETY: (addr, layout) is a live allocation from this allocator.
                    unsafe { LENS.dealloc(addr as *mut u8, layout) };
                    expected.push(Expected::Dealloc { addr, size: layout.size(), align: layout.align() });
                }
            }
        }
        // Clean up remaining allocations (also recorded, also expected).
        for (addr, layout) in live.drain(..) {
            // SAFETY: still live, allocated by this allocator.
            unsafe { LENS.dealloc(addr as *mut u8, layout) };
            expected.push(Expected::Dealloc { addr, size: layout.size(), align: layout.align() });
        }

        // Read this case's trace segment.
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let segment = &content[usize::try_from(*start).expect("offset fits")..];
        *start = content.len() as u64;
        drop(start);

        let mut got: Vec<Expected> = Vec::new();
        let mut last_seq: u64 = 0;
        for line in segment.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(line).expect("trace lines are JSON");
            let ty = v["event_type"].as_str().expect("event_type");
            let p = &v["payload"];
            if let Some(seq) = p.get("seq").and_then(serde_json::Value::as_u64) {
                prop_assert!(seq > last_seq, "seq not strictly increasing: {seq} after {last_seq}");
                last_seq = seq;
            }
            match ty {
                "memlens.alloc" => got.push(Expected::Alloc {
                    addr: parse_addr(&p["addr"]),
                    size: p["size"].as_u64().expect("size") as usize,
                    align: p["align"].as_u64().expect("align") as usize,
                }),
                "memlens.dealloc" => got.push(Expected::Dealloc {
                    addr: parse_addr(&p["addr"]),
                    size: p["size"].as_u64().expect("size") as usize,
                    align: p["align"].as_u64().expect("align") as usize,
                }),
                "memlens.realloc" => got.push(Expected::Realloc {
                    old: parse_addr(&p["old_addr"]),
                    new: parse_addr(&p["new_addr"]),
                    old_size: p["old_size"].as_u64().expect("old_size") as usize,
                    new_size: p["new_size"].as_u64().expect("new_size") as usize,
                    align: p["align"].as_u64().expect("align") as usize,
                }),
                "memlens.meta" | "memlens.loss" => {}
                other => prop_assert!(false, "unexpected event_type {other}"),
            }
        }

        prop_assert_eq!(&got, &expected, "trace events must equal executed ops, in order");
    }
}
