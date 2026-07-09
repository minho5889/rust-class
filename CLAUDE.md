# CLAUDE.md — Project Constitution (v3)

Project codename: **goldeneye**. This repository is a structured learning environment
for **Rust on AWS**. The learner (Minho) is new to Rust and wants to master it as a
systems-engineering language — leaning on Rust's memory-management model (ownership,
borrowing, zero-cost abstractions) — and deploy it across four AWS compute targets:

1. **AWS Lambda** (serverless functions, `cargo-lambda`, ARM64/Graviton)
2. **AWS Lambda MicroVMs** (Firecracker-based isolated stateful sandboxes, June 2026)
3. **Amazon ECS on Fargate** (containerized Rust services)
4. **Amazon EC2** (Rust binaries as long-running systemd services)

Regions: **us-east-1** (primary), **ap-northeast-1** (secondary; feature parity
unverified — check before assuming). AWS resources carry the `project=goldeneye`
tag; physical names are CDK-generated except the two lake buckets (see IaC).

**Goals:** the learner can independently write, test, and deploy idiomatic Rust to
all four targets *and explain why it's efficient on each* (ownership, no GC, small
binaries); every deployed artifact has a spec trail. **Non-goals:** production
traffic, exhaustive AWS coverage, WASM/frontend Rust.

**Doc discipline:** this file is auto-loaded every session — it carries only rules
that matter in *most* sessions. Niche detail lives in `research/` reports or the
relevant spec and is linked, not inlined. One source of truth per fact.

## Session ritual

Read this file first in every session, then:

1. **Read `MEMORY.md`** — learner profile, progress state, decision log.
2. **Read `SKILLS.md`** — curriculum/skill tree; find the current level.
3. At the **end of every working session, update `MEMORY.md`**. Not optional.

## The four-doc spec pipeline (AI-DLC + Kiro-style)

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
| `intent.md` | What the learner meant (verbatim prompt, distilled intent, assumptions, rejected readings) | None — audited by `intent-assurance`, which logs to `_assurance/` and **never edits** |
| `requirements.md` | What must be true — EARS lines tagged **[P]** property / **[E]** example / **[O]** operational, plus learning requirements | ✋ Human (spec-engineer notify/revise loop) |
| `design.md` | How it works — high-level + detailed; every element cites REQ IDs; **Properties** section per [P] requirement | ✋ Human |
| `tasks.md` | The work — main task (deliverable) → sub task (one bolt) → action item (one commit); layers may collapse | ✋ Human approves shape once; bolts then execute freely |
| `evidence.md` | What actually happened — deploys, metrics, counterexamples, learnings | None — append-only |

