//! [P] R2 + R3 + R10a for the replay engine.
//!
//! - Generator: synthetic valid-by-construction traces (fresh or recycled
//!   addresses, reallocs, nested scopes) — the engine's input domain per
//!   design's Properties table.
//! - Reference: a deliberately naive interpreter written HERE, sharing no
//!   code with the engine. If engine and reference agree on every generated
//!   trace and every prefix, the fold is right (or both are wrong the same
//!   way — which the adversarial and real-trace tests then attack).

use memlens_replay::{Allocation, Event, EventKind, LiveSet, live_bytes, replay, validate};
use proptest::prelude::*;

// ---------- synthetic trace generator ----------

#[derive(Debug, Clone)]
enum GenOp {
    Alloc {
        size: u64,
        align_pow: u8,
        recycle: bool,
    },
    Grow {
        pick: usize,
        factor: u64,
    },
    Free {
        pick: usize,
    },
    Scope {
        enter: bool,
    },
    Marker,
}

fn gen_ops() -> impl Strategy<Value = Vec<GenOp>> {
    prop::collection::vec(
        prop_oneof![
            4 => (1u64..=65536, 0u8..=4, any::<bool>())
                .prop_map(|(size, align_pow, recycle)| GenOp::Alloc { size, align_pow, recycle }),
            2 => (any::<usize>(), 1u64..=4).prop_map(|(pick, factor)| GenOp::Grow { pick, factor }),
            2 => any::<usize>().prop_map(|pick| GenOp::Free { pick }),
            1 => any::<bool>().prop_map(|enter| GenOp::Scope { enter }),
            1 => Just(GenOp::Marker),
        ],
        1..60,
    )
}

/// Interpret GenOps into a valid event sequence.
fn build_trace(ops: &[GenOp]) -> Vec<Event> {
    let mut events = Vec::new();
    let mut seq = 0u64;
    let mut next_addr = 0x1000u64;
    let mut live: Vec<(u64, u64, u64)> = Vec::new(); // (addr, size, align)
    let mut freed: Vec<u64> = Vec::new();
    let mut depth = 0u32;
    let mut scope_n = 0u32;
    let mut open: Vec<String> = Vec::new();
    for op in ops {
        seq += 1;
        match op {
            GenOp::Alloc {
                size,
                align_pow,
                recycle,
            } => {
                // Address reuse after free is legal and must be handled.
                let addr = if *recycle && !freed.is_empty() {
                    freed.pop().expect("nonempty")
                } else {
                    next_addr += 0x100;
                    next_addr
                };
                let align = 1u64 << align_pow;
                live.push((addr, *size, align));
                events.push(Event {
                    seq,
                    kind: EventKind::Alloc {
                        addr,
                        size: *size,
                        align,
                    },
                });
            }
            GenOp::Grow { pick, factor } => {
                if live.is_empty() {
                    seq -= 1;
                    continue;
                }
                let i = pick % live.len();
                let (old_addr, old_size, align) = live[i];
                // Sometimes grow in place (same addr), sometimes move.
                let new_addr = if factor % 2 == 0 {
                    old_addr
                } else {
                    next_addr += 0x100;
                    next_addr
                };
                let new_size = (old_size * factor).clamp(1, 262_144);
                if new_addr != old_addr {
                    freed.push(old_addr);
                }
                live[i] = (new_addr, new_size, align);
                events.push(Event {
                    seq,
                    kind: EventKind::Realloc {
                        old_addr,
                        new_addr,
                        old_size,
                        new_size,
                        align,
                    },
                });
            }
            GenOp::Free { pick } => {
                if live.is_empty() {
                    seq -= 1;
                    continue;
                }
                let i = pick % live.len();
                let (addr, size, align) = live.swap_remove(i);
                freed.push(addr);
                events.push(Event {
                    seq,
                    kind: EventKind::Dealloc { addr, size, align },
                });
            }
            GenOp::Scope { enter } => {
                if *enter {
                    scope_n += 1;
                    depth += 1;
                    let label = format!("s{scope_n}");
                    open.push(label.clone());
                    events.push(Event {
                        seq,
                        kind: EventKind::ScopeEnter { label },
                    });
                } else if depth > 0 {
                    depth -= 1;
                    let label = open.pop().expect("open scope");
                    events.push(Event {
                        seq,
                        kind: EventKind::ScopeExit { label },
                    });
                } else {
                    seq -= 1;
                }
            }
            GenOp::Marker => events.push(Event {
                seq,
                kind: EventKind::Marker {
                    label: "m".into(),
                    kind: Some("move".into()),
                },
            }),
        }
    }
    events
}

