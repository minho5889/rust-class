# Intent — 004 glake-traits

> **Doc 1 of 4. WRITE-ONCE.** Audited post-hoc by `intent-assurance`
> (`_assurance/intent-review.md`). No human gate.

**Created:** 2026-07-09 · **Spec status:** active

## Raw prompt (verbatim)

> "great can you make 2 more parts further?"

Said immediately after the Part-1/Part-2 course map (ramp + glake sittings) was
delivered under the materials-ahead directive ("my plan is for you to do as much
as I can. meaning you develop all the materials ahead. so I can just follow
along later").

## Distilled intent

Author **Part 3 of the course** ahead of time: the curriculum for spec 004 from
the approved Phase-1 plan — *"glake → lib+bin with query traits, thiserror
chain, proptest suite; traits, generics, modules, error design, API-guidelines
rubric."* glake v0 (std-only, hand-made) grows into glake v1: filters arrive,
errors become a designed type, and the crates we deliberately withheld in 003
(`clap`, `serde`, `thiserror`) finally enter — **as a measured comparison**, with
the lens showing exactly what the hand-written version saved and what the
abstractions cost.

## Learning goal

SKILLS **1c** (traits, generics, static vs dynamic dispatch, error patterns,
modules, closures/iterator adapters) + the constitution's API rubric
(C-COMMON-TRAITS, C-GOOD-ERR) applied to a real public API. Explicitly out:
async (005), any network/AWS.

## Assumptions made

- "2 more parts" = course Parts 3 and 4 = the next two Phase-1 specs (004, 005),
  authored materials-ahead like Parts 1–2 (worksheets + validated references).
- Fast-path pipeline per the standing pattern; gates presented for ack at the
  end rather than blocking authoring (the learner's directive is to build ahead).
- Part 3 builds on the *reference* v0 shape (lib+bin); a learner whose own
  crates/glake diverges reconciles in sitting G (the refactor sitting).
- 003's "What we're not building yet" list is Part 3's contract: filtering,
  traits/generics/custom errors, serde/clap.

## Rejected interpretations

- ❌ "2 more parts" = two more ramp steps — the ramp is complete at 8 and the
  message followed the two-Part course map.
- ❌ Skip to AWS/Lambda (006) — the approved plan sequences traits (004) and
  async (005) first; fundamentals-before-cloud is the learner's own stated
  preference.
- ❌ Replace the hand scanner with serde outright — the intent of 003 was
  earning intuition; 004's point is *comparing*, keeping both behind a trait.

## Addendum (append-only)

_(none yet)_
