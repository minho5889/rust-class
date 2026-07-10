-- 04-research-volatility.sql
-- feeds: research/ (review queue sizing)
--        (telemetry rule: every standing query names the doc it feeds)
-- Research findings by volatility tag.
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT payload ->> 'volatility' AS volatility, count(*) AS findings
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', payload: 'JSON'})
WHERE event_type = 'research.finding'
GROUP BY volatility
ORDER BY findings DESC;
