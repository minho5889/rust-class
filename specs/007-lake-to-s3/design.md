# Design — 007 lake-to-s3

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

One trait is the whole architecture — and its exact spelling is a lesson:

```rust
pub trait ObjectStore: Send + Sync + 'static {
    // NOT `async fn` sugar: the desugared form lets us SAY the future is Send,
    // which `tokio::task::JoinSet::spawn` demands and the sugar can't promise
    // on stable Rust. This is the async-fn-in-trait lesson, taken on purpose.
    fn list(&self, prefix: &str)
        -> impl Future<Output = Result<Vec<ObjectMeta>, StoreError>> + Send; // key, size, etag (bare-hex md5)
    fn put(&self, key: String, body: Vec<u8>)
        -> impl Future<Output = Result<(), StoreError>> + Send;
}
```

```
                    ┌─────────────── crates/lake-store ────────────────┐
                    │  the trait · ObjectMeta · StoreError             │
                    │  FakeStore  (feature "fake": BTreeMap + ledger)  │
                    │  S3Store    (aws-sdk-s3, ~60 lines)              │
                    └───────┬──────────────────────────┬───────────────┘
      crates/lake-sync ─────┘                          └───── crates/hello-lambda
      (CLI: plan + run, generic over S)                       (ingest: door → put)
```

Three crates, one seam. `lake-store` holds the trait and both impls (the
fake behind a `fake` feature that only dev-dependencies turn on); `lake-sync`
and the evolved `hello-lambda` both depend on it — **neither depends on the
other**, so the S8 size measurement isolates exactly what the SDK costs the
Lambda, unpolluted by clap or the CLI's dep tree.

Everything interesting — walking the lake, planning uploads, idempotence,
bounded concurrency, conservation — happens in code that has **never heard
of AWS**. The properties run at 256+ generated lakes per case against the
fake; the real `S3Store` is a thin translation layer you can read in one
sitting. This is 004's trait lesson graduating into a testing strategy:
*put the seam where the un-testable thing starts.*

Idempotence works like the lake itself — by content, not memory: `list` the
`raw/` prefix once, compare each local file's **size and md5-etag** against
what's there, upload only the differing. Keys are derived mechanically:
`raw/` + the file's **path relative to the lake root** (so `traces/…` and
`dt=bad-ts/…` map verbatim — nothing in the walker's output is special-
cased). Three etag facts the design pins because each one silently kills
the `uploaded 0` demo if missed: (1) the SDK returns the etag **wrapped in
double quotes** — `S3Store::list` strips them, with a comment saying why;
(2) etag = md5 holds for single-part PUT under **SSE-S3** (which S10
mandates) and breaks under SSE-KMS and multipart — one honest paragraph
where it would bite; (3) the fake's `list` computes md5-of-stored-bytes in
the **same bare-hex format**, so the fake and reality can't drift on
spelling. The ingest side is idempotent for free: the key contains the
`event_id`, so a retried event overwrites itself.

## The shape

