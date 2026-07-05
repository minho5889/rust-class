# Insight 001 — Assurance verdicts predicted gate rework exactly

**Date:** 2026-07-05 · **Consumer:** MEMORY.md (pipeline tuning) ·
**Query:** `queries/scan.sh` (gate funnel + doc churn) + `specs/002-memory-lens/_assurance/*`

## Observation

Across spec 002's three gated docs, every doc needed **exactly one pre-gate
revision**, and the auditor's verdict ranked the severity correctly:

| Doc | Auditor verdict | Findings | Human outcome |
|---|---|---|---|
| requirements.md | 78% | 2 MAJOR-class (compound EARS, mis-tagged [P]) | approved on rev 2, zero human change requests |
| design.md | 74% (lowest) | 2 contract-level MAJORs (realloc payload, seq ordering) | approved on rev 2, zero human change requests |
| tasks.md | 86% (highest) | 3 MEDIUMs | rev 2, awaiting gate |

Doc-write churn from the lake corroborates: design.md 12 writes (most contested),
requirements.md 4, intent.md 1 (write-once held).

## So what

1. **The human gate has so far been a formality when the auditor scored ≥74 and
   its findings were addressed** — zero learner change-requests post-audit. The
   "assurance before attention" bet is paying: learner review time is spent
   approving, not debugging docs.
2. Tentative threshold calibration: verdicts in the 70s = real defects present;
   80s = polish. Too few data points to act on — recheck after 2–3 more specs.

## Caveats

n=3 docs, one spec, one day. The learner approved quickly partly due to trust;
a harder spec (first AWS deploy) is the real test.
