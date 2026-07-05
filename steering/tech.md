# Steering — Tech Stack

## Project identity

- Codename: **goldeneye** — all AWS resources prefixed `goldeneye-`.
- Regions: **us-east-1** (primary), **ap-northeast-1** (secondary). Both support
  Lambda MicroVMs, so all four compute targets stay in-region.

## Language & toolchain

- **Rust stable** (via `rustup`), edition 2024. No nightly features.
- `cargo fmt` + `cargo clippy -- -D warnings` gate every commit.
- Errors: `thiserror` (libraries) / `anyhow` (binaries). Logging: `tracing`.
- Async: `tokio`. Serialization: `serde` / `serde_json`.
- HTTP services: `axum`. AWS access: official `aws-sdk-*` crates + `aws-config`.

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
| ECS Fargate | Docker multi-stage (`rust:slim` → distroless/`scratch`), ECR | container | Static musl builds preferred for `scratch` |
| EC2 | release binary + systemd unit, user-data bootstrap | Graviton instance | Compare ops burden vs the managed options |

## Data lake (goldeneye telemetry)

- v1 (live): hooks append envelope JSONL to `datalake/raw-local/` (schema:
  `datalake/schema/envelope.v1.json`), committed to git.
- Wave 2: `s3://goldeneye-lake` (raw JSONL+zstd → curated Parquet, Hive-partitioned),
  scans via DuckDB (default) or Athena. Wave 3: Rust ingest Lambda +
  `s3://goldeneye-discovery` insight cards. See `datalake/README.md`.

## Infrastructure as code

Phase 1: none — `cargo lambda deploy` and AWS CLI, to keep the learning surface
small. Phase 2 (after Lambda mastery): introduce AWS SAM or CDK — decision deferred
to the learner (tracked in `MEMORY.md` open questions).

## Constraints

- Keep everything free-tier/minimal-cost where possible; tear down after exercises.
- No secrets in the repo or in telemetry payloads. Credentials via `aws configure` /
  environment only. Telegram tokens live in env vars (`TELEGRAM_BOT_TOKEN`,
  `TELEGRAM_CHAT_ID`), never in files.
