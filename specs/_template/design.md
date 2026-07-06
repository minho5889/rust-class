# Design — <NNN short-name>

> **Doc 3 of 4. Derived from approved `requirements.md`.** Every design element must
> cite the REQ IDs it satisfies — an element citing none is scope creep made visible.
> Audited by `spec-auditor` (traceability + completeness) before review.
> ✋ **Human gate.**

**Status:** drafting
**Approved:** — · **Assurance verdict:** —

> **Readability rule (CLAUDE.md):** open with the plain-English shape a reviewer
> can veto from; keep detailed interfaces/properties below; audit trail collapsed
> at the end.

## In plain words

_2–4 sentences: how the solution works, at a level someone could sanity-check
without reading the detail. Name the pieces and how they connect._

## High-level design (the shape)

_A narrative the learner can read alone and veto the approach: what runs where,
what talks to what, why this shape. Diagram if useful._

## Detailed design

_Components, interfaces, data flow, failure modes. Each subsection header cites its
requirements, e.g. "### Handler (R1, R2)"._

## Key decisions

| Decision | Options considered | Chosen | Why | REQs |
|---|---|---|---|---|

## Properties (one per [P] requirement)

| REQ | Property (∀ inputs, precondition ⇒ invariant) | Generation strategy |
|---|---|---|
| R1 | For any <input domain> where <precondition>, <invariant> | <proptest strategy: ranges, shapes, edge weights> |

## Rust concepts in play (teaching hook)

_Which ownership/borrowing/allocation lessons this design will surface, and where._

- …

## Error handling

_What can fail and how each failure surfaces (Result types, retries, logs)._

## Verification strategy

_How [P] properties, [E] examples, and [O] operational checks will each be executed;
what "done" looks like in `evidence.md`._

## Changelog

| Date | Change | Trigger (change protocol) | Re-gated? |
|---|---|---|---|
