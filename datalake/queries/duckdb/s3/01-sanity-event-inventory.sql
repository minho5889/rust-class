-- 01-sanity-event-inventory.sql
-- feeds: sanity (scan.sh section 1)
--        (telemetry rule: every standing query names the doc it feeds)
-- Event inventory — what does the lake hold, by type?
-- Variant: S3 (s3://goldeneye-lake). NOT runnable in the authoring
-- session (no AWS credentials there — see NOTES.md); first live run
-- is deploy day, sitting U (S12). Differs from local/ ONLY in the
-- preamble and the FROM paths — the logic is generated from one
-- source (this is checkable: diff the two files).
-- S3 counting term (S12): after deploy day this reads synced
-- events.jsonl AND curl-ingested evt-*.json objects, so the
-- total = local count + events ingested since the sync.
-- ── s3 preamble ──────────────────────────────────────────────────────
-- httpfs speaks https/s3; the secret pulls whatever the ambient AWS
-- credential chain has (env vars, ~/.aws, SSO) — same chain lake-sync
-- and the CLI use. Region must match the bucket (us-east-1).
INSTALL httpfs;
LOAD httpfs;
CREATE OR REPLACE SECRET goldeneye (TYPE s3, PROVIDER credential_chain);

SELECT event_type, count(*) AS n
FROM read_json('s3://goldeneye-lake/raw/dt=*/*.json*',
             format = 'newline_delimited',
             columns = {event_type: 'VARCHAR'})
GROUP BY event_type
ORDER BY n DESC, event_type;
