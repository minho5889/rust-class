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

#[cfg(feature = "memlens")]
pub mod event;
#[cfg(feature = "memlens")]
#[doc(hidden)] // public only for the crate's own failure-injection tests
pub mod record;
#[cfg(feature = "memlens")]
pub use record::{LensSession, flush, session};

/// Feature-off `session`: a no-op, so instrumented programs compile
/// unchanged in both configurations (requirement R16/R6 interplay).
#[cfg(not(feature = "memlens"))]
#[must_use]
pub fn session(_program: &str) -> LensSession {
    LensSession { _priv: () }
}

/// Feature-off session guard (no-op).
#[cfg(not(feature = "memlens"))]
pub struct LensSession {
    _priv: (),
}

use std::alloc::{GlobalAlloc, Layout, System};

// ---------- Teaching macros (bolt 1.4) ----------
//
// The macros expand to calls on `$crate::__…` helpers that exist in BOTH
// feature configurations (no-ops when the lens is off), so annotated learner
// code compiles unchanged either way (R16/R6).

/// Wrap a block in a labeled scope: emits `scope_enter` before and
/// `scope_exit` after — the exit via a `Drop` guard, so early `return`s and
/// panics still close the scope. That guard *is* the lesson: deterministic
/// cleanup is not a convention in Rust, it's the type system running your
/// destructor on every path out.
#[macro_export]
macro_rules! lens_scope {
    ($label:expr, $body:block) => {{
        let _lens_guard = $crate::__scope_enter($label);
        $body
    }};
}

/// Label a variable in the trace (`marker`, no kind): ties a human name to
/// the surrounding events.
#[macro_export]
macro_rules! lens_var {
    ($v:ident) => {
        $crate::__marker(None, concat!("var:", stringify!($v)))
    };
}

/// Explicitly drop a value, marking the spot: the `dealloc` events that
/// follow this marker are ownership doing its job.
#[macro_export]
macro_rules! lens_drop {
    ($v:ident) => {{
        $crate::__marker(Some("drop"), concat!("drop:", stringify!($v)));
        drop($v);
    }};
}

/// Mark a move. The marker is the callout: note the **absence** of
/// allocation events at this seq — ownership transfer copies nothing (R12).
#[macro_export]
macro_rules! lens_move {
    ($e:expr) => {{
        $crate::__marker(Some("move"), concat!("move:", stringify!($e)));
        $e
    }};
}

/// Mark a borrow — same zero-allocation callout as [`lens_move!`].
#[macro_export]
macro_rules! lens_borrow {
    ($e:expr) => {{
        $crate::__marker(Some("borrow"), concat!("borrow:", stringify!($e)));
        $e
    }};
}

/// RAII guard returned by [`lens_scope!`]; records `scope_exit` on drop.
pub struct ScopeGuard {
    #[cfg(feature = "memlens")]
    label: String,
}

#[cfg(feature = "memlens")]
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        record::record_scoped(|seq| {
            event::TraceEvent::ScopeExit(event::ScopePayload {
                label: std::mem::take(&mut self.label),
                kind: None,
                seq,
            })
        });
    }
}

#[doc(hidden)]
#[cfg(feature = "memlens")]
pub fn __scope_enter(label: &str) -> ScopeGuard {
    record::record_scoped(|seq| {
        event::TraceEvent::ScopeEnter(event::ScopePayload {
            label: label.to_owned(),
            kind: None,
            seq,
        })
    });
    ScopeGuard {
        label: label.to_owned(),
    }
}

#[doc(hidden)]
#[cfg(not(feature = "memlens"))]
pub fn __scope_enter(_label: &str) -> ScopeGuard {
    ScopeGuard {}
}

#[doc(hidden)]
#[cfg(feature = "memlens")]
pub fn __marker(kind: Option<&str>, label: &str) {
    record::record_scoped(|seq| {
        event::TraceEvent::Marker(event::ScopePayload {
            label: label.to_owned(),
            kind: kind.map(str::to_owned),
            seq,
        })
    });
}

#[doc(hidden)]
#[cfg(not(feature = "memlens"))]
pub fn __marker(_kind: Option<&str>, _label: &str) {}

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
        let ptr = unsafe { self.inner.alloc(layout) };
        #[cfg(feature = "memlens")]
        if !ptr.is_null() {
            record::record_alloc(ptr as usize, layout.size(), layout.align());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        #[cfg(feature = "memlens")]
        record::record_dealloc(ptr as usize, layout.size(), layout.align());
        // SAFETY: the caller guarantees `ptr` was allocated by *this*
        // allocator with this exact `layout`; since we forward all
        // allocations to `inner`, `ptr` came from `inner` and may be
        // returned to it.
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same argument as `alloc`; zeroing is the inner
        // allocator's obligation.
        let ptr = unsafe { self.inner.alloc_zeroed(layout) };
        #[cfg(feature = "memlens")]
        if !ptr.is_null() {
            record::record_alloc(ptr as usize, layout.size(), layout.align());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller guarantees `ptr`/`layout` describe a live
        // allocation from this allocator and `new_size` is non-zero and
        // does not overflow; all of it is forwarded unchanged to `inner`,
        // which allocated `ptr`.
        let new_ptr = unsafe { self.inner.realloc(ptr, layout, new_size) };
        #[cfg(feature = "memlens")]
        if !new_ptr.is_null() {
            record::record_realloc(
                ptr as usize,
                new_ptr as usize,
                layout.size(),
                new_size,
                layout.align(),
            );
        }
        new_ptr
    }
}
