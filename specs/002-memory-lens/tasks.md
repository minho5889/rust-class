# Tasks — 002 memory-lens

> **Doc 4 of 4. Derived from approved `design.md`.** ✋ **Human gate on the plan's
> shape, once** — bolts then execute without re-asking. Constitution rule embedded
> throughout: **[P] property tests are written before the code they test.**

**Status:** awaiting-review
**Approved:** — · **Assurance verdict:** tasks 86% → revised per
`_assurance/tasks-review.md` (all MEDIUMs + minors addressed, rev 2)

Layer grain: main task = deliverable (design component) · sub task = one bolt ·
action item = one commit. Layers collapse where a deliverable is one bolt.

## 1. `crates/memlens` — capture (R1, R4–R8, R14b, R16; balance R2 is proven in task 2)

**Deliverable:** any playground program, instrumented with two lines + macros,
emits a valid `memlens.v1` trace into the lake's `dt=` partitions.

### 1.1 Bolt: workspace, skeleton, build guard
- [ ] 1.1.1 Root `Cargo.toml` workspace (`members = ["crates/*"]`) with shared
      `[profile.release]` (thin LTO, codegen-units=1, panic=abort, strip — per
      steering/tech.md 2026-07-05); `crates/memlens` skeleton with `memlens`
      feature flag, `#![deny(unsafe_op_in_unsafe_fn)]`; passthrough `MemLens<A>`
      (feature off)
- [ ] 1.1.2 R14b compile guard + `trybuild` compile-fail test proving release+feature
      builds die with the teaching message
- [ ] 1.1.3 `datalake/schema/memlens.v1.json` (envelope-compatible; distinct realloc
      payload with old→new lineage; `meta` and `loss` shapes included) + serde event
      structs; schema round-trip [E] test

### 1.2 Bolt: the tracking allocator (R1 property-first)
- [ ] 1.2.1 **[P] R1 test first**: proptest ops strategy (`Alloc(1..64KiB, align
      1|2|4|8|16) | Grow(idx, 1..4) | Free(idx)`, valid-by-construction) executing
      against `MemLens<System>`, parsing its own trace — red
- [ ] 1.2.2 Writer: `Mutex<BufWriter<File>>`, **seq assigned inside the lock**,
      `MEMLENS_TRACE` env + `dt=` default path; `memlens.meta` header event emitted
      on session start; flush via `LensSession` drop guard **and** `atexit`-style
      hook (per design)
- [ ] 1.2.3 Reentrancy guard: const-init `thread_local!` + `try_with` fallback-to-
      forward; `unsafe impl GlobalAlloc` recording alloc/realloc/dealloc, with a
      `// SAFETY:` comment on every unsafe block — R1 green at ≥256 cases,
      `proptest-regressions/` committed (may split into two commits: guard, then
      impl)

### 1.3 Bolt: failure paths + feature-off verification
- [ ] 1.3.1 R7a/b [E] tests: sink on closed/full fd → no panic, loss marker with
      dropped count
- [ ] 1.3.2 R6 [E] check: feature-off build produces no trace + symbol-absence
      script (`nm | grep -c memlens == 0`)
- [ ] 1.3.3 R8 [E] end-to-end: a real emitted trace file validates against
      `memlens.v1.json` (not just struct round-trip)

### 1.4 Bolt: teaching macros
- [ ] 1.4.1 `lens_scope!` (Drop-guard exit), `lens_var!`, `lens_drop!`, `lens_move!`,
      `lens_borrow!` emitting labeled/marker events
- [ ] 1.4.2 R4 [E] fixture tests: scope enter/exit ordering vs inner allocations
      (incl. early-return and panic paths) **and** variable labels + move/borrow
      marker emission; R16 verified by the example's diff
- [ ] 1.4.3 R5 [E] fixture: `Vec` doubling + `HashMap` rehash traces show
      reconstructable growth chains via realloc lineage

## 2. `crates/memlens-replay` — replay engine (R2, R3, R10a)

**Deliverable:** validated, deterministic trace replay — the pure core the viewer
mirrors. One bolt. Cross-crate mechanism: `memlens-replay` dev-depends on
`memlens` and regenerates harness traces in-process; checked-in golden fixtures
exist only for the viewer's JS mirror (2.1.4 → 3.1.1).

### 2.1 Bolt: properties first, then the fold
- [ ] 2.1.1 **[P] R2 tests first**: `validate()` accepts all harness-generated
      traces; adversarial synthetic traces (shuffled, duplicated, mismatched sizes)
      are rejected with the offending seq — red
- [ ] 2.1.2 **[P] R3 + R10a tests first**: independent naive reference fold (kept
      deliberately separate from engine code); ∀ trace, ∀ t: `live_bytes` = prefix
      sum, `replay` ≡ reference, deterministic across calls — red
- [ ] 2.1.3 Implement `validate`/`replay`/`live_bytes` (realloc = remove old_addr,
      insert new_addr; scope-label attachment) — all four [P] suites green,
      ≥256 cases, `proptest-regressions/` committed
- [ ] 2.1.4 Generate golden fixtures (`fixtures/*.jsonl` + expected live-set
      snapshots) for the viewer's JS mirror

## 3. `viewer/memlens.html` — the lens (R9–R13, R15)

**Deliverable:** one self-contained file; open trace → scrub memory.

### 3.1 Bolt: JS fold mirror + indexes
- [ ] 3.1.1 JSONL streaming parse; JS fold mirroring `replay()`; golden-fixture
      test page must reproduce Rust snapshots bit-for-bit
- [ ] 3.1.2 Prefix-sum (live bytes) + lifetime interval index (live set / timeline
      bars) built once at load

### 3.2 Bolt: panels + polish (the "great UI/UX" bolt)
- [ ] 3.2.1 Heap timeline canvas (address-bucket lanes, allocation lifetime bars)
      + scrubber; live-set table with scope labels at scrub point (R10b)
- [ ] 3.2.2 Live-bytes area chart with scope bands; collections panel (realloc
      lineage → capacity staircase) (R9, R5)
- [ ] 3.2.3 Zero-cost markers with click-through callout (R12); honesty footer (R15)
- [ ] 3.2.4 R13 check: synthetic 100k-event trace scrubs < 100 ms/step; drag-drop +
      file input; error banner on malformed traces

## 4. Operations — the learner session (R9–R13, R14a, R15, L1–L5)

### 4.1 Bolt: first light
- [ ] 4.1.1 `playground/02-collections-lens/`: prepared exercise (Vec growth,
      String vs &str, Box, HashMap, a move and a borrow — annotated)
- [ ] 4.1.2 Learner runs it, opens the trace, scrubs; observations → `evidence.md`
      (R9–R13, R15)
- [ ] 4.1.3 R14a: hyperfine (or 10-run median) feature-off vs feature-on, debug
      builds, fixed workload → absolute times, ratio, trace size → `evidence.md`
- [ ] 4.1.4 Learner explains L1–L5 unaided; explanations (and gaps) → `evidence.md`

## Operations checklist

- [ ] `cargo fmt` + `cargo clippy -- -D warnings` clean
- [ ] All [P] properties pass (≥256 cases); `proptest-regressions/` committed
- [ ] All [E] tests pass
- [ ] All [O] requirements observed → numbers in `evidence.md`
- [ ] `evidence.md` learnings written; `MEMORY.md` + `SKILLS.md` updated
- [ ] `property-auditor` run: every [P] REQ has a faithful, passing property

Template deviation (declared): the "deployed to AWS target" and "torn down" lines
are omitted — this unit is local-only by requirement R11; nothing to deploy or
tear down.
