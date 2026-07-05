//! [E] R5 — collection growth is reconstructable from the trace.
//!
//! Two different growth mechanics, both lessons:
//! - `Vec` grows via **realloc lineage**: old_addr → new_addr links chain
//!   into the capacity staircase.
//! - `HashMap` (hashbrown) grows via **alloc new table + dealloc old** — no
//!   realloc events at all. The pattern is still reconstructable (increasing
//!   alloc sizes, each older table freed), and the difference is itself a
//!   teaching point recorded in evidence.md.

#![cfg(feature = "memlens")]

use memlens::{MemLens, lens_scope};
use std::alloc::System;

#[global_allocator]
static LENS: MemLens<System> = MemLens::system();

#[test]
fn vec_and_hashmap_growth_reconstructable() {
    let path = std::env::temp_dir().join(format!("memlens-r5-{}.jsonl", std::process::id()));
    // The harness already initialized the sink (this binary's global
    // allocator is the lens) — retarget it to this test's own file.
    memlens::record::__retarget(&path);
    let _session = memlens::session("r5-fixture");

    // lens_scope! is an expression — the block's value passes through.
    let final_vec_addr = lens_scope!("vec-growth", {
        let mut v: Vec<u64> = Vec::new();
        for i in 0..1000u64 {
            v.push(i);
        }
        v.as_ptr() as usize
    });

    lens_scope!("map-growth", {
        let mut m = std::collections::HashMap::new();
        for i in 0..1000u32 {
            m.insert(i, u64::from(i));
        }
        assert_eq!(m.len(), 1000);
    });

    memlens::flush();
    let content = std::fs::read_to_string(&path).expect("trace readable");
    let events: Vec<serde_json::Value> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("valid JSONL"))
        .collect();
    let window = |label: &str| -> &[serde_json::Value] {
        let enter = events
            .iter()
            .position(|e| {
                e["event_type"] == "memlens.scope_enter" && e["payload"]["label"] == label
            })
            .expect("enter");
        let exit = events
            .iter()
            .position(|e| e["event_type"] == "memlens.scope_exit" && e["payload"]["label"] == label)
            .expect("exit");
        &events[enter..=exit]
    };

    // --- Vec: follow the realloc lineage chain to the final buffer addr.
    let vec_events = window("vec-growth");
    let reallocs: Vec<(&str, &str, u64, u64)> = vec_events
        .iter()
        .filter(|e| e["event_type"] == "memlens.realloc")
        .map(|e| {
            (
                e["payload"]["old_addr"].as_str().expect("old_addr"),
                e["payload"]["new_addr"].as_str().expect("new_addr"),
                e["payload"]["old_size"].as_u64().expect("old_size"),
                e["payload"]["new_size"].as_u64().expect("new_size"),
            )
        })
        .collect();
    assert!(
        reallocs.len() >= 5,
        "1000 u64 pushes should produce several capacity doublings, got {}",
        reallocs.len()
    );
    // Sizes strictly increase along the chain, and lineage links connect:
    // each realloc's old_addr is the previous step's new_addr (or the
    // original alloc's addr).
    let mut grew = true;
    for w in reallocs.windows(2) {
        grew &= w[1].2 == w[0].3; // next old_size == prev new_size
        grew &= w[1].0 == w[0].1; // next old_addr == prev new_addr
        assert!(w[1].3 > w[0].3, "capacity staircase must increase");
    }
    assert!(grew, "realloc lineage chain must be connected");
    let final_hex = format!("0x{final_vec_addr:x}");
    assert_eq!(
        reallocs.last().expect("nonempty").1,
        final_hex,
        "chain must end at the vec's final buffer address"
    );

    // --- HashMap: growth = increasing alloc sizes, older tables freed.
    let map_events = window("map-growth");
    let allocs: Vec<u64> = map_events
        .iter()
        .filter(|e| e["event_type"] == "memlens.alloc")
        .map(|e| e["payload"]["size"].as_u64().expect("size"))
        .collect();
    let deallocs = map_events
        .iter()
        .filter(|e| e["event_type"] == "memlens.dealloc")
        .count();
    assert!(
        allocs.len() >= 3,
        "map growth should allocate successive tables, got {allocs:?}"
    );
    let max = allocs.iter().max().expect("nonempty");
    let increasing = allocs.iter().filter(|s| **s > 64).collect::<Vec<_>>();
    assert!(
        increasing.windows(2).all(|w| w[1] > w[0]),
        "table sizes should increase: {increasing:?}"
    );
    assert!(
        deallocs >= increasing.len().saturating_sub(1),
        "older tables must be freed (alloc+dealloc growth, not realloc): \
         {deallocs} deallocs for {allocs:?}"
    );
    assert!(
        *max > 8_192,
        "final table for 1000 entries should be > 8 KiB"
    );
}
