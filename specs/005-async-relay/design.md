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

**The one footgun, named up front:** this only works if `main` keeps **no**
`tx` clone of its own. The natural first draft —
`let state = AppState { tx: tx.clone(), … }` with `tx` still alive in `main` —
deadlocks forever at `writer.await`: the writer's `recv()` never returns
`None` because one sender still exists. Move `tx` into `AppState`; when the
serve future finishes and the router drops, the *last* sender drops, and the
drain begins. (This is sitting N's checkpoint question — if you can predict
the hang before running it, you understand channel ownership.)

## The shape

| Part | What it is | Rust you learn | REQs |
|---|---|---|---|
| **main.rs** | `#[tokio::main]`, clap args, build router (moving `tx` in — see footgun), spawn writer, `axum::serve(...).with_graceful_shutdown(ctrl_c)`, then `writer.await`; `#[cfg(feature = "lens")]` global-allocator install (same optional-dep pattern as glake) | async main, task handles, shutdown ordering | A6a/b, A7, A10 |
| **routes.rs** | `POST /events` + `GET /healthz` handlers; `AppState { tx: mpsc::Sender<String>, counters: Arc<Counters> }` cloned per request (each `String` is one formatted JSONL line) | axum extractors, `State`, why `AppState: Clone + Send + Sync` | A1–A3 |
| **validate.rs** | thin: calls the **glake lib**'s public API — classification (verdict + first-missing-key in `REQUIRED_KEYS` order) and the 004 comparable-day rule — and maps the outcome to 202/400 per the requirements' fixed error order | reusing your own public API | A1, A2, A8 |
| **writer.rs** | the single writer task: `while let Some(line) = rx.recv().await` → `tokio::fs` append to `dt=<day>/events.jsonl` (create dirs, open-append, per-day handle cache — a `HashMap<String, File>`, bounded in practice by distinct days seen), flush **after every line** (lab tool: durability over throughput), final flush-all on channel close | ownership transfer, why no `Mutex<File>`, async I/O, drain-then-flush | A4–A6 |
| **counters** | `Arc<Counters>` of `AtomicU64`s (received/accepted/rejected) | `Arc`, atomics vs `Mutex` for counters | A3 |
| **Cargo.toml lints** | `clippy::unwrap_used = "deny"` on the crate (first deployable-shaped crate) | the no-unwrap discipline, mechanized | A9 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Concurrent writes | `Mutex<File>` in handlers · **mpsc + single writer task** | channel | the Rust-idiomatic answer and the T2/T3 lesson: transfer ownership, don't share the sink; also keeps handler latency off the disk |
| Door policy | tolerant (glake-style bad-ts bucket) · **strict 400** | strict | a writer that accepts what it can't partition corrupts the lake; tolerant-reader/strict-writer asymmetry is the A2 teaching point |
| Day source | server receipt time · **event's own `ts`** | event ts | lake semantics: partitions mean "when it happened"; receipt time would silently shear late events into wrong days |
| Body format | JSONL batch · **one envelope per POST** | one | smallest teaching shape; hooks emit one event at a time; batching is 007's problem |
| Channel capacity | unbounded · **bounded (256)** | bounded | `send(line).await` under a full channel makes the handler *wait* — backpressure for free, and the 202 honestly means "enqueued" (requirements A1); unbounded hides overload until OOM |
| Writer I/O | blocking `std::fs` in the task · `spawn_blocking` · **`tokio::fs` + `AsyncWriteExt`** | tokio::fs | teaches async I/O properly (and IS `spawn_blocking` under the hood — said aloud in sitting M); per-line flush keeps A10's lens trace and the 1.7 live demo readable |
| Validation | reimplement · **path-dep on the learner's glake lib** | glake lib | the 004 T4 payoff — first external caller of their own API; drift impossible |
| Shutdown signal | shared `AtomicBool` · cancellation token · **drop the senders** | drop tx | channel close IS the drain signal; zero extra state, teaches what channel ownership means |
| Health counters | `Mutex<Counts>` · **`Arc<AtomicU64>`s** | atomics | counters are the textbook atomics case; contrasts with the file, which atomics can't fix |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| A4 | ∀ concurrent mixed batches: multiset(lines on disk) = multiset(bodies that got 202) ∧ rejected ∉ disk | proptest generates `Vec<Msg>` (valid envelopes — `ts` drawn from a **multi-day domain** — ∪ 004's malformed strategies ∪ occasional exact duplicates); test body owns a **multi-thread** `tokio::runtime::Runtime` and `block_on`s the scenario: each request fired in its own `tokio::spawn` against a cloned router (genuine parallelism — `join_all` over un-spawned `oneshot` futures runs them on one task and is NOT concurrency), temp lake, trigger shutdown, join everything, assert on disk. ≥256 cases |
| A5 | ∀ accepted events: on-disk `dt=` folder = the event's comparable day ∧ `glake stats` total = accepted count | same generator (multi-day domain matters here); after A4's run, walk the temp lake with the glake lib and compare per-day maps |

Async×proptest is a co-written hard bit: proptest drives a **sync** test body
that owns the `Runtime` — generation stays deterministic, only the scenario is
async.

**Honesty about "red", planned as the plan (not a fallback):** the sitting-L
naive handler appends with `writeln!`-style direct writes. On Linux,
`O_APPEND` write(2) calls are atomic for small buffers, so the strawman **may
well survive** the A4 tearing check even under real parallelism — the OS is
quietly saving it. Sitting M teaches exactly that: run A4 against the strawman;
*either* it tears (great — a real red) *or* it holds and we name the savior
(single write syscall + O_APPEND) and why it's not a guarantee you can build
on (two-write formatting, partial writes, retries, non-Linux). The channel
version then passes **by construction**, not by OS mercy. Conservation
(rejected-not-on-disk, nothing lost at shutdown) can still go genuinely red
against the strawman — shutdown-mid-queue is where it visibly loses events.

## How we verify

- A4/A5 proptests (≥256 cases, seeds committed) written before the writer task.
- [E]s: A1/A2/A3 as `#[tokio::test]` via `oneshot`; **A6a/A6b via a
  `CARGO_BIN_EXE` child process sent a real SIGINT** (`kill -INT <pid>`),
  asserting refused-late-request, exit 0, and 202'd-events-on-disk — the
  abstract-shutdown-future shortcut never exercises the actual `ctrl_c`
  wiring; A7 via `CARGO_BIN_EXE` (named test, sitting K).
- [O]s recorded in evidence: A8 `cargo tree` check, A9 lint denied in
  Cargo.toml, A10 lens session numbers.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-4 directive | pending combined ack |
| 2026-07-09 | Rev 2 per design+tasks audit (72%): drop-tx deadlock footgun named + made sitting-N checkpoint (MAJOR-2); A4 concurrency fixed to multi-thread runtime + spawn-per-request, strawman-may-hold honesty promoted from fallback to the plan (MAJOR-1); A6 verified via real child + SIGINT; bounded-channel and writer-I/O decision rows added; 202-means-enqueued aligned with requirements rev 2; lens wiring, A9 lint element, multi-day generator domain, glake API surface named | 005 audits | this combined gate |

</details>
