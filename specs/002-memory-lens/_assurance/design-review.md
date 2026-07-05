# Design Review — 002 memory-lens

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-05
**Audited doc:** `../design.md` (Status: drafting)
**Inputs:** `../requirements.md` (Status: approved, 2026-07-05) only, plus
`CLAUDE.md` (Properties/pipeline rules) and `SKILLS.md` (teaching mapping).

---

## Traceability map

All 19 EARS lines (R1–R6, R7a/b, R8, R9, R10a/b, R11–R13, R14a/b, R15, R16) and
L1–L5 checked against design elements:

| REQ | Design element | Status |
|---|---|---|
| R1 | Capture: `GlobalAlloc` impl + `AtomicU64` seq | ✅ (but see Finding 2) |
| R2 | Capture + Properties row | ⚠️ validator undesigned (Finding 3) |
| R3 | Replay engine `live_bytes` + Properties row | ✅ |
| R4 | Teaching macros (`lens_scope!` Drop guard) | ✅ |
| R5 | Collections panel + realloc-lineage decision | ⚠️ payload can't carry lineage (Finding 1) |
| R6 | Feature gating, passthrough + symbol-absence | ✅ (see Finding 8) |
| R7a/b | Failure path (drop + loss marker, no panic) | ✅ |
| R8 | Trace format, `memlens.v1.json` schema | ✅ (see Finding 9) |
| R9 | Viewer layout (three panels at once) | ✅ |
| R10a | Replay engine `replay()` + Properties row | ✅ (but see Finding 2) |
| R10b | Viewer live-set table with scope labels | ⚠️ perf mechanism gap (Finding 4) |
| R11 | Single-file viewer, no deps | ✅ |
| R12 | Marker events + zero-cost callouts | ✅ |
| R13 | Canvas + precomputed prefix sums | ⚠️ covers live-bytes only (Finding 4) |
| R14a | "Overhead measured honestly" (one clause) | ⚠️ no measurement design (Finding 5) |
| R14b | `compile_error!` build guard | ✅ (see Finding 7) |
| R15 | Honesty footer | ✅ |
| R16 | Ergonomics: dep + two lines + opt-in macros | ✅ |
| L1–L5 | "Rust concepts in play" section | ✅ present; SKILLS mapping thin (Finding 10) |

No design element cites zero REQs; no scope creep found. The JS-mirror +
golden-fixtures scheme cites R3/R10a/R11 via the Key decisions table and is a
legitimate implementation strategy for R10b, not creep. Properties table has a
row for all four [P] requirements (R1, R2, R3, R10a). Key decisions: five rows,
all with genuine alternatives and real reasons — no straw men detected.

---

## Findings (by severity)

### MAJOR

**1. The realloc event payload cannot represent the lineage the design itself
depends on.** The trace format specifies one payload shape for all ops:
`{op, addr: "0x…", size, align, seq, label?}` — a single address and a single
size. But (a) the Key decisions row "Realloc identity" chooses *explicit lineage
(old→new addr)* precisely because "growth chains need old→new links; address
reuse would corrupt chains" (R5), (b) the replay fold does "realloc
removes+inserts" — remove *which* addr, insert *which*?, and (c) R2 requires a
realloc event to match "the latest live allocation of that addr", which needs
the old (addr, size) pair while R1 needs the new one. As written, the schema,
the fold, and the decision table contradict each other; a human approving this
doc would be approving an unimplementable contract.
**Fix:** give `memlens.realloc` its own payload:
`{op:"realloc", old_addr, new_addr, old_size, new_size, align, seq, label?}`,
update the fold description ("remove old_addr, insert new_addr"), and reflect
it in `memlens.v1.json` and the R1/R2 property statements.

**2. Cross-thread write ordering breaks the "events 1..t in order" assumption.**
`seq` comes from a lock-free `fetch_add`, but the event is serialized and
written later under a *separate* `Mutex<BufWriter>`. Under concurrency, thread
B can win the mutex after thread A took the earlier seq — so **file order ≠ seq
order**. Yet the replay engine folds "events `1..=t`" over a slice, R10a's
invariant is defined as "applying events 1..t in order", and the viewer "parses
JSONL in a streaming loop". Nothing says who sorts. Every [P] property and the
R3 prefix fold silently assume an ordering the capture path does not guarantee.
**Fix:** pick one and state it: (a) assign `seq` *inside* the writer lock so
file order == seq order (simplest; the lock exists anyway and the design
already chose "simple over clever"), or (b) specify that `memlens-replay` and
the viewer sort by `seq` before folding, and add an out-of-order fixture to the
R10a generation strategy. Option (a) also makes the "monotonic ordering without
a global lock" claim honest — currently there *is* a global lock, so the
lock-free seq buys nothing.

### MEDIUM

**3. R2's property references a "validator" that exists nowhere in the design.**
The generation strategy says adversarial synthetic traces "must be *rejected*
by the validator — tests the checker itself." The only candidate is the replay
engine's `Result<LiveSet, TraceError>`, but it is never named as the R2 trace-
balance checker, and the Error handling section frames `TraceError` as
"malformed traces" (parse-level), not balance violations (dealloc without prior
alloc, size mismatch).
**Fix:** add an explicit element to `memlens-replay` — e.g.
`fn validate(events: &[Event]) -> Result<(), TraceError>` enforcing the R2
invariants (unknown-addr dealloc/realloc, size mismatch, duplicate live addr) —
cite R2 on it, and make the property row point at it.

