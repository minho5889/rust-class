# goldeneye data lake

Everything the AI-DLC workflow exhales — agent events, gate decisions, assurance
verdicts, property-test counterexamples, build/deploy metrics, learning events —
captured as envelope events and archived for pattern mining.

```
capture (hooks → raw-local JSONL, committed)          ← v1, THIS REPO, live now
  → s3://goldeneye-lake/raw/       JSONL+zstd, Hive-partitioned   ← Wave 2
  → s3://goldeneye-lake/curated/   Parquet, compacted             ← Wave 2
  → scan: DuckDB (default) / Athena                               ← Wave 2
  → s3://goldeneye-discovery/      promoted insight cards only    ← Wave 3
```

## v1 (current): local raw zone

- Hooks (`.claude/hooks/telemetry.sh`) append events to
  `raw-local/dt=YYYY-MM-DD/events.jsonl` — one JSON object per line, envelope schema
  in `schema/envelope.v1.json`.
- Capture granularity (decision 2026-07-05): `session.start` + spec-doc/gate events
  only. Turn-end capture (`Stop` hook) was removed — it fired every turn, produced
  low-value events, and forced a telemetry-only commit per reply. Turn boundaries
  are recoverable from event timestamps if ever needed.
- Files are **committed with normal work commits** (sessions are ephemeral remote
  containers; git is the durability layer until S3 exists). Private repo; raw
  prompts in intent events are permitted by learner decision (2026-07-05).
- **Never** put secrets/credentials in payloads.

## Conventions (fixed now so Wave 2 doesn't migrate data)

- Project prefix: `goldeneye-`. Regions: us-east-1 (primary), ap-northeast-1
  (secondary). Buckets: `goldeneye-lake`, `goldeneye-discovery` (suffix with account
  id if names are taken, e.g. `goldeneye-lake-123456789012`).
- Partitioning: `event_type=<family>/dt=YYYY-MM-DD/` (Hive-style) in S3; local v1
  partitions by `dt=` only, split by family at Wave-2 sync time.
- Raw is append-only, JSONL. Curated is Parquet (zstd), one table per event family.
- Discovery holds **insight cards only** (markdown + JSON pair, each embedding the
  query that produced it, so every insight is reproducible) — never raw data.
- Lifecycle (Wave 2): raw → IA at 30d → Glacier at 90d.

## Deliberately out of scope

Kinesis/Firehose, dashboards, Lake Formation, Iceberg/Delta — revisit as future
Units of Work only if scale demands.
