# Intent — 003 rust-bedrock

> **Doc 1 of 4. WRITE-ONCE.** Clarifications go in the Addendum; a change of
> meaning supersedes the spec. Audited post-hoc by `intent-assurance`
> (`_assurance/intent-review.md`), which never edits this file. No human gate.

**Created:** 2026-07-05 · **Spec status:** active

## Raw prompt (verbatim)

Learner directive (2026-07-05), with LinkedIn profile attached:

> "Rather than that spec I want you to plan out 6 stacks as my phase 1 of
> learning rust using the environment I mentioned. How would you list out the 6
> different specs in order? Here is my profile. Ask me anything… Sorry 6 specs.
> You better research best rust study materials as well."

Then, on the proposed plan: **"Great kick off spec 3."**

Profile facts that shaped scope (from LinkedIn, logged in MEMORY.md): AWS
Professional Services AI/ML consultant; 3+ yrs prior as a Cloud Support Engineer
troubleshooting **Serverless** applications on AWS; new to Rust; bilingual
EN/KR. Answered planning questions: **10+ h/week**, personal AWS account ready,
**build-first + hands-on exercises**, Phase-2 agentic tilt.

## Distilled intent

Spec 003 is the **first spec of Phase 1** and the entry point into Rust itself.
Build **`glake` v0** — a small, dependency-free (std-only) command-line tool that
reads the goldeneye data lake's local JSONL and reports on it (validate against
the envelope schema shape, count events by type/day, surface basic stats). The
tool must be genuinely useful to the project (goldeneye already has a lake worth
inspecting), but its real job is to be the vehicle for **Level-1a + core-1b Rust
fundamentals** — ownership, borrowing, `String`/`&str`, structs, enums, `match`,
`Option`/`Result`, iterators — learned by building, not by reading. memlens
instruments it so the learner watches his own first Rust program's memory.

## Learning goal

SKILLS Level **1a** (all) + Level **1b** partial: ownership/moves, borrowing
(`&T`/`&mut T`), stack vs heap, `String` vs `&str`, slices, `Vec`. Explicitly
**out**: traits/generics (004), async (005), error *libraries* (004 —
`std::result` + manual matching only here). Warm-ups bound: Rustlings
(variables→enums→error_handling→iterators), 100-Exercises ch1–4 (see
`research/rust-study-materials.md`).

## Assumptions made

- "6 stacks" = 6 specs (learner self-corrected: "Sorry 6 specs").
- "that spec" the learner set aside = the old datalake-v1-as-003 idea; glake is
  its std-only, fundamentals-first reframing (the lake work proper is spec 007).
- std-only is a deliberate constraint (no serde) so the learner writes parsing
  and ownership by hand before reaching for crates — the pedagogy is the point.
- CLI scope kept small enough for one spec at 10+ h/week: validate + stats +
  filter, reading the existing `datalake/raw-local/**/*.jsonl`. No writing to
  the lake, no S3 (007), no query DSL (that's glake's 004 evolution).
- Build-first: external materials are optional warm-ups, never gates.

## Rejected interpretations

- ❌ Start with a toy (guessing game / fibonacci) — rejected: the learner is a
  senior engineer who wants tools he'll use; a real CLI over real data motivates.
- ❌ Use `serde`/`clap` from the start — rejected: hides ownership and parsing,
  the exact muscles 003 must build. They arrive deliberately later.
- ❌ Make 003 a Lambda already — rejected: async + deploy before fundamentals
  would overwhelm; every study source sequences fundamentals first.
- ❌ Skip fundamentals because he's an AWS expert — rejected: AWS-expert ≠
  Rust-expert; the borrow checker is new regardless of cloud seniority.

## Addendum (append-only)

_(none yet)_
