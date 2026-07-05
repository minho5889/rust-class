# Design — 002 memory-lens

> **Doc 3 of 4. Derived from approved `requirements.md`.** Audited by `spec-auditor`
> before review. ✋ **Human gate.**

**Status:** approved
**Approved:** 2026-07-05 by Minho · **Assurance verdict:** design 74% → revised per
`_assurance/design-review.md` (both MAJORs + all MEDIUMs addressed, rev 2)

## High-level design (the shape)

Three pieces, connected by one file format:

```
your Rust program                     trace file                    browser
┌─────────────────────────┐   JSONL   ┌──────────────┐   open in   ┌─────────────────┐
│ #[global_allocator]     │  ───────► │ memlens.*    │  ─────────► │ viewer/         │
│ MemLens<System>         │           │ envelope     │             │ memlens.html    │
│ + lens_scope!/lens_var! │           │ events       │             │ (single file,   │
│   teaching macros       │           │ (→ datalake) │             │  no server)     │
└─────────────────────────┘           └──────────────┘             └─────────────────┘
     crates/memlens                                        replay engine: crates/memlens-replay
     (capture, R1–R8, R14b, R16)                           (fold, R3/R10a properties) + JS mirror
```

1. **`crates/memlens`** — a tracking allocator: a `MemLens<A>` wrapper around any
   `GlobalAlloc` (default `System`) that records every alloc/realloc/dealloc as an
   envelope event, plus teaching macros that mark scopes, drops, moves, and borrows.
   Feature-flagged; compile-fails in release/bench profiles.
2. **`crates/memlens-replay`** — a headless engine that folds a trace into "the set
   of live allocations at sequence point t" and "live bytes at t". This tiny pure
   function is where the unit's [P] properties live. The viewer's JS reimplements
   the same fold and is pinned to the Rust engine by golden fixture files.
3. **`viewer/memlens.html`** — one self-contained HTML file (vanilla JS + canvas,
   no build step, no server). Open it, drop a trace in, scrub.

Why this shape: the trace file is the contract. Capture and viewing are fully
decoupled — any program can be lensed, any trace can be viewed later, and traces
accumulate in the data lake for future units to mine.

## Detailed design

### Capture: `MemLens<A: GlobalAlloc>` (R1, R2, R6, R7a/b, R14b)

- `unsafe impl GlobalAlloc for MemLens<A>` forwards to the inner allocator, then
  records the event. **`seq` is assigned inside the writer lock** (below), so file
  order and sequence order are the same thing by construction — replay and the
  viewer can stream without sorting. (A lock-free `AtomicU64` seq was considered
  and rejected: seq taken outside the lock can be written out of order by
  concurrent threads, silently breaking every replay invariant. The lock already
  exists; one ordering authority beats two. This rejection is itself the teaching
  moment on ordering.)
- **Reentrancy guard (the key trick):** recording an event itself allocates
  (JSON string, buffer growth). A `thread_local` `IN_LENS: Cell<bool>` flag makes
  the allocator transparent while the recorder runs — the tracer never traces
  itself. The cell is **const-initialized** (`thread_local!` with `const { … }`)
  and accessed via `try_with`, falling back to plain forwarding (no recording) if
  TLS is unavailable during thread init/teardown — those windows lose events, never
  panic (R7a). Teaching moment for `thread_local!` and why `GlobalAlloc` must be
  reentrancy-aware.
- **Writer:** a `Mutex<BufWriter<File>>` (path from `MEMLENS_TRACE` env, default
  `datalake/raw-local/traces/dt=YYYY-MM-DD/<name>-<pid>.jsonl`, matching the lake's
  partition convention). Simple over clever: correctness first, overhead measured
  honestly (R14a). Flush on `Drop` of a `LensSession` guard and via an
  `atexit`-style hook.
- **Failure path (R7a/b):** all writes are best-effort; on error the event is
  dropped and a `dropped_events: AtomicU64` counter increments; the first
  successful write afterwards (or trace end) emits one `memlens.loss` marker with
  the count. No `panic!`, no `abort`, ever, in the allocator path.
- **Feature gating (R6):** everything is behind `#[cfg(feature = "memlens")]`; with
  the feature off, `MemLens<A>` still exists as a zero-field passthrough whose
  methods inline to the inner allocator — so the learner's two-line setup (R16)
  compiles unchanged in both configurations; only recording disappears (verified by
  symbol-absence check, task-level).
- **Build guard (R14b):** `#[cfg(all(feature = "memlens", not(debug_assertions)))]
  compile_error!("memlens is a teaching instrument: never in release/bench builds");`
  — `not(debug_assertions)` is a *proxy* for "release/bench profile" (it holds for
  default `release` and `bench`, but a custom profile could re-enable
  debug-assertions); the proxy and its limits are documented at the guard.

### Teaching macros (R4, R12, R16)

- `lens_scope!("label", { … })` — emits `scope_enter`/`scope_exit` events around the
  block (exit emitted by a `Drop` guard, so early returns and panics still close
  the scope correctly — ownership teaching for free).
