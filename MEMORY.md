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
- **Phase:** Wave 1 complete; spec 002 in flight through pipeline v2.
- **Active Unit of Work:** `002-memory-lens` — tracking allocator + trace replay +
  single-file memory dashboard. Intent audited 90%; requirements approved
  (gate 1, 2026-07-05); design approved (gate 2, same day, after 2 MAJOR audit
  fixes); tasks.md rev 2 **awaiting gate 3**. Construction starts on approval.
- The former "datalake-v1 as 002" plan shifts to a later number; memory-lens
  took 002 (learner's dashboard idea, scoped to memory-only per correction).
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
| 2026-07-05 | Turn-end telemetry (`Stop` hook) removed; capture = session.start + spec/gate events only | Fired every turn → noise events + a telemetry-only commit per reply; turn boundaries derivable from timestamps |
| 2026-07-05 | Git strategy from 002's close: merge to `main` + tag `spec/NNN-name` at spec close; branch-per-spec (`claude/NNN-*`) thereafter | `main` = sum of approved work; tags = spec-boundary markers for the lake; learner asked for worthwhile fixes to be implemented |
| 2026-07-05 | **IaC = AWS CDK v2 with TypeScript** (over SAM/Terraform/Pulumi); CDK app in `infra/` beside the cargo workspace; introduced when the first AWS-deploying spec constructs | Learner decision; conventions to be refined from CDK deep-research report |
| 2026-07-05 | Research reports live in `research/` as `<topic>.md` (no dates in filenames — date in header, versions via git) | Learner requested a research folder with three reports (best practices / Rust-on-AWS / CDK); learner amended: no date in the name |
| 2026-07-05 | **Lake reframed: gradebook + lab notebook.** Traces = primary citizen (Parquet/DuckDB where volume is real); learning analytics (mistake ledger) = novel value; process telemetry = plain JSONL forever, curation pipeline killed. Queries must name their consumer doc | Old design borrowed big-data architecture for KB-scale data; insight cards had no reader. Learner approved reframing |
| 2026-07-05 | **Research protocol v2** (research/README.md): unverified-claim backlog, mid-run completeness critic, lens-diverse verifiers, mandatory volatility tags + review_by, findings emitted as `research.*` lake events, adoption delta stage | First 3 runs silently dropped 62–77% of extracted claims and found the MicroVMs gap post-hoc (evidence: insights-local/002) |
| 2026-07-05 | **`steering/` deleted; consolidated into CLAUDE.md v3** (product/tech/structure merged, conventions + file maps deduplicated). Doc set = CLAUDE.md + MEMORY.md + SKILLS.md + research/ | Steering was a Kiro mechanism reimplemented atop Claude Code's native auto-loaded CLAUDE.md; caused real drift (same facts maintained in two places twice in one day). Learner: "yes do it" |

## Concepts mastered

_(Move items here from `SKILLS.md` when demonstrated, with date and evidence.)_

- _none yet_

## Open questions for the learner

- AWS credentials: are they configured in this environment? (Blocks Wave 2.)
- Telegram: provide `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` as env vars when you
  want gate notifications live (hook already falls back gracefully without them).

## Session log

### 2026-07-05 — Session 5: gate 3 approved; construction begins (bolt 1.1 ✅)
- Learner approved tasks.md (gate 3) — all three 002 gates now passed.
- **First Rust in the repo.** Bolt 1.1 complete, three commits: (1.1.1)
  workspace + shared release profile + `MemLens<A>` passthrough allocator with
  per-block SAFETY contracts + feature-off integration test; (1.1.2) R14b
  compile guard proven by a cargo-invoking test (deviation from trybuild noted —
  trybuild can't vary profiles); (1.1.3) `memlens.v1.json` schema + serde
  payload structs with round-trip tests. fmt/clippy/tests green in both feature
  configurations. Toolchain: rustc 1.94.1, edition 2024.
- Concepts introduced (not yet learner-verified): `GlobalAlloc` + unsafe
  contracts, const fn constructors for allocator statics, integration-test
  binaries as the only place to install `#[global_allocator]`.
- **Next: bolt 1.2** — the tracking allocator proper, R1 property test FIRST
  (red), then writer (seq under lock) + reentrancy guard to make it green.

### 2026-07-05 — Session 4: research triad + conventions hardening
- Three deep-research workflows (≈320 agents, 3-vote adversarial verification):
  `research/typescript-cdk-for-goldeneye.md` (25/25 confirmed),
  `rust-best-practices-and-big-tech.md` (24/25, 1 refuted),
  `rust-on-aws-compute.md` (24/25, 1 refuted).
- Adopted into steering/CLAUDE: CDK construct choices + 2 ARM64 explicit-flag
  traps; tags-over-physical-names; unsafe policy (forbid everywhere, memlens =
  unsafe classroom); shared release profile (thin LTO/cu=1/panic=abort/strip);
  clippy::unwrap_used; distroless-static over scratch; no-QEMU cross-compile;
  Lambda thread-pool sizing rule; Graviton arch-backend checklist.
- Honest gaps flagged: Lambda MicroVMs / Managed Instances / SnapStart claims
  did not survive verification; ap-northeast-1 parity unverified. Refuted en
  route: "ARM64 blocks cargo-fuzz" and AWS's "125 ms because Firecracker is
  Rust" attribution.
- 002 tasks.md (still awaiting gate 3) updated pre-gate with the new
  conventions (1.1.1 release profile + lint gates, 1.2.3 SAFETY comments).
- IaC decision recorded: TypeScript CDK v2, app in `infra/`.

### 2026-07-05 — Session 3: spec 002-memory-lens through the pipeline
- Learner clarified the dashboard idea: **solely** for understanding Rust memory at
  runtime (not workflow observability). IDE cut, voice + non-voice helpers deferred.
- Full pipeline run: intent.md (write-once, verbatim prompts) → intent-assurance
  blind audit **90%** → requirements draft → spec-auditor **78%** (compound EARS
  lines, mis-tagged R10 → became flagship [P] replay property) → rev 2 → **gate 1
  approved** → design draft → spec-auditor **74%** (2 MAJORs: realloc payload
  lacked old→new lineage; lock-free seq outside writer lock broke ordering) →
  rev 2 → **gate 2 approved** → tasks draft → auditor **86%** → rev 2 →
  gate 3 pending.
- Assurance-before-attention demonstrably worked: 2 contract-level design bugs and
  a property-tagging error caught by subagents before human review.
- gate.approved events recorded in the lake; telemetry hooks live all session.

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
