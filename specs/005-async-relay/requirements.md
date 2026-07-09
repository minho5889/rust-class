# Requirements — 005 async-relay (relay v0)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

A tiny local web service, **relay**, that does one job: receive telemetry
events over HTTP and land them in the lake — safely, even when many arrive at
once.

```console
$ relay --lake datalake/raw-local &
relay: listening on 127.0.0.1:7311

$ curl -s -X POST localhost:7311/events -d @one-event.json
HTTP/1.1 202 Accepted

$ curl -s -X POST localhost:7311/events -d '{"not":"an envelope"}'
HTTP/1.1 400 Bad Request
{"error":"missing key: event_id"}

$ curl -s localhost:7311/healthz
{"received":42,"accepted":41,"rejected":1}

$ kill -INT %1        # ctrl-c: finishes in-flight work, flushes, THEN exits
relay: drained, 41 events on disk. bye.
```

Why this is the lesson: today the shell hooks append to the lake one process
at a time. A web service gets **many requests at the same time** — and two
handlers writing one file concurrently will tear lines. The fix is the most
Rust-shaped idea in this course: don't share the file — **send the line
through a channel to the one task that owns the file**. Ownership transfer
instead of locks. If you can explain why that compiles (and why sharing the
`File` wouldn't), you understand `Send`.

And the door policy differs from glake on purpose: **glake is a tolerant
reader** (malformed lines get counted, never crash the scan) but **relay is a
strict gatekeeper** (malformed events get a 400 and never touch the lake).
Validation reuses *your* glake library — the first stranger to call your
public API is you.

## What we're *not* building yet

- Anything AWS — this exact handler becomes a Lambda in **006**.
- Batching, retries, S3 → **007**. TLS/auth: none — localhost lab tool.
- No queue systems, no databases, no background daemons.

## What you'll learn building it

- **T1 · async/await + tokio** — what `.await` actually does; tasks vs threads.
- **T2 · Channels as ownership transfer** — `mpsc`, the single-writer pattern.
- **T3 · Send/Sync, felt** — why the compiler lets state cross `tokio::spawn`.
- **T4 · Shared state done right** — handler counters vs the owned file.
- **T5 · Graceful shutdown** — drain, flush, then exit; no accepted event lost.
- **T6 · Testing async** — properties that fire concurrent requests.

---

## Precise acceptance criteria

> Tags: **[P]** property · **[E]** example · **[O]** operational.

**The routes**
- **[E] A1** — `POST /events` with a valid envelope (all `REQUIRED_KEYS`
  present, top-level) returns **202** and appends the event as one JSONL line
  to `<lake>/dt=<YYYY-MM-DD>/events.jsonl`, day taken from the event's `ts`.
- **[E] A2** — `POST /events` with an invalid body (not JSON, missing a
  required key, or a `ts` that yields no valid `YYYY-MM-DD` day) returns
  **400** with a one-line JSON error naming the first problem; nothing is
  written to the lake.
- **[E] A3** — `GET /healthz` returns **200** with live counters
  `{received, accepted, rejected}` that add up (`received = accepted + rejected`).

**Concurrency & conservation**
- **[P] A4** — acceptance conservation: for any generated mix of valid and
  invalid events fired **concurrently**, after shutdown: (# of 202 responses)
  = (# of lines on disk); every line on disk parses back as exactly one of
  the accepted events (no torn, merged, or duplicated lines); rejected events
  appear nowhere.
- **[P] A5** — partition correctness: every line lands in the `dt=` folder
  matching its own `ts` day; `glake stats` over the resulting lake agrees
  with the relay's accepted count.

**Lifecycle**
- **[E] A6** — on ctrl-c (SIGINT): the listener stops accepting, in-flight
  requests finish, the writer drains the channel and flushes, then the
  process exits 0. Every event that got a 202 is on disk afterwards.
- **[E] A7** — `--lake <dir>` chooses the lake root (default
  `datalake/raw-local`); `--port <n>` chooses the port (default 7311); binds
  `127.0.0.1` only.

**Hygiene**
- **[O] A8** — dependency policy: `tokio`, `axum`, `serde_json`, `clap`,
  `thiserror`, `anyhow` (bin only), `tracing` allowed; validation logic comes
  from the learner's `glake` lib (path dep), not reimplemented.
- **[O] A9** — no `unwrap()`/`expect()` in handler paths
  (`clippy::unwrap_used` on the crate — first deployable-shaped crate, the
  constitution's Cloudflare lesson applies from here on).
- **[O] A10** — the lens, on a service: one short relay session under
  `--features lens` (POST a few dozen events); tokio-runtime vs handler vs
  writer allocations eyeballed in the viewer, notes → `evidence.md`.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft (with design+tasks); 003/004 audit lessons pre-applied: single-clause EARS, [P] strategies live in design, test-first in tasks, lens optional-dep | Part-4 directive | pending combined ack |

</details>
