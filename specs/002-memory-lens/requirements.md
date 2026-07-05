# Requirements — 002 memory-lens

> **Doc 2 of 4. Derived from `intent.md` ONLY.** Audited by `spec-auditor` before
> review. ✋ **Human gate.**

**Status:** approved
**Approved:** 2026-07-05 by Minho ("All correct move on") · **Assurance verdicts:**
intent 90% · requirements 78% → revised per `_assurance/requirements-review.md`
(all MAJOR/MEDIUM findings addressed, rev 2)

## User stories

- As a learner, I want to run any of my Rust programs with instrumentation and then
  *watch* what it did in memory, so that ownership, allocation, and drop become
  visible phenomena instead of compiler abstractions.
- As a learner, I want collection types' behavior (growth, reallocation, rehashing)
  shown explicitly, so that I understand what `Vec::push`, `String::from`, or
  `HashMap::insert` really costs.
- As a learner, I want the *absence* of allocations to be visible too (moves,
  borrows), so that I see why Rust is efficient where GC languages are not.

## Acceptance criteria (EARS + verification class)

### A. Trace capture (the `goldeneye-memlens` tracking allocator + macros)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R1 | WHEN an instrumented program performs a heap allocation, reallocation, or deallocation THE SYSTEM SHALL record a trace event with operation kind, address, size, alignment, and monotonic sequence number | [P] |
| R2 | WHEN a deallocation or reallocation event is recorded THE SYSTEM SHALL have previously recorded a matching allocation of the same address and size (trace balance; live-bytes non-negativity follows as a corollary) | [P] |
| R3 | WHEN live bytes are computed at any prefix of the trace THE SYSTEM SHALL yield exactly (bytes allocated − bytes freed) over that prefix | [P] |
| R4 | WHEN a scope or variable annotated with a memlens teaching macro is created or dropped THE SYSTEM SHALL record a labeled event ordered consistently with the allocation events inside that scope (creation before them, drop after them) | [E] |
| R5 | WHEN a `Vec<T>` or `HashMap<K,V>` grows beyond capacity inside a traced region THE SYSTEM SHALL record the corresponding reallocation events from which the growth pattern is reconstructable | [E] |
| R6 | WHERE the `memlens` cargo feature is disabled THE SYSTEM SHALL leave the program unchanged by observable checks: no trace output is produced, the global allocator is not replaced, and no memlens symbols appear in the release binary | [E] |
| R7a | IF the trace sink cannot be written (e.g., disk full) THEN THE SYSTEM SHALL NOT panic or abort the traced program | [E] |
| R7b | IF trace events are dropped due to a sink failure THEN THE SYSTEM SHALL record a single loss marker once writing resumes or at trace end | [E] |
| R8 | THE SYSTEM SHALL write traces as envelope-compatible JSONL (family `memlens.*`, schema registered in `datalake/schema/`) so traces land in the goldeneye lake | [E] |
| R16 | WHEN instrumenting a playground exercise THE SYSTEM SHALL require only: adding the crate dependency, one global-allocator declaration, and macros at the sites the learner chooses to annotate — no other program changes | [E] |

### B. Replay engine + dashboard (the memory lens viewer)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R9 | WHEN the learner opens a trace file in the viewer THE SYSTEM SHALL render, from the same trace at once: a scrubbable heap timeline, a live-bytes graph, and a collections panel showing capacity growth | [O] |
| R10a | WHEN a trace is replayed to any sequence point t THE SYSTEM SHALL compute a live-allocation set identical to the result of applying events 1..t in order (replay determinism — the headless engine the viewer sits on) | [P] |
| R10b | WHEN the learner scrubs to any point in the timeline THE viewer SHALL display the R10a live set with its scope labels | [O] |
| R11 | THE viewer SHALL run fully client-side from a single self-contained file (no server, no AWS dependency) | [O] |
| R12 | WHEN a traced region contains a move or borrow annotated via macro THE viewer SHALL render an explicit zero-allocation marker at that point (the "zero-cost" callout) | [O] |
| R13 | WHEN the learner scrubs a trace of ≤ 100k events THE viewer SHALL update the display within 100 ms per scrub step | [O] |

