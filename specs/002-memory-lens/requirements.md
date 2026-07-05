# Requirements — 002 memory-lens

> **Doc 2 of 4. Derived from `intent.md` ONLY.** Audited by `spec-auditor` before
> review. ✋ **Human gate.**

**Status:** drafting
**Approved:** — · **Assurance verdict:** intent 90% (see `_assurance/intent-review.md`)

## User stories

- As a learner, I want to run any of my Rust programs with instrumentation and then
  *watch* what it did in memory, so that ownership, allocation, and drop become
  visible phenomena instead of compiler abstractions.
- As a learner, I want collection types' behavior (growth, reallocation) shown
  explicitly, so that I understand what `Vec::push` or `String::from` really costs.
- As a learner, I want the *absence* of allocations to be visible too (moves,
  borrows), so that I see why Rust is efficient where GC languages are not.

## Acceptance criteria (EARS + verification class)

### A. Trace capture (the `goldeneye-memlens` tracking allocator + macros)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R1 | WHEN an instrumented program performs a heap allocation, reallocation, or deallocation THE SYSTEM SHALL record a trace event with operation kind, address, size, alignment, and monotonic sequence number | [P] |
| R2 | THE SYSTEM SHALL produce traces that balance: every deallocation event matches a prior allocation of the same address and size, and live-bytes computed from the trace never goes negative | [P] |
| R3 | WHEN tracing ends THE SYSTEM SHALL report final live bytes equal to (total allocated − total freed) computed from the event stream | [P] |
| R4 | WHEN a scope or variable annotated with a memlens teaching macro is created or dropped THE SYSTEM SHALL record a labeled scope/drop event at the correct sequence position | [E] |
| R5 | WHEN a `Vec<T>` grows beyond capacity inside a traced region THE trace SHALL contain the corresponding reallocation events showing the capacity growth pattern | [E] |
| R6 | WHILE the `memlens` feature flag is disabled THE SYSTEM SHALL add zero code to the binary (no trace output, no allocator wrapping) | [E] |
| R7 | IF the trace sink cannot be written (e.g., disk full) THEN THE SYSTEM SHALL not panic the traced program; it SHALL drop events and record a single loss marker | [E] |
| R8 | THE SYSTEM SHALL write traces as envelope-compatible JSONL (family `memlens.*`, schema registered in `datalake/schema/`) so traces land in the goldeneye lake | [E] |

### B. Dashboard (the memory lens viewer)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R9 | WHEN the learner opens a trace file in the viewer THE SYSTEM SHALL render, from the same trace at once: a scrubbable heap timeline, a live-bytes graph, and a collections panel showing capacity growth | [O] |
| R10 | WHEN the learner scrubs to any point in the timeline THE SYSTEM SHALL show the set of live allocations at that instant with their scope labels | [O] |
| R11 | THE viewer SHALL run fully client-side from a single self-contained file (no server, no AWS dependency) | [O] |
| R12 | WHEN a traced region contains a move or borrow annotated via macro THE viewer SHALL make the absence of allocation events at that point explicit (the "zero-cost" callout) | [O] |
| R13 | THE viewer SHALL remain responsive (scrub without visible lag) for traces from the class's playground exercises, sized ≤ 100k events | [O] |

### C. Honesty constraints

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R14 | THE SYSTEM SHALL document measured tracing overhead, and tracing SHALL never be enabled in benchmark or production builds | [O] |
| R15 | THE viewer SHALL label what it cannot see (stack frames, non-annotated borrows) rather than implying the heap trace is the whole memory story | [O] |

## Learning requirements

| ID | The learner SHALL be able to explain… |
|---|---|
| L1 | what the `GlobalAlloc` trait is, why implementing it requires `unsafe`, and what invariants the implementer promises |
| L2 | why moves and borrows produce no allocation events, and what that means for Rust vs GC languages |
| L3 | `Vec`'s growth strategy as observed in a real trace (amortized doubling), and `String` vs `&str` allocation behavior |
| L4 | why every allocation in a trace has a deterministic matching free (ownership + `Drop`), with a real trace as evidence |

## Out of scope

- IDE/editing features of any kind (cut in intent).
- Voice or chat interfaces (deferred; possible late-stage unit).
- Workflow/pipeline observability dashboards (explicitly rejected in intent).
- Async/`tokio` allocation attribution (later unit — sync programs first).
- Production-grade profiling accuracy (dhat/bytehound exist; this is a teaching lens).
- Hosting the viewer on AWS (later; v0 is a local file).

## Open questions (answered before approval)

- [ ] From intent audit: confirm the IDE cut and voice-bot deferral (proceeded on
      "Lets go" but never explicitly re-confirmed).
- [ ] From intent audit: confirm v0 audience = the learner running programs locally
      (not other users, not deployed AWS workloads).
- [ ] R13 threshold: is 100k events the right v0 bound, or should we target bigger
      traces from day one?

## Changelog

| Date | Change | Trigger (change protocol) | Re-gated? |
|---|---|---|---|
