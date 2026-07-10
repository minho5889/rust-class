-- 05-research-review-queue.sql
-- feeds: research/ (re-verify routine)
--        (telemetry rule: every standing query names the doc it feeds)
-- Volatile findings with their review_by dates — check them
-- against today; past-due claims get re-verified, not trusted.
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

SELECT payload ->> 'topic'                  AS topic,
       payload ->> 'review_by'              AS review_by,
       substr(payload ->> 'claim', 1, 100)  AS claim
FROM read_json('s3://goldeneye-lake/raw/dt=*/*.json*',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', payload: 'JSON'})
WHERE event_type = 'research.finding'
  AND (payload ->> 'volatility') = 'volatile'
ORDER BY review_by, topic;
