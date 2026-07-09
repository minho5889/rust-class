# Intent Assurance Review — 005 async-relay

**Auditor:** intent-assurance (fresh context) · **Date:** 2026-07-09
**Input:** raw prompt (verbatim) + delivery context, derived blind before reading `intent.md`.

## 1. Blind derivation (recorded before reading intent.md)

**Raw prompt:** "great can you make 2 more parts further?"

**(a) My distilled intent.** The learner approves the just-delivered Part-1/Part-2
course map for spec 003 ("great") and asks Claude to extend the course by
authoring the **next two parts** — Parts 3 and 4 — in the same materials-ahead
style (worksheets, hint ladders, checkpoints, pre-validated solutions) so the
learner can follow along self-paced. Per the approved Phase-1 plan, Part 3 =
spec 004 (glake-traits) and Part 4 = spec 005 (async-relay: local axum service
receiving envelope events; async/await, tokio, channels, Send/Sync, graceful
shutdown). This spec's share of the request: author Part 4 (spec 005) in full,
ahead of time.

**(b) Alternative readings considered blind:**

1. **"2 more parts" = two more units *within* spec 003** (e.g., ramp steps 9–10
   or sittings G–H), not two new curriculum parts. Requires reading "parts" as
   fine-grained units — weak, since the delivery was explicitly labeled
   Part-1/Part-2, so "2 more parts" naturally continues that numbering.
2. **Map/outline only** — produce a course *plan* for Parts 3–4, not fully
   authored materials. Requires reading "make" as "map out" — contradicted by
   the standing materials-ahead directive ("you develop all the materials
   ahead. so I can just follow along later").
3. **Claude picks the next two topics freely** rather than following the
   approved Phase-1 sequence. Requires ignoring the approved plan in MEMORY.md
   — implausible given the pipeline discipline.

No phonetic-garble reading of "parts"/"further" seems plausible; "further" reads
as voice-to-text emphasis meaning "continue onward".

**(c) Assumptions my primary reading requires:** "parts" continues the delivered
Part numbering; sequence follows the approved Phase-1 plan (004 then 005);
"make" = full materials-ahead authorship with validated solutions; the request
splits across two specs (two intent docs); Part-4 scope = the async-relay line
in MEMORY.md.

## 2. Comparison against the drafted intent.md

**Agreement — core meaning: full.** The draft's distilled intent is exactly my
primary reading: Part 4 of the course = spec 005 from the approved plan,
authored materials-ahead, quoting the same MEMORY.md scope line. Its first
assumption ("2 more parts" = Parts 3 and 4 = specs 004 and 005, materials-ahead)
matches my assumption (a)/(c) verbatim in substance. The learning-goal framing
(SKILLS 1d/2a; async foundations; AWS/TLS/batching explicitly out) is a faithful
elaboration of the plan line, not new meaning.

**Elaborations beyond the raw prompt (acceptable, and properly flagged):**
- The concrete service shape (`relay`, `POST /events`, appends to `dt=`
  partitions, 006 lifts the handler into Lambda) goes beyond what the prompt
  says, but derives directly from the approved plan entry and is the natural
  smallest reading of it.
- "Relay depends on the learner's own `glake` lib from Part 3" is a
  design-leaning coupling assumption not present in the prompt — but the draft
  correctly lists it under *Assumptions made* and provides a reconciliation
  path (sitting K), so it is visible to downstream gates rather than smuggled in.
- Fast-path pipeline and local-only/no-deploy boundary: consistent with the
  standing pattern and with 006 owning deployment.

**Alternative readings — coverage check:**
- The draft's rejected interpretations (jump to Lambda, replace hooks now,
  message-queue system) address Part-4 *scope* ambiguity; mine addressed the
  *"2 more parts"* phrase ambiguity. The draft resolves my alternatives 1 and 3
  implicitly via its first assumption; my alternative 2 (map-only, not full
  materials) is not explicitly rejected, but the materials-ahead directive
  quoted in the draft rules it out — a nice-to-have omission, not a divergence.
- None of the draft's rejected readings look wrongly rejected; each rejection
  cites a plan- or pedagogy-grounded reason.

## 3. Score

Same meaning, same scope boundary, same sequencing assumption; the only deltas
are (i) draft elaborates the service shape further than the prompt strictly
supports (justified by the approved plan and flagged as assumptions) and (ii) it
does not explicitly log the weak "outline-only" alternative reading. Neither
changes what the learner meant.

**Confidence: 95.**

VERDICT: 95% — Independent reading matches the draft (Part 4 = spec 005 async-relay, authored materials-ahead per the approved plan); the only unlogged alternative is the weak "course-map-only, not full materials" reading, which the standing directive rules out.
