-- 08-evidence-reconciliation.sql
-- feeds: specs/007-lake-to-s3 evidence.md (S11/S12 identity)
--        (telemetry rule: every standing query names the doc it feeds)
-- The reconciliation IDENTITY (the 003 R10 lesson: reconcile,
-- don't naively compare): process events + trace lines must
-- equal `glake stats <lake-root>`'s total, because glake walks
-- BOTH partitions while the standing queries read them apart.
-- trace_rows counts parsed JSON rows, trace_newlines counts
-- raw newlines — equal exactly when every trace line is one
-- JSON object, which is itself worth watching.
-- Variant: S3 (s3://goldeneye-lake). NOT runnable in the authoring
-- session (no AWS credentials there — see NOTES.md); first live run
-- is deploy day, sitting U (S12). Differs from local/ ONLY in the
-- preamble and the FROM paths — the logic is generated from one
-- source (this is checkable: diff the two files).
-- S3 identity terms (S12): process_events here = local count
-- + events curl-ingested since the sync (each ingested event
-- is its own evt-*.json object); state both terms when
-- recording the identity in evidence.md.
-- ── s3 preamble ──────────────────────────────────────────────────────
-- httpfs speaks https/s3; the secret pulls whatever the ambient AWS
-- credential chain has (env vars, ~/.aws, SSO) — same chain lake-sync
-- and the CLI use. Region must match the bucket (us-east-1).
INSTALL httpfs;
LOAD httpfs;
CREATE OR REPLACE SECRET goldeneye (TYPE s3, PROVIDER credential_chain);

WITH terms AS (
  SELECT
    (SELECT count(*) FROM read_json('s3://goldeneye-lake/raw/dt=*/*.json*',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR'})) AS process_events,
    (SELECT count(*)
     FROM read_json('s3://goldeneye-lake/raw/traces/dt=*/*.jsonl', format = 'newline_delimited',
                    columns = {event_id: 'VARCHAR'})) AS trace_rows,
    (SELECT sum(length(content) - length(replace(content, chr(10), '')))
     FROM read_text('s3://goldeneye-lake/raw/traces/dt=*/*.jsonl')) AS trace_newlines
)
SELECT process_events, trace_rows, trace_newlines,
       process_events + trace_rows AS identity_total -- = `glake stats` total
FROM terms;
