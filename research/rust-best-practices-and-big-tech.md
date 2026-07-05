# Rust Best Practices & Big-Tech Production Lessons

**Researched:** 2026-07-05 · **Method:** deep-research workflow (6 angles, 27
sources, 110 claims extracted, 25 verified by 3-vote adversarial panels — 24
confirmed, **1 refuted**) · **Purpose:** engineering conventions + teaching
material for goldeneye

## Executive summary

Production Rust practice is well-codified in official sources: the rust-lang API
Guidelines give a checkable design rubric, the Rustonomicon defines the
unsafe-hygiene model (safe public APIs over audited unsafe internals), and
Cargo's release defaults leave 10–20%+ performance and real binary size on the
table until you opt in (`lto`, `codegen-units=1`, `panic=abort`, `strip`). The
big-tech evidence (Cloudflare Pingora, Google Android, Discord) is directionally
consistent and spectacular — but all self-reported, and the verifiers attached
honest caveats to every number. Highest-leverage adoptions for goldeneye are
listed at the bottom; all have been applied to steering.

## Part 1 — Verified engineering practices

### API design (the design.md rubric)
The official [API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
*(9-0)*: eagerly implement applicable common traits (`C-COMMON-TRAITS`,
`C-SEND-SYNC`); error types meaningful and well-behaved — `std::error::Error +
Display + Send + Sync`, which is **exactly what `thiserror` derives**, so our
existing convention is the downstream implementation of this upstream rule
(`C-GOOD-ERR`); doc examples use `?`, never `unwrap` (`C-QUESTION-MARK`);
newtypes for static distinctions, dedicated types over bare `bool`/`Option`,
builders for complex construction (`C-NEWTYPE`, `C-CUSTOM-TYPE`, `C-BUILDER`).

### Unsafe hygiene (the memlens doctrine)
From the [Rustonomicon](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
*(9-0)*: safe Rust's guarantee is that safe-code clients can never cause UB;
`unsafe` has exactly two roles (declare an uncheckable contract / assert you
verified one); the std-library norm is unsafe confined behind rigorously audited
safe APIs. Enforceable via `#![forbid(unsafe_code)]` — **per-crate only** (not
dependencies/build scripts). Verifier caveat worth teaching: the guarantee is
conditional on the unsafe code being *correct* (the RUDRA study found 264
ecosystem soundness bugs) — which is why memlens's `GlobalAlloc` core warrants
`#![deny(unsafe_op_in_unsafe_fn)]`, a `// SAFETY:` comment on every unsafe
block, and eventually Miri.

### Release-profile tuning (the Lambda-relevant one)
*(15-0 across 5 claims, [perf book](https://nnethercote.github.io/perf-book/build-configuration.html), [Cargo book](https://doc.rust-lang.org/cargo/reference/profiles.html))*
Cargo release defaults are conservative: `lto=false` (local thin only),
`codegen-units=16`, `panic=unwind`, no symbol strip. Opt-ins for deployables:
- `lto = "thin"` — ~10–20%+ runtime and smaller binaries at modest link cost
  (fat LTO only when chasing minimum size)
- `codegen-units = 1`, `strip = "symbols"` (1.77+ auto-strips debuginfo only)
- `panic = "abort"` — smaller binaries, no unwind tables, and **safe to combine
  with a normal test suite**: tests/benches/build scripts ignore the panic
  setting *(3-0, verified against cargo master)*
- PGO: +10%+ possible but setup-heavy — not worth it for a learning repo until
  a measured problem exists. Gains are workload-dependent: benchmark, don't assume.

### Fuzzing
`cargo-fuzz` is the rust-fuzz org's recommended frontend (libFuzzer only) and
**requires nightly** (unstable `-Z` sanitizer flags; stabilization PR still open
as of July 2026) *(6-0)*. **Refuted claim, for the record:** "aarch64 Linux is
not in the supported list" lost 0-3 — **ARM64/Graviton does not block fuzzing.**
Phase-2 addition once we parse untrusted input (envelope/trace parsing is the
natural first target).

## Part 2 — Big-tech evidence (with the caveats the verifiers demanded)

- **Cloudflare Pingora** *(9-0)*: Rust proxy serving >1T requests/day; ~70% less
  CPU, ~67% less memory than the NGINX/Lua predecessor; no service-code crash in
  hundreds of trillions of requests. Caveats: vendor-reported; a purpose-built
  rewrite vs a generalized stack (architecture + dropping Lua did part of the
  work). **And the counter-lesson: Cloudflare's Nov 2025 global outage was a
  Rust `Result::unwrap()` panic** — memory safety does not prevent panic-driven
  failures. Direct consequence: no `unwrap()`/`expect()` in handler paths,
  enforced via `clippy::unwrap_used`.
- **Google Android** *(9-0 + 3-0)*: memory-safety vulns 76% of total (2019) →
  24% (2024) → <20% (2025); ~5 MLOC of Rust at ~0.2 vulns/MLOC vs ~1000/MLOC
  historical C/C++ — a >1000× density reduction. Plus the velocity result: Rust
  changes have ~4× lower rollback rate and ~25% less review time than C++.
  Caveats: self-classified data; 0.2/MLOC rests on one pre-release finding in
  young code; velocity measured on experienced teams — a beginner should expect
  the borrow checker to be a tax before it's a dividend.
- **Discord Read States (Go→Rust)** *(9-0)*: Go's runtime forces a GC run every
  2 minutes (hardcoded `forcegcperiod`, independently confirmed in Go's source —
  still true), and each run scanned a multi-million-entry LRU cache, causing
  periodic latency spikes no tuning could fix. Rust with basic optimization beat
  the hand-tuned Go service on latency, CPU, and memory — **because ownership
  frees memory deterministically at scope exit; no collector ever scans live
  data.** This is the memory-management through-line of the whole class, and
  memlens will make it *visible*. Caveats: Go ≤1.10 only was tested;
  single-service, self-reported.

## What does NOT transfer to goldeneye

Trillion-request rewrite economics; custom proxies/runtimes; org-scale
rollback/review metrics; vendor benchmark magnitudes (70% CPU, microsecond
latencies) — those are motivation, not targets.

## Adopted now (applied to steering 2026-07-05)

1. `#![forbid(unsafe_code)]` on every crate **except** `memlens`; in memlens:
   `#![deny(unsafe_op_in_unsafe_fn)]` + `// SAFETY:` on every unsafe block.
2. Shared workspace `[profile.release]`: `lto="thin"`, `codegen-units=1`,
   `panic="abort"`, `strip="symbols"`.
3. API Guidelines checklist as the design.md rubric for public APIs.
4. No `unwrap()`/`expect()` in handler/service paths — `clippy::unwrap_used`
   (the Cloudflare outage lesson).
5. Proptest-first pipeline — unchanged; it *is* the executable-invariants
   culture these sources describe.

**Later:** cargo-fuzz nightly lane (first untrusted-input parser), Miri for
memlens's unsafe core, fat LTO/PGO only on measured need.

## Volatility & review

**Volatile (review by 2026-10-05):** cargo-fuzz nightly requirement (`-Csanitize`
stabilization PR open); aws-lc-rs as default SDK crypto provider. **Annual
(2027-07):** all benchmark magnitudes (Pingora/Android/Discord numbers, LTO
gains). **Stable:** API Guidelines, Nomicon unsafe model, Cargo profile
semantics, Go's forced-GC mechanism.

## Open questions the research could not settle

- Async/tokio production patterns: no claims survived verification — re-research
  when the first tokio service spec (Fargate) approaches.
- AWS's own Rust engineering (Firecracker, S3 ShardStore formal methods): absent
  from the verified set — fold into the Rust-on-AWS report's follow-ups.
- Does `panic=abort` measurably cut Rust Lambda cold start on ARM64, and by how
  much? → **flagged as a future evidence.md benchmark experiment.**
- cargo-fuzz on stable once `-Csanitize` stabilizes (rust-lang/rust #123617).