// ---------- the naive reference interpreter (no shared code) ----------

fn reference_live(events: &[Event], t: u64) -> std::collections::HashMap<u64, (u64, u64)> {
    let mut live = std::collections::HashMap::new();
    for e in events {
        if e.seq > t {
            break;
        }
        match &e.kind {
            EventKind::Alloc { addr, size, align } => {
                live.insert(*addr, (*size, *align));
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
                live.remove(old_addr);
                live.insert(*new_addr, (*new_size, *align));
            }
            _ => {}
        }
    }
    live
}

fn reference_bytes(events: &[Event], t: u64) -> u64 {
    reference_live(events, t).values().map(|(s, _)| *s).sum()
}

// ---------- properties ----------

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// R2: the validator accepts every valid-by-construction trace.
    #[test]
    fn r2_validator_accepts_valid_traces(ops in gen_ops()) {
        let events = build_trace(&ops);
        prop_assert_eq!(validate(&events), Ok(()));
    }

    /// R2 adversarial: the validator REJECTS guaranteed-broken mutations —
    /// this tests the checker itself, not just happy paths.
    #[test]
    fn r2_validator_rejects_corrupted_traces(
        ops in gen_ops(),
        mutation in 0u8..4,
        pick in any::<usize>(),
    ) {
        let mut events = build_trace(&ops);
        // Need at least one balance-affecting event to corrupt.
        let heap_idx: Vec<usize> = events.iter().enumerate()
            .filter(|(_, e)| matches!(e.kind,
                EventKind::Alloc{..} | EventKind::Dealloc{..} | EventKind::Realloc{..}))
            .map(|(i, _)| i)
            .collect();
        prop_assume!(!heap_idx.is_empty());
        let i = heap_idx[pick % heap_idx.len()];
        match mutation {
            0 => {
                // Swap with a later event WITHOUT renumbering: seq disorder.
                prop_assume!(events.len() >= 2 && i + 1 < events.len());
                events.swap(i, i + 1);
            }
            1 => {
                // Duplicate a heap event, renumbering seqs afterwards:
                // double alloc / double free / stale realloc.
                //
                // Triage 2026-07-05 (test bug, see _assurance/triage-log.md):
                // an EXACT no-op realloc (old==new addr AND old==new size)
                // duplicates into a still-valid trace — the universal
                // "duplication invalidates" claim was unsound at that corner.
                // Exclude only that corner; in-place reallocs with different
                // sizes remain valid targets (their duplicate has a stale
                // old_size and is genuinely invalid).
                let noop = matches!(&events[i].kind,
                    EventKind::Realloc { old_addr, new_addr, old_size, new_size, .. }
                        if old_addr == new_addr && old_size == new_size);
                prop_assume!(!noop);
                let dup = events[i].clone();
                events.insert(i + 1, dup);
                for (n, e) in events.iter_mut().enumerate() {
                    e.seq = n as u64 + 1;
                }
            }
            2 => {
                // Corrupt a size by +1: dealloc/realloc mismatch, or make an
                // alloc whose later matching free now disagrees.
                let has_pair = match &mut events[i].kind {
                    EventKind::Dealloc { size, .. } => { *size += 1; true }
                    EventKind::Realloc { old_size, .. } => { *old_size += 1; true }
                    _ => false, // Alloc (or non-heap, excluded by heap_idx)
                };
                prop_assume!(has_pair);
            }
            _ => {
                // Delete an alloc but keep the rest (renumber): later
                // dealloc/realloc of that address is now unbalanced...
                let removed_addr = match &events[i].kind {
                    EventKind::Alloc { addr, .. } => *addr,
                    _ => { prop_assume!(false); unreachable!() }
                };
                // ...but only if something later actually references it.
                let referenced = events[i+1..].iter().any(|e| matches!(&e.kind,
                    EventKind::Dealloc { addr, .. } if *addr == removed_addr)
                    || matches!(&e.kind, EventKind::Realloc { old_addr, .. } if *old_addr == removed_addr));
                prop_assume!(referenced);
                events.remove(i);
                for (n, e) in events.iter_mut().enumerate() {
                    e.seq = n as u64 + 1;
                }
            }
        }
        prop_assert!(validate(&events).is_err(), "corruption {mutation} at {i} undetected");
    }

    /// R3: live_bytes equals the reference at EVERY prefix point.
    #[test]
    fn r3_live_bytes_equals_reference_at_every_prefix(ops in gen_ops()) {
        let events = build_trace(&ops);
        let max = events.last().map_or(0, |e| e.seq);
        for t in 0..=max + 1 {
            prop_assert_eq!(live_bytes(&events, t), reference_bytes(&events, t), "at t={}", t);
        }
    }

    /// R10a: replay equals the reference at every prefix, and is
    /// deterministic across repeated calls and cloned inputs.
    #[test]
    fn r10a_replay_equals_reference_and_is_deterministic(ops in gen_ops()) {
        let events = build_trace(&ops);
        let max = events.last().map_or(0, |e| e.seq);
        for t in [0, max / 2, max, max + 7] {
            let engine: LiveSet = replay(&events, t);
            let reference = reference_live(&events, t);
            prop_assert_eq!(engine.len(), reference.len(), "cardinality at t={}", t);
            for (addr, a) in &engine {
                let (size, align) = reference.get(addr)
                    .unwrap_or_else(|| panic!("engine has {addr:#x} the reference lacks at t={t}"));
                prop_assert_eq!(a.size, *size);
                prop_assert_eq!(a.align, *align);
                prop_assert!(a.born_seq <= t, "born in the future");
            }
            // Determinism: same input → identical result, cloned input too.
            let again = replay(&events, t);
            let cloned = replay(&events.clone(), t);
            prop_assert_eq!(&engine, &again);
            prop_assert_eq!(&engine, &cloned);
        }
    }

    /// R2 corollary made explicit: on valid traces, live_bytes never
    /// underflows (saturating_sub in the engine never actually saturates).
    #[test]
    fn r2_corollary_live_bytes_never_negative(ops in gen_ops()) {
        let events = build_trace(&ops);
        prop_assume!(validate(&events).is_ok());
        let mut running: i128 = 0;
        for e in &events {
            match &e.kind {
                EventKind::Alloc { size, .. } => running += i128::from(*size),
                EventKind::Dealloc { size, .. } => running -= i128::from(*size),
                EventKind::Realloc { old_size, new_size, .. } => {
                    running += i128::from(*new_size) - i128::from(*old_size);
                }
                _ => {}
            }
            prop_assert!(running >= 0, "balanced trace drove live bytes negative");
        }
    }
}

