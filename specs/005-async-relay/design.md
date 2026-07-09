# Design — 005 async-relay (relay v0)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

One rule shapes the whole service: **the file has one owner.**

```
                    ┌────────────────────── tokio runtime ──────────────────────┐
 curl ─► POST /events ─► handler: validate (glake lib) ─► tx.send(line) ──┐     │
 curl ─► POST /events ─► handler: validate ─► 400, counted, dropped       │     │
 curl ─► GET  /healthz ─► counters (atomics)                              ▼     │
                                                          mpsc channel ─► writer task
                    └──────────────────────────────────────── owns the files ───┘
                                            ctrl-c ─► stop accepting ─► drop tx
                                                      ─► writer drains ─► flush ─► exit
```

Handlers never touch a file. A valid event becomes a `String` line and is
**sent** — ownership and all — through an `mpsc` channel to the one task that
holds the open files. That task writes lines one at a time, so concurrent
requests can't interleave bytes (A4). It's memlens's writer-lock lesson in
async clothes: *order and integrity come from single ownership of the sink.*

Shutdown is the reverse: closing the channel (dropping the last sender) is
itself the shutdown signal — the writer's `recv()` loop ends naturally, it
flushes, and only then does `main` return (A6). No flags, no sleeps.

## The shape

| Part | What it is | Rust you learn | REQs |
|---|---|---|---|
| **main.rs** | `#[tokio::main]`, clap args, build router, spawn writer, `axum::serve(...).with_graceful_shutdown(ctrl_c)` , then `writer.await` | async main, task handles, shutdown ordering | A6, A7 |
| **routes.rs** | `POST /events` + `GET /healthz` handlers; `AppState { tx: mpsc::Sender<Line>, counters: Arc<Counters> }` cloned per request | axum extractors, `State`, why `AppState: Clone + Send + Sync` | A1–A3 |
| **validate.rs** | thin: calls the **glake lib** (`classify` + required keys + day-from-ts); maps its verdict to 202/400 | reusing your own public API | A1, A2, A8 |
| **writer.rs** | the single writer task: `while let Some(line) = rx.recv().await` → append to `dt=<day>/events.jsonl` (create dirs, open-append, per-day handle cache), flush on channel close | ownership transfer, why no `Mutex<File>`, drain-then-flush | A4–A6 |
| **counters** | `Arc<Counters>` of `AtomicU64`s (received/accepted/rejected) | `Arc`, atomics vs `Mutex` for counters | A3 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Concurrent writes | `Mutex<File>` in handlers · **mpsc + single writer task** | channel | the Rust-idiomatic answer and the T2/T3 lesson: transfer ownership, don't share the sink; also keeps handler latency off the disk |
| Door policy | tolerant (glake-style bad-ts bucket) · **strict 400** | strict | a writer that accepts what it can't partition corrupts the lake; tolerant-reader/strict-writer asymmetry is the A2 teaching point |
| Day source | server receipt time · **event's own `ts`** | event ts | lake semantics: partitions mean "when it happened"; receipt time would silently shear late events into wrong days |
| Body format | JSONL batch · **one envelope per POST** | one | smallest teaching shape; hooks emit one event at a time; batching is 007's problem |
| Validation | reimplement · **path-dep on the learner's glake lib** | glake lib | the 004 T4 payoff — first external caller of their own API; drift impossible |
| Shutdown signal | shared `AtomicBool` · cancellation token · **drop the senders** | drop tx | channel close IS the drain signal; zero extra state, teaches what channel ownership means |
| Health counters | `Mutex<Counts>` · **`Arc<AtomicU64>`s** | atomics | counters are the textbook atomics case; contrasts with the file, which atomics can't fix |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| A4 | ∀ concurrent mixed batches: #202 = #lines on disk ∧ each line round-trips to exactly one accepted event ∧ rejected ∉ disk | proptest generates `Vec<Msg>` (valid envelopes with distinct `event_id`s ∪ 004's malformed strategies); test body builds the router with a **temp lake**, `block_on`: fire all via `tower::ServiceExt::oneshot` under `join_all`, trigger shutdown, then assert on disk. ≥256 cases |
| A5 | ∀ accepted events: on-disk `dt=` folder = `ts[0..10]` ∧ `glake stats` total = accepted count | same generator; after A4's run, walk the temp lake with the glake lib and compare per-day maps |

Async×proptest is a co-written hard bit: proptest drives a **sync** test body
that owns a `tokio::runtime::Runtime` and `block_on`s the async scenario —
generation stays deterministic, only the scenario is async. Torn lines may not
reproduce under `join_all`'s cooperative scheduling even with a broken
implementation (single-threaded poll order can hide the race) — which is
itself a T6 lesson: A4 red-first runs against the naive `Mutex<File>`-less
*shared-append* strawman in sitting M to see a real failure, then the channel
makes it pass. If the strawman won't tear under the test runtime, the sitting
records that honestly and the property still pins conservation.

## How we verify

- A4/A5 proptests (≥256 cases, seeds committed) written before the writer task.
- [E]s as `#[tokio::test]` examples: A1/A2/A3 via `oneshot`, A6 via spawning
  the real server on port 0 + SIGINT-equivalent (trigger the shutdown future),
  A7 via clap `CARGO_BIN_EXE` run.
- [O]s recorded in evidence: A8 `cargo tree` check, A9 clippy lint on, A10
  lens session numbers.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-4 directive | pending combined ack |

</details>
