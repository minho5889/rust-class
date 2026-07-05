//! The trace recorder — bolt 1.2 wires this into the allocator.
//!
//! Stubbed no-op for task 1.2.1 (the R1 property test is written against this
//! API surface and must be RED before the implementation exists — the
//! constitution's test-first rule).

use crate::event::MetaPayload;

/// Begin a traced session: emits the `memlens.meta` header event and returns
/// a guard whose `Drop` flushes the trace sink.
#[must_use]
pub fn session(program: &str) -> LensSession {
    let _ = MetaPayload {
        program: program.to_owned(),
        pid: std::process::id(),
        started_at: String::new(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    };
    LensSession { _priv: () }
}

/// Guard for a traced session; flushes the sink on drop (and, with the
/// implementation in place, emits a final loss marker if events were
/// dropped).
pub struct LensSession {
    _priv: (),
}

pub(crate) fn record_alloc(_addr: usize, _size: usize, _align: usize) {}
pub(crate) fn record_dealloc(_addr: usize, _size: usize, _align: usize) {}
pub(crate) fn record_realloc(
    _old_addr: usize,
    _new_addr: usize,
    _old_size: usize,
    _new_size: usize,
    _align: usize,
) {
}
