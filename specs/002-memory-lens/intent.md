# Intent — 002 memory-lens

> **Doc 1 of 4. WRITE-ONCE.** Clarifications go in the Addendum; a change of meaning
> supersedes the spec. Audited post-hoc by `intent-assurance` (see
> `_assurance/intent-review.md`), which never edits this file. No human gate.

**Created:** 2026-07-05 · **Spec status:** active

## Raw prompt (verbatim)

Initial ask (2026-07-05):

> What I want to have is nice front end with great UI UX design with smooth
> interface with bots next and visualization of memory allocations, collections,
> event or session or contract information all at once like a dashboard. So you see
> all the behind the scene as you have small IDE as well as a bot who you can
> interact with voice chat. Before you take any action what do you think jonestly
> about my idea

Correction after Claude's assessment (same day):

> Nono dashboard is soley for to understand whats going on when they run the
> program in rust memory!!!

Go-ahead:

> Lets go

## Distilled intent

Build a **memory lens** for the class: an instrument that records what a running
Rust program does in memory (allocations, deallocations, collection growth, drops)
and a **dashboard with polished, smooth UI/UX** that visualizes those recordings so
the learner can *see* Rust's memory model in action — the "behind the scenes" of
every exercise. The dashboard's purpose is solely understanding Rust runtime memory
behavior, per the learner's correction.

## Learning goal

- SKILLS 1b (entire section — this is the through-line made visible): ownership,
  borrowing, stack vs heap, `Box`, collections growth, drop order.
- The instrument itself teaches: `GlobalAlloc` trait, `unsafe` boundaries,
  zero-cost-abstraction measurement (SKILLS 1b/3).

## Assumptions made

- "collections" = Rust collection types (`Vec`, `String`, `HashMap`) and their
  allocation/growth behavior — not data-lake collections.
- "event or session or contract information" from the initial ask is superseded by
  the correction ("solely… rust memory"): the dashboard scope is memory traces;
  workflow/pipeline observability is NOT part of this unit.
- The "small IDE" is cut and "voice chat bot" is deferred — Claude recommended this
  scope reduction and the learner proceeded with "Lets go" without objection.
  Flagged here because it was never explicitly re-confirmed.
- Instrumentation is acceptable via a custom tracking allocator + teaching macros
  behind a feature flag (overhead acceptable in learning runs, never in benchmarks).
- v0 delivery may be a self-contained local HTML viewer (no AWS dependency);
  polish ("great UI UX") applies to the visualization experience, not to hosting.
- Traces flow into the goldeneye data lake conventions (envelope-compatible), so
  later units can mine them.

## Rejected interpretations

- ❌ Workflow-observability dashboard (pipeline states, gates, sessions) — Claude's
  own initial misread; explicitly corrected by the learner ("Nono… soley…").
- ❌ Full product suite (IDE + voice bot + dashboard) — IDE cut (no Rust learning
  value, large TS effort), voice deferred to a possible late-stage unit.
- ❌ Production memory profiler (dhat/bytehound replacement) — this is a teaching
  instrument; prior art informs it, correctness > completeness > performance.

## Addendum (append-only)

- **2026-07-05 (learner):** "there is no need of learner's involvement because
  we are building an internal tool that learner can use later but this itself
  is not a learning material." → memory-lens is **internal infrastructure**;
  its pedagogical value is realized later, when class exercises use it. The
  unit closes without a learner session; learning requirements L1–L5 defer to
  future exercises (see requirements changelog, same date).
