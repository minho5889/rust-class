//! Minimal traced program — used by the R6 (feature-off) and R8 (schema)
//! verification tests, and as the smallest possible instrumentation example:
//! one static + one session call (requirement R16).

use memlens::MemLens;
use std::alloc::System;

#[global_allocator]
static LENS: MemLens<System> = MemLens::system();

fn main() {
    let _session = memlens::session("demo");

    // Vec growth: repeated pushes force capacity doublings (reallocs).
    let mut v = Vec::new();
    for i in 0..1000u32 {
        v.push(i);
    }

    // String: one allocation, freed deterministically at drop.
    let s = String::from("goldeneye ").repeat(50);
    drop(s);

    // HashMap: growth appears as alloc(new table) + dealloc(old table).
    let mut m = std::collections::HashMap::new();
    for i in 0..500u32 {
        m.insert(i, u64::from(i) * 2);
    }

    println!("demo done: v={} m={}", v.len(), m.len());
}
