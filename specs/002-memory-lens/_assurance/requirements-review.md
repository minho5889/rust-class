# Requirements Review — 002 memory-lens

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-05
**Audited doc:** `../requirements.md` (Status: drafting)
**Inputs:** `../intent.md` only, plus `_assurance/intent-review.md` (protocol check),
`CLAUDE.md` (tag definitions), `SKILLS.md` (learning mapping).

---

## Protocol check: intent-assurance verdict

Intent verdict was **90% (≥ 80)** — no mandatory escalation. The two optional
confirmations it recommended are both present in Open questions:

- ✅ IDE cut / voice-bot deferral confirmation — present (Q1).
- ✅ v0 audience = local single-learner — present (Q2).
- ⚠️ Nuance dropped: the intent audit's first confirmation also asked *"whether a
  non-voice interactive helper is wanted in v0 or later"* (the middle reading of
  "a bot who you can interact with voice chat"). Q1 only asks about the cut/deferral
  as drafted. See Finding 6.

## Traceability map (summary)

| Intent element | Covered by |
|---|---|
| Record allocations/deallocations/growth/drops | R1–R5 |
| Dashboard, memory-only, "all at once" | R9–R12, Out-of-scope list |
| Polished/smooth UI/UX | R13 only (see Finding 8) |
| Feature-flag instrumentation, overhead honesty | R6, R14 |
| v0 local HTML viewer, no AWS | R11 |
| Envelope/data-lake compatibility (intent assumption) | R8 |
| Teaching-instrument honesty (not a profiler) | R15 |
| Learning goal (SKILLS 1b + GlobalAlloc/unsafe/zero-cost) | L1–L4, gaps in Finding 3 |

No orphan requirements: every R traces to a distilled-intent clause or a stated
intent assumption (R8 and R11 trace to assumptions, which the intent audit already
flagged as drafter-added but correctly placed). No scope creep found.

---

## Findings (by severity)

### MAJOR

**1. Compound requirements: R2, R7, R14 each carry two obligations in one line.**
This breaks one-SHALL-per-ID and muddies counterexample triage (which half failed?).
- R2 = (a) every dealloc matches a prior alloc of same address+size, (b) live-bytes
  never negative. Note (b) is a corollary of (a) — if kept, keep it as a separate,
  explicitly redundant sanity property.
- R7 = (a) SHALL not panic the traced program, (b) SHALL drop events and record one
  loss marker. Two distinct behaviors, separately testable.
- R14 = (a) SHALL document measured overhead [O], (b) tracing SHALL never be enabled
  in benchmark/production builds — different verification classes entirely.
**Fix:** split into R2a/R2b [P]+[P], R7a/R7b [E]+[E], R14a/R14b [O]+[O or E].

**2. R10 is tagged [O] but its core is the unit's best [P] candidate.**
"Live allocations at instant t with scope labels" is a pure function of the trace
(replay/fold up to sequence n). Per CLAUDE.md, [P] requirements drive the
proptest-first workflow and are explicitly teaching tools; tagging the whole thing
[O] ("open it and look") forfeits that for the one computation most worth proving
(∀ valid trace, ∀ t: live set from replay ≡ live set displayed / live-bytes graph
value). R9's *rendering* is legitimately [O].
**Fix:** split R10 into R10a [P] — state-reconstruction invariant on the (headless)
trace-replay function, over generated valid traces — and R10b [O] — the viewer
displays that set at the scrub position. Same consideration applies to the
live-bytes graph in R9.

### MEDIUM

**3. Learning-goal coverage gaps vs intent.**
Intent names "SKILLS 1b (entire section)": ownership, borrowing, **stack vs heap,
`Box`**, collections growth, drop order. L1–L4 cover ownership/borrowing/drops/
collections/GlobalAlloc, but:
- No learning requirement for `Box<T>` / stack-vs-heap / "when allocation happens"
  (SKILLS 1b line 4) — the single most lens-visible concept (a `Box::new` is exactly
  one trace event; a stack value is exactly zero).
- `HashMap` is named in the intent's own definition of "collections" but appears in
  no R or L line (R5 and L3 are Vec/String only).
- `Rc`/`Arc` (SKILLS 1b line 5) absent — acceptable to defer, but then say so in
  Out of scope rather than silently.