- `lens_var!(x)` / `lens_drop!(x)` — labels a variable; ties its address to a name.
- `lens_move!(x → y)` and `lens_borrow!(&x)` — emit marker events the viewer renders
  as **zero-allocation callouts** (R12): "a move happened here; note no alloc events."
- Ergonomics (R16): instrumenting a program = add dependency + two lines
  (`#[global_allocator] static A: MemLens<System> = MemLens::system();`) + macros
  only where the learner wants labels.

### Trace format (R8)

Envelope v1-compatible JSONL, `actor: "memlens"`, `event_type: memlens.{alloc |
realloc | dealloc | scope_enter | scope_exit | marker | loss | meta}`. Payloads:

- `alloc` / `dealloc`: `{addr: "0x…", size, align, seq, label?}`
- `realloc` (its own shape — lineage is the point): `{old_addr, new_addr,
  old_size, new_size, align, seq}` — R2 checks (old_addr, old_size) against the
  live set; R1/R5 use (new_addr, new_size); the collections panel chains
  old→new links into growth staircases.
- `scope_enter`/`scope_exit`/`marker`: `{label, kind?, seq}`.

A `memlens.meta` header event records program name, pid, start time, memlens
version. Schema registered as `datalake/schema/memlens.v1.json`. Addresses are
opaque strings — useful for matching, meaningless across runs (ASLR), and stated
so (R15 honesty).

### Replay engine: `crates/memlens-replay` (R3, R10a)

Pure function, no I/O in the core:

```rust
fn validate(events: &[Event]) -> Result<(), TraceError>  // R2: trace balance
fn replay(events: &[Event], t: u64) -> LiveSet   // BTreeMap<Addr, Allocation>
fn live_bytes(events: &[Event], t: u64) -> u64   // prefix fold
```

- `validate` is the explicit R2 checker: every `dealloc`/`realloc` must reference
  the live allocation's exact (addr, size); violations yield
  `TraceError::Unbalanced { seq }`. The R2 property tests target this function —
  including adversarial synthetic traces that it must *reject*.
- `LiveSet` = fold of events `1..=t`: alloc inserts, dealloc removes, realloc
  removes its `old_addr` and inserts its `new_addr`; scope labels attach to
  allocations whose events fall inside the scope's enter/exit window.
- This is deliberately a *pure fold over an immutable slice* — the properties
  (below) test it exhaustively, and the viewer JS mirrors exactly this function.
- Golden fixtures: `memlens-replay/fixtures/*.jsonl` + expected live-set snapshots;
  the JS fold must reproduce them bit-for-bit (checked by a task-level test page).

### Viewer: `viewer/memlens.html` (R9–R13, R15)

- Single file, inline CSS/JS, zero dependencies (R11). Loads a trace via file
  input or drag-drop; parses JSONL in a streaming loop.
- **Layout (R9, all visible at once):** top — scrubbable heap timeline (canvas:
  x = sequence, y = address-bucket lanes, bars = allocation lifetimes); middle —
  live-bytes area chart with scope bands; bottom-left — collections panel (realloc
  chains grouped by address lineage → capacity staircase, the `Vec` doubling /
  `HashMap` rehash picture, R5); bottom-right — live-set table at the scrub point
  with scope labels (R10b).
- **Performance (R13, R10b):** canvas rendering (no DOM per event). Two indexes
  precomputed once at load: prefix-sum arrays for live-*bytes* at any t (O(log n)
  lookup), and a **lifetime interval index** — each allocation's [birth seq,
  death seq) — which serves double duty as the timeline's bars *and* the
  live-*set* query (allocations whose interval contains t), so the scrub-point
  table (R10b) is O(log n + live count), not a re-fold; 100k events ≪ 100 ms per
  step on any modern laptop.
- **Zero-cost callouts (R12):** `marker` events render as flagged points on the
  timeline; clicking one shows "move/borrow here — 0 bytes allocated."
- **Honesty footer (R15):** a fixed panel: "This lens sees the heap only. Stack
  frames, registers, and non-annotated borrows are invisible here — that
  invisibility is the point: they cost nothing at runtime."

## Key decisions

| Decision | Options considered | Chosen | Why | REQs |
|---|---|---|---|---|
| Writer concurrency | lock-free ring buffer; per-thread files; `Mutex<BufWriter>` | `Mutex<BufWriter>` + reentrancy guard | Teaching clarity and provable correctness beat throughput; overhead is measured and documented, not hidden | R1, R14a |
| `seq` assignment | `AtomicU64` outside the lock; inside the writer lock | inside the writer lock | Atomic-outside lets concurrent threads write out of seq order, breaking replay's "events 1..t in order" contract; one ordering authority, file order ≡ seq order by construction | R1, R3, R10a |
| Event format | compact binary; JSONL | JSONL (envelope v1) | Lake compatibility, human-readable for learning, size fine at ≤100k events | R8, R13 |
| Where the fold lives | JS only; Rust only (WASM in viewer); Rust + JS mirror | Rust engine (properties) + JS mirror pinned by golden fixtures | [P] tests belong in Rust/proptest; WASM would break "single self-contained file" simplicity at v0 | R3, R10a, R11 |
| Viewer stack | React/deps + bundler; vanilla JS + canvas | vanilla + canvas, one file | R11 (no server/build), R13 (canvas perf), zero toolchain for the learner | R9, R11, R13 |
| Realloc identity | track by address equality; explicit lineage (old→new addr) | lineage from realloc events | Growth chains (collections panel) need old→new links; address reuse would corrupt chains | R5 |