/// Scope labels attach to allocations born inside them (unit-level check of
/// the fold's scope tracking; the [O] viewer test shows them to the human).
#[test]
fn scope_labels_attach_to_allocations() {
    let events = vec![
        Event {
            seq: 1,
            kind: EventKind::ScopeEnter {
                label: "outer".into(),
            },
        },
        Event {
            seq: 2,
            kind: EventKind::Alloc {
                addr: 0x10,
                size: 8,
                align: 8,
            },
        },
        Event {
            seq: 3,
            kind: EventKind::ScopeEnter {
                label: "inner".into(),
            },
        },
        Event {
            seq: 4,
            kind: EventKind::Alloc {
                addr: 0x20,
                size: 16,
                align: 8,
            },
        },
        Event {
            seq: 5,
            kind: EventKind::ScopeExit {
                label: "inner".into(),
            },
        },
        Event {
            seq: 6,
            kind: EventKind::Alloc {
                addr: 0x30,
                size: 32,
                align: 8,
            },
        },
    ];
    assert_eq!(validate(&events), Ok(()));
    let live = replay(&events, 6);
    let scopes = |a: u64| -> Vec<String> { live[&a].scopes.clone() };
    assert_eq!(scopes(0x10), vec!["outer"]);
    assert_eq!(scopes(0x20), vec!["outer", "inner"]);
    assert_eq!(scopes(0x30), vec!["outer"]);
    let alloc: &Allocation = &live[&0x20];
    assert_eq!(alloc.lineage_root, 0x20);
}