**Fix:** add L5 (Box / stack-vs-heap, evidenced by a trace showing allocation vs
no-allocation), extend R5 or L3 to cover HashMap (or explicitly scope collections
= Vec/String for v0), and add an Out-of-scope line for Rc/Arc if deferred.

**4. R6 is untestable as written and misuses WHILE.**
"SHALL add zero code to the binary" — binaries differ for unrelated reasons
(metadata, layout); "zero code" has no test procedure. Also, EARS `WHILE` is for
continuous runtime states; a disabled cargo feature is build-time configuration
(`WHERE`-pattern or ubiquitous territory).
**Fix:** rewrite as, e.g., "WHERE the `memlens` feature is disabled, THE SYSTEM
SHALL produce no trace output, SHALL NOT replace the global allocator, and the
binary SHALL contain no memlens symbols (verified via `nm`/symbol inspection)"
— and split, per Finding 1's rule.

**5. R14(b) "never enabled in benchmark or production builds" has no verification
mechanism.** As an [O] line it needs an operational check, or it's a wish.
**Fix:** state the mechanism — e.g., CI/`cargo` guard asserting `bench`/`release`
profiles do not activate the `memlens` feature — and tag that check [E] or [O]
accordingly.

**6. Open question Q1 drops the "non-voice interactive helper" nuance** from the
intent audit's recommended confirmation (alt-A residual: cutting the *bot* entirely
goes beyond deferring *voice*).
**Fix:** extend Q1: "…and whether a non-voice (text) interactive helper is wanted
in v0 or later."

### MINOR

**7. Vague phrases that will bite at design/verification time:**
- R4 "at the correct sequence position" — correct relative to what? Define:
  ordered consistently with the allocation/deallocation events of the annotated
  scope (drop event after the scope's frees begin? before? pick one).
- R12 "make the absence of allocation events explicit" — define the minimum
  observable (a visible zero-cost marker/callout element in the timeline).
- R13 "without visible lag" — quantify (e.g., scrub update < 100 ms at 100k events)
  so the open question about the 100k bound has a measurable partner.

**8. "Great UI/UX / polished, smooth" from the distilled intent is operationalized
only as R13 responsiveness.** Probably the right call (taste isn't EARS-able), but
it's an implicit decision — record it, e.g., a note that polish is verified [O] at
the human design/evidence gates, so the intent's most-emphasized adjective isn't
silently narrowed to latency.

**9. EARS subject drift:** R5 says "THE trace SHALL contain…" — the trace is an
artifact, not the system. Rephrase as "THE SYSTEM SHALL record…". R5 could also be
[P] (∀ push sequences, realloc events reflect capacity growth) — at minimum note
why [E] was chosen (fixed growth factor is impl-defined, fine).

**10. L1–L4 have no verification mechanism.** CLAUDE.md exempts learning
requirements from [P]/[E]/[O], but say how they're checked (learner explains
unaided in session; recorded in MEMORY.md/evidence.md) so "SHALL be able to
explain" isn't unfalsifiable.

**11. No requirement covers instrumentation ergonomics.** The first user story
says "run *any* of my Rust programs with instrumentation," but no R defines what
opting a program in looks like (feature flag + one-line allocator install?
workspace crates only?). One [E] line would anchor the design.

---

## What is already good

- Honest, pre-filled Open questions section (including its own R13 threshold doubt).
- Out-of-scope list mirrors the intent's rejected readings exactly — the
  workflow-dashboard misread cannot re-enter through this doc.
- R2/R3 as [P] lines are genuine allocator invariants, not decorative properties.
- R15 (label what the lens cannot see) is an unusually mature honesty requirement
  that directly serves the teaching intent.

## Score rationale

Traceability and scope discipline are strong; nothing here is a meaning-level
error. But three compound lines (per "untestable = unapprovable", each must be one
testable statement), one untestable/mis-patterned line (R6), and a mis-tagged
flagship [P] (R10) mean the human would be reviewing lines that must change anyway.
Fix Findings 1–6 and this is a fast approval.

VERDICT: 78% — Strong traceability and honest scoping, but three compound EARS lines (R2/R7/R14), an untestable R6, a mis-tagged property-grade R10, and SKILLS-1b learning gaps (Box/stack-vs-heap, HashMap) need one revision pass before human review.
