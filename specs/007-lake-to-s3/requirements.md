# Requirements — 007 lake-to-s3 (the lake gets a home)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Three deliverables, one idea: **the lake becomes durable, without ever
becoming magical.**

```console
$ lake-sync --bucket goldeneye-lake --dry-run datalake/raw-local
plan: upload 7, skip 0

$ lake-sync --bucket goldeneye-lake datalake/raw-local
uploaded 7, skipped 0

$ lake-sync --bucket goldeneye-lake datalake/raw-local     # run it again
uploaded 0, skipped 7                                       # idempotent — proven, not vibes

$ duckdb -c "SELECT event_type, count(*) FROM read_json_auto(
    's3://goldeneye-lake/raw/dt=*/events.jsonl', format='newline_delimited')
  GROUP BY 1 ORDER BY 2 DESC"                               # scan.sh, but over S3
```

1. **`lake-sync`** (you write it): walks your local lake with your own glake
   walker and uploads what's missing to `s3://goldeneye-lake/raw/…`. The
   trick that makes it *testable without AWS*: S3 hides behind a **trait you
   design** (`ObjectStore`), with an in-memory fake for tests and the real
   `aws-sdk-s3` behind the same seam. Your 004 trait lesson becomes a
   testing superpower: two property tests prove conservation and idempotence
   against the fake, at 256+ generated lakes.
2. **The ingest Lambda**: 006's handler, sink upgraded — accepted events
   become S3 objects (`raw/dt=<day>/evt-<event_id>.json`) instead of log
   lines. Same door, same 202/400, same property pinning it.
3. **The stateful stack + scan pack**: `goldeneye-lake`/`goldeneye-discovery`
   buckets in their own termination-protected CDK stack (never torn down
   with the compute), and DuckDB SQL that runs the standing queries over
   `s3://` — validated locally first against `datalake/raw-local`.
   (Cost, said plainly: the retained buckets at this volume run to **cents
   per month** — that ongoing cost is part of what you're approving at this
   gate. Everything else still tears down same-day.)

And the headline **measurement**: adding `aws-sdk-s3` to the 006 binary.
The constitution says "~10 MB+, check cargo-bloat" — you'll produce the real
before/after numbers on the real ARM64 artifact.

**What runs where (honesty box):** authored with no AWS credentials. *(local)*
lines were validated in-session — properties against the fake, both real
binaries built for ARM64, `cdk synth` + cdk-nag, DuckDB against the local
lake. *(deploy day)* lines run in sitting U on the learner's account.

## What we're *not* building yet

- Parquet/compaction/cataloging (Glue, Athena) — raw JSONL zones only; the
  Parquet curriculum is the memlens thread's future unit.
- Two-way sync or S3-as-source-of-truth — local stays the working copy;
  S3 is the durable home and query target.
- Multipart upload (files are KB-scale; single PUT; the multipart etag
  caveat is *taught* where idempotence checks meet it, not implemented).
