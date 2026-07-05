# research/ — verified deep-research reports

Reports from the deep-research workflow. Naming: `<topic>.md` (no dates — the
research date lives in each report's header; re-running a topic updates the
file, git history keeps versions).

| Report | Status |
|---|---|
| `rust-best-practices-and-big-tech.md` | ✅ 2026-07-05 (24/25 confirmed, 1 refuted) |
| `rust-on-aws-compute.md` | ✅ 2026-07-05 (24/25 confirmed, 1 refuted) |
| `typescript-cdk-for-goldeneye.md` | ✅ 2026-07-05 (25/25 confirmed) |

These feed: CLAUDE.md conventions, `SKILLS.md` curriculum, CI gates, and the
design of AWS-deploying specs. Findings are ranked by confidence; every claim
carries its sources.

## goldeneye research protocol v2 (adopted 2026-07-05)

Lessons from the first three runs (evidence: `datalake/insights-local/002-*`).
Every future deep-research run follows these rules — bake them into the
workflow prompt/args:

1. **No silent drops.** Claims extracted but not verified (budget cap) are
   listed in a report appendix as *"extracted, unverified — do not cite"*, and
   verification budget is spent by relevance-to-question ranking, not
   extraction order. (First runs silently dropped 62–77% of extractions.)
2. **Mid-run completeness critic.** Between verify and synthesize: compare
   surviving claims against the question's sub-parts; re-run targeted searches
   for empty buckets *before* writing the report. (MicroVMs came back empty and
   we learned it post-hoc.)
3. **Lens-diverse verification.** Verifier panel uses distinct lenses —
   source-authority (primary vs blog), reproducibility (is there a repo?),
   recency (still true?) — instead of three identical refuters.
4. **Volatility tags, mandatory.** Every finding: `stable` (definitional/
   official — no review) / `annual` (benchmarks, prices — review yearly) /
   `volatile` (pre-1.0 tools, open PRs, unverified service facts — `review_by`
   +3 months). Each report carries a *Volatility & review* section.
5. **Findings land in the lake.** Every run emits `research.run`, `.finding`
   (with volatility), `.open_question`, `.refuted` events per
   `datalake/schema/research.v1.json` — a finding invisible to the lake doesn't
   exist. Adoptions emit `research.adoption` linking claim → file → commit.
6. **Adoption delta stage.** The run's final agent reads CLAUDE.md + the
   relevant spec and emits a structured proposed-changes list, so
   finding → repo change is reviewable, not narrative.
7. **Re-verify routine.** `datalake/queries/scan.sh` surfaces findings past
   `review_by`; expired volatile claims get single-fact re-checks (cheap), not
   whole-topic re-runs.
