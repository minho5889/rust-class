# Intent assurance review — 004 glake-traits

**Auditor:** intent-assurance (fresh context) · **Date:** 2026-07-09
**Input:** raw prompt (verbatim) + delivery context, derived blind *before* reading `intent.md`.

## 1. Blind derivation (written before reading the draft)

**Raw prompt:** "great can you make 2 more parts further?"

**Context available:** sent immediately after delivery of the Part-1/Part-2
course map (ramp steps 1–8 + glake v0 sittings A–F, spec 003), under the
standing materials-ahead directive; the approved Phase-1 plan sequences
spec 004 (glake lib+bin: query traits, thiserror chain, proptest; traits,
generics, modules, error design, API rubric) then 005 (async relay).

**My distilled intent:** the learner approves the delivered material and asks
Claude to extend the pre-authored course by two more numbered parts — Parts 3
and 4 — in the same materials-ahead style (worksheets, hint ladders, validated
references). Per the approved plan, Part 3 = spec 004, Part 4 = spec 005. This
spec covers the Part-3 half.

**Alternative readings I considered:**

1. **"2 more parts" = more segments inside the existing material** (extra ramp
   steps or extra 003 sittings). *Requires:* "parts" being loose slang, ignoring
   the Part-1/Part-2 taxonomy the message directly replied to. Weak — the ramp
   is complete at 8 and the reply anchors to the two-Part map just shown.
2. **"make" = outline only** — extend the course *map* by two parts, defer full
   worksheet authoring. *Requires:* "make a part" meaning "sketch a part",
   against the explicit standing directive "you develop all the materials
   ahead… so I can just follow along later". Weak, but it is the one reading
   that changes scope materially (map vs. full authored curriculum).
3. **Two arbitrary new parts beyond the Phase-1 plan** (e.g. jump to AWS).
   *Requires:* ignoring the approved plan's 004→005 ordering and "further"
   implying continuation of the existing trail. Weakest.

**Assumptions my primary reading requires:** "parts" tracks the numbered
course-map taxonomy; the Phase-1 plan defines the content of Parts 3–4;
materials-ahead means fully authored + validated, not outlined; splitting the
single prompt into two Units of Work (004 and 005), one per part, is a faithful
decomposition.

## 2. Comparison with the drafted `intent.md`

**Distilled intent — agreement: strong.** The draft reads the prompt exactly as
my primary reading: Parts 3–4 = specs 004–005 per the approved plan, authored
materials-ahead; this doc is the Part-3 (004) half. Same meaning.

**Assumptions — agreement with notes.**
- "2 more parts = Parts 3 and 4 = specs 004/005, materials-ahead like Parts
  1–2" — identical to my derivation.
- "Part 3 builds on the reference v0 shape; divergence reconciles in sitting G"
  and "003's 'not building yet' list is Part 3's contract" — sound elaborations
  from spec-003 context; not derivable from the prompt alone but consistent
  with it and correctly filed as assumptions.
- "Fast-path pipeline; gates presented for ack at the end rather than blocking
  authoring" — a **process** assumption, not a meaning assumption. The
  constitution's fast path allows one combined approval for small
  well-understood units, so this is defensible, but it stretches "great can you
  make 2 more parts further?" slightly: the learner asked for materials, not
  for a specific gating posture. Minor; flagging, not diverging.

**One elaboration to note:** the draft's framing that new crates enter "as a
measured comparison, with the lens showing exactly what the hand-written
version saved and what the abstractions cost" is design flavor imported from
the 003 trajectory, not from the raw prompt or the Phase-1 plan wording. It is
plausible and pedagogically consistent, but it pre-commits a comparative
methodology at the intent layer. If requirements later drop the
measured-comparison angle, that is not an intent violation — the prompt itself
does not demand it.

**Rejected readings — coverage check.** The draft rejects: more ramp steps
(= my alt 1, same grounds), skip-to-AWS (= my alt 3), and serde-replaces-hand-
scanner (a content-level reading I hadn't generated; reasonable to reject).
The draft does **not** list my alt 2 (map/outline only, authoring deferred).
It is implicitly rejected by the materials-ahead assumption, and the standing
directive makes it very unlikely to be what the learner meant, so this is a
completeness gap, not a divergence.

## 3. Score

Same core meaning; identical primary reading, identical decomposition into
004/005; one unlisted (weak) alternative reading and two mild elaborations
that go beyond the prompt but not against it.

**Confidence: 93** — above the 80 escalation threshold; no questions escalate
to the requirements gate.

VERDICT: 93% — Independent reading matches the draft (Parts 3–4 = specs 004/005, materials-ahead); best unlisted alternative — "make 2 more parts" as course-map extension only, authoring deferred — is implicitly rejected by the standing directive and does not warrant escalation.