### C. Honesty constraints

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R14a | THE SYSTEM SHALL have its tracing overhead measured and documented in this spec's `evidence.md` | [O] |
| R14b | IF the `memlens` feature is enabled in a release or benchmark build profile THEN the build SHALL fail with a compile error (guard in the crate itself) | [E] |
| R15 | THE viewer SHALL label what it cannot see (stack frames, non-annotated borrows) rather than implying the heap trace is the whole memory story | [O] |

## Learning requirements

Verified at spec close: the learner explains each item unaided; the explanation (or
its gaps) is recorded in `evidence.md`.

| ID | The learner SHALL be able to explain… |
|---|---|
| L1 | what the `GlobalAlloc` trait is, why implementing it requires `unsafe`, and what invariants the implementer promises |
| L2 | why moves and borrows produce no allocation events, and what that means for Rust vs GC languages |
| L3 | `Vec`'s growth strategy and `HashMap`'s rehashing as observed in real traces; `String` vs `&str` allocation behavior |
| L4 | why every allocation in a trace has a deterministic matching free (ownership + `Drop`), with a real trace as evidence |
| L5 | stack vs heap: which variables in an annotated exercise never appear in the heap trace, and why `Box<T>` moves a value to the heap |

## Out of scope

- IDE/editing features of any kind (cut in intent).
- Voice or chat interfaces (deferred; possible late-stage unit).
- Workflow/pipeline observability dashboards (explicitly rejected in intent).
- Async/`tokio` allocation attribution (later unit — sync programs first).
- `Rc`/`Arc` refcount visualization (deferred to a later unit; needs wrapper types,
  not just the allocator).
- Production-grade profiling accuracy (dhat/bytehound exist; this is a teaching lens).
- Hosting the viewer on AWS (later; v0 is a local file).

## Open questions (answered before approval)

- [x] IDE cut and voice-bot deferral confirmed; non-voice interactive helper also
      deferred (not in these requirements) — learner, 2026-07-05 ("All correct").
- [x] v0 audience = the learner running programs locally — confirmed 2026-07-05.
- [x] R13 bound (100k events / 100 ms) confirmed — 2026-07-05.

## Changelog

| Date | Change | Trigger (change protocol) | Re-gated? |
|---|---|---|---|
| 2026-07-05 | Rev 2: split R2/R7/R14, R10→R10a[P]+R10b[O], R6 rewritten as WHERE-pattern with observables, added R16 (ergonomics), R14b compile guard, HashMap in R5/L3, added L5 (Box/stack-vs-heap), Rc/Arc explicitly deferred, vague phrases concretized | spec-auditor findings 1–6 (78%) | pre-gate revision, no approval yet |
| 2026-07-05 | R5 clarified during construction: `Vec` grows via realloc lineage as specified, but `HashMap` (hashbrown) grows via alloc-new-table + dealloc-old — **no realloc events exist for it**. R5's "reallocation events" is read as "growth events (realloc lineage or alloc+dealloc pairs), reconstructable" — the [E] fixture tests exactly that, and the difference is itself a teaching point | growth_fixture construction finding (change protocol: minor clarification, meaning of "reconstructable" unchanged) | flagged for learner ack in the construction report |
| 2026-07-05 | **Unit reclassified as internal tooling** (intent Addendum, learner directive): L1–L5 are no longer unit-close gates — they defer to the future class exercises that will *use* the lens. Learner-session [O] verifications (R9, R10b, R12, R15 visual; old tasks 4.1.2/4.1.4) are replaced by automated browser verification (Playwright/Chromium screenshot + DOM checks). No requirement texts change; only their verification vehicle | learner clarification 2026-07-05 | learner directive is itself the approval |
