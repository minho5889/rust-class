//! # memlens-replay — the headless replay engine
//!
//! A memlens trace is a fact log; this crate folds it into *state*: the set
//! of live allocations (and live bytes) at any sequence point `t`. The
//! viewer's JavaScript mirrors exactly this fold, pinned by golden fixtures.
//!
//! Pure by design: parse bytes → `Vec<Event>` → fold. No I/O in the core,
//! no unsafe anywhere (`#![forbid(unsafe_code)]` — the lens crate is the
//! project's only unsafe zone). The trace **file** is the contract with
//! `memlens`; no Rust types are shared, so the two crates can only drift in
//! ways the schema tests catch.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// One seq-carrying trace event (the `memlens.meta` header is parsed
/// separately — it has no place in the fold).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub seq: u64,
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Alloc {
        addr: u64,
        size: u64,
        align: u64,
    },
    Dealloc {
        addr: u64,
        size: u64,
        align: u64,
    },
    Realloc {
        old_addr: u64,
        new_addr: u64,
        old_size: u64,
        new_size: u64,
        align: u64,
    },
    ScopeEnter {
        label: String,
    },
    ScopeExit {
        label: String,
    },
    Marker {
        label: String,
        kind: Option<String>,
    },
    Loss {
        dropped: u64,
    },
}

/// Program metadata from the `memlens.meta` header event.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Meta {
    pub program: String,
    pub pid: u64,
    pub started_at: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trace {
    pub meta: Option<Meta>,
    pub events: Vec<Event>,
}

/// A live allocation at some point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    pub size: u64,
    pub align: u64,
    /// seq of the event that created this block (alloc or realloc).
    pub born_seq: u64,
    /// Scope-label stack at birth (innermost last).
    pub scopes: Vec<String>,
    /// Address of the first allocation in this block's realloc lineage —
    /// the key the collections panel chains growth staircases by.
    pub lineage_root: u64,
}

/// Live allocations keyed by address. `BTreeMap` so iteration order is
/// deterministic — determinism is requirement R10a, not a nicety.
pub type LiveSet = BTreeMap<u64, Allocation>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TraceError {
    #[error("line {line}: not valid JSON: {msg}")]
    BadJson { line: usize, msg: String },
    #[error("line {line}: missing or malformed field {field}")]
    BadField { line: usize, field: &'static str },
    #[error("seq {seq}: not strictly increasing (previous {prev})")]
    SeqOrder { seq: u64, prev: u64 },
    #[error("seq {seq}: {op} of {addr:#x} does not match a live allocation (trace unbalanced)")]
    Unbalanced {
        seq: u64,
        op: &'static str,
        addr: u64,
    },
    #[error("seq {seq}: alloc at {addr:#x} but that address is already live")]
    DoubleAlloc { seq: u64, addr: u64 },
    #[error("seq {seq}: scope_exit '{label}' does not match open scope")]
    ScopeMismatch { seq: u64, label: String },
    #[error("seq {seq}: trace contains a loss marker ({dropped} dropped) — balance is undefined")]
    ContainsLoss { seq: u64, dropped: u64 },
}

/// Parse envelope JSONL into a [`Trace`]. Non-memlens lines are rejected;
/// malformed lines fail loudly with their line number.
pub fn parse(input: &str) -> Result<Trace, TraceError> {
    let mut trace = Trace::default();
    for (i, line) in input.lines().enumerate() {
        let n = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).map_err(|e| TraceError::BadJson {
            line: n,
            msg: e.to_string(),
        })?;
        let ty = v["event_type"].as_str().ok_or(TraceError::BadField {
            line: n,
            field: "event_type",
        })?;
        let p = &v["payload"];
        let field = |f: &'static str| -> Result<u64, TraceError> {
            p[f].as_u64()
                .ok_or(TraceError::BadField { line: n, field: f })
        };
        let addr_field = |f: &'static str| -> Result<u64, TraceError> {
            p[f].as_str()
                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                .ok_or(TraceError::BadField { line: n, field: f })
        };
        let label = || -> Result<String, TraceError> {
            Ok(p["label"]
                .as_str()
                .ok_or(TraceError::BadField {
                    line: n,
                    field: "label",
                })?
                .to_owned())
        };
        match ty {
            "memlens.meta" => {
                trace.meta = Some(Meta {
                    program: p["program"].as_str().unwrap_or_default().to_owned(),
                    pid: p["pid"].as_u64().unwrap_or_default(),
                    started_at: p["started_at"].as_str().unwrap_or_default().to_owned(),
                    version: p["version"].as_str().unwrap_or_default().to_owned(),
                });
            }
            "memlens.alloc" | "memlens.dealloc" => {
                let kind = if ty == "memlens.alloc" {
                    EventKind::Alloc {
                        addr: addr_field("addr")?,
                        size: field("size")?,
                        align: field("align")?,
                    }
                } else {
                    EventKind::Dealloc {
                        addr: addr_field("addr")?,
                        size: field("size")?,
                        align: field("align")?,
                    }
                };
                trace.events.push(Event {
                    seq: field("seq")?,
                    kind,
                });
            }
            "memlens.realloc" => trace.events.push(Event {
                seq: field("seq")?,
                kind: EventKind::Realloc {
                    old_addr: addr_field("old_addr")?,
                    new_addr: addr_field("new_addr")?,
                    old_size: field("old_size")?,
                    new_size: field("new_size")?,
                    align: field("align")?,
                },
            }),
            "memlens.scope_enter" => trace.events.push(Event {
                seq: field("seq")?,
                kind: EventKind::ScopeEnter { label: label()? },
            }),
            "memlens.scope_exit" => trace.events.push(Event {
                seq: field("seq")?,
                kind: EventKind::ScopeExit { label: label()? },
            }),
            "memlens.marker" => trace.events.push(Event {
                seq: field("seq")?,
                kind: EventKind::Marker {
                    label: label()?,
                    kind: p["kind"].as_str().map(str::to_owned),
                },
            }),
            "memlens.loss" => trace.events.push(Event {
                seq: field("seq")?,
                kind: EventKind::Loss {
                    dropped: field("dropped")?,
                },
            }),
            _ => {
                return Err(TraceError::BadField {
                    line: n,
                    field: "event_type",
                });
            }
        }
    }
    Ok(trace)
}

