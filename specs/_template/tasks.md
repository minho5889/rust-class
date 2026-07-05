# Tasks — <NNN short-name>

> **Doc 4 of 4. Derived from approved `design.md`.** ✋ **Human gate on the plan's
> shape, once** — bolts then execute without re-asking. Three layers defined by
> *grain*, not ceremony — **collapse layers for small units** (a small unit may go
> straight from one main task to action items).

**Status:** drafting
**Approved:** — · **Assurance verdict:** —

| Layer | Grain | Done means |
|---|---|---|
| Main task `N.` | A deliverable (maps to a design component) | Its part of the system demonstrably works |
| Sub task `N.M` | One bolt — a single working session | Compiles, tests pass, committable checkpoint |
| Action item `N.M.K` | One atomic action — one commit's worth | Verifiable in isolation |

Rules: a parent checks off only when all children are checked. [P] property tests are
written **before** the code they test (first action items of their sub task). Note
deviations inline; contradictions with approved docs trigger the change protocol.

## 1. <Main task — deliverable> (R1, R2)

### 1.1 <Sub task — one bolt>
- [ ] 1.1.1 Write property test for R1 (per design Properties table)
- [ ] 1.1.2 …

## Operations checklist

- [ ] `cargo fmt` + `cargo clippy -- -D warnings` clean
- [ ] All [P] properties pass (≥256 cases); `proptest-regressions/` committed
- [ ] All [E] tests pass
- [ ] All [O] requirements observed on the real AWS target → numbers in `evidence.md`
- [ ] Torn down (if the resource costs money at idle)
- [ ] `evidence.md` learnings written; `MEMORY.md` + `SKILLS.md` updated
- [ ] `property-auditor` run: every [P] REQ has a matching passing property
