# Requirements Assurance Review — 006 hello-lambda

**Auditor:** spec-auditor (fresh context)
**Date:** 2026-07-10
**Doc audited:** `specs/006-hello-lambda/requirements.md` (rev of 2026-07-10, awaiting-review)
**Audited against:** `specs/006-hello-lambda/intent.md` only. Context consulted:
`specs/005-async-relay/requirements.md` (H1/H2/H4 are defined by reference to it),
`specs/006-hello-lambda/_assurance/intent-review.md` (92% — above the 80% bar, so
no open-questions obligation applies), CLAUDE.md conventions.

---

## Checks that PASS (recorded so the human doesn't re-do them)

- **Readable-first standard:** plain-words transcript, honesty box, "not building",
  "what you'll learn" all present and genuinely readable; precise criteria below
  the fold; changelog collapsed. Meets the house shape.
- **Status header** format exact (`**Status:** awaiting-review`).
- **Traceability, forward:** every H-line maps to the intent's plan sentence
  ("relay handler as ARM64 Lambda + Function URL" → H1–H5/H8; "infra/ CDK born" →
  H8; "cold-start experiment" → H10; "cargo-bloat" → H6; "lambda_runtime, serde
  events" → H7; teardown/cost discipline → H11). No orphan requirements.
- **Scope-tag coherence** (red-team question answered): *(deploy day)* [O]s
  cannot block *doc approval* — they gate **spec close**, which by design waits
  for sitting Q, with evidence.md as the landing zone. The honesty box states
  this explicitly. Coherent; no fix needed.
- **Tagging present** on every line; [P] strategy correctly deferred to design
  per pipeline rules (only a pointer lives here).
- **No-unwrap rule** carried (H7, with `expect_used` added — stricter than the
  constitution, deliberately).
- **Learning goal coverage:** SKILLS 2a mapped via T1–T5; H3 hard-wires the T4
  lesson into an acceptance line — good practice.

---

## Findings

### MAJOR-1 — H4's "for any generated body… may never drift" is false at the size boundary
Relay's door includes axum's stock **413** above its default body limit
(005 A2, accepted as-is). hello-lambda has no such limit in code; Function URLs
carry up to ~6 MB and the platform rejects larger payloads *before the handler
runs*, with its own status/body. So over the full input space the two doors
**provably drift**, and the property as worded ("The two doors may never drift")
is either false or depends on an unstated size bound in the generation strategy.
Request-size behavior of the Lambda door is otherwise unspecified anywhere in
the doc.
**Fix:** bound H4's strategy explicitly (generated bodies stay under axum's
default limit) and add one sentence acknowledging the size-limit asymmetry as an
accepted, out-of-relation divergence — the exact move 005 A2 made with its 413
note. Optionally note non-UTF-8/binary bodies resolve via clause (1) of the door
check (fails JSON parse → 400 "not a JSON object") so the strategy may include
them deliberately or exclude them explicitly (see MINOR-5).

### MAJOR-2 — H1/H2's stdout purity contradicts H7's tracing allowance
H1 demands **exactly one** compact JSON line on stdout per accepted event; H2
demands **nothing** on stdout for rejected bodies. H7 allows
`tracing`+`tracing-subscriber`, whose default writer is **stdout** — a single
log line during any request violates both, and H5 makes these stdout assertions
test obligations. As written the doc is internally inconsistent and the
reference tests cannot pass without an unstated convention.
**Fix:** either scope the invariant to *event lines* ("the only **event** line
on stdout is…") or, better, require tracing output → **stderr** (both streams
reach CloudWatch, so nothing is lost and the "stdout = the lake in exile"
lesson stays clean). One clause in H1/H2 or H7.