## Properties (one per [P] requirement)

| REQ | Property (∀ inputs, precondition ⇒ invariant) | Generation strategy |
|---|---|---|
| R1 | For any generated sequence of harness operations (allocate/grow/free boxed buffers), every operation appears in the trace exactly once, with strictly increasing `seq` | proptest: `Vec<Op>` where `Op ∈ {Alloc(size 1..64KiB, align ∈ {1,2,4,8,16}), Grow(idx, factor 1..4), Free(idx)}`, valid-by-construction indices; execute against `MemLens<System>`, parse own trace |
| R2 | For any such trace, every `dealloc`/`realloc` event's (addr, size) matches the latest live allocation of that addr; corollary: live bytes never negative at any prefix | same harness traces + separately, adversarial *synthetic* traces (shuffled/duplicated events) must be *rejected* by the validator — tests the checker itself |
| R3 | For any trace and any prefix t: `live_bytes(events, t) == Σ alloc sizes − Σ freed sizes` over that prefix, computed by an independent naive reference fold | same generated traces; reference fold written separately from the engine (different author-step, no shared code) |
| R10a | For any valid trace and any t: `replay(events, t)` equals the reference interpreter's live map; and replay is deterministic across repeated calls and event-slice copies | generated traces from R1 strategy + edge cases via shrinking (empty trace, t=0, t=max, realloc chains) |

Default ≥256 cases each; `proptest-regressions/` committed (constitution).

## Rust concepts in play (teaching hook)

- `GlobalAlloc` + `unsafe impl` — what the contract really promises (L1); SKILLS 1b
  (stack vs heap / allocation) — note: allocators themselves aren't yet a SKILLS
  line; this unit adds depth to 1b rather than mapping 1:1.
- `thread_local!` + reentrancy — why the tracer must not trace itself; SKILLS 1d
  (`Send`/`Sync` territory, previewed early).
- `Mutex` as the single ordering authority for `seq` — and why the "obvious"
  lock-free atomic was wrong; SKILLS 1d preview.
- `Drop` guards in `lens_scope!` — deterministic cleanup even on early return/panic
  (L4); SKILLS 1b.
- Moves/borrows produce **no events** — the zero-cost story made visible (L2, R12).
- `Vec`/`HashMap` growth observed live (L3); `Box` moving values to the heap (L5).

## Error handling

- Allocator path: infallible by construction — errors degrade to dropped events +
  one loss marker (R7a/b). The traced program's behavior is never altered.
- Replay engine: `Result<LiveSet, TraceError>` (`thiserror`) — malformed traces
  fail loudly with the offending seq number.
- Viewer: parse errors → banner with line number; never a blank screen.

## Verification strategy

- **[P]** R1/R2/R3/R10a: proptest suites in `memlens` (harness-driven) and
  `memlens-replay` (trace-driven), written before implementation (constitution).
- **[E]** R4/R5 (fixture programs with known shapes), R6 (build example twice,
  assert no trace + `nm`-based symbol absence in a test script), R7a/b (sink pointed
  at a full/closed fd), R8 (schema validation against `memlens.v1.json`), R14b
  (compile-fail test via `trybuild`), R16 (the example diff *is* the test).
- **[O]** R9, R10b, R11, R12, R13, R15: learner session — run a prepared exercise,
  scrub the trace, record observations in `evidence.md`.
- **[O]** R14a overhead procedure: a fixed workload binary (the collections
  exercise, ~100k events) built in debug twice — feature off vs on — timed with
  `hyperfine` (or 10-run manual median); report absolute times, ratio, and trace
  size in `evidence.md`. No claims beyond that workload.

## Changelog

| Date | Change | Trigger (change protocol) | Re-gated? |
|---|---|---|---|
| 2026-07-05 | Rev 2: dedicated realloc payload with old→new lineage; seq assigned inside writer lock (file order ≡ seq order); explicit `validate()` for R2; lifetime interval index for R10b/R13 live-set queries; R14a measurement procedure (hyperfine, fixed workload); const-init TLS + `try_with` fallback; R14b proxy documented; trace path follows `dt=` partitions; R6/R16 passthrough interplay clarified; SKILLS 1d citations | spec-auditor findings 1–6 + minors (74%) | pre-gate revision, no approval yet |
