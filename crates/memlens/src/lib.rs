//! # memlens — the goldeneye memory lens
//!
//! A *teaching* tracking allocator: it wraps any [`GlobalAlloc`] and records
//! every heap allocation, reallocation, and deallocation as trace events, so
//! the learner can *watch* Rust's memory model at work — `Vec` doubling,
//! `String`s moving without copying, `Drop` freeing memory at scope exit.
//!
//! ## The two big ideas
//!
//! 1. **Rust has no runtime to ask about the heap.** In a GC language, the
//!    runtime keeps score. Rust compiles to bare metal — so to see the heap we
//!    interpose our own allocator via the [`GlobalAlloc`] trait. That trait is
//!    `unsafe` because the *implementor* promises things the compiler cannot
//!    check (see below).
//! 2. **The lens must never change the program it observes** (beyond timing):
//!    with the `memlens` cargo feature **off**, `MemLens<A>` is a transparent
//!    passthrough that inlines to the inner allocator — zero recording code in
//!    the binary (requirement R6). With the feature **on**, recording is added
//!    (later bolts) — and a compile guard refuses release/bench builds
//!    (requirement R14b): the lens is for learning runs, never benchmarks.
//!
//! ## Why `unsafe`, and what we promise
//!
//! `GlobalAlloc` is an `unsafe trait`: the compiler cannot verify that an
//! allocator returns valid memory, so the implementor *declares* they uphold
//! the contract (the Nomicon's first role of `unsafe`). Every `unsafe` block
//! in this crate carries a `// SAFETY:` comment stating why the contract
//! holds (the second role: asserting a checked obligation). This crate is the
//! project's only sanctioned home for unsafe code — everything else is
//! `#![forbid(unsafe_code)]`.

// R14b: the lens must never ship in a release or bench build. There is no
// direct cfg for "profile", so `not(debug_assertions)` is the standard proxy:
// it holds for the default `release` and `bench` profiles. (A custom profile
// that re-enables debug-assertions would slip past this guard — acceptable
// for a learning repo, documented in design.md.)
#[cfg(all(feature = "memlens", not(debug_assertions)))]
compile_error!(
    "memlens is a teaching instrument: never enable the `memlens` feature in \
     release/bench builds. Trace in debug builds; benchmark without the lens."
);

use std::alloc::{GlobalAlloc, Layout, System};

/// A wrapper around any global allocator that (with the `memlens` feature)
/// records every heap operation as a trace event.
///
/// Install it as the program's global allocator — this is the *entire*
/// integration surface (requirement R16):
///
/// ```
/// use memlens::MemLens;
///
/// #[global_allocator]
/// static LENS: MemLens<std::alloc::System> = MemLens::system();
///
/// let v = vec![1u8, 2, 3]; // with `memlens` on: recorded as an alloc event
/// assert_eq!(v.len(), 3);
/// ```
///
/// # Ownership note (teaching)
///
/// `MemLens` *owns* its inner allocator by value — no `Box`, no indirection.
/// A `#[global_allocator]` static must be constructible in a `const` context
/// and must never allocate to exist (an allocator that allocates to construct
/// itself would need... an allocator). This is why both constructors are
/// `const fn`.
pub struct MemLens<A> {
    inner: A,
}

impl MemLens<System> {
    /// A lens over the standard [`System`] allocator — the common case.
    #[must_use]
    pub const fn system() -> Self {
        Self { inner: System }
    }
}

impl<A> MemLens<A> {
    /// Wrap an arbitrary allocator.
    #[must_use]
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }
}

// SAFETY (trait-level contract): `MemLens` forwards every call to `self.inner`
// with the *identical* `Layout`, and returns the inner allocator's result
// unmodified. It therefore upholds `GlobalAlloc`'s contract exactly as well
// as the inner allocator does: memory it returns is valid for `layout`, and
// it never unwinds. Recording (feature `memlens`, later bolts) only observes;
// it never alters layouts, pointers, or control flow.
unsafe impl<A: GlobalAlloc> GlobalAlloc for MemLens<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds `alloc`'s precondition (non-zero-sized
        // `layout`); we forward the same layout to an allocator bound by the
        // same contract.
        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the caller guarantees `ptr` was allocated by *this*
        // allocator with this exact `layout`; since we forward all
        // allocations to `inner`, `ptr` came from `inner` and may be
        // returned to it.
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same argument as `alloc`; zeroing is the inner
        // allocator's obligation.
        unsafe { self.inner.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller guarantees `ptr`/`layout` describe a live
        // allocation from this allocator and `new_size` is non-zero and
        // does not overflow; all of it is forwarded unchanged to `inner`,
        // which allocated `ptr`.
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }
}
