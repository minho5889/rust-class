# Spec Audit — 006 hello-lambda: design.md + tasks.md (combined)

**Auditor:** spec-auditor (fresh context, adversarial)
**Date:** 2026-07-10
**Inputs:** design.md audited against approved-pending requirements.md; tasks.md
against design.md. Cross-read: 005 design.md + `_reference/relay/src/`
(validate.rs, routes.rs), 004 `_reference/glake/src/parser.rs`, CLAUDE.md AWS
conventions, `research/typescript-cdk-for-goldeneye.md`, 006 intent.md +
intent-review (92%).

---

## Traceability matrix (summary)

| REQ | Design element | Task | Status |
|---|---|---|---|
| H1 | handler.rs row | 1.2 | covered — but see MAJOR-2 (stdout clause untestable as designed) |
| H2 | handler.rs / door rows | 1.2 | covered — same stdout-absence caveat |
| H3 | handler.rs + state.rs | 1.2 | covered — see MODERATE-6 (instance id / started-at) |
| H4 | Properties table | 1.1 (red-first ✓) | covered — see MODERATE-5 (harness under-specified) |
| H5 | "How we verify" | 1.2 | covered |
| H6 | verify section | 1.3 | covered; bloat host-target caveat honestly stated ✓ |
| H7 | verify (lints only) | 1.2 (lints only) | **partial** — tracing dep has no design element (MODERATE-4); no-aws-sdk check unassigned (MINOR-9) |
| H8 | infra row + verify | 1.4 | covered — see MAJOR-3 (nag rule / v3 API / Serverless pack) |
| H9–H11 | deploy-day section | 1.5 | covered |

No design element cites zero REQs (no scope creep). Every Key-decisions row has
genuine alternatives and a real reason. "Rust you learn" column present and maps
to SKILLS 2a (lambda_http model, provided.al2023, cold starts) plus the T4
per-instance-state lesson. Property is red-first in tasks with the red commit
named. Coached-mode split (learner writes Rust; Claude writes TS, learner
reviews; property co-written) is stated in intent, design decision row, and
tasks 1.1/1.2/1.4 consistently — no violations found.

---

## Findings (by severity)

### MAJOR-1 — Past-tense validation claims are not true in the repo at `awaiting-review`

