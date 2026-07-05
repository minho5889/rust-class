# Steering — Tech Stack

## Language & toolchain

- **Rust stable** (via `rustup`), edition 2024. No nightly features.
- `cargo fmt` + `cargo clippy -- -D warnings` gate every commit.
- Errors: `thiserror` (libraries) / `anyhow` (binaries). Logging: `tracing`.
- Async: `tokio`. Serialization: `serde` / `serde_json`.
- HTTP services: `axum`. AWS access: official `aws-sdk-*` crates + `aws-config`.

## Target architecture

- **ARM64 (Graviton) everywhere** — `aarch64-unknown-linux-gnu` (or `-musl` for
  static Fargate images). Rationale: cheaper on Lambda/Fargate/EC2, and Lambda
  MicroVMs is ARM64-only.

## Per-target tooling

| Target | Build/deploy | Runtime | Notes |
|---|---|---|---|
| Lambda | `cargo-lambda` (`build --release --arm64`, `deploy`) | `provided.al2023` | Rust GA on Lambda since Nov 2025; ~15 ms cold starts |
| Lambda MicroVMs | `aws-sdk` lifecycle calls + console/CLI | Firecracker microVM | Launched 2026-06-22; ARM64 only; ≤16 vCPU / 32 GB / 32 GB disk / 8 h; regions: us-east-1/2, us-west-2, eu-west-1, ap-northeast-1 |
| ECS Fargate | Docker multi-stage (`rust:slim` → distroless/`scratch`), ECR | container | Static musl builds preferred for `scratch` |
| EC2 | release binary + systemd unit, user-data bootstrap | Graviton instance | Compare ops burden vs the managed options |

## Infrastructure as code

Phase 1: none — `cargo lambda deploy` and AWS CLI, to keep the learning surface
small. Phase 2 (after Level 2a is mastered): introduce AWS SAM or CDK — decision
deferred to the learner (tracked in `MEMORY.md` open questions).

## Constraints

- Default region: TBD (must support MicroVMs — see `MEMORY.md` open questions).
- Keep everything free-tier/minimal-cost where possible; tear down after exercises.
- No secrets in the repo. Credentials via `aws configure` / environment only.
