# Tasks — 004 glake-traits (glake v1)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Three new ramp steps (9–11: traits, generics/dyn, closures+adapters), then
**four sittings** (G→J) evolving your own `crates/glake`. Same rules as Part 2:
you write, worksheets steer, properties come red-first, one commit per move.
All materials pre-authored and validated against the 004 reference.

**Done means:** filters work, errors are designed, both parsers agree under a
512-case property, and you've *measured* hand vs serde in the lens.

---

## 0. Ramp extension (`playground/ramp/`, no spec)

- [ ] 0.2 Steps 9–11, one concept each: **9 traits** (define + implement twice),
      **10 generics vs `dyn`** (same function both ways; when each), **11
      closures & iterator adapters** (map/filter/collect over real lake lines).
      Worksheets + validated solutions, `my-solution.rs` + one commit per step.

## 1. glake v1, four sittings (learner writes, Claude coaches)

### Sitting G — the refactor *(T4; F12 groundwork)*
- [ ] 1.1 Split your glake into lib + thin bin (reference shape); modules
      `scan/classify/walk/tally`; everything still green afterwards — the
      refactor-under-tests lesson. *(commits: split, then green)*

### Sitting H — errors by design *(F5, F6)*
- [ ] 1.2 `thiserror` dep; `GlakeError` with io source chain; lib paths return
      `Result<_, GlakeError>`; bin maps to one stderr line, same exit codes;
      C-GOOD-ERR checklist pass recorded. *(commits: enum, then threaded)*

### Sitting I — clap + filters, test-first *(F1–F4, F10; T5)*
- [ ] 1.3 **F3 partition property first** (red) — reusing your R9 generator.
- [ ] 1.4 `clap` derive CLI; `Filter` struct + closure predicate; `--type`,
      `--since` (+ bad-ts exclusion note); `(filtered from M)` output; property
      + fixtures green. *(commits: red, clap, green)*

### Sitting J — the trait, the rival, the measurement *(F7–F9; T1, T2, T6)*
- [ ] 1.5 **F8 equivalence property first** (red): both-parsers-agree, against
      stubs.
- [ ] 1.6 `EventParser` trait; `HandParser` wrapping your scanner (owned at the
      boundary); `SerdeParser`; `--parser` via `Box<dyn …>`; property green —
      expect and triage the duplicate-key counterexample (planned lesson).
      *(commits: red, hand, serde+green)*
- [ ] 1.7 **The measurement (L→evidence):** stats on the real lake under
      `--features lens`, once per parser; open both traces in the viewer;
      allocation counts + deltas → `evidence.md`. You drive.

## 2. Close-out (Claude, machine work)

- [ ] 2.1 evidence.md (T1–T6 notes, F9 numbers, property outcomes, triage log);
      property-auditor run; SKILLS 1c updates; MEMORY; main FF +
      `spec-close/004-glake-traits` marker.

---

## Operations checklist

- [ ] fmt + clippy clean (both feature configs); F3/F8 ≥256 cases, red-first,
      seeds committed on failure; fixtures green; F6/F12 rubric + F9 numbers in
      evidence; property-auditor pass.

*Deviation declared: no AWS deploy/teardown — local-only unit.*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-3 directive | pending combined ack |

</details>
