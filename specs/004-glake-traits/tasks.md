# Tasks — 004 glake-traits (glake v1)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Three new ramp steps (9–11: traits, generics/dyn, closures+adapters), then
**four sittings** (G→J) evolving your own `crates/glake`. Same rules as Part 2:
you write, worksheets steer, properties come red-first, one commit per move.
Materials (step 9–11 worksheets + sitting guides G–J) are authored and
validated against the 004 reference **before this gate is presented** —
if you're reading this at the gate, they exist.

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
- [ ] 1.1 Reconcile your glake to the reference shape — lib + thin bin,
      modules `scan/classify/walk/tally` (if you followed sittings A–F you
      already have this: G audits it, generalizes `tally`, adds the F12
      derives, bumps to 0.2.0); everything still green afterwards — the
      refactor-under-tests lesson. *(commits: refactor, then green)*

### Sitting H — errors by design *(F5, F6)*
- [ ] 1.2 `thiserror` dep; `GlakeError` with io source chain; lib paths return
      `Result<_, GlakeError>`; bin maps to one stderr line, same exit codes;
      C-GOOD-ERR checklist pass recorded. *(commits: enum, then threaded)*

### Sitting I — clap + filters, test-first *(F1–F4, F10; T5)*
- [ ] 1.3 **F3 partition property first** (red) — reusing your R9 generator.
- [ ] 1.4 `clap` derive CLI (filter flags rejected on `validate`, usage test);
      `Filter` struct + `verdict` method; `--type`, `--since` (+ bad-ts
      excluded count, F2b); `(filtered from M)` output; property + fixtures
      green. *(commits: red, clap, green)*

### Sitting J — the trait, the rival, the measurement *(F7–F9, F13; T1, T2, T6)*
- [ ] 1.5 **Equivalence property first** (red): the *naive* both-parsers-agree
      property over the full mixed strategy, against stub parsers (`todo!()`)
      — deliberately naive; it meets its counterexample in 1.7 and the triage
      reshapes it into F8a/F8b (the planned spec-bug lesson).
- [ ] 1.6 `EventParser` trait + `ClassifiedLine`; `HandParser` wrapping your
      scanner (owned only at the boundary); the generic pipeline
      (`fn run<P: EventParser>`) + monomorphized unit tests (F13); F8a green
      for hand-vs-hand. *(commits: red, trait+hand)*
- [ ] 1.7 `SerdeParser`; `--parser` via `Box<dyn …>` in the bin only; the
      naive property meets its real, shrunk counterexample → **spec-bug
      triage** → reshape into F8a (green) + co-written **F8b** (divergence
      containment), walking its three classes — leniency, escapes, duplicate
      keys — via shrunk inputs and pinned property arms. Any *unclassifiable*
      divergence → normal triage. *(commits: serde+F8a green, F8b)*
- [ ] 1.8 Hygiene sweep (Claude drives): F11 `cargo tree` check, F12
      C-COMMON-TRAITS rubric review, fmt/clippy both configs — outputs
      recorded for evidence.
- [ ] 1.9 **The measurement (F9 → evidence):** stats on the real lake under
      `--features lens`, once per parser; open both traces in the viewer;
      allocation counts + deltas → `evidence.md`. You drive.

## 2. Close-out (Claude, machine work)

- [ ] 2.1 evidence.md (T1–T6 notes, F9 numbers, property outcomes, triage log
      if any); property-auditor run; SKILLS 1c updates; MEMORY; main FF +
      `spec-close/004-glake-traits` marker.

---

## Operations checklist

- [ ] fmt + clippy clean (both feature configs); F3/F8a/F8b ≥256 cases,
      red-first, seeds committed on genuine failure; F13 monomorphized tests +
      fixtures green (incl. `validate --type` → exit 2); F6/F11/F12 records +
      F9 numbers in evidence; property-auditor pass.

*Deviation declared: no AWS deploy/teardown — local-only unit.*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-3 directive | pending combined ack |
| 2026-07-09 | Rev 2.1 (materials reconciliation): 1.1 reworded — A–F leavers already own the split, G audits/generalizes; 1.5/1.7 reworded to the naive-property→counterexample→F8a/F8b dramaturgy the guides and NOTES actually teach; "debugger" → shrunk inputs + pinned arms | sitting-guide authoring | this combined gate |
| 2026-07-09 | Rev 2 per design+tasks audit (62%): materials claim made truthful-at-gate (MAJOR-3); sitting J restructured for F8a/F8b + F13 (trait→hand→generic tests→serde→divergence walk); hygiene task 1.8 owns F11/F12; sitting I uses `verdict` + validate-rejects-filters test | 004 audits | this combined gate |

</details>