**4. R13/R10b performance design covers live-*bytes* but not the live-*set*.**
"Prefix-sum arrays precomputed once at load so any scrub position is O(log n)"
is true for the live-bytes chart only. The bottom-right live-set *table* at an
arbitrary t (R10b) is not derivable from a prefix sum — reconstructing "the set
of live allocations at t" naively is O(t) per scrub step, which is exactly what
the 100 ms / 100k-events budget (R13) exists to constrain.
**Fix:** one sentence of mechanism, e.g. allocation lifetimes as intervals
`[alloc_seq, dealloc_seq)` indexed once at load (the timeline already computes
these bars — reuse them), or checkpoint snapshots every k events + delta
replay. Then the ≪100 ms claim is designed, not asserted.

**5. R14a has no design element.** The overhead requirement gets one adverb
("measured honestly") and an [O] verification line, but the design never says
*what* is measured or *how*: which fixture workload, feature-on vs feature-off
wall time? allocation-path latency? trace-file size? Without this, the learner
session in the verification strategy has no procedure to follow.
**Fix:** add a short "Overhead measurement" element: run the R5 fixture program
N times with feature on/off, report wall-time ratio and events/sec + trace
size in `evidence.md`. Two sentences suffice; cite R14a.

**6. The `thread_local` reentrancy guard has undesigned edge cases that
threaten the "no `panic!`, ever, in the allocator path" claim (R7a).**
`GlobalAlloc` can be invoked before the guard's TLS is initialized and during
thread teardown, where `LocalKey::with` **panics** (access after destruction).
A lazily-initialized `thread_local!` can itself allocate on first access on
some platforms, re-entering the allocator before the flag exists.
**Fix:** specify `thread_local! { static IN_LENS: Cell<bool> = const { Cell::new(false) } }`
(const-init, non-allocating) and use `try_with`, forwarding to the inner
allocator *without recording* whenever TLS is unavailable (counted as dropped
events → R7b loss marker). This is also a better teaching beat than the current
one-liner.

### MINOR

**7. R14b guard uses `not(debug_assertions)` as a proxy for "release or
benchmark build profile".** A custom profile inheriting `release` with
`debug-assertions = true` (or `[profile.release] debug-assertions = true`)
slips through; conversely a dev profile with them off would falsely fail.
Acceptable for a teaching crate, but the design should *say* it's a proxy and
note the escape hatch, so the `trybuild` test in the verification strategy
tests the real mechanism.

**8. R6 vs R16 tension left implicit.** With the feature off, the learner's
`#[global_allocator] static A: MemLens<System>` line (which R16 says stays put)
still *does* install `MemLens` — R6's literal "the global allocator is not
replaced" is satisfied only in the observable sense (passthrough inlines away;
symbol-absence check). The design clearly intends the observable reading; add
one sentence saying so, so the human isn't left to reconcile it.

**9. Default trace path deviates from lake layout.**
`datalake/raw-local/traces/<name>-<pid>.jsonl` vs the lake's
`datalake/raw-local/dt=YYYY-MM-DD/*.jsonl` Hive-partition convention
(CLAUDE.md). R8's whole point is "traces land in the goldeneye lake" — either
use `dt=` partitioning (`…/dt=YYYY-MM-DD/memlens-<name>-<pid>.jsonl`) or record
the divergence as a Key decision.

**10. SKILLS mapping in "Rust concepts in play" is thin.** Only the first
bullet cites a SKILLS item (1b). Map the rest: `Drop` guards → 1b drop order;
`Vec`/`HashMap`/`String` growth → 1b slices/`Vec` growth; `Box` → 1b stack vs
heap; `AtomicU64`/`Sync` → 1d (worth flagging as deliberately ahead of the
curriculum path — the doc's "first taste" framing is good, make the 1d pointer
explicit). Note `GlobalAlloc`/`unsafe impl` appears nowhere in SKILLS.md — fine
for the design, but the skill tree should gain a line at spec close.

**11. Verification-strategy nit:** the [O] row says "R9–R13", a range that
sweeps in R10a — which is [P] and already listed there. Write "R9, R10b,
R11–R13". Also, the R10a property row's "edge cases via shrinking" is slightly
backwards — shrinking minimizes failures, it doesn't generate edge cases; list
empty trace / t=0 / t=max as explicit strategy cases (the row half-does this
already).

---

## What is already good

- The trace-file-as-contract shape is exactly right for the requirements: it
  makes R11 trivial, R8 structural, and decouples every [O] from every [P].
- Rust engine + JS mirror pinned by golden fixtures is a genuinely good answer
  to the "where do properties live for a browser tool" problem, and the Key
  decisions row shows real alternatives (WASM considered and rejected for a
  reason).
- The R2 property's "the checker itself is tested with adversarial traces" is
  a sophisticated move — it just needs the checker to exist (Finding 3).
- Failure-path design (best-effort + loss marker) maps R7a/R7b cleanly and the
  reentrancy guard is the right core idea (it needs Finding 6's hardening).
- Error handling and verification strategy assign every requirement to a
  concrete test mechanism — rare and welcome.

## Score rationale

Traceability is complete (19/19 EARS + L1–L5), the Properties table covers all
four [P]s with mostly concrete strategies, and decisions are genuine. But the
two MAJORs are internal contradictions in the doc's own contract layer — the
realloc payload cannot express the lineage the design requires, and the
capture/replay pair disagrees about event ordering under threads. A human
approving this design would be freezing a trace schema that must change. Fix
Findings 1–2 (each is a few lines), address 3–6, and this approves fast.

VERDICT: 74% — Complete traceability and a strong shape, but the realloc payload contradicts the lineage decision (R5/R2) and the seq-vs-file-order gap undermines every [P] property; both contract-level MAJORs must be fixed before the human gate.
