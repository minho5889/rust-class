-- 02-memory-gate-funnel.sql
-- feeds: MEMORY.md (pipeline tuning)
--        (telemetry rule: every standing query names the doc it feeds)
-- Gate funnel per spec: notifications sent vs approvals landed.
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

SELECT spec_id,
       count(*) FILTER (event_type = 'gate.notified') AS notified,
       count(*) FILTER (event_type = 'gate.approved') AS approved
FROM read_json('s3://goldeneye-lake/raw/dt=*/*.json*',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR', spec_id: 'VARCHAR'})
WHERE event_type LIKE 'gate.%'
GROUP BY spec_id
ORDER BY spec_id;
