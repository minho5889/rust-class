-- 02-memory-gate-funnel.sql
-- feeds: MEMORY.md (pipeline tuning)
--        (telemetry rule: every standing query names the doc it feeds)
-- Gate funnel per spec: notifications sent vs approvals landed.
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT spec_id,
       count(*) FILTER (event_type = 'gate.notified') AS notified,
       count(*) FILTER (event_type = 'gate.approved') AS approved
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', spec_id: 'VARCHAR'})
WHERE event_type LIKE 'gate.%'
GROUP BY spec_id
ORDER BY spec_id;