- **Deletion or drift-repair** — sync is one-way and additive; the local
  lake never deletes files in Phase 1, so remote orphans cannot arise; a
  local deletion would simply leave the S3 copy in place (stated so the
  conservation law's domain is explicit).
- **Concurrent sync runs** — one operator, one machine; last-writer-wins on
  identical content is harmless; anything smarter is out of scope.
- Any cross-region story (us-east-1 only; ap-northeast-1 parity is a later
  verification unit).

## What you'll learn building it

- **T1 · A trait as a seam** — design `ObjectStore` (native async fn in
  trait), implement it twice: `FakeStore` for laws, `S3Store` for reality.
- **T2 · Property-testing side effects** — generate whole little lakes on
  disk; assert conservation and idempotence against the fake's ledger.
- **T3 · The SDK, measured** — aws-config/client model, why the client is
  built once (cold start), and the size bill on the real artifact.
- **T4 · Streaming + buffer reuse** — read files line-streamed with one
  reused buffer; bounded-concurrency uploads (a 005 channel-lesson echo).
- **T5 · Idempotence as design** — identity-derived keys (path for sync,
  `event_id` for ingest) and etag/size compare; why "safe to run twice" is
  an architecture property, not a flag.
- **T6 · Query federation** — the same SQL over local paths and `s3://`;
  what DuckDB needs (httpfs, credentials) and what it doesn't (a server).

---

## Precise acceptance criteria

> Tags: **[P]** property · **[E]** example · **[O]** operational.
> Scope: *(local)* validated in-session · *(deploy day)* sitting U →
> evidence.md.

**Layout & plan** *(local)*
- **[E] S1** — key derivation, normative: sync maps every file the glake
  walker finds to `raw/` + its **path relative to the lake root** — so
  `dt=<day>/events.jsonl` → `raw/dt=<day>/events.jsonl` and the real lake's
  `traces/dt=<day>/<file>.jsonl` → `raw/traces/dt=<day>/<file>.jsonl`,
  verbatim, `dt=bad-ts` included (relay's quarantine partition is legal lake
  content and syncs like any other). Ingest writes
  `raw/dt=<day>/evt-<event_id>.json`. Nothing is ever written outside
  `raw/`.
- **[E] S2** — `--dry-run` prints the upload/skip plan and performs zero
  puts (fake's ledger shows none).

**The laws** *(local)*
- **[P] S3** — sync conservation: for any generated local lake, after sync
  the multiset of lines across the fake store's objects equals the multiset
  of lines across the local files — nothing lost, duplicated, or invented.
  The store holds **one object per key, and a re-put replaces it** (real S3
  semantics; the fake must model this — it's what keeps the law true across
  re-syncs).
- **[P] S4** — idempotence: an immediate second sync performs **zero** puts;
  mutating or adding one local file and re-syncing uploads exactly the
  changed/new objects and no others — where mutation arms include a
  **same-size change** (so the checksum branch of the compare is
  load-bearing, not dead code behind the size check) — and after that second
  sync, **S3's conservation equality holds again** against the now-current
  local lake.

**Failure behavior** *(local)*
- **[E] S5** — unreadable path → one stderr line, exit 2 (glake's error
  discipline); a put that fails mid-run → the object is named on stderr,
  exit is non-zero, and already-uploaded objects are reported (partial
  progress is stated, not hidden).

**The ingest Lambda** *(local tests; e2e is U)*
- **[E] S6** — an accepted event results in exactly one `put` at its S1 key
  (fake-store test); rejected bodies produce zero puts; the 202/400 door
  behavior is unchanged (006 H4 property still green after the evolution).
- **[E] S6b** — **supersession, explicit:** the put **replaces** 006's
  stdout event line (H1's sink retires; the emit seam is where the swap
  happens), and the evolved crate legitimately breaks 006 H7's "no
  `aws-sdk-*`" rule. Both 006 lines get change-protocol amendments — logged
  in 006's changelog — when sitting T lands the evolution; until then 006
  stands as approved for its own sittings, which run first.

**Hygiene & review** *(local)*
- **[O] S7** — `ObjectStore` reviewed against the API rubric (C-COMMON-TRAITS
  on its data types, C-GOOD-ERR on its error); `aws-sdk-s3` appears **only**
  in the real impl module; the sync core is generic over the trait
  (monomorphized — the F13 pattern, second appearance); the review also
  checks T4's streaming discipline (one reused line buffer, no
  whole-file-into-String reads) and notes that the checksum crate is pure
  Rust used for compatibility, not security (Graviton hardware-backend
  checklist: n/a by construction, recorded).
- **[O] S8** — the size bill: `cargo lambda build --release --arm64` for the
  006 baseline vs the 007 ingest; both sizes + `cargo bloat` deltas recorded
  in evidence.md; if the delta departs wildly from the research reports'
  ~10 MB expectation, say so and explain.
- **[E] S9** — uploads run with bounded concurrency (≤4 in flight; the
  fake's gauge proves the bound is respected under a many-file lake).

**Infrastructure** *(local synth; deploy is U)*
- **[O] S10** — the stateful stack: `goldeneye-lake` + `goldeneye-discovery`
  (the only hardcoded physical names allowed; the discovery bucket is
  created now so the lake's stateful footprint is born complete — its
  research/insight-zone purpose is Wave 3 per `datalake/README.md`, and no
  Phase-1 requirement writes to it), versioning off / SSE-S3 on / all
  public access blocked, `RemovalPolicy.RETAIN` + termination protection,
  split from the stateless stack; the ingest function's role carries an
  **explicit policy statement**: `s3:PutObject` on
  `goldeneye-lake/raw/*` — written by hand, not via a broad grant helper,
  so the least-privilege claim is literally readable in the template;
  `cdk synth` + cdk-nag (both packs) clean-or-suppressed.

**The scan pack** *(local against the real local lake; s3 variant is U)*
- **[O] S11** — `datalake/queries/duckdb/` holds the standing queries in two
  variants (local path, `s3://` path), each header naming the doc it feeds
  (telemetry rule). The S3 variants read **both** key shapes with the
  pinned glob `raw/dt=*/*.json*` plus `raw/traces/…` where a query wants
  traces (valid because every object body is newline-delimited JSON —
  ingest writes exactly one compact line). Verification is a
  **reconciliation identity, not naive equality** (the 003 R10 lesson):
  scan-pack event totals + trace-file line counts = `glake stats` total.
  The local variants were executed in-session against `datalake/raw-local`
  with that identity shown; outputs recorded.

**Deploy day** *(all deploy day)*
- **[O] S12** — stateful stack deployed (buckets exist, protection on);
  real backlog synced with real numbers matching the dry-run plan; re-run
  shows `uploaded 0`; ingest deployed, one curl → one object in
  `raw/dt=…/`; DuckDB-over-S3 runs the scan pack and reconciles:
  S3-side event count = local count **+ events ingested via curl since the
  sync** (the identity is written into evidence with its terms); stateless
  stack torn down same sitting, **buckets retained**; costs + timestamps →
  evidence.md.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-10 | Initial fast-path draft (with design+tasks); scope tags per the no-credentials reality; fake-store law-testing pattern chosen so [P]s stay local | Part-6 directive (full loop) | pending combined ack |
| 2026-07-10 | Rev 2.1 (reference reconciliation): dry-run example drops its "(N events)" decoration — sync moves bytes, not judgments, and the plan doesn't parse lines | reference build | this combined gate |
| 2026-07-10 | Rev 2 per requirements audit (68%) + design audit (58%): S11/S12 rewritten as reconciliation identities with the pinned two-shape glob (M1 — the 003 R10 lesson, nearly re-learned); S1 key rule = raw/ + path-relative-to-root, `traces/` and `dt=bad-ts` covered verbatim (M2); S6b states the 006 H1/H7 supersession + change protocol (M3); S3/S4 pin put-replaces-key, the same-size mutation arm, and conservation-after-re-sync (M4, design M2); S10 explicit hand-written PutObject policy + discovery-bucket purpose; deletion/concurrency domain stated; retained-bucket cost in plain words; T4/T5 wording fixed | 007 audits | this combined gate |

</details>
