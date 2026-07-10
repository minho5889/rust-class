-- 06-specs-open-questions.sql
-- feeds: future specs (question owners)
--        (telemetry rule: every standing query names the doc it feeds)
-- Open research questions by topic.
-- Variant: S3 (s3://goldeneye-lake). NOT runnable in the authoring
-- session (no AWS credentials there — see NOTES.md); first live run
-- is deploy day, sitting U (S12). Differs from local/ ONLY in the
-- preamble and the FROM paths — the logic is generated from one
-- source (this is checkable: diff the two files).
-- ── s3 preamble ──────────────────────────────────────────────────────
-- httpfs speaks https/s3; the secret pulls whatever the ambient AWS
-- credential chain has (env vars, ~/.aws, SSO) — same chain lake-sync
-- and the CLI use. Region must match the bucket (us-east-1).
INSTALL httpfs;
LOAD httpfs;
CREATE OR REPLACE SECRET goldeneye (TYPE s3, PROVIDER credential_chain);

SELECT payload ->> 'topic' AS topic, count(*) AS open_questions
FROM read_json('s3://goldeneye-lake/raw/dt=*/*.json*',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', payload: 'JSON'})
WHERE event_type = 'research.open_question'
GROUP BY topic
ORDER BY open_questions DESC, topic;
