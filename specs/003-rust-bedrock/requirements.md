# Requirements — 003 rust-bedrock

> **Doc 2 of 4. Derived from `intent.md` ONLY.** Audited by `spec-auditor`
> before review. ✋ **Human gate.**

**Status:** awaiting-review
**Approved:** — · **Assurance verdicts:** intent 90% · requirements 72% → revised
rev 2 per `_assurance/requirements-review.md` (all MAJOR/MODERATE addressed)

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

> "Well-formed envelope" = a JSON object carrying every key in the **`required`
> set of `datalake/schema/envelope.v1.json`** (the single source of truth:
> `event_id`, `ts`, `session_id`, `actor`, `event_type`, `schema_version`,
> `payload`; `spec_id` is nullable/optional). v0 checks **key presence only** —
> it does *not* validate `ts` as RFC3339 or `schema_version == 1` (deliberate;
> deeper validation is a candidate 004 feature). glake reads the required-key
> list from the schema file, not a frozen inline copy.

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R1a | WHEN `glake validate <path>` finds a JSONL line that is not a well-formed envelope (any required key absent, per the schema registry) THE SYSTEM SHALL report it naming the file and 1-based line number | [E] |
| R1b | WHEN `glake validate <path>` completes THE SYSTEM SHALL exit non-zero if and only if at least one malformed line was found | [E] |
| R2 | WHEN `glake stats <path>` runs THE SYSTEM SHALL print counts of events grouped by `event_type` and, separately, by `dt=` day, plus the grand total | [E] |
| R3a | WHEN the input path is a directory THE SYSTEM SHALL recurse into `dt=*/` partitions and process every `*.jsonl` file within | [E] |
| R3b | WHEN the input path is a single file THE SYSTEM SHALL process just that file | [E] |
| R5 | IF a line is empty or whitespace-only THEN THE SYSTEM SHALL skip it — neither counted as an event nor reported as malformed | [E] |
| R6 | IF the path does not exist or cannot be read THEN THE SYSTEM SHALL print a clear error to stderr and exit non-zero, without panicking | [E] |

### B. Correctness properties (hand-rolled parser)

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R8 | THE minimal JSON-value extraction glake uses SHALL, for any input string, either return the requested top-level string field or a "not found / not that type" result — never panicking and never reading past the input | [P] |
| R9 | WHEN `stats` counts events THE per-`event_type` totals AND the per-`dt=`-day totals SHALL each sum to the grand total of valid events (no double-count, no drop, on either grouping axis) | [P] |

### C. Instrumentation & operational

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R4 | THE default-feature dependency tree SHALL contain no external crates — only `std`; a check (e.g. `cargo tree`) SHALL fail if any is added; `memlens` SHALL appear only under `--features lens` | [O] |
| R7a | WHERE the `lens` cargo feature is enabled THE SYSTEM SHALL install `memlens` as its global allocator and emit a trace | [E] |
| R7b | WHERE the `lens` feature is disabled (default) THE SYSTEM SHALL compile out all memlens code — no memlens symbols in the binary | [O] |
| R10 | THE SYSTEM SHALL run as `glake stats datalake/raw-local` against the repo's real lake and produce counts cross-checked equal to `datalake/queries/scan.sh` | [O] |

> Note (std-only vs the lens): std-only is the *functional* rule (R4). The lens
> is the sole, in-repo, off-by-default exception (R7a/b) — it observes, never
> changes glake's behavior.

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
| L5 | iterators + `HashMap`: counting/grouping expressed as iterator chains over borrowed data, aggregated via `HashMap::entry` |
| L6 | **memory observed**: glake run under `--features lens`; evidence.md notes what the trace reveals about `&str`-vs-`String` and `Vec`/`HashMap` growth in parse→count (ties L2/L5 to real allocation behavior — the reason R7 exists) |

> SKILLS note (per requirements audit MODERATE-5): L5 exercises **iterators** and
> **`HashMap`** — both std, both usable without authoring traits/generics, so no
> 003/004 boundary crossing — but neither is currently an enumerated Level-1b
> line item (only `Vec` is). Curriculum bookkeeping: add them to SKILLS 1b at
> close, or track here. Not a scope defect, just an unlisted std skill.

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
| 2026-07-05 | Rev 2: R1 now validates against the schema registry's required set incl. **`actor`** (was missing — would have passed malformed lines) and references the registry not an inline copy; R4 retagged [P]→[O] (build constraint, not a property); R3→R3a/R3b and R7→R7a/R7b (compound EARS split, R7b→[O]); R9 extended to conserve the by-day axis too; added L6 (observe memory in the lens) + SKILLS note on L5/HashMap | spec-auditor 72% (MAJOR-1/2, MODERATE-3/4/5, MINOR-6) | pre-gate revision |
