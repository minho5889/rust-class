-- 03-memory-doc-churn.sql
-- feeds: MEMORY.md (pipeline tuning)
--        (telemetry rule: every standing query names the doc it feeds)
-- Doc-write churn per spec document (a revisions proxy).
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT payload ->> 'file' AS file, count(*) AS writes
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', payload: 'JSON'})
WHERE event_type = 'spec.doc_written'
GROUP BY file
ORDER BY writes DESC, file;
