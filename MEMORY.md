# MEMORY.md — Living Memory

> Claude: read this at the start of every session; append to it at the end of every
> session. Keep entries short and factual. Newest session-log entries go on top.

## Learner profile (updated 2026-07-05 from LinkedIn)

- **Name / contact:** Minho Lee (minho5889@gmail.com) — Toronto, ON.
- **Day job:** Associate Delivery Consultant AI/ML, AWS Professional Services
  (Dec 2025–): architecting/delivering **GenAI and agentic AI solutions on
  AWS**. Before that: **Cloud Support Engineer, 3 yrs 4 mos — troubleshooting
  and optimizing Serverless applications on AWS**. Western University. Earlier
  career: 2,000+ hrs EN⇄KR medical interpretation (bilingual).
- **Consequence for the class:** AWS-expert / Rust-beginner. Skip all AWS-101;
  lean into Rust language depth; his serverless *operations* instincts
  (cold starts, throttling, observability) are assets to build on, not teach.
  GenAI/agentic day job → MicroVMs (AI-code sandboxes) and Bedrock-from-Rust
  are high-motivation later units. Time-constrained working professional.
- **Rust experience:** Beginner — no substantial prior Rust knowledge.
- **Goal:** Rust as a systems-engineering language, memory management as the
  through-line, deployed across Lambda, Lambda MicroVMs, ECS Fargate, EC2.
- **Methodology:** AI-DLC + Kiro-style four-doc pipeline (see CLAUDE.md).
- **Learning style:** Fundamentals first ("the bedrock"), then hands-on
  building; likes process/telemetry thinking (designed the data lake himself).

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
| 2026-07-05 | **002 reclassified as internal tooling** (intent Addendum): learner sessions are not close gates for infrastructure units; automated browser verification substitutes for visual [O] checks; pedagogy defers to the exercises that use the tool | Learner directive mid-construction |
| 2026-07-05 | **IaC = AWS CDK v2 with TypeScript** (over SAM/Terraform/Pulumi); CDK app in `infra/` beside the cargo workspace; introduced when the first AWS-deploying spec constructs | Learner decision; conventions to be refined from CDK deep-research report |
| 2026-07-05 | Research reports live in `research/` as `<topic>.md` (no dates in filenames — date in header, versions via git) | Learner requested a research folder with three reports (best practices / Rust-on-AWS / CDK); learner amended: no date in the name |
| 2026-07-05 | **Lake reframed: gradebook + lab notebook.** Traces = primary citizen (Parquet/DuckDB where volume is real); learning analytics (mistake ledger) = novel value; process telemetry = plain JSONL forever, curation pipeline killed. Queries must name their consumer doc | Old design borrowed big-data architecture for KB-scale data; insight cards had no reader. Learner approved reframing |
| 2026-07-05 | **Research protocol v2** (research/README.md): unverified-claim backlog, mid-run completeness critic, lens-diverse verifiers, mandatory volatility tags + review_by, findings emitted as `research.*` lake events, adoption delta stage | First 3 runs silently dropped 62–77% of extracted claims and found the MicroVMs gap post-hoc (evidence: insights-local/002) |
| 2026-07-05 | **Phase 1 = coached mode (learner writes, Claude coaches)**, one concept per sitting, ramp before tool. Learner is a complete beginner and wants to learn by writing, not reading. Infrastructure specs (002) stay Claude-builds. `playground/ramp/` (no spec) precedes glake; glake built dead-simple first, one capability per bolt. See CLAUDE.md "Learning mode" | Learner: "is this a good spec for me to learn rust? I am a complete beginner" → chose "I write it, you coach" + "add a gentle ramp first" |
| 2026-07-05 | **Spec docs must be human-readable first** — plain-English opener ("In plain words / What it does / What we're not building / What you'll learn") before the precise tagged criteria; audit trail collapsed at end. Baked into CLAUDE.md + all four templates; 003 requirements retrofitted. Closed specs (002) left as-is | Learner found 002/003 requirements hard to read — the human gate fails if the reviewer can't read the doc |
| 2026-07-05 | **`steering/` deleted; consolidated into CLAUDE.md v3** (product/tech/structure merged, conventions + file maps deduplicated). Doc set = CLAUDE.md + MEMORY.md + SKILLS.md + research/ | Steering was a Kiro mechanism reimplemented atop Claude Code's native auto-loaded CLAUDE.md; caused real drift (same facts maintained in two places twice in one day). Learner: "yes do it" |

## Concepts mastered

_(Move items here from `SKILLS.md` when demonstrated, with date and evidence.)_

- _none yet_

## Open questions for the learner

- AWS credentials: are they configured in this environment? (Blocks Wave 2.)
- Telegram: provide `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` as env vars when you
  want gate notifications live (hook already falls back gracefully without them).

## Phase 1 plan (approved direction 2026-07-05; each spec still gates individually)

