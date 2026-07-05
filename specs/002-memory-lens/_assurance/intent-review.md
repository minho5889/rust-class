# Intent Assurance Review — 002 memory-lens

**Auditor:** intent-assurance (fresh context) · **Date:** 2026-07-05
**Input:** raw prompts (verbatim, 3 messages) · **Audited doc:** `../intent.md`
**Protocol note:** blind derivation was completed BEFORE reading `intent.md`.

---

## 1. Blind derivation (pre-read)

### 1a. My distilled intent

Build a polished, smooth web dashboard whose **sole purpose** (per correction #2:
"soley for to understand whats going on when they run the program in rust
memory!!!") is to make visible what happens in memory while a Rust program runs —
allocations, deallocations/drops, and collection (Vec/String/HashMap) growth — as a
learning aid. The original ask bundled a small embedded IDE and a voice-chat bot;
the main assistant recommended narrowing to memory visualization via a tracking
allocator, cutting the IDE and deferring voice. The learner's correction reads as
*endorsing and hardening* that narrowing (dashboard = memory only), and "Lets go"
approves the scoped plan.

### 1b. Alternative readings considered blind

- **A — Maximal scope.** "Nono" only clarified the *dashboard's* purpose and did
  not concede the IDE/voice-bot cuts; "Lets go" green-lights the full original
  vision (dashboard + mini IDE + interactive bot).
  *Requires:* reading "Nono" as pushback against the cuts rather than against a
  content misread — but the learner restated the dashboard's purpose, not "keep
  the IDE", so this is the weaker reading. Residual risk: an *interactive bot*
  (text, not voice) may still be wanted even if voice is deferred; the phrase was
  "a bot who you can interact with voice chat", and cutting the bot entirely goes
  slightly beyond deferring voice.
- **B — Semantics-deep.** "collections" is GC-background vocabulary (the learner
  is new to Rust, coming from GC languages), so the tool should visualize
  ownership/lifetime/drop *semantics* (who owns what, when drops fire), not merely
  allocator byte counts. "event or session or contract information" is
  voice-to-text garble (possibly "context").
  *Requires:* reading "collections" as free/drop events rather than Rust
  collection types. Both readings converge if the instrument records drops.
- **C — Audience/deployment.** "when *they* run the program" suggests a tool for
  other learners or for observing deployed AWS workloads' memory (Lambda/Fargate/
  EC2), implying remote telemetry or multi-user scope rather than a local
  single-learner tool.
  *Requires:* taking the third-person pronoun literally in noisy voice-to-text;
  weak, but it silently sets the v0 delivery target (local vs hosted).

### 1c. Assumptions my primary reading requires

- "bots next" is garble (widgets / "bot's nest" / "bots inside") and carries no
  independent requirement beyond the bot already named later in the sentence.
- "Lets go" is consent to the assistant's recommended scope as amended by
  correction #2, not to the original maximal ask.
- "event or session or contract information" is superseded by "solely… rust
  memory" and is out of scope.

## 2. Comparison against the drafted `intent.md`

| Point | Draft | Blind derivation | Agreement |
|---|---|---|---|
| Core deliverable | Memory recorder + polished dashboard, memory-only scope | Same | ✅ |
| "collections" | Rust collection types + growth; drops included | Same; my alt-B (drop semantics) is substantively covered because the instrument records drops and drop order (SKILLS 1b) | ✅ |
| event/session/contract | Superseded by correction, out of scope | Same | ✅ |
| IDE / voice bot | Cut / deferred; **honestly flagged as never explicitly re-confirmed** | Same reading; same residual risk (my alt-A) | ✅ — the draft self-flags the exact soft spot |
| Rejected readings | Workflow-observability dashboard; full suite; production profiler | Matches my alt-A; workflow-dashboard rejection is consistent with the correction | ✅ |
| Delivery target | v0 local HTML viewer, no AWS | Not derivable from prompts either way; my alt-C ("when *they* run") is not discussed | ⚠️ minor gap |
| Data-lake compatibility of traces | Assumed | Not present in any raw prompt — drafter-added scope, though low-risk and consistent with repo conventions | ⚠️ minor addition |

### Divergences (all minor)

1. **Alt-C unaddressed.** The draft assumes a local, single-learner tool ("v0 …
   local HTML viewer") without noting that "when they run the program" could be
   read as other users / deployed workloads. Low probability; costless to confirm
   at the requirements gate.
2. **Interactive-bot nuance.** The draft cuts the bot entirely (voice "deferred").
   A non-voice interactive bot is a conceivable middle reading of "a bot who you
   can interact with voice chat". Covered in spirit by the draft's own flag that
   the cut was never explicitly re-confirmed.
3. **Drafter-added assumptions** (data-lake envelope compatibility; "polish applies
   to visualization, not hosting") go beyond the prompts. Both are plausible and
   flagged as assumptions rather than smuggled into the distilled intent — correct
   placement.

### Alternatives the draft handled well

- The GC-vocabulary reading of "collections" is effectively absorbed (growth AND
  drops both in scope).
- The workflow-observability misread is explicitly named and rejected with the
  learner's own correction as evidence.
- The unconfirmed IDE/voice cut is flagged rather than hidden — exactly what an
  intent doc should do.

## 3. Confidence score

Core meaning is identical; the draft's assumptions section is honest about the one
real risk (unconfirmed scope cut). Remaining divergences are minor, cheap-to-ask
items suitable for the requirements gate, not meaning-level disagreements.

**Score: 90/100** (≥ 80 — no mandatory escalation; two optional confirmations
recommended below).

### Recommended (optional) confirmations for the requirements gate

- Confirm the IDE cut and voice-bot deferral explicitly (the draft itself flags
  this); ask whether a *non-voice* interactive helper is wanted in v0 or later.
- Confirm the audience/runtime: local single-learner runs only in v0, or should
  traces from programs run elsewhere (other users / AWS targets) be in scope?

VERDICT: 90% — Independent reading matches the drafted intent (memory-only teaching dashboard via tracking allocator); best alternative reading is the maximal-scope one (IDE + interactive bot survive "Nono"), which the draft already flags as never explicitly re-confirmed.
