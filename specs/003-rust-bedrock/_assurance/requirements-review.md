# Requirements Assurance Review — 003 rust-bedrock

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-05
**Audited file:** `specs/003-rust-bedrock/requirements.md` (against `intent.md` only).
I never edit the spec docs — findings and fixes live here.

Sources consulted: `intent.md`, `_assurance/intent-review.md` (verdict 90%),
`CLAUDE.md` ([P]/[E]/[O] definitions, EARS discipline), `MEMORY.md` (Phase-1 plan
row 003; learner profile AWS-expert/Rust-beginner/build-first), `SKILLS.md`
(Level 1a/1b), and `datalake/schema/envelope.v1.json` (the envelope R1 validates).

---

## Intent-advisory resolution check (required)

The intent audit closed at 90% with two advisories. Both are resolved in the doc:

- **Advisory 1 (memlens vs std-only).** Resolved. R4 carves out `memlens` as the
  sole exception behind an off-by-default `lens` feature; R7 specifies both feature
  states; Open-questions line 1 marks it `[x]` resolved. Clean.
- **Advisory 2 (filter in v0?).** Resolved. Out-of-scope + Open-questions line 2
  defer `--type`/`--day` filter to 004. Consistent with the plan row. Clean.

Both divergent readings surface as answered open questions. Requirement satisfied.

---

## Findings (ordered by severity)

### MAJOR-1 — R1's "well-formed envelope" field list is wrong (omits `actor`) and duplicates the schema registry

R1 defines a well-formed envelope as having `event_id`, `ts`, `session_id`,
`event_type`, `schema_version`, `payload` — **six** fields. The single source of
truth, `datalake/schema/envelope.v1.json` (`required`, line 7), lists **seven**:
it also requires **`actor`**. As written, `glake validate` would pass an envelope
that is missing `actor`, i.e. it would fail to catch a genuinely malformed line —
the exact job R1 exists to do. (Note `spec_id` is correctly excluded: it is
nullable/optional in the schema.)

There are two defects here:
1. The list is factually incomplete (missing `actor`).
2. Hardcoding the field list into the requirement duplicates the schema and
   invites drift — a direct violation of CLAUDE.md's "one source of truth per
   fact." Intent says "validate against the envelope schema **shape**," which
   points at the registry, not a frozen copy of it.

**Fix:** Restate R1 to validate against the required-key set of
`datalake/schema/envelope.v1.json` (add `actor`; ideally reference the registry as
the authority rather than enumerating keys inline). Decide explicitly whether
"shape" also checks `schema_version == 1` and `ts` RFC3339 — if not, say so, so a
reviewer knows presence-only is intentional.

### MAJOR-2 — R4 is mis-tagged [P]; it has no input domain, invariant, or generation strategy

R4 ("THE SYSTEM SHALL use only the Rust standard library …") is a build/structural
constraint, not a property. Per CLAUDE.md a [P] requirement must yield a formal
property with *inputs, preconditions, invariant, and a generation strategy*, and a
`proptest` written before the code. R4 has none of these — there is nothing to
generate and no runtime input domain. Tagging it [P] would force an author to
invent a fake property (as happened in 002's audit history), and it fails the
design-gate's Properties-table requirement.

This is the direct answer to audit question (a): **R4 should not be [P].** It is a
verifiable operational fact — assert via `cargo tree` / a dependency-deny check in
CI that the crate's non-dev dependency set is empty (with `memlens` gated behind
the `lens` feature). That is an **[O]** (operational) check; an **[E]** single
deterministic assertion is also defensible. It is not [P].

