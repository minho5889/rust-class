# Requirements — 003 rust-bedrock

> **Doc 2 of 4. Derived from `intent.md` ONLY.** Audited by `spec-auditor`
> before review. ✋ **Human gate.**

**Status:** awaiting-review
**Approved:** — · **Assurance verdicts:** intent 90% (`_assurance/intent-review.md`)

## User stories

- As the learner, I want to build a real CLI over goldeneye's own data lake, so
  that my first Rust program is a tool I'll actually reuse — not a toy.
- As the learner, I want to write JSONL parsing and stats **by hand with std
  only**, so that ownership, borrowing, and `String`/`&str` are muscles I build
  rather than crates that hide them.
- As the learner, I want `glake` instrumented so I can watch its memory in the
  lens, so that Level-1b concepts are visible on my own code.

## Acceptance criteria (EARS + verification class)

### A. `glake` functionality (std-only)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R1 | WHEN `glake validate <path>` runs over a lake dir or file THE SYSTEM SHALL report each JSONL line that is not a well-formed envelope (missing any of `event_id`,`ts`,`session_id`,`event_type`,`schema_version`,`payload`), naming file and line number, and exit non-zero if any are found | [E] |
| R2 | WHEN `glake stats <path>` runs THE SYSTEM SHALL print counts of events grouped by `event_type` and by `dt=` day, plus the total | [E] |
| R3 | WHEN the input path is a directory THE SYSTEM SHALL recurse into `dt=*/` partitions and process every `*.jsonl` file; WHEN it is a single file, process just that file | [E] |
| R4 | THE SYSTEM SHALL use **only the Rust standard library** for its functionality — no external crates for parsing, CLI, or serialization (the in-repo `memlens` crate behind an off-by-default feature is the sole exception, see R7) | [P] |
| R5 | IF a line is empty or whitespace THEN THE SYSTEM SHALL skip it without counting it as malformed | [E] |
| R6 | IF the path does not exist or cannot be read THEN THE SYSTEM SHALL print a clear error to stderr and exit non-zero, without panicking | [E] |

### B. Correctness properties (hand-rolled parser)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R8 | THE minimal JSON-value extraction glake uses SHALL, for any input string, either return the requested top-level string/number field or a "not found / not that type" result — never panic and never read past the input | [P] |
| R9 | WHEN `stats` counts events THE SYSTEM SHALL report a per-`event_type` total whose sum equals the grand total of valid events (no double-counting, no drops) | [P] |

### C. Instrumentation & learning

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R7 | WHERE the `lens` cargo feature is enabled THE SYSTEM SHALL install `memlens` as its global allocator and emit a trace; WHERE disabled, glake SHALL contain no memlens code (default) — resolving the std-only/instrumentation tension: std-only is the functional rule, the lens is opt-in and in-repo | [E] |
| R10 | THE SYSTEM SHALL be usable as `glake stats datalake/raw-local` against the repo's real lake and produce correct counts cross-checked against `datalake/queries/scan.sh` | [O] |

## Learning requirements

Verified at close (this is internal tooling per project norms — "verified" =
demonstrated in code/tests + a short written note in `evidence.md`; no live
learner session required, per the 002 precedent):

| ID | The code SHALL demonstrate, and evidence.md SHALL note… |
|---|---|
| L1 | ownership & moves: where values move vs are borrowed in the parse→count pipeline |
| L2 | `&str` vs `String`: parsing borrows slices of the input line (`&str`) rather than allocating, and where an owned `String` is genuinely needed |
| L3 | `enum` + `match`: event kinds / parse outcomes modeled as enums, exhaustively matched |
| L4 | `Option`/`Result` + `?`: fallible steps returning `Result`, no `unwrap` in the tool's logic paths |
| L5 | iterators: counting/grouping expressed as iterator chains over borrowed data |

## Out of scope

- Traits, generics, custom error types / `thiserror` (spec 004).
- Async, tokio, any network or S3 (specs 005/007).
- `serde`, `clap`, or any external functional dependency (deliberate — R4).
- Writing to the lake, a query DSL, or DuckDB integration (004/007).
- Deployment of any kind (006+).

## Open questions (answered before approval)

- [x] memlens vs std-only — resolved by R7 (opt-in, in-repo, off by default).
- [x] filter in v0? — **deferred to 004**; v0 is validate + stats only (a
      `--type`/`--day` filter is a natural first 004 trait-based feature).

## Changelog

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-05 | Initial draft; intent advisories resolved (R7 lens tension; filter deferred) | intent audit 90% | first gate |
