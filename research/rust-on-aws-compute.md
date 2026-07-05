# Rust on AWS Compute — Lambda, EC2, Fargate, MicroVMs

**Researched:** 2026-07-05 · **Method:** deep-research workflow (5 angles, 22
sources, 48 claims extracted, 25 verified by 3-vote adversarial panels — 24
confirmed, **1 refuted**) · **Scope:** goldeneye's four targets, ARM64,
us-east-1 / ap-northeast-1

## Executive summary

Rust on AWS ARM64 is a production-grade path with verified numbers: GA on Lambda
since Nov 2025 (AWS Support + SLA, all regions including both of ours), ~16–28 ms
cold starts (5–8× faster than Python/Node), and ARM64 winning **every genuine
cost comparison** in the December 2025 benchmark set (229 wins, 0 for x86). The
two biggest practical gotchas are architecture-specific crate features (a missing
hardware-crypto backend made ARM64 *look slower than x86* until enabled — then 4–5×
faster) and QEMU-emulated Docker builds (15–25× slower than native; use
`cargo-zigbuild` + `$BUILDPLATFORM`). **Honest gap: no claims about Lambda
MicroVMs, Lambda Managed Instances, or SnapStart survived verification** — those
corners of the head-to-head remain open questions, not facts.

## Lambda (verified core)

- **GA Nov 2025**, backed by AWS Support + Lambda SLA, all regions. GA covers the
  Rust runtime client on OS-only runtimes — there is no dedicated managed Rust
  runtime. *(3-0)*
- **Deployment model settled**: native binary named `bootstrap` on
  `provided.al2023`; `lambda_runtime` (+ `lambda_http`) + cargo-lambda
  (`cargo lambda build --release --arm64`). Directly validates our CLAUDE.md
  conventions. *(3-0 ×2)*