/// R2 — trace balance: every dealloc/realloc must reference the live
/// allocation's exact (addr, size); seq strictly increases; scopes nest.
/// A corollary the property tests exploit: a balanced trace can never drive
/// live bytes negative.
pub fn validate(events: &[Event]) -> Result<(), TraceError> {
    let mut live: BTreeMap<u64, (u64, u64)> = BTreeMap::new(); // addr -> (size, align)
    let mut scopes: Vec<String> = Vec::new();
    let mut prev = 0u64;
    for e in events {
        if e.seq <= prev {
            return Err(TraceError::SeqOrder { seq: e.seq, prev });
        }
        prev = e.seq;
        match &e.kind {
            EventKind::Alloc { addr, size, align } => {
                if live.insert(*addr, (*size, *align)).is_some() {
                    return Err(TraceError::DoubleAlloc {
                        seq: e.seq,
                        addr: *addr,
                    });
                }
            }
            EventKind::Dealloc { addr, size, .. } => match live.remove(addr) {
                Some((s, _)) if s == *size => {}
                _ => {
                    return Err(TraceError::Unbalanced {
                        seq: e.seq,
                        op: "dealloc",
                        addr: *addr,
                    });
                }
            },
            EventKind::Realloc {
                old_addr,
                new_addr,
                old_size,
                new_size,
                align,
            } => {
                match live.remove(old_addr) {
                    Some((s, _)) if s == *old_size => {}
                    _ => {
                        return Err(TraceError::Unbalanced {
                            seq: e.seq,
                            op: "realloc",
                            addr: *old_addr,
                        });
                    }
                }
                if live.insert(*new_addr, (*new_size, *align)).is_some() {
                    return Err(TraceError::DoubleAlloc {
                        seq: e.seq,
                        addr: *new_addr,
                    });
                }
            }
            EventKind::ScopeEnter { label } => scopes.push(label.clone()),
            EventKind::ScopeExit { label } => {
                if scopes.pop().as_deref() != Some(label.as_str()) {
                    return Err(TraceError::ScopeMismatch {
                        seq: e.seq,
                        label: label.clone(),
                    });
                }
            }
            EventKind::Marker { .. } => {}
            EventKind::Loss { dropped } => {
                return Err(TraceError::ContainsLoss {
                    seq: e.seq,
                    dropped: *dropped,
                });
            }
        }
    }
    Ok(())
}

/// R10a — the live-allocation set after applying events with `seq <= t`,
/// in order. Pure fold: same events, same `t` → same result, always.
#[must_use]
pub fn replay(events: &[Event], t: u64) -> LiveSet {
    let mut live = LiveSet::new();
    let mut scopes: Vec<String> = Vec::new();
    for e in events.iter().take_while(|e| e.seq <= t) {
        match &e.kind {
            EventKind::Alloc { addr, size, align } => {
                live.insert(
                    *addr,
                    Allocation {
                        size: *size,
                        align: *align,
                        born_seq: e.seq,
                        scopes: scopes.clone(),
                        lineage_root: *addr,
                    },
                );
            }
            EventKind::Dealloc { addr, .. } => {
                live.remove(addr);
            }
            EventKind::Realloc {
                old_addr,
                new_addr,
                new_size,
                align,
                ..
            } => {
                let root = live.remove(old_addr).map_or(*old_addr, |a| a.lineage_root);
                live.insert(
                    *new_addr,
                    Allocation {
                        size: *new_size,
                        align: *align,
                        born_seq: e.seq,
                        scopes: scopes.clone(),
                        lineage_root: root,
                    },
                );
            }
            EventKind::ScopeEnter { label } => scopes.push(label.clone()),
            EventKind::ScopeExit { .. } => {
                scopes.pop();
            }
            EventKind::Marker { .. } | EventKind::Loss { .. } => {}
        }
    }
    live
}

/// R3 — live bytes after the prefix `seq <= t`: exactly
/// Σ alloc'd − Σ freed over that prefix.
#[must_use]
pub fn live_bytes(events: &[Event], t: u64) -> u64 {
    let mut bytes: u64 = 0;
    for e in events.iter().take_while(|e| e.seq <= t) {
        match &e.kind {
            EventKind::Alloc { size, .. } => bytes += size,
            EventKind::Dealloc { size, .. } => bytes = bytes.saturating_sub(*size),
            EventKind::Realloc {
                old_size, new_size, ..
            } => {
                bytes = bytes.saturating_sub(*old_size) + new_size;
            }
            _ => {}
        }
    }
    bytes
}
