# Intent — 007 lake-to-s3

> **Doc 1 of 4. WRITE-ONCE.** Audited post-hoc by `intent-assurance`
> (`_assurance/intent-review.md`). No human gate.

**Created:** 2026-07-10 · **Spec status:** active

## Raw prompt (verbatim)

> "do part 5 and part 6 as well if we do not have spec then make them. Do not
> make me to intervene this is full loop you have full green lgihts."

Same message as 006's intent — "part 5 and part 6" covers both. This doc
records the Part-6 half.

## Distilled intent

Author **Part 6 of the course** ahead of time: spec 007 from the approved
Phase-1 plan — *"CDK stateful stack (lake buckets) + Rust ingest Lambda +
backlog sync + DuckDB-over-S3 scan pack; aws-sdk-s3, batching, buffer reuse,
streaming serde."* The goldeneye lake gets its **real home**: the
`goldeneye-lake` / `goldeneye-discovery` buckets (the only two hardcoded
names the constitution allows) in a termination-protected stateful stack;
a **`lake-sync`** CLI the learner writes to push the local
`datalake/raw-local` backlog to S3 idempotently; 006's Lambda evolved so
accepted events land as S3 objects instead of log lines; and a **DuckDB scan
pack** so the standing queries run over `s3://` the way `scan.sh` runs over
local files. Wave 2 of the original data-lake plan, arriving as a class unit
exactly as `datalake/README.md` promised.

**Full-loop directive:** same as 006 — build docs, audits, reference,
guides, critic autonomously; present acks at the end.

## Learning goal

SKILLS **2e** (the goldeneye data lake on S3) deepening **2a** (the SDK
inside Lambda) and **1c** (a trait as a seam: fake vs real `ObjectStore`,
property-tested against the fake — 004's lesson turned into a testing
superpower). The headline measurement: **what does `aws-sdk-s3` cost** the
006 binary (size, cold-start implications) — the constitution's "~10 MB,
check cargo-bloat" rule, measured instead of quoted.

## Assumptions made

- Same environment reality as 006: **no AWS credentials in the authoring
  session**. Everything S3-shaped is validated against a learner-written
  in-memory fake (plus compile-level validation of the real SDK impl and the
  real ARM64 artifact); bucket creation, the real backlog sync, ingest e2e,
  and DuckDB-over-S3 are deploy-day moves (sitting U) on the learner's
  account. DuckDB queries are validated in-session against the **local**
  lake via the installed duckdb engine; the s3:// variants differ only in
  path + httpfs setup.
- The ingest Lambda **evolves `crates/hello-lambda` in place** (same
  deployable, its sink grows up); no new crate name mid-course.
- The lake bucket is **retained** after the sitting (it IS the lake —
  stateful stack, termination-protected); the stateless stack still tears
  down same-day. Ongoing cost ≈ cents at this volume; flagged to the
  learner at the gate (AI proposes, human disposes).
- Sync direction is one-way local→S3 (the local lake stays the working copy
  in Phase 1; S3 is durable home + query target).

## Rejected interpretations

- ❌ Full lake platform (Firehose/Glue/Athena/partitioning jobs) — the plan
  names S3 + DuckDB + two Rust programs; exhaustive AWS coverage is a
  stated non-goal.
- ❌ Parquet conversion now — the memlens→Parquet/DuckDB curriculum is its
  own thread; raw JSONL zones first (datalake/README's own sequencing).
- ❌ Retire the local lake / the hooks — S3 becomes the durable copy;
  local-first telemetry keeps working offline.
- ❌ Deploy from the authoring session — no credentials; deploy day is the
  learner's, by design.

## Addendum (append-only)

_(none yet)_
