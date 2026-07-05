//! Feature-off smoke test: with `memlens` disabled, `MemLens` must behave as
//! a transparent global allocator (requirement R6's behavioral half — the
//! symbol-absence half is checked by a build script in bolt 1.3).
//!
//! An integration test is its own binary, so it can install a
//! `#[global_allocator]` — something a unit test inside the library cannot do
//! (there is exactly one global allocator per program).

use memlens::MemLens;
use std::alloc::System;

#[global_allocator]
static LENS: MemLens<System> = MemLens::system();

#[test]
fn heap_types_work_through_the_lens() {
    // Vec growth exercises alloc + realloc paths.
    let mut v = Vec::new();
    for i in 0..1000u32 {
        v.push(i);
    }
    assert_eq!(v.len(), 1000);
    assert_eq!(v[999], 999);

    // String exercises alloc + realloc; drop at scope end exercises dealloc.
    let s = String::from("goldeneye").repeat(100);
    assert_eq!(s.len(), 900);

    // Zeroed allocation path.
    let z = vec![0u8; 4096];
    assert!(z.iter().all(|&b| b == 0));
}
