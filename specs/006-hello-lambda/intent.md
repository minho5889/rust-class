# Intent — 006 hello-lambda

> **Doc 1 of 4. WRITE-ONCE.** Audited post-hoc by `intent-assurance`
> (`_assurance/intent-review.md`). No human gate.

**Created:** 2026-07-10 · **Spec status:** active

## Raw prompt (verbatim)

> "do part 5 and part 6 as well if we do not have spec then make them. Do not
> make me to intervene this is full loop you have full green lgihts."

Said the day after Parts 3–4 (specs 004/005) were delivered materials-ahead.
"Part 5 and part 6" continue the course numbering; no specs 006/007 existed,
so this spec is created by that instruction. This doc records the Part-5 half.

## Distilled intent

Author **Part 5 of the course** ahead of time: spec 006 from the approved
Phase-1 plan — *"relay handler as ARM64 Lambda + Function URL; `infra/` CDK
born; panic=abort cold-start experiment; lambda_runtime, serde events, release
profile in anger, cargo-bloat."* The door the learner built in 005 gets lifted
into **AWS Lambda**: same validation, same 202/400 contract, but the server is
now one async function on Graviton, the "disk" is CloudWatch logs (until 007
gives events a real S3 home), and the numbers that made Rust the choice —
cold start, binary size, memory floor — stop being claims and get measured.

**Full-loop directive:** build everything without pausing for the learner —
docs, audits, revisions, validated reference, `infra/`, sitting guides, critic
pass. Human gates are presented as acks at the end, not as blocking pauses.

## Learning goal

SKILLS **2a** (Lambda: lambda_http/lambda_runtime model, provided.al2023,
ARM64 build+deploy, cold starts) with Level-3 measurement habits (release
profile in anger, cargo-bloat). Explicitly out: any S3/data plane (007), other
compute targets (later specs).

## Assumptions made

- "part 5 / part 6" = the next two Phase-1 specs (006 hello-lambda,
  007 lake-to-s3), authored materials-ahead like Parts 1–4.
- **This session has no AWS credentials.** Local validation covers tests,
  properties, the real `cargo lambda build --release --arm64` artifact, and
  `cdk synth` + cdk-nag. Deploy, curl-against-the-real-URL, the cold-start
  experiment, and teardown are **deploy-day sitting moves on the learner's
  personal account** (confirmed ready, decision 2026-07-08) with Claude
  co-driving. The docs mark which acceptance lines are which.
- "Full green lights" authorizes the autonomous build loop; it does not
  delete the gate ritual — status stays `awaiting-review` until the learner
  acks (write-once honesty: pipeline rules survive enthusiasm).
- The learner writes the Rust (coached mode); Claude writes the TypeScript
  CDK (it's not the learning target, and the learner reviews it as the AWS
  professional they are) — the same split 002 used for infrastructure.

## Rejected interpretations

- ❌ Parts 5–6 = more local Rust fundamentals — the approved plan says 006 is
  the first deploy, and the learner's own sequencing preference was
  fundamentals-*then*-cloud; fundamentals are done (ramp 1–13, sittings A–N).
- ❌ Deploy from this session to some shared/sandbox account — no credentials
  exist here, and cost/account decisions belong to the learner (AI proposes,
  human disposes).
- ❌ Skip the gates entirely because of "full green lights" — the acks move
  to the end; they don't disappear.

## Addendum (append-only)

_(none yet)_
