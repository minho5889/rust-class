-- 07-skills-learning-ledger.sql
-- feeds: SKILLS.md (mastery evidence)
--        (telemetry rule: every standing query names the doc it feeds)
-- The mistake ledger's size: learning.* events captured so far
-- (0 until the coached sittings start logging fights).
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

SELECT count(*) AS learning_events
FROM read_json('s3://goldeneye-lake/raw/dt=*/*.json*',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR'})
WHERE event_type LIKE 'learning.%';
