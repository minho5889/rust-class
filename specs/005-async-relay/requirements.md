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
strict gatekeeper** (events that fail the door check get a 400 and never touch
the lake). "Strict" here means exactly two things — the event must pass
*your glake library's* classification (all required keys present, top level),
and it must carry a `ts` the relay can partition by day. It is **not** full
JSON-Schema validation (no `schema_version` value check, no RFC3339 audit) —
that asymmetry is deliberate and written down below. The first stranger to
call your public API is you.

## What we're *not* building yet

- Anything AWS — this exact handler becomes a Lambda in **006**.
- Batching, retries, S3 → **007**. TLS/auth: none — localhost lab tool.
- No queue systems, no databases, no background daemons.
- **Full JSON-Schema validation** — no `schema_version` value check, no RFC3339
  audit, no payload-shape check. The door is glake's key-presence rule plus a
  partitionable day. Anything tighter waits for a spec that needs it.
- **Deduplication** — the relay appends what it accepts, duplicates included.
  `event_id` uniqueness is the *writer's* promise (hooks generate UUIDs);
  enforcing it is a reader/curation concern, not this door's.
- **Cross-process file locking** — the relay's channel serializes *its own*
  writes; the shell hooks may still append to the same live lake concurrently,
  exactly as they coexist today. Property runs use a temp lake for this reason.

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

> **The door check, defined once** (A1/A2 use it): a body is **valid** iff
> (1) it parses as a JSON object, (2) the glake lib's classification finds all
> `REQUIRED_KEYS` at top level, and (3) its `ts` has a **comparable day** —
> the first 10 characters match the `YYYY-MM-DD` digit pattern (004's F2 rule,
> from the glake lib). Weird-but-valid days (e.g. `1999-01-01`) are accepted;
> partitions mean what the event claims. This is deliberately *stricter than
> glake the reader* (which buckets bad-ts instead of rejecting) and
> deliberately *weaker than the JSON Schema* (see "not building").

**The routes**
- **[E] A1** — `POST /events` with a valid body returns **202** (accepted =
  validated and enqueued for the writer; on disk no later than shutdown — A4,
  A6b) and the event is appended as one JSONL line to
  `<lake>/dt=<day>/events.jsonl`, day from the event's own `ts`. The example
  fixture is a real hook-emitted envelope from the live lake.
- **[E] A2** — `POST /events` with an invalid body returns **400** with a
  one-line JSON error naming the **first** problem in this fixed order:
  not-a-JSON-object → first missing key in `REQUIRED_KEYS` order → no
  comparable day. Nothing is written to the lake. (Bodies over axum's default
  size limit get its stock 413 — accepted as-is, not customized.)
- **[E] A3** — `GET /healthz` returns **200** with counters
  `{received, accepted, rejected}`; **once no requests are in flight**,
  `received = accepted + rejected`. (Mid-flight snapshots may transiently
  disagree — the counters are independent atomics; that's the T4 lesson,
  stated, not hidden.)

**Concurrency & conservation**
- **[P] A4** — acceptance conservation: for any generated mix of valid and
  invalid events fired **concurrently** (temp lake), after shutdown the
  multiset of lines on disk equals the multiset of bodies that received 202 —
  nothing torn, merged, lost, or invented; duplicate submissions yield
  duplicate lines (by design); rejected bodies appear nowhere.
- **[P] A5** — partition correctness: every line lands in the `dt=` folder
  matching its own `ts` day, and `glake stats` over the resulting temp lake
  agrees with the relay's accepted count.

**Lifecycle**
- **[E] A6a** — on ctrl-c (SIGINT): the listener stops accepting new
  connections (late requests are refused) and in-flight requests complete.
- **[E] A6b** — after A6a, the writer drains the channel, flushes, and the
  process exits 0; every event that got a 202 is on disk afterwards.
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
| 2026-07-09 | Initial fast-path draft (with design+tasks) | Part-4 directive | pending combined ack |
| 2026-07-09 | Rev 2 per requirements audit (68%): the door check defined once (glake key-presence + comparable day; explicitly weaker than the JSON Schema, stricter than glake-the-reader — MAJOR-1); A3 pinned to quiescent consistency (MAJOR-2); A4 rewritten as multiset equality + explicit no-dedup policy (MAJOR-3); A2 gets a fixed first-problem order + 413 note; A6 split into A6a/A6b with late-request behavior; cross-process hook coexistence + temp-lake rule stated; A1 fixture = real hook envelope | 005 audits | this combined gate |

</details>
