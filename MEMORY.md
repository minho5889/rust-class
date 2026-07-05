# MEMORY.md — Living Memory

> Claude: read this at the start of every session; append to it at the end of every
> session. Keep entries short and factual. Newest session-log entries go on top.

## Learner profile

- **Name / contact:** Minho (minho5889@gmail.com)
- **Rust experience:** Beginner — no substantial prior Rust knowledge.
- **Goal:** Learn Rust as a systems-engineering language with a focus on memory
  management and efficiency, deployed on AWS.
- **Target AWS services:** Lambda, Lambda MicroVMs (June 2026), ECS Fargate, EC2.
- **Methodology:** AI-DLC + Kiro-style spec-driven development (no Kiro IDE),
  extended by the learner's own four-doc pipeline design — see `CLAUDE.md` v2.
- **Learning style:** Fundamentals first ("the bedrock"), then hands-on building;
  likes process/telemetry thinking (designed the data lake himself).

## Current state

- **Project codename:** goldeneye. Regions: us-east-1 (primary), ap-northeast-1
  (secondary).
- **Phase:** Wave 1 complete (pipeline v2 codified, telemetry v1 live, no AWS yet).
- **Active Unit of Work:** none. Next spec number is **002** (001 was retired —
  see decision log). Wave 2 candidate: `002-datalake-v1` (S3 zones + first scan).
- **Toolchain:** unverified — check `rustup`, `cargo-lambda`, Docker, AWS CLI at the
  start of the first hands-on session.

## Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-05 | Adopt AI-DLC three-phase workflow with Kiro-style specs | Learner requested borrowing AI-DLC logic without the Kiro IDE |
| 2026-07-05 | ARM64/Graviton default everywhere | Cheaper; MicroVMs is ARM64-only |
| 2026-07-05 | Pipeline v2: four docs (intent → requirements → design → tasks) + gate-less evidence.md; assurance-before-attention subagents; change protocol; fast path; [P]/[E]/[O] verification tags | Learner's design + Claude's improvement suggestions, all approved |
| 2026-07-05 | Property-based testing via `proptest`, Kiro-style: EARS requirements → properties, test-first, counterexample triage (spec/code/test bug), regressions committed | Spec-as-executable-contract; strong teaching value |
| 2026-07-05 | Telegram notifications are one-way v1 (notify only); approval = in-session or Status-line edit. Two-way bot deferred (future Rust Lambda UoW) | Avoid listener infrastructure now |
| 2026-07-05 | goldeneye data lake: hooks → local JSONL (v1, committed to git) → S3 `goldeneye-lake` raw/curated (Wave 2) → `goldeneye-discovery` insight cards (Wave 3). DuckDB default scan engine, Athena optional. No Kinesis/dashboards/Iceberg | Learner's idea; sized for a one-learner repo; doubles as curriculum |
| 2026-07-05 | Project name **goldeneye**; us-east-1 primary, ap-northeast-1 secondary; raw prompts may be committed (private repo) | Learner decision |
| 2026-07-05 | Spec 001-hello-rust-lambda (draft, never approved) **removed**; numbering starts at 002 | Learner said "don't make one"; predated pipeline v2 (no intent.md); recoverable at commit 3bba9d5 |

## Concepts mastered

_(Move items here from `SKILLS.md` when demonstrated, with date and evidence.)_

- _none yet_

## Open questions for the learner

- AWS credentials: are they configured in this environment? (Blocks Wave 2.)
- Telegram: provide `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` as env vars when you
  want gate notifications live (hook already falls back gracefully without them).
- IaC preference (SAM vs CDK) — deferred until after Lambda mastery.

## Session log

### 2026-07-05 — Session 2: Pipeline v2 + goldeneye telemetry (Wave 1)
- Researched property-based testing and Kiro's correctness feature (EARS → extracted
  properties → generated cases → shrinking → spec/code/test triage); learner
  approved all improvement suggestions.
- Codified pipeline v2: rewrote `CLAUDE.md`; rebuilt `specs/_template/` as five docs
  with `_assurance/` sidecar; added `.claude/agents/` (intent-assurance,
  spec-auditor, property-auditor); wired hooks (`settings.json` → telemetry.sh,
  on-doc-write.sh) — tested in sandbox: valid envelopes, gate detection, template
  exclusion, Telegram fallback all working.
- Created `datalake/` (envelope.v1.json schema registry, raw-local zone, README with
  S3 naming fixed: goldeneye-lake / goldeneye-discovery).
- Updated steering (tech: proptest conventions, goldeneye identity, data lake;
  structure: new tree). Removed spec 001 per learner instruction.
- **Next:** learner reviews Wave 1; then `002-datalake-v1` through the new pipeline
  (needs AWS credentials), or start SKILLS Level 0/1 with a playground exercise.

### 2026-07-05 — Session 1: Foundation setup
- Created initial scaffolding: CLAUDE.md v1, MEMORY.md, SKILLS.md, steering/,
  specs/_template (3 docs), spec 001 draft (later removed).
- Researched: AI-DLC, Kiro spec-driven development, Lambda MicroVMs launch
  (2026-06-22), Rust-on-AWS tooling (cargo-lambda, Rust GA on Lambda Nov 2025).
