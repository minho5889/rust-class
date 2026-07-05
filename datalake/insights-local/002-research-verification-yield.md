# Insight 002 — Research verification: high survival, and the kills mattered most

**Date:** 2026-07-05 · **Consumer:** research/README.md (methodology) ·
**Query:** `queries/scan.sh` (research.* events)

## Observation

Three deep-research runs, 318 agents total:

| Run | Extracted | Verified | Confirmed | Killed | Verified/Extracted |
|---|---|---|---|---|---|
| typescript-cdk | 65 | 25 | 25 | 0 | 38% |
| best-practices | 110 | 25 | 24 | 1 | 23% |
| rust-on-aws | 48 | 25 | 24 | 1 | 52% |

1. **Verification bottleneck is real**: 62–77% of extracted claims were never
   verified (budget cap), silently. This is the single biggest methodology gap —
   now addressed by the unverified-backlog rule in research/README.md.
2. **97% of verified claims survived** (73/75) — but the 2 kills were
   disproportionately valuable: one prevented a false constraint on our CI
   design (ARM64 "can't fuzz" — it can), one corrected AWS marketing causality
   (Firecracker 125 ms "because Rust"). Adversarial verification earns its cost
   on the kills, not the confirms.
3. **Absence-of-evidence was a finding**: MicroVMs produced zero surviving
   claims across two runs (12 open questions logged in the lake). The
   completeness-critic upgrade exists because this was discovered post-hoc.

## So what

Keep 3-vote adversarial verification (the kills justify it); fix the silent-drop
problem (backlog rule) and the late gap discovery (mid-run critic) — both now in
the research protocol.

## Caveats

n=3 runs, same harness/day. Survival rate may reflect conservative extraction
rather than verification quality.
