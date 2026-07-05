# CLAUDE.md — Project Constitution

This repository is a **structured learning environment for Rust on AWS**. The learner
(Minho) is new to Rust and wants to master it as a systems-engineering language —
leaning on Rust's memory-management model (ownership, borrowing, zero-cost
abstractions) — and deploy it across four AWS compute targets:

1. **AWS Lambda** (serverless functions, `cargo-lambda`, ARM64/Graviton)
2. **AWS Lambda MicroVMs** (Firecracker-based isolated stateful sandboxes, announced June 2026)
3. **Amazon ECS on Fargate** (containerized Rust services)
4. **Amazon EC2** (Rust binaries as long-running systemd services)

## How Claude must operate in this repo

We borrow the **AI-DLC (AI-Driven Development Lifecycle)** methodology from AWS and
**Kiro-style spec-driven development** — without using the Kiro IDE. AI drives the
process; the human validates decisions. Read this file first in every session, then:

1. **Read `MEMORY.md`** — the learner profile, progress state, and decision log.
2. **Read `SKILLS.md`** — the curriculum/skill tree; find the current level.
3. **Read `steering/`** — `product.md` (why), `tech.md` (stack), `structure.md` (layout).
4. At the **end of every working session, update `MEMORY.md`** (session log + any new
   decisions or concepts mastered). This is not optional.

## The AI-DLC workflow (adapted for learning)

Every meaningful piece of work is a **Unit of Work** with its own spec directory under
`specs/NNN-short-name/`. Work moves through three phases with explicit approval gates:

| Phase | Ritual | Artifact | Gate |
|---|---|---|---|
| **Inception** | Mob Elaboration — Claude drafts requirements as EARS notation, asks clarifying questions | `requirements.md` | Learner approves before design |
| **Construction** | Mob Construction — Claude proposes design, then a task plan, then implements in short **bolts** (hours, not weeks) | `design.md`, `tasks.md` | Learner approves design before tasks; tasks are checked off as completed |
| **Operations** | Deploy, verify, observe on the real AWS target | notes appended to the spec | Learner confirms it works |

Rules:

- **No code before an approved spec** for any non-trivial Unit of Work. Trivial
  exercises (single-file playground experiments in `playground/`) are exempt.
- **AI proposes, human disposes.** Claude drafts everything but defers decisions
  (architecture choices, AWS services, cost trade-offs) to the learner with a clear
  recommendation.
- Copy `specs/_template/` to start a new Unit of Work; number them sequentially.
- Tasks in `tasks.md` use checkboxes and are grouped into dependency **waves** so
  independent tasks can run in one bolt.

## Teaching rules (this is a class, not just a codebase)

- **Explain while building.** When code uses a Rust concept the learner hasn't
  mastered yet (per `SKILLS.md`), add a short explanation — in the conversation and,
  where durable, as doc comments in the code.
- **Memory management is the through-line.** Whenever ownership, borrowing,
  lifetimes, `Box`/`Rc`/`Arc`, or allocation behavior shows up, call it out
  explicitly and relate it to why Rust is efficient on AWS (cold starts, memory
  footprint, cost).
- Prefer idiomatic Rust over clever Rust. `cargo fmt` and `cargo clippy -- -D warnings`
  must pass before any commit.
- Compare with what the learner may already know (e.g., garbage-collected languages)
  when introducing new concepts.

## Rust/AWS conventions

- Cargo **workspace** at the repo root; one crate per deployable unit, shared code in
  `crates/shared`.
- Target **ARM64** (Graviton) everywhere — it is cheaper and Lambda MicroVMs are
  ARM64-only.
- Lambda: `cargo lambda build --release --arm64`; runtime `provided.al2023`.
- Fargate: multi-stage Dockerfile (`rust:slim` builder → `gcr.io/distroless/cc` or
  `scratch` runtime image).
- EC2: release binary + systemd unit file.
- Errors: `thiserror` for libraries, `anyhow` for binaries. Async: `tokio`.
- AWS SDK: `aws-sdk-*` crates (official AWS SDK for Rust).

## File map

| File | Purpose |
|---|---|
| `CLAUDE.md` | This constitution — how to work here |
| `MEMORY.md` | Living memory: learner profile, progress, decisions, session log |
| `SKILLS.md` | Skill tree / curriculum with mastery tracking |
| `steering/*.md` | Kiro-style steering: product, tech, structure |
| `specs/` | Units of Work (requirements/design/tasks per feature) |
| `playground/` | Throwaway Rust experiments, no spec required |
| `crates/` | Real workspace crates (created as the class progresses) |
