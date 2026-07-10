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
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
WITH terms AS (
  SELECT
    (SELECT count(*) FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR'})) AS process_events,
    (SELECT count(*)
     FROM read_json('datalake/raw-local/traces/dt=*/*.jsonl', format = 'newline_delimited',
                    columns = {event_id: 'VARCHAR'})) AS trace_rows,
    (SELECT sum(length(content) - length(replace(content, chr(10), '')))
     FROM read_text('datalake/raw-local/traces/dt=*/*.jsonl')) AS trace_newlines
)
SELECT process_events, trace_rows, trace_newlines,
       process_events + trace_rows AS identity_total -- = `glake stats` total
FROM terms;