design.md "How we verify" says *"the reference was validated with all of
these"*; tasks.md says sitting guides O–Q *"are authored and validated against
the 006 reference before this gate is presented."* Both docs are already
`Status: awaiting-review`, yet the repo contains **no**
`specs/006-hello-lambda/_reference/`, no `sittings/`, no `crates/hello-lambda`,
and no `infra/` (Glob confirms: only the four docs + `_assurance/`). Either the
status flip is premature or the claims are forward-dated — both break the gate's
honesty (the reviewer is being told validation happened when it hasn't).
**Fix:** build the reference + materials + infra and record the actual command
outputs before presenting the gate, or reword both passages to future
commitments and hold status at `drafting` until they're true.

### MAJOR-2 — H1/H2's stdout clauses are untestable under the design as written

H1: accepted → *"exactly one compact JSON line on stdout"*; H2: *"nothing is
emitted to stdout for rejected"*; H5 claims all of H1–H3 are proven by
`#[tokio::test]`s. But the handler.rs row emits via `println!("{line}")` inside
the handler — an in-process test **cannot observe its own process's stdout**
(libtest capture isn't assertable; there is no seam). As designed, the tests can
prove status codes but not the one-line/no-line claims, so H5's promise is
hollow for exactly the clause 007 depends on. **Fix (small):** make the handler
core return the emit decision — e.g. door wrapper returns
`Result<EmitLine(String), Rejection>` or the handler takes a `&mut impl
io::Write` sink — with the single `println!`/`writeln!(stdout)` at the outer
edge. Tests assert the returned/written line (exactly one, compact, parses back
to the body); H9 verifies real stdout→CloudWatch on deploy day. This also
silences the 256-case property-run print noise for free.

### MAJOR-3 — The cdk-nag suppression story is unverified and the API/pack choice contradicts the research

Three connected problems in the infra row + verify section:

1. **No rule ID is named** for the AuthType=NONE suppression, and it is not
   established that the AwsSolutions pack *has* a Function-URL-auth rule (its
   Lambda coverage is famously thin — AwsSolutions-L1/runtime). If no rule
   fires, `NagSuppressions` metadata for a never-triggering rule is dead text,
   "zero unsuppressed findings" passes **vacuously**, and the centerpiece of the
   auth decision ("the suppression text IS the security lesson") silently
   evaporates. **Fix:** during reference validation, run synth and record which
   rule (if any) actually fires with its ID; if none does, restate the decision
   as a written risk note in the stack file + an explicit template assertion on
   `AuthType: NONE` (the lesson survives; the mechanism changes), and say so in
   design.md.
2. **Stale API:** design says "cdk-nag `AwsSolutions` **aspect**" — the research
   report (Guardrails, 3-0 verified) says v3 registers via CDK-native
   `Validations.of()`, *not* Aspects, and warns v2-era snippets are stale.
3. **Missing pack:** CLAUDE.md and the research both mandate **AwsSolutions +
   Serverless** packs; the design names only AwsSolutions.

### MODERATE-4 — H7's `tracing` (+subscriber) has no design element, and the stdout story ignores it

No row wires a subscriber; nothing says where tracing output goes. Two real
consequences: (a) a requirement (part of H7) is satisfied by no design element —
a traceability gap; (b) tracing_subscriber's fmt default writer is **stdout**,
so tracing lines interleave with the H1 event lines in the very stream H9 greps.
Note that routing tracing to stderr does *not* keep it out of CloudWatch —
Lambda merges stdout and stderr into the same log stream — so the design must
name the discrimination story explicitly: subscriber → `io::stderr` (keeps
stdout pure = "one line per accepted event, and nothing else, on stdout"
becomes a checkable claim), event line remains the only bare-JSON stdout line,
and sitting Q filters on that shape. One sentence in the main.rs row fixes it.

### MODERATE-5 — H4 harness: which relay, and the liveness precondition

The property compares against "relay via its router `oneshot`", but: (a)
`crates/relay` does not exist yet — 005 is itself materials-ahead and
unexecuted; the reference build must dep on
`specs/005-async-relay/_reference/relay` (a workspace-external path dep) while
the learner's sitting-O run should compare against **their** `crates/relay`.
Design names neither path; tasks never state the precondition "005 sittings
K–N complete" for sitting O. (b) Relay's 202 path does `tx.send(...).await` —
the oneshot harness must construct `AppState` with a **live receiver held open**
or every accepted case comes back 500 ("writer unavailable") and the property
compares garbage. (c) Comparing "(status, error_text)" — parsed text, not raw
bytes — is right; keep it that way (axum `Json` vs hand-built body need not be
byte-identical). **Fix:** one sentence in the Properties row naming the dev-dep
path + the open-rx harness; one precondition line in tasks sitting O.

### MODERATE-6 — Instance-id scheme: unspecified randomness, and `started` semantics

state.rs: "instance id (from init timestamp + a few random bytes)". (a) H7's
dependency list contains no randomness source (`rand`/`uuid`/`getrandom` all
absent) — so "random bytes" is either a new dep that violates the H7 list as
written, or hand-rolled `/dev/urandom`, or a weak hasher hack; the design must
pick. (b) Timestamp alone collides when a scale burst cold-starts several
instances in the same second — exactly the two-curls-hit-different-instances
demo H3/T4 depends on distinguishing. (c) If the `OnceLock` is initialized
lazily in the handler, `started` records **first-request** time, not init time
— slightly dishonest for the cold-start narrative; initialize in `main` before
`lambda_http::run`. **Cheapest honest fix:** derive the id from
`AWS_LAMBDA_LOG_STREAM_NAME` (unique per instance, zero deps, and lets sitting
Q cross-check `/healthz`'s `instance` against the CloudWatch stream name — a
better lesson), with a timestamp fallback for local tests.

### MINOR-7 — Task ordering: 1.1 needs a crate that 1.2 creates

1.1 writes the red H4 property "hello-lambda's door stubbed (`todo!()`)", but
`cargo new crates/hello-lambda` happens in 1.2. The stub and the test need a
crate to live in. Move crate creation (+ stub door) to the front of 1.1, or
split "scaffold" out as 1.0.

### MINOR-8 — Synth-time bundling prerequisite unstated

`RustFunction` **compiles the crate at `cdk synth`** (local `cargo-lambda ≥
0.12` or `forcedDockerBundling: true` — research, Lambda section). Design lists
cargo-lambda+zig under the *build* bullet only; sitting P's "run `npx cdk
synth`" and deploy day inherit the same prerequisite silently, and synth time
includes a full cross-compile (worth saying so the learner doesn't read a
2-minute synth as a hang). One clause in the verify section.

### MINOR-9 — H7's "No `aws-sdk-*`" check is unassigned

005 verified its dep policy via a `cargo tree` check (A8); 006's ops checklist
covers lints but not the dependency-policy assertion. Add "cargo tree shows no
aws-sdk-*" to the operations checklist / 1.2.

---

## Red-team items that check out (no finding)

- **Function URL → method+path routing:** sound. Function URLs deliver the API
  GW v2 payload; lambda_http surfaces it as a real `http::Request`, so
  `req.method()` + `req.uri().path()` (rawPath, e.g. `/events`) are available;
  a bare Function URL accepts any path, and the design's "else 404" handles
  that. Hand-built Requests for H5 work because the handler never touches the
  request context.
- **Rust stdout buffering:** `std::io::Stdout` is line-buffered via `LineWriter`
  even when piped, and `println!` holds the lock per call — the one-line claim
  doesn't tear within a process.
- **Property discipline:** one property is honestly the right count here; the
  drift argument in design.md is good.
- **Decisions table:** all six rows have real alternatives and reasons; the
  reference-profile-replication row is a subtle catch done right.

## Verdict rationale

Structure, traceability, teaching mapping, red-first discipline, and the
coached-mode split are in genuinely good shape. But three majors block the gate
as-is: the docs assert validation that hasn't happened in-repo, the flagship
H1/H2 stdout claims can't be tested by the tests that claim to prove them, and
the cdk-nag suppression — the named centerpiece of the auth decision —
references an unnamed, possibly nonexistent rule via a stale API with a missing
pack. All are cheaply fixable (a seam, a sentence, and actually running the
reference build).

VERDICT: 68% — Traceability, red-first, and the coached split are solid, but the gate is blocked by unverified past-tense validation claims, an untestable H1/H2 stdout clause behind an in-handler println!, and an unnamed (possibly nonexistent) cdk-nag rule anchoring the NONE-auth suppression story.
