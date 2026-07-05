//! Trace event payloads — the `memlens.v1` format (requirement R8).
//!
//! One JSONL line per event, envelope-compatible with the goldeneye data
//! lake (`datalake/schema/envelope.v1.json`): the envelope's `event_type`
//! carries the operation (`memlens.alloc`, `memlens.realloc`, …) and these
//! structs are the `payload`. Registered schema:
//! `datalake/schema/memlens.v1.json`.
//!
//! ## Teaching notes
//!
//! - **Addresses are opaque strings** (`"0x5591a3c04ba0"`). They are useful
//!   for matching an allocation to its free *within one run*, and meaningless
//!   across runs — ASLR randomizes the address space every execution. The
//!   viewer says so (requirement R15).
//! - **`realloc` has its own shape** carrying the old→new *lineage*. A
//!   reallocation is conceptually "free old, allocate new, copy" — one event
//!   with both addresses lets the replay engine keep the books balanced (R2)
//!   and lets the collections panel chain `Vec`'s growth into a staircase (R5).
//! - **`seq` is the trace's clock.** Wall-clock time is unreliable at
//!   nanosecond scale; a monotonic sequence number assigned under the writer
//!   lock gives every event a total order, and file order equals seq order by
//!   construction (design decision after the auditor caught the lock-free
//!   version being reorderable).

use serde::{Deserialize, Serialize};

/// Payload for `memlens.alloc` and `memlens.dealloc` (and zeroed allocs,
/// which are recorded as ordinary allocs — the zeroing is invisible to the
/// heap ledger).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocPayload {
    /// Opaque pointer, formatted `0x…`; meaningless across runs (ASLR).
    pub addr: String,
    pub size: usize,
    pub align: usize,
    pub seq: u64,
    /// Teaching label attached by `lens_var!`-style macros (bolt 1.4).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub label: Option<String>,
}

/// Payload for `memlens.realloc` — carries the old→new lineage (R2, R5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReallocPayload {
    pub old_addr: String,
    pub new_addr: String,
    pub old_size: usize,
    pub new_size: usize,
    pub align: usize,
    pub seq: u64,
}

/// Payload for `memlens.scope_enter`, `memlens.scope_exit`, and
/// `memlens.marker` (moves/borrows — the zero-allocation callouts, R12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopePayload {
    pub label: String,
    /// Marker kind: `"move"` | `"borrow"` | `"drop"` (markers only).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kind: Option<String>,
    pub seq: u64,
}

/// Payload for `memlens.loss` — emitted once after sink failures, carrying
/// how many events were dropped (R7b). The trace stays honest about its own
/// gaps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LossPayload {
    pub dropped: u64,
    pub seq: u64,
}

/// Payload for `memlens.meta` — one header event per trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaPayload {
    pub program: String,
    pub pid: u32,
    /// RFC3339 UTC.
    pub started_at: String,
    /// memlens crate version that wrote the trace.
    pub version: String,
}

/// A complete trace event: the operation (→ envelope `event_type`) plus its
/// payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEvent {
    Alloc(AllocPayload),
    Dealloc(AllocPayload),
    Realloc(ReallocPayload),
    ScopeEnter(ScopePayload),
    ScopeExit(ScopePayload),
    Marker(ScopePayload),
    Loss(LossPayload),
    Meta(MetaPayload),
}

impl TraceEvent {
    /// The envelope `event_type` for this event.
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::Alloc(_) => "memlens.alloc",
            Self::Dealloc(_) => "memlens.dealloc",
            Self::Realloc(_) => "memlens.realloc",
            Self::ScopeEnter(_) => "memlens.scope_enter",
            Self::ScopeExit(_) => "memlens.scope_exit",
            Self::Marker(_) => "memlens.marker",
            Self::Loss(_) => "memlens.loss",
            Self::Meta(_) => "memlens.meta",
        }
    }

    /// Serialize just the payload (the envelope wrapper is the writer's job).
    ///
    /// # Errors
    /// Returns `serde_json`'s error if serialization fails (practically:
    /// never for these plain-data types).
    pub fn payload_json(&self) -> Result<String, serde_json::Error> {
        match self {
            Self::Alloc(p) | Self::Dealloc(p) => serde_json::to_string(p),
            Self::Realloc(p) => serde_json::to_string(p),
            Self::ScopeEnter(p) | Self::ScopeExit(p) | Self::Marker(p) => serde_json::to_string(p),
            Self::Loss(p) => serde_json::to_string(p),
            Self::Meta(p) => serde_json::to_string(p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R8 round-trip: every payload survives serialize → deserialize intact,
    /// and field names match the registered schema (spot-checked).
    #[test]
    fn payloads_round_trip() {
        let alloc = AllocPayload {
            addr: "0x5591a3c04ba0".into(),
            size: 48,
            align: 8,
            seq: 7,
            label: Some("names".into()),
        };
        let json = serde_json::to_string(&alloc).unwrap();
        assert!(json.contains("\"addr\"") && json.contains("\"seq\""));
        assert_eq!(alloc, serde_json::from_str(&json).unwrap());

        let realloc = ReallocPayload {
            old_addr: "0x10".into(),
            new_addr: "0x20".into(),
            old_size: 8,
            new_size: 16,
            align: 8,
            seq: 8,
        };
        let json = serde_json::to_string(&realloc).unwrap();
        assert!(json.contains("\"old_addr\"") && json.contains("\"new_addr\""));
        assert_eq!(realloc, serde_json::from_str(&json).unwrap());

        let scope = ScopePayload {
            label: "build_names".into(),
            kind: None,
            seq: 9,
        };
        let json = serde_json::to_string(&scope).unwrap();
        // `kind: None` must be omitted entirely, not serialized as null.
        assert!(!json.contains("kind"));
        assert_eq!(scope, serde_json::from_str::<ScopePayload>(&json).unwrap());

        let loss = LossPayload {
            dropped: 3,
            seq: 10,
        };
        assert_eq!(
            loss,
            serde_json::from_str(&serde_json::to_string(&loss).unwrap()).unwrap()
        );

        let meta = MetaPayload {
            program: "playground-02".into(),
            pid: 4242,
            started_at: "2026-07-05T09:00:00Z".into(),
            version: "0.1.0".into(),
        };
        assert_eq!(
            meta,
            serde_json::from_str(&serde_json::to_string(&meta).unwrap()).unwrap()
        );
    }

    #[test]
    fn event_types_are_the_memlens_family() {
        let e = TraceEvent::Loss(LossPayload { dropped: 1, seq: 1 });
        assert_eq!(e.event_type(), "memlens.loss");
        assert!(e.payload_json().unwrap().contains("\"dropped\":1"));
    }
}
