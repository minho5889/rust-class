# goldeneye data lake — the class's gradebook and lab notebook

Reframed 2026-07-05 (learner-approved): **not a mini enterprise data platform.**
Three workloads, in priority order, each sized honestly:

## 1. memlens traces — the lake's primary citizen

The only data we produce at real volume (~100k events per traced run). This is
where columnar formats genuinely pay off, and where the queries *are* the
curriculum: allocation-size distributions across exercises, `Vec` growth
patterns before/after an optimization, "how did my allocation behavior change
between playground 02 and 07."

- Land in `raw-local/traces/dt=YYYY-MM-DD/` as `memlens.v1` JSONL (spec 002).
- **Wave 2 (S3, as a class unit):** traces → Parquet (zstd) in
  `s3://goldeneye-lake/traces/`, DuckDB query pack over them. Athena/Glue only
  when it teaches something.

## 2. Learning analytics — the mistake ledger (the novel part)

`learning.*` events measure the learner learning: compiler-error classes
encountered (borrow-checker fights!), time-to-green on property tests, triaged
counterexamples, concepts exercised per session. Standing query: *"which error
classes stopped recurring?"* — spaced-repetition input, and **SKILLS.md mastery
promotions get evidence attached instead of vibes.**

## 3. Process telemetry — small forever, no ceremony

Sessions, spec docs, gates, assurance verdicts, research findings. Kilobytes.
**Plain JSONL in git indefinitely** — no curation pipeline, no compaction job
(explicitly killed from the old plan), queryable with jq/DuckDB as-is.

```
capture (hooks + research runs + memlens) → raw-local/dt=YYYY-MM-DD/*.jsonl (committed)
traces (volume)  → Wave 2: s3://goldeneye-lake/traces/ (Parquet) → DuckDB
insights         → insights-local/ now → s3://goldeneye-discovery/ later
```

## Rules

- Envelope schema: `schema/envelope.v1.json`; payload families:
  `schema/research.v1.json`, `schema/memlens.v1.json` (with spec 002).
  **No secrets in payloads, ever.**
- **Research runs must emit events** (`research.run/finding/open_question/
  refuted/adoption`) alongside their markdown report — findings invisible to
  the lake don't exist. Volatility-tag every finding; `volatile` ⇒ `review_by`.
- **Every standing query names the document it feeds** (SKILLS.md, MEMORY.md,
  a spec's evidence.md) — a query with no consumer doesn't get built.
  Query pack: `queries/`. Insight cards: `insights-local/` (md + json pair,
  each embedding its query; promoted to `s3://goldeneye-discovery` in Wave 3).
- Naming/regions unchanged: `goldeneye-lake` / `goldeneye-discovery`,
  us-east-1 primary, ap-northeast-1 secondary (parity unverified).
- Capture granularity (2026-07-05): session.start + spec/gate/research events;
  turn-end capture removed as noise.

## Explicitly out of scope

Process-event curation/compaction, Kinesis/Firehose, dashboards (memory-lens
viewer excepted — it reads traces, not this lake's process events), Lake
Formation, Iceberg/Delta. Revisit only if volume proves the reframing wrong.
