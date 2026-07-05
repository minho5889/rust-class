# Steering — Tech Stack

## Project identity

- Codename: **goldeneye** — all AWS resources prefixed `goldeneye-`.
- Regions: **us-east-1** (primary), **ap-northeast-1** (secondary). Both support
  Lambda MicroVMs, so all four compute targets stay in-region.

## Language & toolchain

- **Rust stable** (via `rustup`), edition 2024. No nightly features (exception
  later: a dedicated cargo-fuzz CI lane may use nightly).
- `cargo fmt` + `cargo clippy -- -D warnings` gate every commit.
- Errors: `thiserror` (libraries) / `anyhow` (binaries). Logging: `tracing`.
- Async: `tokio`. Serialization: `serde` / `serde_json`.
- HTTP services: `axum`. AWS access: official `aws-sdk-*` crates + `aws-config`.

Conventions adopted 2026-07-05 from `research/rust-best-practices-and-big-tech.md`:

- **Unsafe policy:** `#![forbid(unsafe_code)]` on every crate **except**
  `memlens`; in memlens `#![deny(unsafe_op_in_unsafe_fn)]` and a `// SAFETY:`
  comment on every unsafe block (Nomicon model: safe public APIs over audited
  unsafe internals). Miri for memlens's core: later.
- **Shared release profile** (workspace root): `lto = "thin"`,
  `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` — tests are
  unaffected (test harness ignores the panic setting). Fat LTO/PGO only on
  measured need.
- **No `unwrap()`/`expect()` in handler/service code paths** —
  `clippy::unwrap_used` on deployable crates (the Cloudflare Nov-2025 outage was
  an unwrap panic in memory-safe Rust).
- **API rubric:** public APIs reviewed against the rust-lang API Guidelines
  checklist (C-COMMON-TRAITS, C-GOOD-ERR, C-NEWTYPE, C-BUILDER) in design.md.

## Testing

- Built-in `#[test]` for [E] example tests; **`proptest`** for [P] property tests
  (chosen over `quickcheck`: multiple strategies per type, better shrinking).
- Conventions: explicit `Strategy` objects over type-based generation;
  `prop_assert!`/`prop_assert_eq!` inside `proptest!` blocks (clean shrink output);
  default ≥256 cases; name property tests after requirement IDs (`prop_r1_…`).
- **Commit `proptest-regressions/`** — failing seeds are permanent regression tests.
- Every counterexample is triaged (spec bug / code bug / test bug) in the spec's
  `_assurance/triage-log.md` before it is fixed.

## Target architecture

- **ARM64 (Graviton) everywhere** — `aarch64-unknown-linux-gnu` (or `-musl` for
  static Fargate images). Cheaper on Lambda/Fargate/EC2; MicroVMs is ARM64-only.

## Per-target tooling

| Target | Build/deploy | Runtime | Notes |
|---|---|---|---|
| Lambda | `cargo-lambda` (`build --release --arm64`, `deploy`) | `provided.al2023` | Rust GA on Lambda since Nov 2025; ~15 ms cold starts |
| Lambda MicroVMs | `aws-sdk` lifecycle calls + console/CLI | Firecracker microVM | Launched 2026-06-22; ARM64 only; ≤16 vCPU / 32 GB / 32 GB disk / 8 h |
| ECS Fargate | Docker multi-stage (`rust:slim` builder → `distroless-static`/`chainguard-static`), ECR | container | musl static binaries (verify with `ldd`); never build ARM64 images under QEMU — `cargo-zigbuild` + `--platform=$BUILDPLATFORM` or native ARM runners |
| EC2 | release binary + systemd unit, user-data bootstrap | Graviton instance | `aarch64-unknown-linux-gnu` is Rust Tier 1; `-Ctarget-cpu=neoverse-n1` when Graviton-only |

## Graviton/Lambda rules (from `research/rust-on-aws-compute.md`, 2026-07-05)

- **Thread pools on Lambda**: size from the memory-derived vCPU count
  (1,769 MB ≈ 1 vCPU, 10,240 MB ≈ 6), never `available_parallelism()` — the
  sandbox over-reports.
- **Arch-backend check**: any crate doing crypto/hash/SIMD work must have its
  ARM64 hardware backend verified active (the sha2 lesson: 4–5× swing).
- **Binary budget**: `aws-sdk-*` adds ~10 MB+ (aws-lc-rs ~4 MB); check with
  `cargo-bloat` in deployable specs.
- MicroVMs / Managed Instances specifics are **unverified** — their specs start
  with primary-source verification.

## Data lake (goldeneye telemetry)

- v1 (live): hooks append envelope JSONL to `datalake/raw-local/` (schema:
  `datalake/schema/envelope.v1.json`), committed to git.
- Wave 2: `s3://goldeneye-lake` (raw JSONL+zstd → curated Parquet, Hive-partitioned),
  scans via DuckDB (default) or Athena. Wave 3: Rust ingest Lambda +
  `s3://goldeneye-discovery` insight cards. See `datalake/README.md`.

## Infrastructure as code

**AWS CDK v2 with TypeScript** (learner decision, 2026-07-05). Phase 1 still uses
`cargo lambda deploy` + AWS CLI; CDK enters with the first AWS-deploying spec.
Conventions (from `research/typescript-cdk-for-goldeneye.md`, verified 2026-07-05):

- CDK app in `infra/` beside the cargo workspace (one repo — AWS best practice).
- Lambda: `cargo-lambda-cdk` `RustFunction` with **explicit
  `architecture: Architecture.ARM_64`** (default is x86_64). Avoid the
  experimental `cdklabs/aws-lambda-rust`.
- Fargate: `DockerImageAsset` with **explicit `platform: Platform.LINUX_ARM64`**
  — unset, it silently builds for the build machine's arch and fails at runtime.
- EC2: Graviton via `InstanceType.of` + AL2023 `ARM_64` AMI; binary as S3 asset;
  daemons via `InitService.systemdConfigFile()`/`enable(…, SYSTEMD)` — never
  `InitCommand` for long-running processes.
- Stacks: stateful (lake buckets, termination-protected) split from stateless
  compute stacks. Guardrail: `cdk-nag` v3 (AwsSolutions + Serverless packs).
- **Naming: tags over physical names.** `project=goldeneye` tag on everything;
  hardcoded physical names only for `goldeneye-lake` / `goldeneye-discovery`
  (cross-spec contract); all other resources use CDK-generated names.
- Lambda MicroVMs CDK/CFN support: **unknown** — the MicroVMs spec must carry
  this as a risk item (fallbacks: raw CfnResource → AwsCustomResource → CLI).

## Constraints

- Keep everything free-tier/minimal-cost where possible; tear down after exercises.
- No secrets in the repo or in telemetry payloads. Credentials via `aws configure` /
  environment only. Telegram tokens live in env vars (`TELEGRAM_BOT_TOKEN`,
  `TELEGRAM_CHAT_ID`), never in files.
