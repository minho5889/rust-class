-- 07-skills-learning-ledger.sql
-- feeds: SKILLS.md (mastery evidence)
--        (telemetry rule: every standing query names the doc it feeds)
-- The mistake ledger's size: learning.* events captured so far
-- (0 until the coached sittings start logging fights).
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT count(*) AS learning_events
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR'})
WHERE event_type LIKE 'learning.%';
