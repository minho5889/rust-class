# CLAUDE.md — Project Constitution (v2)

Project codename: **goldeneye**. This repository is a structured learning environment
for **Rust on AWS**. The learner (Minho) is new to Rust and wants to master it as a
systems-engineering language — leaning on Rust's memory-management model (ownership,
borrowing, zero-cost abstractions) — and deploy it across four AWS compute targets:

1. **AWS Lambda** (serverless functions, `cargo-lambda`, ARM64/Graviton)
2. **AWS Lambda MicroVMs** (Firecracker-based isolated stateful sandboxes, June 2026)
3. **Amazon ECS on Fargate** (containerized Rust services)
4. **Amazon EC2** (Rust binaries as long-running systemd services)

Regions: **us-east-1** (primary), **ap-northeast-1** (secondary). All AWS resources
are prefixed `goldeneye-`.

## Session ritual

Read this file first in every session, then:

1. **Read `MEMORY.md`** — learner profile, progress state, decision log.
2. **Read `SKILLS.md`** — curriculum/skill tree; find the current level.
3. **Read `steering/`** — `product.md` (why), `tech.md` (stack), `structure.md` (layout).
4. At the **end of every working session, update `MEMORY.md`**. Not optional.

## The four-doc spec pipeline (AI-DLC + Kiro-style, v2)

Every meaningful piece of work is a **Unit of Work** in `specs/NNN-short-name/`
(copy `specs/_template/`). Each document freezes one kind of decision so that
disagreements are caught at the cheapest layer: *meaning → behavior → shape → work*.
Each doc derives **only** from the doc above it.

```
raw prompt ─► intent.md ─► requirements.md ─► design.md ─► tasks.md ─► bolts
                 │              ✋ human          ✋ human       ✋ human
                 └─► intent-assurance bot (audits, never edits)
                                                          evidence.md (append-only, no gate)
```

| Doc | Freezes | Gate |
|---|---|---|
| `intent.md` | What the learner meant (verbatim prompt, distilled intent, assumptions, rejected readings) | None — audited by the `intent-assurance` subagent, which logs to `_assurance/` and **never edits** the doc |
| `requirements.md` | What must be true — EARS lines, each tagged **[P]** property-testable / **[E]** example-testable / **[O]** operationally-verified, plus learning requirements | ✋ Human approves (spec-engineer notify/revise loop) |
| `design.md` | How it works — high-level narrative + detailed design; every element cites REQ IDs; formal **Properties** section for each [P] requirement | ✋ Human approves |
| `tasks.md` | The work — main task (deliverable) → sub task (one bolt) → action item (one commit); layers may collapse for small units | ✋ Human approves shape once; bolts then execute freely |
| `evidence.md` | What actually happened — deploys, metrics, counterexamples, learnings | None — append-only |

**Status lifecycle** (in every gated doc's header, exact format matters — hooks grep it):
`**Status:** drafting | awaiting-review | revising | approved | superseded`

### Pipeline rules

- **Write-once intent.** `intent.md` is never edited after creation. Small
  clarifications go in its append-only *Addendum* section; a change of meaning
  supersedes the whole spec (new spec, note at the top of the old one).
- **Assurance before attention.** Before any doc is set to `awaiting-review`, run the
  matching fresh-context subagent audit (`.claude/agents/`): `intent-assurance` for
  intent, `spec-auditor` for requirements/design, `property-auditor` before a spec
  closes. Verdicts go to `specs/NNN-*/_assurance/`. The human only reviews
  pre-audited docs; embed the one-line verdict in the review notification.
- **Change protocol.** When downstream work contradicts an approved doc: halt that
  item, amend the upstream doc with a changelog entry, re-gate **only the amendment**.
  Never let code silently diverge from spec.
- **Fast path.** Small, well-understood units may draft all four docs in one shot and
  take a single combined approval. Ceremony scales with risk. Trivial playground
  experiments need no spec at all.
- **Untestable = unapprovable.** Every EARS requirement must carry a [P]/[E]/[O] tag.
  If none fits, the requirement is rewritten before review.
- **AI proposes, human disposes.** Claude drafts everything; architecture, AWS
  service, and cost decisions go to the learner with a clear recommendation.

### Property-based testing (Kiro-style correctness)

EARS requirements are universal statements — treat them as executable properties:

- Each **[P]** requirement gets a formal property in `design.md` (inputs,
  preconditions, invariant, generation strategy) and a `proptest` test written
  **before** the code it tests (first action items of the sub task).
- Default ≥256 cases; commit `proptest-regressions/` — failing seeds are permanent
  regression tests.
- **Counterexample triage** (logged in `_assurance/`): a failing property is a
  **spec bug** (amend requirements, re-gate), a **code bug** (fix, keep the seed), or
  a **test bug** (fix the generation strategy). Classify before fixing.

## Telemetry (the goldeneye data lake, v1 capture)

Hooks in `.claude/settings.json` append envelope events (schema:
`datalake/schema/envelope.v1.json`) to `datalake/raw-local/dt=YYYY-MM-DD/*.jsonl`.
Spec-doc writes and gate notifications are captured automatically. Rules:

- **Never put secrets/credentials in event payloads.**
- Telemetry files are committed with normal work commits (this is a private repo;
  raw prompts in intent events are allowed by the learner's decision, 2026-07-05).
- S3 zones (`goldeneye-lake` raw/curated, `goldeneye-discovery` insights) are
  Wave 2 — do not create AWS resources for telemetry until that spec is approved.

## Teaching rules (this is a class, not just a codebase)

- **Explain while building.** Concepts not yet mastered (per `SKILLS.md`) get a short
  explanation in conversation and, where durable, as `///` doc comments.
- **Memory management is the through-line.** Ownership, borrowing, lifetimes,
  `Box`/`Rc`/`Arc`, allocation — call them out and relate them to AWS efficiency
  (cold starts, memory footprint, cost).
- **Properties are teaching tools.** Writing an invariant and reading a shrunk
  counterexample are both lessons; record notable counterexamples in `evidence.md`.
- Prefer idiomatic Rust. `cargo fmt` and `cargo clippy -- -D warnings` must pass
  before any commit. Compare with GC languages when introducing concepts.

## Rust/AWS conventions

- Cargo **workspace** at repo root; one crate per deployable, shared code in
  `crates/shared`. Target **ARM64** (Graviton) everywhere.
- Lambda: `cargo lambda build --release --arm64`; runtime `provided.al2023`.
- Fargate: multi-stage Dockerfile (`rust:slim` → distroless/`scratch`).
- EC2: release binary + systemd unit.
- Errors: `thiserror` (libs) / `anyhow` (bins). Async: `tokio`. Tests: built-in +
  `proptest`. Logging: `tracing`. AWS: official `aws-sdk-*` crates.

## File map

| Path | Purpose |
|---|---|
| `CLAUDE.md` | This constitution |
| `MEMORY.md` | Living memory: profile, progress, decisions, session log |
| `SKILLS.md` | Skill tree / curriculum with mastery tracking |
| `steering/*.md` | Kiro-style steering: product, tech, structure |
| `specs/` | Units of Work (five docs + `_assurance/` sidecar each) |
| `datalake/` | Telemetry: schema registry + local raw zone (v1) |
| `.claude/agents/` | Assurance subagents (intent-assurance, spec-auditor, property-auditor) |
| `.claude/hooks/` | Telemetry capture + gate notification scripts |
| `playground/` | Throwaway experiments, no spec required |
| `crates/` | Real workspace crates (created as the class progresses) |