### MODERATE-1 — The 005 reference is a soft pin on an unapproved document
H1 ("005's door check, verbatim"), H2 ("005 A2"), and H4 ("005's A4 message
strategy") define this spec's contract by reference — but 005's requirements are
themselves **awaiting-review**. If the 005 gate amends the door check or the
first-problem order, 006's meaning silently changes with no re-gate trigger; the
change protocol only fires on contradictions with *approved* docs. Partial
mitigations exist (H1's parenthetical summarizes the check; H4 is a live
cross-crate test that pins behavior operationally), but the doc-level pin is
soft, and "A4 message strategy" actually lives in 005's *design*, one more hop.
**Fix:** cite by version ("005 requirements rev 2.2's door-check block") or
inline the three-clause check (it is four lines in 005 — cheap), and add a
changelog-adjacent note: any amendment to 005's door ⇒ halt, amend 006 H1/H2/H4,
re-gate the amendment.

### MODERATE-2 — Intent's "memory floor" is never measured
The intent names three claims to stop repeating and start measuring: "cold
start, binary size, **memory floor**." T5 promises "cold start **and memory**…
from your own CloudWatch REPORT lines." H10 tabulates only **init + duration**;
the REPORT line's *Max Memory Used* column — the memory-floor number, and the
GC-runtime comparison's sharpest axis — is dropped. Coverage gap against the
intent.
**Fix:** add "max memory used" to H10's tabulated columns at both 128 MB and
512 MB; the GC-baseline paragraph should compare it too.

### MODERATE-3 — H3 is compound with a mixed, partly mis-tagged payload
One [E] line carries three obligations: (a) healthz 200 + counter shape,
(b) catch-all 404 for any other method/path (an unwanted-behavior/IF-THEN
pattern, a different stimulus class), and (c) "the doc and code must say out
loud…" — a documentation requirement that is not example-testable at all; it is
[O]-shaped. The changelog claims "single-clause EARS" was a pre-applied lesson;
H3 is the line where it wasn't.
**Fix:** split into H3a [E] (healthz), H3b [E] (404 catch-all), H3c [O]
(per-instance disclosure in `///` docs + sitting text).

### MODERATE-4 — The endpoint's auth posture is punted wholly to design
"Auth on the Function URL beyond the lab decision in design.md" forward-
references a decision (AuthType=NONE, public internet endpoint) that is a
*what-must-be-true* fact, not a shape choice — exactly what this gate exists to
put in front of the human. A reviewer approving requirements alone would not
learn the endpoint is unauthenticated. The combined fast-path gate softens this,
but the doc should stand on its own.
**Fix:** one [O] line: auth mode is a recorded design decision; exposure is
bounded by same-sitting teardown (H11); healthz/error responses and the stdout
event line carry no secrets (constitution's telemetry rule, restated at the
place it now matters).

### MINOR-1 — H5 is tagged [E] but is an [O]-shaped meta-requirement
It mandates the *means of proof* (local `#[tokio::test]`s, no AWS) rather than
an example behavior. Retag [O] or fold into each line. Note it also silently
sets the test seam H1/H2's stdout assertions need (see MAJOR-2) — the design
must provide an observable emit path.

### MINOR-2 — Plain-words transcript teaches a wrong shape
`"instance":"i-3f9a…"` mimics the EC2 instance-ID format; design defines the id
as init-timestamp + random bytes. On a Lambda spec whose T4 lesson is *what an
instance is*, the fake `i-` prefix is actively misleading. Fix the transcript.

### MINOR-3 — H6's "(expected: single-digit MB)" is neither bound nor clearly a note
If the artifact comes out 12 MB, does H6 fail? As written, no — but a reviewer
can't tell. State "recorded, not bounded (the 007 baseline)" or make it a bound.
Also say where the size/bloat numbers land (evidence.md, presumably).

### MINOR-4 — Intent-review's one flagged assumption isn't surfaced at this gate
The 92% intent audit asked for "one line of confirmation at the requirements
gate ack" on the least prompt-grounded assumption (Claude writes the TypeScript,
learner reviews). H8 births `infra/` without saying who writes it. Advisory
(verdict ≥ 80%, so no formal obligation): one clause in H8 lets the ack cover it.

### MINOR-5 — Non-UTF-8 / binary bodies covered only implicitly
They fall out of door-clause (1) (JSON parse fails → 400), but neither H2 nor
the H4 strategy pointer says whether the property generates them. One clause
resolves it; can be folded into the MAJOR-1 fix.

---

## Cold-start experiment done-ness check (red-team item, mostly OK)

H10 is judgeable as written: ≥5 cold starts × 2 memory sizes, REPORT numbers
tabulated, one *cited* comparison paragraph — a measurement [O], not a pass/fail
bound, which is the right shape. Gaps: the missing memory column (MODERATE-2)
and, at most a note, T3 promises the release profile "justifies itself" in cold
start, but H10 has no profile A/B — the claim is only *argued from* the
measured numbers, never isolated. Acceptable for scope; the sitting guide
should not oversell it.

---

VERDICT: 74% — Strong, readable, fully traceable draft whose two MAJORs (H4's door-equivalence property is falsifiable at the size boundary; H1/H2 stdout purity contradicts H7's tracing allowance) plus the soft pin on unapproved 005 need one revision pass before the human gate.
