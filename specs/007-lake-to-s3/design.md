# Design — 007 lake-to-s3

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

One trait is the whole architecture:

```rust
pub trait ObjectStore {
    async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>, StoreError>; // key, size, etag
    async fn put(&self, key: &str, body: Vec<u8>) -> Result<(), StoreError>;
}
```

```
                       ┌── FakeStore (BTreeMap + a ledger: puts counted,
 lake-sync core ───────┤     max-in-flight gauged)  ← every test, every property
 (generic over S)      └── S3Store (aws-sdk-s3)     ← reality, deploy day
 ingest handler ───────┘
```

Everything interesting — walking the lake, planning uploads, idempotence,
bounded concurrency, conservation — happens in code that has **never heard
of AWS**. The properties run at 256+ generated lakes per case against the
fake; the real `S3Store` is a thin translation layer you can read in one
sitting. This is 004's trait lesson graduating into a testing strategy:
*put the seam where the un-testable thing starts.*

Idempotence works like the lake itself — by content, not memory: `list` the
`raw/` prefix once, compare each local file's **size and md5-etag** against
what's there, upload only the differing. (S3's etag *is* the md5 for
single-part puts — and the multipart caveat gets one honest paragraph where
it would bite.) The ingest side is idempotent for free: the key contains the
`event_id`, so a retried event overwrites itself.

## The shape

| Part | What it is | Rust you learn | REQs |
|---|---|---|---|
| **crates/lake-sync/src/store.rs** | the trait above + `ObjectMeta {key,size,etag}` + `StoreError` (thiserror, C-GOOD-ERR); **native async fn in trait** (stable) — and the honest note: that choice keeps the seam generic-only (`S: ObjectStore`), no `dyn`, which is exactly what sync needs (F13 pattern redux) | async fn in trait, trait-as-seam | S7 |
| **fake.rs** (test support, `pub` in lib for the ingest crate's tests too) | `Mutex<BTreeMap<String, Vec<u8>>>` + put counter + in-flight gauge (`AtomicU32` high-water mark) | interior mutability for test ledgers | S2–S4, S9 |
| **plan.rs** | pure: local walk (glake's `jsonl_files`) + remote `list` → `Plan { upload: Vec<_>, skip: Vec<_> }`; md5 via a tiny md5 crate; size checked first (cheap), etag second | pure-core/impure-shell layout | S1, S2, S5 |
| **run.rs** | executes a Plan: streams each file (`BufRead::read_line` into one reused `String`), single PUT per file, `JoinSet` + `Semaphore(4)` bound | buffer reuse, bounded fan-out (005's backpressure echo) | S9, T4 |
| **main.rs (lake-sync bin)** | clap: `--bucket`, `--dry-run`, `--prefix raw/`, path; stderr/exit discipline per glake | — | S2, S5 |
| **crates/hello-lambda** (evolved) | the sink swap: `println!` line → `store.put(ingest_key(event), line)`; `S3Store` built once in `main` before the runtime loop (`OnceLock`), bucket name from env (`LAKE_BUCKET`, set by CDK) | client-once cold-start discipline | S6, T3 |
| **infra/lib/stateful-stack.ts** | the two buckets (SSE-S3, BPA on, RETAIN, termination protection), exported bucket ref; stateless stack wires `LAKE_BUCKET` env + `grantPut` scoped `raw/*` | — | S10 |
| **datalake/queries/duckdb/** | `NN-<consumer>.sql` in `local/` and `s3/` variants + README (httpfs install, credential chain note); each header names the doc it feeds | — | S11 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Test seam | mock the SDK (smithy mocks) · localstack/docker · **own trait + in-memory fake** | own trait | the learner *designs* the seam (T1) instead of consuming a mocking framework; properties stay dependency-free and fast; localstack teaches ops, not Rust |
| Trait dispatch | `dyn ObjectStore` · **generic `S: ObjectStore`** | generic | async fn in trait isn't (yet) dyn-friendly without boxing helpers; sync has exactly two impls chosen at compile time — the honest F13 answer. The *contrast* with 004's CLI `dyn` seam is called out in sitting R |
| Idempotence check | remember state locally (manifest file) · always upload · **remote list + size/md5-etag compare** | list+compare | the store itself is the source of truth (no second thing to corrupt); teaches etag semantics incl. the multipart caveat; one LIST is cheaper than N heads at this scale |
| Sync upload unit | per-event objects · **per-file objects (mirror the local layout)** | per-file | keys mirror `dt=*/events.jsonl` exactly — `glake` and DuckDB read both sides identically; per-event explodes object count for zero query gain |
| Ingest key | append to a shared object (impossible) · per-day rollup (needs read-modify-write races) · **`evt-<event_id>.json` per event** | per event | S3 has no append; event_id key makes retries self-idempotent; DuckDB globs both shapes with one pattern |
| md5 dependency | hand-roll (unsafe curiosity) · openssl · **`md5` crate (pure Rust, tiny)** | md5 crate | it's a compatibility checksum here, not security (said in a comment — C-GOOD-ERR of crypto hygiene); pure Rust keeps the ARM64 cross-build trivial |
| Concurrency bound | unbounded `join_all` · **`JoinSet` + `Semaphore(4)`** | JoinSet+Semaphore | 005's bounded-channel lesson re-cast for fan-out; the fake's gauge makes the bound *testable* (S9) |
| SDK features | default (aws-lc-rs TLS) · **rustls/ring feature set if the ARM64 cross-build fights** | default first, documented fallback | aws-lc-rs ships pregenerated aarch64 bindings and built clean in the authoring environment; the fallback is written down so a learner on a stranger machine isn't stranded |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| S3 | ∀ generated lakes: multiset(lines in fake store after sync) = multiset(local lines) | 003's line-set generator builds a real tempdir lake (1–8 files across 1–4 `dt=` days, 0–40 lines each, incl. blanks/malformed — sync moves *bytes*, not judgments); sync against a fresh fake; compare multisets. ≥256 cases |
| S4 | ∀ generated lakes: second sync ⇒ ledger shows 0 puts; then mutate-or-add one file ⇒ exactly the changed keys upload | same generator; fake's put-counter reset between passes; mutation arm picks {append a line, touch new file} |

Both run against the **fake** — that's the point (T2). The real `S3Store`
is covered by compile + review + deploy-day S12, and by the honesty that a
translation layer this thin has nowhere for logic bugs to live (list-map,
put-passthrough — ~60 lines).

## How we verify

**In-session (the reference was validated with all of these):**
- S3/S4 proptests ≥256, red-first against a stub plan/run core.
- S1/S2/S5/S6/S9 example tests (fake store; tempdir lakes; CLI via
  `CARGO_BIN_EXE`); 006's H4 property re-run green post-evolution.
- fmt + clippy (`unwrap_used`/`expect_used` deny on both crates) clean.
- Both binaries: `cargo lambda build --release --arm64` (ingest) and
  `cargo build --release --target aarch64-unknown-linux-gnu` (lake-sync);
  sizes + bloat deltas recorded (S8).
- `npx cdk synth` both stacks + cdk-nag; template asserts: RETAIN,
  termination protection, BPA, the scoped `raw/*` grant, `LAKE_BUCKET` env.
- Scan pack local variants executed with DuckDB against `datalake/raw-local`;
  outputs recorded; totals cross-checked against `glake stats`.

**Deploy day (sitting U → evidence.md):** S12 end-to-end, with the buckets
retained and everything else torn down.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft | Part-6 directive (full loop) | pending combined ack |

</details>
