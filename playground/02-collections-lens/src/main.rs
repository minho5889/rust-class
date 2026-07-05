//! # Exercise 02 — collections under the lens
//!
//! Run it twice:
//! ```sh
//! cargo run                     # plain — the lens compiles to nothing
//! cargo run --features lens     # traced — then open the trace in viewer/memlens.html
//! ```
//! Everything this program does on the heap becomes visible. Read the code,
//! predict the trace, then scrub the viewer and check yourself.

use memlens::{MemLens, lens_borrow, lens_drop, lens_move, lens_scope, lens_var};
use std::alloc::System;

// The entire instrumentation surface (R16): one static + one session call.
#[global_allocator]
static LENS: MemLens<System> = MemLens::system();

fn main() {
    let _session = memlens::session("02-collections-lens");

    // ── Lesson 1: Vec grows by amortized doubling ────────────────────────
    // Prediction: 1000 pushes will NOT touch the allocator 1000 times.
    // Watch the collections panel: one alloc, then a staircase of reallocs
    // (4 → 8 → 16 … elements), each realloc linked old_addr → new_addr.
    let numbers = lens_scope!("vec-doubling", {
        let mut v: Vec<u64> = Vec::new();
        for i in 0..1000 {
            v.push(i);
        }
        lens_var!(v);
        v
    });

    // ── Lesson 2: a move costs zero bytes ────────────────────────────────
    // `numbers` transfers ownership to `owned`. Click the ◆ marker in the
    // viewer: no alloc events at that seq. The 8000-byte buffer did not
    // move; only the (ptr, len, cap) triple — three words on the stack —
    // changed hands. This is what "zero-cost" means, on screen.
    let owned = lens_move!(numbers);

    // ── Lesson 3: a borrow costs zero bytes too ──────────────────────────
    // `total` reads through a &-reference. No events. The borrow checker's
    // rules exist precisely so this can be free AND safe.
    let total: u64 = lens_borrow!(&owned).iter().sum();
    println!("sum of 0..1000 = {total}");

    // ── Lesson 4: String allocates, &str does not ─────────────────────────
    lens_scope!("string-vs-str", {
        let literal: &str = "goldeneye"; // no event: &str points into the binary
        let heap_copy = String::from(literal); // alloc: 9 bytes land on the heap
        lens_var!(heap_copy);
        let grown = heap_copy + " rides again"; // realloc/alloc: growth visible
        lens_var!(grown);
        println!("{grown}, says {literal}");
        // Scope exit → both Strings drop → watch the deallocs, in reverse
        // declaration order, with zero garbage-collector involvement.
    });

    // ── Lesson 5: Box moves a value TO the heap, explicitly ──────────────
    lens_scope!("box", {
        let on_stack = [0u8; 64]; // no event: arrays live on the stack
        let boxed = Box::new(on_stack); // alloc(64): the copy to the heap is visible
        lens_var!(boxed);
        lens_drop!(boxed); // dealloc(64), exactly here — not "eventually"
    });

    // ── Lesson 6: HashMap grows differently than Vec ─────────────────────
    // Prediction to check in the viewer: NO realloc chain. hashbrown
    // allocates a bigger table, rehashes, then frees the old one — growth
    // appears as alloc/dealloc pairs of increasing size. Two collection
    // types, two different bargains with the allocator.
    lens_scope!("hashmap-rehash", {
        let mut squares = std::collections::HashMap::new();
        for i in 0..1000u32 {
            squares.insert(i, u64::from(i) * u64::from(i));
        }
        lens_var!(squares);
        println!("31^2 = {}", squares[&31]);
    });

    // `owned` (8 KiB) drops here, at end of main — the very last dealloc in
    // the trace. Ownership is a schedule, and the trace is its receipt.
}
