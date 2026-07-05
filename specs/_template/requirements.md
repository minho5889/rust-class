# Requirements — <NNN short-name>

> **Doc 2 of 4. Derived from `intent.md` ONLY** — if it can't be written from
> intent.md alone, intent.md was incomplete. Audited by `spec-auditor` before
> review (verdict in `_assurance/`). ✋ **Human gate:** loop
> drafting → awaiting-review → revising until approved.

**Status:** drafting
**Approved:** — · **Assurance verdict:** —

## User stories

- As a learner, I want <capability>, so that <learning outcome>.

## Acceptance criteria (EARS + verification class)

Every requirement carries an ID and a verification tag. **A requirement that fits no
tag is untestable and must be rewritten before review.**

- **[P]** property-testable → becomes a `proptest` property ("for any valid input…")
- **[E]** example-testable → unit/integration test with chosen examples
- **[O]** operationally-verified → observed on the real AWS target, recorded in `evidence.md`

| ID | Requirement (EARS) | Tag |
|---|---|---|
| R1 | WHEN <trigger> THE SYSTEM SHALL <behavior> | [P] |
| R2 | IF <error condition> THEN THE SYSTEM SHALL <behavior> | [E] |
| R3 | THE SYSTEM SHALL <constraint, e.g. run on arm64 / cold-start ≤ X ms> | [O] |

## Learning requirements

| ID | The learner SHALL be able to explain… |
|---|---|
| L1 | <concept>, after completing this unit |

## Out of scope

- …

## Open questions (answered before approval)

- [ ] …

## Changelog

| Date | Change | Trigger (change protocol) | Re-gated? |
|---|---|---|---|