- **Cold starts**: ~16 ms init on ARM64 (independent, Dec 2025); AWS's own
  reproducible benchmark: 19–28 ms across memory configs, ARM64 slightly ahead.
  Workload-dependent — a light-workload binary showed 66–68 ms (still 5–6× ahead
  of Python/Node). Rust essentially eliminates the problem SnapStart exists to
  solve for Java. Caveat: 20 samples/config; Go/Java/.NET weren't in the test
  set. *(3-0 ×3, [cebert benchmarks](https://github.com/cebert/aws-lambda-performance-benchmarks), [AWS blog](https://aws.amazon.com/blogs/compute/optimizing-compute-intensive-serverless-workloads-with-multi-threaded-rust-on-aws-lambda/))*
- **ARM64 = default, verified**: 15–20% cheaper per AWS (list GB-second exactly
  20% below x86), 41–47% cheaper for Rust light workloads independently; ARM64
  won all 229 genuine cost comparisons. Cold-start init 13–24% faster across
  tested runtimes *(cost 3-0; cold-start 2-1 — cite the 13–24% range, not the
  wider figures sometimes quoted; Java behaves differently and wasn't tested)*.
- **CPU scales with memory**: 1,769 MB ≈ 1 vCPU, 10,240 MB ≈ 6 vCPUs; rayon hit
  6.73× on 6 workers. **Size thread pools from the memory-derived vCPU count,
  not `available_parallelism()`** — the sandbox may report more cores than the
  throttled allocation and oversubscribe. Below ~1,769 MB, CPU is fractional.
  *(3-0 ×2)*
- **Binary bloat warning**: `aws-sdk-*` crates can add 10 MB+ to the deployed
  binary — ~4 MB from `aws-lc-rs` (default crypto provider) alone — plus slower
  compiles. Budget for it; measure with `cargo-bloat`. *(3-0)*

## The Graviton gotcha to memorize

Enabling the `sha2` crate's hardware backend changed ARM64 SHA-256 from
*slower than x86* (~158 vs ~152 ms) to *4.3× faster* (~35 vs ~152 ms) — a 4–5×
swing from one feature flag. Durable lesson (the literal `asm` flag is
sha2-0.10-specific; 0.11 auto-detects): **when a crate does crypto/hash/SIMD
work, verify its ARM64 hardware backend is actually active before trusting any
benchmark.** Press coverage misframed this as an ARM-vs-x86 gap; it was a
configuration gap. *(3-0 ×3, 2-1 ×1)*

## Fargate / EC2 (long-running services)

- **musl static linking is the proven container pattern**: zero dynamic-lib
  dependencies kills the glibc-mismatch failure class; `muslrust` maintains
  build images for both arches. **Refinement to our convention: prefer
  `distroless-static` or `chainguard-static` over bare `scratch`** — non-root
  user + CA certs, still no shell. Caveats: `aarch64-…-musl` is Tier 2; vendored
  C deps (openssl) or the binary isn't actually static — verify with `ldd`; musl
  has DNS/allocator quirks. *(3-0 ×3)*
- **Cross-compilation**: QEMU-emulated multi-arch builds are 15–25× slower
  (~50 min vs 2–3 min). Fix: `--platform=$BUILDPLATFORM` build stage +
  `cargo-zigbuild` to musl targets, copy per-arch binaries into matching layers;
  or native ARM64 CI runners (GitHub has free ones). *(3-0 ×2, medium confidence
  — blog-grade primary source, strongly corroborated)*
- **No toolchain risk on ARM64 servers**: `aarch64-unknown-linux-gnu` is Rust
  **Tier 1 with host tools** (since 1.49, RFC motivated by Graviton). Graviton2+
  LSE atomics help lock-heavy tokio services; Rust 1.57+ already uses LSE via
  runtime dispatch — `-Ctarget-cpu=neoverse-n1` removes the fallback when
  deploying Graviton-only. *(3-0 ×3)*

## Refuted, for the record

AWS's claim that Lambda/Fargate environments start "in under 125 ms *because*
Firecracker is written in Rust" failed verification (0-3) — the speed is real
but the causal attribution to the implementation language is marketing. Good
epistemics lesson: love Rust, distrust vibes.

## Honest gaps (nothing survived verification — treat as unknown)

1. **Lambda MicroVMs**: the ARM64-only / 16 vCPU / 32 GB / 8 h limits and
   lifecycle API we recorded from launch coverage, aws-sdk-rust support, tokio
   sizing against its vCPU model — all unverified. **The MicroVMs spec must
   start with primary-source verification of its own constraints.**
2. **Lambda Managed Instances** (Rust support Mar 2026): pricing/warm semantics
   unverified.
3. Production case studies with numbers for Rust on Fargate/EC2 (the Lambda side
   is well-grounded; the long-running side isn't).
4. **ap-northeast-1 feature parity** for the full goldeneye stack — verify
   before assuming the secondary region supports everything.

## Volatility & review

**Volatile (review by 2026-10-05):** everything MicroVMs/Managed Instances
(unverified); sha2 0.10 `asm` flag specifics (0.11 auto-detects). **Annual
(2027-07):** all cold-start/cost benchmark numbers; aws-sdk binary-size figures.
**Stable:** GA status, bootstrap/provided.al2023 deployment model, Tier-1
platform status, memory→vCPU proportionality.

## Adopted into goldeneye (applied 2026-07-05)

1. Container images: `distroless-static`/`chainguard-static` over bare scratch
   (steering + CLAUDE.md refinement).
2. Cross-compile rule: never build ARM64 images under QEMU; `cargo-zigbuild` +
   `$BUILDPLATFORM` or native ARM runners.
3. Thread-pool sizing rule for Lambda: derive from memory-based vCPU count.
4. Graviton checklist item: verify arch-specific hardware backends on
   crypto/hash/SIMD crates.
5. Curriculum: the four-target capstone gains a verified baseline table to beat;
   the future Lambda spec gains the `panic=abort` cold-start experiment (from
   the best-practices report) and a `cargo-bloat` budget check for aws-sdk deps.
