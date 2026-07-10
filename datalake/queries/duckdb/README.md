# DuckDB scan pack — the standing queries, federated (spec 007, S11)

`scan.sh` (next door) runs the standing queries with jq over the local
lake. This pack is the same set of questions as SQL, in **two variants
that differ only in paths**:

| Dir | Reads | Runnable |
|---|---|---|
| `local/` | `datalake/raw-local/…` | anywhere, now — validated in-session against the real local lake (outputs recorded in `specs/007-lake-to-s3/_reference/NOTES.md`) |
| `s3/` | `s3://goldeneye-lake/raw/…` | **only after sitting U deploys the lake** — needs AWS credentials + the DuckDB `httpfs` extension (each file carries its own preamble). NOT runnable in the authoring session, by design (no credentials there). |

That's T6, the whole lesson: the same SQL over local files and `s3://` —
what DuckDB needs is an extension and a credential chain, not a server.

## Rules (the telemetry constitution, applied)

- **Every query names the document it feeds** in its header
  (`-- feeds: …`) — a query with no consumer doesn't get built.
- File naming: `NN-<consumer>-<what>.sql`.
- The events relation pins its **columns explicitly** (`read_json` with
  `columns={…}`, not `read_json_auto`): auto-inference samples the data
  and can type `payload` differently as the lake grows — a standing query
  must not have a weather-dependent schema.

## The pinned S3 glob (S11, normative)

```
s3://goldeneye-lake/raw/dt=*/*.json*
```

matches BOTH key shapes the lake contains after deploy day: lake-sync's
per-file objects (`raw/dt=<day>/events.jsonl`) and the ingest Lambda's
per-event objects (`raw/dt=<day>/evt-<event_id>.json`). Valid because
every object body is newline-delimited JSON — ingest writes exactly one
compact line. Queries that want traces read `raw/traces/dt=*/*.jsonl`
alongside.

## Running

```console
# local (from the repo root; duckdb CLI or the installed python module):
$ python3 -c "import duckdb; print(duckdb.sql(open('datalake/queries/duckdb/local/01-sanity-event-inventory.sql').read()))"

# s3 (deploy day): same, with the s3/ variant — the preamble INSTALLs
# httpfs and creates a credential-chain secret; region is us-east-1.
```

## Verification is a reconciliation identity, not naive equality

(The 003 R10 lesson.) `08-evidence-reconciliation.sql` computes the
identity's left side; the right side is the learner's own walker:

```
process events (dt=*/events.jsonl) + trace lines (traces/dt=*/*.jsonl)
  = `glake stats datalake/raw-local` total
```

because glake walks BOTH partitions while the standing queries read them
apart. In-session run (2026-07-10T23:24Z, instant matters — hooks append
to the lake DURING sessions): **350 + 64 = 414 = glake's `6 files · 414
events`**. On deploy day the S3 identity gains a term: S3-side event
count = local count at sync time **+ events curl-ingested since** (each
becomes its own `evt-*.json` object) — S12 records it with its terms in
evidence.md.
