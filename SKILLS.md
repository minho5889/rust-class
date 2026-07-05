# SKILLS.md — Skill Tree & Curriculum

Mastery tracking for the Rust-on-AWS class. Statuses: `[ ]` not started,
`[~]` in progress, `[x]` mastered (with evidence noted in `MEMORY.md`).

A skill counts as **mastered** only when the learner can apply it without help —
e.g., fix a borrow-checker error unaided, or deploy a Lambda from scratch.

---

## Level 0 — Environment (prerequisite)

- [ ] Install `rustup`, understand stable vs nightly toolchains
- [ ] `cargo` basics: `new`, `build`, `run`, `test`, `fmt`, `clippy`, workspaces
- [ ] Install `cargo-lambda`, Docker, AWS CLI v2; configure credentials
- [ ] Cross-compilation to `aarch64-unknown-linux-gnu` (ARM64/Graviton)

## Level 1 — Rust fundamentals (the bedrock)

### 1a. Core language
- [ ] Variables, mutability, shadowing, scalar & compound types
- [ ] Functions, control flow, pattern matching (`match`, `if let`, `let else`)
- [ ] Structs, enums, `impl` blocks, methods
- [ ] `Option<T>` and `Result<T, E>` — no null, no exceptions

### 1b. Memory management (the reason we're here)
- [ ] **Ownership**: move semantics, `Copy` vs `Clone`, drop order
- [ ] **Borrowing**: `&T` vs `&mut T`, the aliasing XOR mutability rule
- [ ] **Lifetimes**: elision rules, explicit annotations, `'static`
- [ ] Stack vs heap: `Box<T>`, when allocation happens, why it matters for cost
- [ ] Shared ownership: `Rc<T>`, `Arc<T>`, interior mutability (`RefCell`, `Mutex`)
- [ ] Slices, `String` vs `&str`, `Vec<T>` growth behavior

### 1c. Abstraction & robustness
- [ ] Traits, generics, trait objects (`dyn Trait`) — static vs dynamic dispatch
- [ ] Error handling patterns: `?`, `thiserror`, `anyhow`
- [ ] Modules, visibility, crate organization
- [ ] Testing: unit tests, integration tests, `cargo test`
- [ ] **Property-based testing with `proptest`**: properties as invariants,
      strategies, shrinking, regression seeds — and how EARS requirements
      become executable properties (our pipeline's [P] tags)

### 1d. Async Rust (required for all AWS work)
- [ ] `async`/`.await`, futures, why Rust async is zero-cost
- [ ] `tokio` runtime: tasks, `spawn`, channels, `select!`
- [ ] `Send`/`Sync` and what the compiler enforces across threads

## Level 2 — AWS compute targets (one Unit of Work each)

### 2a. AWS Lambda (start here — fastest feedback loop)
- [ ] `lambda_runtime` + `cargo lambda new/watch/build/deploy`
- [ ] Handler signatures, `serde` for event payloads, API Gateway events
- [ ] Cold starts: why Rust lands at ~15 ms; memory sizing vs cost
- [ ] Structured logging with `tracing` → CloudWatch
- 📦 Spec: first hands-on Unit of Work, to be created via pipeline v2

### 2b. AWS Lambda MicroVMs (new — June 2026)
- [ ] Concept: Firecracker microVM sandboxes vs regular Lambda functions —
      stateful, isolated, full lifecycle control, up to 8 h / 16 vCPU / 32 GB
- [ ] MicroVM lifecycle API: create, snapshot/resume, terminate from Rust
      (`aws-sdk` crates)
- [ ] Use case build: a sandbox that executes untrusted/generated code safely
- [ ] When to choose MicroVMs vs Lambda vs Fargate (isolation, state, duration)

### 2c. Amazon ECS on Fargate
- [ ] Containerizing Rust: multi-stage Dockerfile, distroless/scratch images,
      static vs dynamic linking (musl vs glibc)
- [ ] A long-running `axum` (or `actix-web`) HTTP service
- [ ] Task definitions, service, ALB; graceful shutdown on SIGTERM
- [ ] Right-sizing: Rust's small memory footprint vs typical container sizing

### 2d. Amazon EC2
- [ ] Build/release pipeline for a systemd-managed Rust daemon
- [ ] Instance selection (Graviton), user-data bootstrap
- [ ] Comparing operational burden: EC2 vs Fargate vs Lambda for the same service

### 2e. The goldeneye data lake (cross-cutting, spans 2a–2c)
- [ ] S3 as a data lake: zones (raw/curated/discovery), Hive partitioning,
      lifecycle to Glacier
- [ ] Columnar thinking: JSONL vs Parquet, why scans get 10–20× cheaper
- [ ] Scanning with DuckDB (local, free) and Athena (serverless SQL)
- [ ] Rust telemetry ingest Lambda: `serde` streaming, buffer reuse,
      `bytes::Bytes` zero-copy batching
- 📦 Specs: `002-datalake-v1` (Wave 2), telemetry ingest + archaeologist (Wave 3)

## Level 3 — Systems engineering & efficiency (the payoff)

- [ ] Profiling: `cargo flamegraph`, heap profiling, `criterion` benchmarks
- [ ] Zero-copy techniques: borrowing in APIs, `bytes::Bytes`, avoiding clones
- [ ] Binary size & startup optimization: LTO, `codegen-units`, `strip`, panic=abort
- [ ] Measuring the money: same workload on all four targets — latency, memory,
      monthly cost comparison (capstone Unit of Work)

---

## Suggested path

```
Level 0 → 1a → 1b → (001: hello Lambda, using only 1a/1b concepts)
       → 1c → 1d → (2a deep dive) → 2b → 2c → 2d → Level 3 capstone
```

Interleave theory and deployment: after Level 1b the learner should already be
shipping a Lambda, because deploying early keeps motivation high and makes the
memory-management lessons concrete (cold start times and RAM bills are feedback).
