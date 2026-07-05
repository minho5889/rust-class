# MEMORY.md — Living Memory

> Claude: read this at the start of every session; append to it at the end of every
> session. Keep entries short and factual. Newest session-log entries go on top.

## Learner profile

- **Name / contact:** Minho (minho5889@gmail.com)
- **Rust experience:** Beginner — no substantial prior Rust knowledge.
- **Goal:** Learn Rust as a systems-engineering language with a focus on memory
  management and efficiency, deployed on AWS.
- **Target AWS services:** Lambda, Lambda MicroVMs (new, June 2026), ECS Fargate, EC2.
- **Methodology preference:** AI-DLC (AWS's AI-Driven Development Lifecycle) +
  Kiro-style spec-driven development, without the Kiro IDE.
- **Learning style:** Wants fundamentals prepared first ("the bedrock"), then
  hands-on building.

## Current state

- **Phase:** 0 — Environment & fundamentals (see `SKILLS.md` Level 0/1).
- **Active Unit of Work:** none yet. First candidate: `specs/001-hello-rust-lambda`.
- **Toolchain installed locally:** unknown — verify `rustup`, `cargo-lambda`, Docker,
  AWS CLI at the start of the first hands-on session.
- **AWS account/region:** not yet confirmed. Note: Lambda MicroVMs is only available
  in us-east-1, us-east-2, us-west-2, eu-west-1, ap-northeast-1 (as of launch).

## Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-05 | Adopt AI-DLC three-phase workflow (Inception/Construction/Operations) with Kiro-style specs (`requirements.md`/`design.md`/`tasks.md` + steering files) | Learner explicitly requested borrowing AI-DLC logic and spec-driven development without the Kiro IDE |
| 2026-07-05 | ARM64/Graviton as default target architecture | Cheaper, faster on Lambda; Lambda MicroVMs is ARM64-only |
| 2026-07-05 | Curriculum ordered: Rust fundamentals → Lambda → MicroVMs → Fargate → EC2 | Lambda has the shortest feedback loop for a beginner; EC2 requires the most ops knowledge |

## Concepts mastered

_(Move items here from `SKILLS.md` as they are demonstrated, with the date and the
evidence — e.g., "ownership: explained borrow-checker error in 001 task 3 unaided.")_

- _none yet_

## Open questions for the learner

- Which AWS region should be the default? (Must be a MicroVMs region if we want all
  four targets in one region — suggest `us-east-1` or `ap-northeast-1`.)
- Infrastructure-as-code preference: AWS SAM, CDK (TypeScript or Rust via cdk8s?), or
  plain CLI first and IaC later? (Recommendation: plain `cargo lambda deploy` + CLI
  first, introduce IaC in Phase 2.)
- Local dev environment: does the learner have Docker and an AWS account with
  credentials configured?

## Session log

### 2026-07-05 — Session 1: Foundation setup
- Created repo scaffolding on branch `claude/rust-aws-learning-44il52`: `CLAUDE.md`,
  `MEMORY.md`, `SKILLS.md`, `steering/` (product/tech/structure), `specs/_template/`,
  and first draft spec `specs/001-hello-rust-lambda/`.
- Researched and encoded: AI-DLC methodology (AWS, re:Invent 2025, open-sourced
  adaptive workflows), Kiro spec-driven development (EARS notation, approval gates,
  steering files), Lambda MicroVMs launch details (June 22, 2026), current Rust-on-AWS
  tooling (`cargo-lambda`, Rust GA on Lambda since Nov 2025, Lambda Managed Instances
  with Rust support since Mar 2026).
- No code written yet — next session should start with toolchain verification and
  Level 0/1 of `SKILLS.md`.