| Part | What it is | Rust you learn | REQs |
|---|---|---|---|
| **crates/lake-store/src/lib.rs** | the trait above (RPITIT with explicit `+ Send` bounds — see the decision row) + `ObjectMeta {key, size, etag}` + `StoreError` (thiserror, C-GOOD-ERR); generic-only seam, no `dyn` (F13 pattern redux) | async fn in trait desugared, trait-as-seam | S7 |
| **lake-store/src/fake.rs** (behind feature `fake`; consumers enable it in dev-dependencies only) | `Mutex<BTreeMap<String, Vec<u8>>>` — **insert replaces, modeling S3's one-object-per-key** (the law S3/S4 lean on) — + a ledger: put counter, and an in-flight gauge whose increment/decrement live **inside `put` itself** (so S9's bound is measured where the work happens, not where it's scheduled); `list` computes bare-hex md5 etags; a `fail_on(key)` injection point for S5's mid-run-failure test | interior mutability for test ledgers | S2–S4, S5, S9 |
| **lake-store/src/s3.rs** | `S3Store` wrapping `aws-sdk-s3` (~60 lines): paginated `list_objects_v2` → `ObjectMeta` with **quotes stripped from etags**; `put_object` passthrough | the SDK, read whole | S7, T3 |
| **crates/lake-sync/src/plan.rs** | pure: local walk (glake's `jsonl_files`) + remote `list` → `Plan { upload, skip }`; key = `raw/` + path-relative-to-root; md5 via a tiny pure-Rust crate; size checked first (cheap), etag second | pure-core/impure-shell layout | S1, S2, S5 |
| **lake-sync/src/run.rs** | executes a Plan against `Arc<S>` (`S: ObjectStore`): streams each file (`BufRead::read_line` into one reused `String`), single PUT per file (owned key + body moved into the task), `JoinSet` + `Arc<Semaphore>(4)` with `acquire_owned` moved into each task | buffer reuse, bounded fan-out (005's backpressure echo), why the spawn forces ownership | S9, T4 |
| **lake-sync/src/main.rs** | clap: `--bucket`, `--dry-run`, `--prefix raw/`, path; stderr/exit discipline per glake | — | S2, S5 |
| **crates/hello-lambda** (evolved) | the sink swap happens **at the emit seam 006 built**: the returned line goes to `store.put(ingest_key(event), line)` instead of `println!` (S6b's supersession, one module); `aws_config` + `S3Store` built once in `main` **before** `lambda_http::run` (async construction happens in plain `async main`, then parked in a `OnceLock`), bucket name from env `LAKE_BUCKET` (set by CDK) | client-once cold-start discipline | S6, S6b, T3 |
| **infra/lib/stateful-stack.ts** | the two buckets (SSE-S3, BPA on, RETAIN, termination protection); the stateless stack imports the lake bucket and attaches a **hand-written `PolicyStatement`** (`s3:PutObject` on `arn:…:goldeneye-lake/raw/*`) — explicit over `grantPut`, whose bundled extras would falsify S10's "nothing wider" | — | S10 |
| **datalake/queries/duckdb/** | `NN-<consumer>.sql` in `local/` and `s3/` variants + README (httpfs install, credential chain note); s3 variants use the pinned glob `raw/dt=*/*.json*` (valid because every object body is newline-delimited JSON) and the reconciliation identities from S11; each header names the doc it feeds | — | S11 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Test seam | mock the SDK (smithy mocks) · localstack/docker · **own trait + in-memory fake** | own trait | the learner *designs* the seam (T1) instead of consuming a mocking framework; properties stay dependency-free and fast; localstack teaches ops, not Rust |
| Trait spelling | bare `async fn` sugar · `trait_variant::make(Send)` · **desugared RPITIT with explicit `+ Send`** | desugared RPITIT | the sugar cannot promise `Send` on stable, and `JoinSet::spawn` (and lambda_http's service bounds) demand it — the bare-sugar version is a guaranteed compile failure at exactly the artifact this unit teaches. Writing `impl Future<…> + Send` by hand IS the async-fn-in-trait lesson; a proc-macro would hide it |
| Trait dispatch | `dyn ObjectStore` · **generic `S: ObjectStore`** | generic | RPITIT isn't dyn-friendly without boxing helpers; sync has exactly two impls chosen at compile time — the honest F13 answer. The *contrast* with 004's CLI `dyn` seam is called out in sitting R |
| Crate layout | one crate, fake in prod surface · hello-lambda depends on lake-sync · **three crates: `lake-store` seam + two consumers** | three crates | a deployable must not depend on a CLI's dep tree (clap/md5 would pollute the Lambda build and confound S8's measurement); the fake ships feature-gated, not in anyone's default surface; the constitution reserves shared crates for exactly this |
| Idempotence check | remember state locally (manifest file) · always upload · **remote list + size/md5-etag compare** | list+compare | the store itself is the source of truth (no second thing to corrupt); teaches etag semantics incl. the multipart caveat; one LIST is cheaper than N heads at this scale |
| Sync upload unit | per-event objects · **per-file objects (mirror the local layout)** | per-file | keys mirror `dt=*/events.jsonl` exactly — `glake` and DuckDB read both sides identically; per-event explodes object count for zero query gain |
| Ingest key | append to a shared object (impossible) · per-day rollup (needs read-modify-write races) · **`evt-<event_id>.json` per event** | per event | S3 has no append; event_id key makes retries self-idempotent; DuckDB globs both shapes with one pattern |
| md5 dependency | hand-roll (unsafe curiosity) · openssl · **`md5` crate (pure Rust, tiny)** | md5 crate | it's a compatibility checksum here, not security (said in a comment — C-GOOD-ERR of crypto hygiene); pure Rust keeps the ARM64 cross-build trivial |
| Concurrency bound | unbounded `join_all` · **`JoinSet` + `Semaphore(4)`** | JoinSet+Semaphore | 005's bounded-channel lesson re-cast for fan-out; the fake's gauge makes the bound *testable* (S9) |
| SDK features | default (aws-lc-rs TLS) · **rustls/ring feature set if the ARM64 cross-build fights** | default first, documented fallback | aws-lc-rs ships pregenerated aarch64 bindings and built clean in the authoring environment; the fallback is written down so a learner on a stranger machine isn't stranded |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| S3 | ∀ generated lakes: multiset(lines in fake store after sync) = multiset(lines in local files) | 003's line-set generator builds a real tempdir lake (1–8 files across 1–4 `dt=` days **including a `traces/dt=…` arm and a `dt=bad-ts` arm** — the generator's domain matches S1's real domain; 0–40 lines each, incl. blanks/malformed — sync moves *bytes*, not judgments); sync against a fresh fake; compare multisets. ≥256 cases |
| S4 | ∀ generated lakes: second sync ⇒ ledger shows 0 puts; then mutate one file ⇒ exactly the changed keys upload **and S3's equality holds again** | same generator; fake's put-counter reset between passes; mutation arm picks {append a line, **replace a line with an equal-length different line** (forces the etag branch — size alone can't catch it), touch new file}; after the re-sync, re-assert the S3 multiset equality against the now-current local lake |

Both run against the **fake** — that's the point (T2). The real `S3Store`
is covered by compile + review + deploy-day S12, and by the honesty that a
translation layer this thin has nowhere for logic bugs to live (list-map
with quote-stripping, put-passthrough — ~60 lines; the quote-strip is the
one line where a "thin" layer can still lie, which is why the design pins
it by name).

## How we verify

**In-session (the reference is validated with all of these before this gate
is presented — if you're reading this at the gate, it exists and passed):**
- S3/S4 proptests ≥256, red-first against a stub plan/run core.
- S1/S2/S5/S6/S9 example tests (fake store incl. the `fail_on` injection for
  S5's mid-run failure; tempdir lakes; CLI via `CARGO_BIN_EXE`); 006's H4
  property re-run green post-evolution.
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
| 2026-07-10 | Rev 2 per design+tasks audit (58%): trait respelled as desugared RPITIT with explicit `+ Send` (M1 — bare sugar cannot satisfy JoinSet's spawn bound on stable; the fix IS the lesson) + ownership shape pinned (Arc<S>, owned keys, acquire_owned); etag cluster fixed (M2 — quote-strip named, fake computes bare-hex md5, same-size mutation arm, SSE-S3-only premise stated); three-crate layout with feature-gated fake (M3 — S8 measurement de-confounded); key derivation rule + pinned glob + supersession task inherited from requirements rev 2 (M4); validation claims re-tensed (M5); gauge moved inside put (D1); fail_on injection (D2); hand-written PolicyStatement over grantPut (D3) | 007 audits | this combined gate |

</details>