**Fix:** Retag R4 as **[O]** (build/dependency assertion) and phrase the
verification (e.g., "the default-feature dependency tree contains only `std`; a CI
check fails on any added crate; `memlens` appears only under `--features lens`").

### MODERATE-3 — Compound EARS lines (two SHALLs / two conditions in one requirement)

CLAUDE.md and the EARS discipline require one testable behavior per line. Three
requirements bundle two:

- **R3** — `WHEN … a directory THE SYSTEM SHALL recurse …; WHEN … a single file,
  process just that file`. Two WHEN clauses, two behaviors. Split into **R3a**
  (directory recursion over `dt=*/` → `*.jsonl`) and **R3b** (single-file path).
- **R7** — `WHERE … enabled … SHALL install memlens and emit a trace; WHERE
  disabled … SHALL contain no memlens code`. Two WHERE branches plus a trailing
  prose justification ("resolving the std-only/instrumentation tension …") that is
  rationale, not a testable clause. Split into **R7a** (enabled: allocator + trace)
  and **R7b** (disabled: compiles out entirely) and move the justification to a
  note.
- **R1** — bundles "report each malformed line (file + line no.)" with "exit
  non-zero if any found." Borderline; the exit-code contract is separately
  testable and arguably deserves its own line. At minimum keep it in view.

**Fix:** Split R3 and R7 as above; consider lifting R1's exit-code clause out.

### MODERATE-4 — Coverage gap: the "watch my own program's memory in the lens" learning objective has no learning requirement

Intent's distilled intent and the third user story make the memlens instrumentation
a *learning* objective ("watch its memory in the lens, so that **Level-1b concepts
are visible on my own code**"). R7 only guarantees the feature exists and emits a
trace — it does not require the learner to *observe or interpret* glake's
allocation behavior. So the pedagogical payoff of the instrumentation is
unaddressed by any L-requirement, even though it is the whole reason R7 exists.

**Fix:** Add **L6** — e.g. "the code SHALL be run under `--features lens` and
`evidence.md` SHALL note what glake's allocation profile reveals about
`&str`-vs-`String` and `Vec` growth in the parse→count pipeline (ties L2/L5 to
observed memory)." This also strengthens the L2/L5 traceability to the through-line.

### MODERATE-5 — Learning-requirement → SKILLS mapping is incomplete for iterators (and stats needs `HashMap`)

Checking L1–L5 against SKILLS Level 1a/1b (audit question c): L1 (ownership/moves),
L2 (`&str`/`String`), L3 (enum+match), L4 (`Option`/`Result`+`?`) all map cleanly
to explicit 1a/1b items and are fully demonstrable by a std-only CLI **without any
traits/generics** — nothing here secretly belongs to 004. **L5 (iterators),
however, is not an enumerated Level 1a/1b skill** — SKILLS lists iterators only as a
Rustlings warm-up, not a tracked line item. Separately, `stats` grouping (R2) will
in practice need `HashMap` (`entry` API), which is std but also absent from the
1a/1b skill list (only `Vec` is listed).

Using iterator adapters and `HashMap` does **not** require defining traits/generics,
so there is no 003/004 boundary violation. The issue is bookkeeping: a learning
requirement (L5) and an implied dependency (`HashMap`) point at skills the
curriculum doesn't track at this level.

**Fix:** Either add "iterators" and "`HashMap` basics" as Level 1b line items in
SKILLS (out of scope for this doc — flag for the curriculum owner), or add a note in
the learning-requirements table that L5/stats exercise std collections beyond the
enumerated 1a/1b set. Do not silently rely on the mapping as-is.

### MINOR-6 — R9 conserves `event_type` totals but not the by-`dt=`-day totals

R2 groups by **both** `event_type` and `dt=` day; R9's conservation property
("sum of per-type totals == grand total of valid events") only covers the type
axis. The day grouping has the same double-count/drop risk and no property guarding
it.

**Fix:** Extend R9 (or add R9b) so the sum over `dt=` days also equals the grand
total of valid events — the same [P] invariant on the other grouping axis.

### MINOR-7 — [P] input domains should be pinned for the design gate; ID ordering is non-sequential

- **R8 is correctly [P]** (this is audit question a's second half): "for any input
  string … never panic and never read past the input" is a genuine total-function
  robustness property with a clear input domain (arbitrary `String`, biased toward
  JSON-like inputs). **R9 is also legitimately [P]** (a conservation property over
  arbitrary valid/invalid line collections). No change needed to their tags — but
  the design's Properties table must state concrete generation strategies (e.g.
  `any::<String>()` plus a hand-written JSON-fragment strategy for R8; a strategy
  emitting mixed valid/malformed envelope lines for R9). Flagging forward so the
  design gate can hold them to it.
- IDs are out of order: section B holds R8/R9, section C holds R7/R10, so R7 is
  referenced by R4 ("see R7") before it appears. Harmless but reads awkwardly.

**Fix:** Add the intended [P] generation strategies as a forward note (or leave for
design); optionally renumber so IDs are monotonic.

### NIT-8 — R7's "contains no memlens code" is partly an [O]/build guarantee, not [E]

The disabled-feature branch (glake compiles out all memlens code) is a
compile/structural guarantee closer to the R14b-style compile guard used in 002 —
an [O] build assertion — than an [E] example. Minor once R7 is split (MODERATE-3):
tag the R7b compile-out clause [O], keep the R7a allocator+trace clause [E].

---

## Affirmations (things done well)

- Traceability is clean: **no orphan requirements** — every R and L traces to a
  distilled-intent element (validate, stats, std-only, memlens, fundamentals,
  real-lake usefulness).
- Out-of-scope section correctly walls off traits/generics/`thiserror`, async,
  `serde`/`clap`, and lake-writing — faithful to the fundamentals-first,
  std-only intent and the 004 boundary. **Nothing in scope contradicts the
  std-only / fundamentals-first constraint** (audit question d): the only std-only
  tension, memlens, is explicitly gated off by default (R4 exception + R7).
- R5, R6 (IF-THEN), R2 (WHEN), R10 ([O], real-lake cross-check against
  `scan.sh`) are well-formed and correctly tagged.
- Learning requirements sensibly reuse the 002 "internal tooling, verify in
  evidence.md, no live learner gate" precedent.

---

VERDICT: 72% — Structurally sound and faithful to a std-only fundamentals intent with both intent advisories resolved, but not review-ready until R1's envelope field list adds `actor` (it currently mis-defines "well-formed") and R4 is retagged off [P]; compound EARS (R3/R7) and a missing memory-observation learning requirement should follow.
