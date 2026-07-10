-- 06-specs-open-questions.sql
-- feeds: future specs (question owners)
--        (telemetry rule: every standing query names the doc it feeds)
-- Open research questions by topic.
-- Variant: LOCAL (datalake/raw-local). Run from the repo root.
SELECT payload ->> 'topic' AS topic, count(*) AS open_questions
FROM read_json('datalake/raw-local/dt=*/events.jsonl',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', payload: 'JSON'})
WHERE event_type = 'research.open_question'
GROUP BY topic
ORDER BY open_questions DESC, topic;
