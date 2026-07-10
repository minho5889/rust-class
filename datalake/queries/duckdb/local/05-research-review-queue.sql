-- 05-research-review-queue.sql
-- feeds: research/ (re-verify routine)
--        (telemetry rule: every standing query names the doc it feeds)
-- Volatile findings with their review_by dates — check them
-- against today; past-due claims get re-verified, not trusted.
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT payload ->> 'topic'                  AS topic,
       payload ->> 'review_by'              AS review_by,
       substr(payload ->> 'claim', 1, 100)  AS claim
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', payload: 'JSON'})
WHERE event_type = 'research.finding'
  AND (payload ->> 'volatility') = 'volatile'
ORDER BY review_by, topic;
