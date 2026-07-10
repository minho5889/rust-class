-- 01-sanity-event-inventory.sql
-- feeds: sanity (scan.sh section 1)
--        (telemetry rule: every standing query names the doc it feeds)
-- Event inventory — what does the lake hold, by type?
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT event_type, count(*) AS n
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR'})
GROUP BY event_type
ORDER BY n DESC, event_type;