Learner constraints: 10+ h/week · personal AWS account ready · build-first +
hands-on exercises (reading optional) · Phase 2 = agentic/GenAI tilt.

| # | Spec | Builds | Rust concepts |
|---|---|---|---|
| 003 | `rust-bedrock` | `glake` CLI v0 (validate/stats the local lake, std-only) | ownership, borrowing, String/&str, enums, match, Option/Result, iterators |
| 004 | `glake-traits` | glake → lib+bin with query traits, thiserror chain, proptest suite | traits, generics, modules, error design, API-guidelines rubric |
| 005 | `async-relay` | local axum service receiving envelope events (hooks can POST) | async/await, tokio, channels, Send/Sync, graceful shutdown |
| 006 | `hello-lambda` | relay handler as ARM64 Lambda + Function URL; `infra/` CDK born; panic=abort cold-start experiment | lambda_runtime, serde events, release profile in anger, cargo-bloat |
| 007 | `lake-to-s3` | CDK stateful stack (lake buckets) + Rust ingest Lambda + backlog sync + DuckDB-over-S3 scan pack | aws-sdk-s3, batching, buffer reuse, streaming serde |
| 008 | `lambda-memlab` | capstone: memlens-instrumented workloads ON Lambda across memory configs; traces → lake; insight cards → discovery | allocator behavior under vCPU scaling; original published-in-repo research |

Phase 2 (agentic tilt, sketched): 009 MicroVMs AI-code sandbox · 010
Bedrock-from-Rust agent core · 011 Fargate/EC2 + cost capstone.

## Session log

### 2026-07-09 — Session 6 (cont.): the whole course authored ahead
- Learner directive: develop ALL materials in advance for self-paced follow-
  along. CLAUDE.md learning-mode gained the materials-ahead model.
- Built + validated reference glake (`specs/003-rust-bedrock/_reference/`):
  17/17 tests, R8/R9 properties at 512 cases, clippy clean both configs,
  std-only tree, R10 reconciliation verified (221 = 157 + 64).
- Workflow-authored 8 ramp worksheets (each solution compile-swept) + 6
  sitting guides; adversarial critic 78% → fixes (2 MAJORs: sitting F vs
  approved R10/walk text → change-protocol amendments requirements rev 4 /
  design rev 3; reference classify vs frozen interface → reference fixed) →
  re-verify **96%**, residues cleaned to 100% of findings addressed.
- ⚠ Pending learner ack (change protocol): R3a whole-lake walk scope + R10
  reconciliation amendment + design layout note (tally.rs).
- **State: the entire trail is built.** Learner starts at
  playground/ramp/step-01-hello/ whenever ready; Claude reviews submissions.


### 2026-07-09 — Session 6: spec 003 fully gated; coached construction begins
- Learner approved requirements ("proceed"), then the combined design+tasks
  fast-path gate ("approve"). All three 003 gates closed same-day.
- Combined audit (68%) caught 5 MAJORs pre-gate: lens dep pattern would have
  failed R4's own check; schema-at-runtime undesigned (→ constant + drift
  test, requirements rev 3); &ts[0..10] latent panic (→ .get + bad-ts bucket);
  properties were scheduled after code (→ test-first, red commits); lifetimes
  un-ramped (→ ramp step 8 added).
- Terminology: ramp exercises = "steps" (never "rungs"). Git strategy
  reconciled: trunk + main mirror + spec-close/NNN marker branches (remote
  refuses tags & deletes). Branch-map artifact published.
- **State: waiting on the learner's ramp step 1** (first Rust program at
  play.rust-lang.org). Everything else is unblocked and done.


### 2026-07-05 — Session 5 (cont.): 002 CONSTRUCTION COMPLETE — spec closed
- Full autonomous run of all 8 bolts on "you do all the bolts": memlens crate
  (tracking allocator, R1 property 256 cases, failure paths, macros),
  memlens-replay (R2/R3/R10a properties, adversarial validator, goldens),
  viewer (single-file, JS fold pinned bit-for-bit to Rust goldens, 1.77ms
  scrub at 100k events), operations (exercise 02, canonical 64-event trace in
  the lake, overhead 1.20×).
- **First real shrunk counterexample**: property-auditor (75%) caught a
  committed untriaged seed — duplicated no-op realloc validly accepted →
  TEST BUG, strategy fixed, seed kept (triage-log.md). PBT audited the test.
- Rescoped mid-run per learner: internal tool, no learner gates; Chromium
  verification 8/8 green (screenshot in _assurance/).
- Spec CLOSED (all gates, properties, [O] verifications, triage done).
  ⚠ merge-to-main + `spec/002-memory-lens` tag are created locally but the
  remote returned 403 — this session may only push the designated branch.
  **Deferred**: learner (or a session with push rights) runs
  `git branch main <head> && git tag spec/002-memory-lens <head> && git push origin main spec/002-memory-lens`.
  The working branch holds the complete history either way.

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