**Status lifecycle** (gated docs' headers; exact format — hooks grep it):
`**Status:** drafting | awaiting-review | revising | approved | superseded`

### Pipeline rules

- **Write-once intent.** Never edited after creation; clarifications in the
  append-only *Addendum*; a change of meaning supersedes the spec.
- **Assurance before attention.** Before any doc goes `awaiting-review`, run the
  matching fresh-context audit (`.claude/agents/`): `intent-assurance`,
  `spec-auditor` (requirements/design/tasks), `property-auditor` before close.
  Verdicts to `specs/NNN-*/_assurance/`; embed the one-line verdict in the review
  notification.
- **Change protocol.** Downstream work contradicting an approved doc: halt that
  item, amend upstream with a changelog entry, re-gate **only the amendment**.
- **Fast path.** Small well-understood units may draft all four docs in one shot,
  one combined approval. Trivial playground experiments need no spec.
- **Untestable = unapprovable.** Every EARS line carries [P]/[E]/[O] or is rewritten.
- **Human-readable first.** A gated doc must open with plain English a non-Rust
  reader can approve from in a minute ("In plain words" / "What it does" / "What
  we're not building" / "What you'll learn"). The precise tagged/testable criteria
  go in a clearly-marked section *below* (reviewers skim; the design step and
  auditors use it); audit trail and changelog go in a collapsed block at the end.
  The gate is only real if the reviewer can actually read the doc — jargon walls
  break it. Applies to all four docs; `_template/` shows the shape.
- **AI proposes, human disposes.** Architecture, AWS service, and cost decisions go
  to the learner with a clear recommendation.

### Property-based testing (Kiro-style correctness)

- Each **[P]** requirement gets a formal property in `design.md` (inputs,
  preconditions, invariant, generation strategy) and a `proptest` test written
  **before** the code it tests. Default ≥256 cases; commit `proptest-regressions/`.
- **Counterexample triage** (logged in `_assurance/triage-log.md`): spec bug
  (amend requirements, re-gate) / code bug (fix, keep seed) / test bug (fix
  strategy). Classify before fixing.

## Telemetry (the goldeneye data lake = gradebook + lab notebook)

Three workloads, priority order: **memlens traces** (the volume; Parquet/DuckDB
curriculum), **learning analytics** (`learning.*` mistake ledger → SKILLS.md
evidence), **process telemetry** (plain JSONL in git forever, no curation).
Hooks append envelope events (`datalake/schema/envelope.v1.json`) to
`datalake/raw-local/dt=YYYY-MM-DD/*.jsonl`. Rules that bind every session:

- **Never put secrets in payloads.** Events commit with normal work commits.
- **Research runs must emit `research.*` events** (protocol:
  `research/README.md` v2) — findings invisible to the lake don't exist.
- **Log the mistake ledger while teaching**: recurring compiler-error classes,
  borrow-checker fights, counterexample triages → `learning.*` events.
- Standing queries (`datalake/queries/scan.sh`) each name the doc they feed;
  insight cards → `datalake/insights-local/`. S3 zones are a Wave-2 class unit.
  Details: `datalake/README.md`.

## Learning mode — how Phase 1 runs (coached, learner writes)

Two kinds of spec, and they run differently:

- **Infrastructure specs** (e.g. 002 memlens): Claude builds, learner reads
  heavily-commented code. Fast; the deliverable is a tool.
- **Learning specs** (Phase 1 fundamentals, 003–008): **the learner writes the
  Rust; Claude coaches.** The deliverable is the learner's understanding; the
  code is the by-product. This is the default for Phase 1.

**Materials-ahead model (learner directive 2026-07-09):** Claude authors the
entire course in advance — a worksheet per ramp step (`playground/ramp/step-NN/`)
and per glake sitting (`specs/003-rust-bedrock/sittings/`), each with goals,
prompts, a hint ladder, expected compiler errors, and checkpoint commands.
Solutions are pre-written and **validated** (they compile and pass) but live
clearly separated (`solution.rs` / `_reference/`) under a don't-peek-until-tried
convention. The learner follows the trail self-paced; Claude reviews submissions
live. Worksheets supersede chat-posed exercises.

Coached-mode rules:
- **One concept per sitting.** The smallest step that compiles and teaches.
- **Ramp before tool.** Throwaway `playground/` exercises (no spec needed)
  isolate a single concept before it appears in a real tool. Then build the tool
  **dead-simple first** and add one capability per bolt.
- **Claude poses, then stops.** Pose the exercise, let the learner write, then
  review: name what's right, and when it's wrong explain *why the compiler
  objects*, don't just paste the fix. Only hand over the answer after they've
  tried and are stuck.
- **Co-write the hard bits.** Property tests, `unsafe`, tricky lifetimes are
  teaching moments written together, not solo learner homework.
- **Log the fights.** Every borrow-checker battle / recurring compiler-error
  class → `learning.*` events (the mistake ledger feeds SKILLS mastery).

## Teaching rules (this is a class, not just a codebase)

- **Explain while building.** Unmastered concepts (per `SKILLS.md`) get a short
  explanation in conversation and, where durable, `///` doc comments.
- **Memory management is the through-line.** Relate ownership/borrowing/allocation
  to AWS efficiency (cold starts, footprint, cost). Compare with GC languages.
- **Properties are teaching tools.** Record notable counterexamples in `evidence.md`.

## Rust conventions

- **Stable Rust**, edition 2024; no nightly (exception: a future cargo-fuzz lane).
- `cargo fmt` + `cargo clippy -- -D warnings` pass before any commit.
- Errors: `thiserror` (libs) / `anyhow` (bins). Async: `tokio`. Logging: `tracing`.
  Serialization: `serde`. HTTP: `axum`. AWS: official `aws-sdk-*` + `aws-config`.
- **Unsafe policy:** `#![forbid(unsafe_code)]` on every crate **except** `memlens`
  (the unsafe classroom): there, `#![deny(unsafe_op_in_unsafe_fn)]` + a
  `// SAFETY:` comment on every unsafe block. Miri for its core: later.
- **Shared release profile** (workspace root): `lto = "thin"`,
  `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` (tests unaffected).
- **No `unwrap()`/`expect()` in handler/service paths** — `clippy::unwrap_used`
  on deployable crates (the Cloudflare Nov-2025 outage lesson).
- **API rubric:** public APIs reviewed in design.md against the rust-lang API
  Guidelines checklist (C-COMMON-TRAITS, C-GOOD-ERR, C-NEWTYPE, C-BUILDER).
- Testing: built-in `#[test]` for [E]; `proptest` for [P] (explicit strategies,
  `prop_assert!`, property tests named after REQ IDs, e.g. `prop_r1_…`).

## AWS conventions

**ARM64 (Graviton) everywhere.** `aarch64-unknown-linux-gnu` is Rust Tier 1.

| Target | Build/deploy | Notes |
|---|---|---|
| Lambda | `cargo lambda build --release --arm64`; `lambda_runtime`/`lambda_http`; runtime `provided.al2023` | Size thread pools from memory-derived vCPUs (1,769 MB ≈ 1 vCPU), never `available_parallelism()`; `aws-sdk-*` adds ~10 MB+ — check `cargo-bloat` |
| Lambda MicroVMs | `aws-sdk` lifecycle calls | ARM64-only; **launch specs unverified — that spec starts with primary-source verification** |
| ECS Fargate | multi-stage Docker: `rust:slim` → `distroless-static`/`chainguard-static` (not bare `scratch`); musl static (verify `ldd`) | **Never build ARM64 images under QEMU** — `cargo-zigbuild` + `--platform=$BUILDPLATFORM`, or native ARM runners |
| EC2 | release binary + systemd unit, user-data bootstrap | Graviton; `-Ctarget-cpu=neoverse-n1` when Graviton-only |

- **Graviton checklist:** any crate doing crypto/hash/SIMD must have its ARM64
  hardware backend verified active (the sha2 4–5× lesson).
- **IaC: AWS CDK v2, TypeScript**, app in `infra/`. `cargo-lambda-cdk`
  `RustFunction` with explicit `Architecture.ARM_64`; `DockerImageAsset` with
  explicit `Platform.LINUX_ARM64` (both default x86 — silent runtime failures).
  Stateful stack (lake buckets, termination-protected) split from stateless.
  `cdk-nag` v3 (AwsSolutions + Serverless). Tags over physical names; hardcoded
  names only for `goldeneye-lake`/`goldeneye-discovery`.
  Details: `research/typescript-cdk-for-goldeneye.md`.
- Keep costs minimal; tear down after exercises. No secrets in the repo or
  telemetry; Telegram tokens via env (`TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`).

## Repository structure & git

```
rust-class/ (goldeneye)
├── CLAUDE.md              # this constitution (auto-loaded; single source of truth)
├── MEMORY.md              # living memory: profile, progress, decisions, session log
├── SKILLS.md              # skill tree / curriculum with mastery tracking
├── README.md              # human-facing overview
├── specs/                 # Units of Work: _template/ + NNN-short-name/ (+ _assurance/)
├── research/              # verified deep-research reports (<topic>.md, no dates)
├── datalake/              # telemetry: schema/ + raw-local/ (+ future S3 zones)
├── .claude/               # agents/ (assurance bots), hooks/ (telemetry, gate notify)
├── infra/                 # TypeScript CDK v2 app (created with first AWS spec)
├── playground/            # throwaway experiments — no spec required
└── crates/                # cargo workspace: shared/ + one crate per deployable
```

- **Specs**: numbered from 002 (001 retired pre-pipeline); never deleted —
  superseded specs point to their successor.
- **Branches/merges** (reconciled 2026-07-09 to this remote's real limits — it
  accepts branch creates/updates only: **no tag pushes, no branch deletes**):
  - `claude/rust-aws-learning-44il52` — the designated **active dev branch**;
    all ongoing work commits here (the harness/PR tracks it). Our trunk.
  - `main` — mirror of the trunk, fast-forwarded at each spec close. Pushed.
  - **Spec boundaries** are marked with `spec-close/NNN-name` **branches** (not
    tags — tags can't push here). `git diff spec-close/002-… spec-close/003-…`
    shows exactly what a spec added.
  - Retired: per-spec dev branches (redundant with the designated branch for a
    solo repo) and tag markers (unpushable). A stray `claude/003-rust-bedrock`
    lingers on the remote from a permissions probe — deletes are refused, so
    ignore it.
- **Commits**: imperative, spec/step-referenced (`003: implement handler`,
  `ramp: step 2 review`). One commit per learner step in coached mode.
  (Terminology: ramp exercises are called **steps** — never "rungs", learner
  preference 2026-07-09.)
- Crate and spec names: kebab-case. Playground stays outside the workspace.
