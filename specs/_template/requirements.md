# Requirements — <NNN short-name>

**Status:** drafting
**Approved:** — · **Assurance:** —

> **Readability rule (see CLAUDE.md):** lead with plain English anyone can read in
> a minute and approve from. Keep the precise, tagged, testable criteria in a
> clearly-marked section *below* — reviewers skim it, the design step and
> auditors use it. Push audit trail/changelog into the collapsed block at the end.

---

## In plain words

_2–5 sentences, no jargon: what we're building, what it does for the user, and
the one design choice that matters most. A non-Rust reader should get it._

## What it does

- _Plain-English bullets of user-facing behavior. Normal sentences, not
  WHEN/SHALL. This is the part that gets read._

## What we're *not* building yet (on purpose)

- _Scope walls, plain language, each with the spec that will pick it up._

## What you'll learn building it

- _Plain list of the concepts this unit exercises (maps to SKILLS). This is a
  class — say what the learner gets, in normal words._

---

## Precise acceptance criteria

> *Skim unless you're writing the design or the tests.* Each item is one testable
> fact. Tags: **[P]** property (holds for all inputs) · **[E]** example (specific
> cases) · **[O]** operational (checked by running it / the build). An item that
> fits no tag is untestable — rewrite it before review.
>
> _Define any shared term (e.g. "well-formed X") once, here, pointing at the
> single source of truth — never a hardcoded copy._

**<group name, plain>**
- **[P|E|O] R1** — <one behavior, as plainly as precision allows>
- **[E] R2** — …

**Learning (verified in `evidence.md`)**
- **L1** — <concept the code must demonstrate>

---

<details><summary>Audit trail & changelog</summary>

_Intent advisories resolved; requirements-audit findings and each revision.
Keep it here, out of the reader's way._

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|

</details>
