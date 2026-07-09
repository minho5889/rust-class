# 005 reference implementation — notes

**What this is:** the validated answer key for spec 005 (`relay/` — relay v0,
lib + thin bin: `main.rs` / `routes.rs` / `validate.rs` / `writer.rs`).
Built against requirements.md + design.md **rev 2**. Not the learner's code;
exists so every sitting checkpoint is validated before the learner reaches it.

## Validation results (2026-07-09)

| Check | default | `--features lens` |
|---|---|---|
| `cargo fmt --check` | pass | — (same source) |
| `cargo clippy --all-targets -- -D warnings` | pass (exit 0) | pass (exit 0) |
| `cargo test` | **14/14** | **14/14** |

Test breakdown (identical both configs): 5 unit (4 door table in
`validate.rs`, 1 day-derivation in `writer.rs`) · 4 routes [E] (A1 ×2 incl.
multiline body, A2, A3) · 2 properties [P] (A4 conservation, A5 partitions —
**256 cases each**, multi-thread runtime, one `tokio::spawn` per request) ·
1 shutdown [E] (A6a/A6b, real child + `kill -INT`) · 2 CLI [E] (A7).
No proptest failure ever occurred, so there is no `proptest-regressions/`
to commit. `cargo tree`: default tree has **zero** memlens (A8/F11-style
check); the direct deps are exactly the A8 allow-list + glake (path) + tower
& proptest as dev-deps.

## Smoke transcript (real binary, scratch lake, curl, SIGINT)

```
$ relay --lake $SCRATCH/lake --port 7311
relay: listening on 127.0.0.1:7311
POST /events (3 valid, distinct days)      → 202, 202, 202
POST /events '{"not":"an envelope"}'       → 400 {"error":"missing key: event_id"}
GET  /healthz                              → {"accepted":3,"received":4,"rejected":1}
kill -INT <pid>                            → exit 0
relay: drained, 3 events on disk. bye.
lake: dt=2026-07-05/events.jsonl (1 line) · dt=2026-07-09/events.jsonl (2 lines)

$ glake stats $SCRATCH/lake        # the REAL 004 reference binary
2 files · 3 events
by type: bolt.done 1 · gate.approved 1 · session.start 1
by day:  2026-07-09 2 · 2026-07-05 1
```

glake's total (3) == relay's accepted counter (3); per-day map matches the
partition dirs exactly. Quiescent invariant holds: 4 = 3 + 1.

## Lens run (A10 groundwork; debug build, trace redirected out of the repo)

`--features lens`, `MEMLENS_TRACE=$SCRATCH/relay-lens/trace.jsonl`, 36 valid
POSTs across 3 days, SIGINT, exit 0, `drained, 36 events`:

- trace: 7,414 lines — 3,694 allocs / 3,621 deallocs / 98 reallocs / 1 meta;
  ~1.26 MB total allocated, net live at exit ≈ 73 blocks (runtime globals +
  the per-day file cache, freed past the last flush).
- ≈ **103 allocs per accepted event** end-to-end (axum request machinery +
  two serde parses + the re-serialized `String` + writer bookkeeping).
  Visible size clusters: 632 B ×217 (per-request buffers), plus small-string
  noise (5–48 B) from JSON keys/values.
- This is a *reference observation*; the learner's own A10 eyeballing
  session (runtime vs handler vs writer in the viewer) is task 1.7 and goes
  to `evidence.md`.

## Semantic decisions beyond the spec text

1. **Blank body → "not a json object".** glake-the-reader distinguishes
   `Blank` from `Unparseable` (a scan must skip blank lines silently); the
   door folds both into the same 400, since the requirements' door check
   clause (1) is "parses as a JSON object" and whitespace doesn't.
2. **The channel speaks `String`, so the writer re-derives the day.** The
   design pins `AppState { tx: mpsc::Sender<String> }`; the handler
   therefore drops `Accepted.day` and the writer re-parses the line and
   applies glake's `day_of_ts` (same one rule, door and writer can't
   drift). See friction #1.
3. **The unreachable no-day line quarantines to `dt=bad-ts`.** Only
   door-validated lines enter the channel, so the writer's `None` day
   branch can't fire — but A9 forbids unwrap, so instead of panicking (or
   silently dropping, which would break A4 conservation) it writes to a
   visible `dt=bad-ts` folder and logs an error.
4. **Two parses per accepted body.** `SerdeParser` parses internally but
   its trait boundary returns only the owned verdict, not the `Value`; the
   door parses again for the compact re-serialization. Chosen over
   extending glake's API (A8 says *reuse*, and 004 is closed).
5. **`send()` failure → 500, counted as `rejected`.** Unreachable (the
   writer outlives the router by construction), but the branch exists
   without unwrap and keeps the quiescent invariant `received = accepted +
   rejected` true on every path.
6. **Writer I/O error after a 202: log loudly, drop that line, keep
   draining.** A lab tool prefers serving the rest of the queue over
   crashing; the 202 promise is already broken either way. Property runs
   never hit it (temp lake).
7. **`writer::run` returns the written count** — it feeds the
   `relay: drained, N events on disk. bye.` goodbye (the requirements'
   console sketch) and gives tests a second conservation witness.
8. **stdout is protocol, stderr is logs.** Exactly two stdout lines
   (`listening on 127.0.0.1:<real port from local_addr()>`, `drained, …`);
   tracing goes to stderr so the A6 test's stdout parse can't be polluted.
   `--port 0` is supported (OS-assigned) and is how the tests avoid port
   collisions.
9. **`expect_used` denied alongside `unwrap_used`.** A9's text says "no
   `unwrap()`/`expect()`"; `clippy::unwrap_used` alone doesn't catch
   `expect`. Tests re-allow at file/module level.
10. **tokio feature list gains `io-util`** (the `AsyncWriteExt` trait that
    gives `File` its `write_all`/`flush`) — required but not named in the
    dependency sketch.
11. **202 body is empty**; the requirements only show the status line.
    400 bodies are the one-line `{"error":"…"}` JSON exactly as specced.
12. **Temp lakes are hand-rolled** (`env::temp_dir()` + pid + per-process
    counter, cleanup in `Drop`) instead of the tempfile dev-dep — one less
    crate and a Drop lesson. Verified: no `relay-test-*` dirs left behind.
13. **A6a's "late requests are refused" is asserted post-exit**
    (connection refused after the process is gone). Refusal *during* the
    graceful window is real (axum closes the listener on signal) but racy
    to test deterministically; the deterministic form still proves the
    listener died with the process.
14. **Duplicates via generator repeat.** The batch strategy occasionally
    repeats a message exactly (weight 1-in-6, ×2) — multiset equality is
    what catches an accidental dedup, per A4's no-dedup policy.

## Spec-text friction (feed the gate summary)

1. **`Sender<String>` vs the day.** The design's routes row fixes the
   channel item as `String`, but the writer needs the day; the reference
   re-derives it (decision #2/#3). A one-line design amendment — channel
   item = `{day, line}` struct — would remove a parse and the unreachable
   quarantine branch. Worth posing to the learner in sitting M as a
   design-tension question rather than silently deviating; the reference
   follows the letter of the design.
2. **healthz key order.** The requirements sketch shows
   `{"received":42,"accepted":41,"rejected":1}`; serde_json's default map
   sorts keys, so the wire emits `{"accepted":…,"received":…,"rejected":…}`.
   Semantically identical (it's a JSON object); noting in case anyone
   greps the literal.
3. **"flush after every line"** is satisfied with `AsyncWriteExt::flush`
   (buffered bytes handed to the kernel). If the spec ever means
   crash-durability, that's `sync_all()` (fsync) — deliberately not done
   for a localhost lab tool; the writer comment says so aloud.
4. **Bounded capacity 256 has no observable acceptance criterion.** It's
   set (and its rationale taught), but nothing in A1–A7 can distinguish
   256 from unbounded without a test that stalls the writer. Fine for v0;
   noting that the decision row is currently enforced by reading the code.
5. **A2's 413 note** (bodies over axum's default limit) is accepted stock
   behavior per the requirements and intentionally has no test — pinning a
   third-party default would make the suite brittle against axum upgrades.

## Layout

```
_reference/relay/
├── Cargo.toml            # [workspace] opt-out, lints, lens feature
├── .gitignore            # /target, /datalake (runtime exhaust)
├── src/lib.rs            # crate docs: the one-owner diagram
├── src/validate.rs       # the strict door (pure; glake SerdeParser)
├── src/routes.rs         # AppState/Counters, POST /events, GET /healthz
├── src/writer.rs         # the single owner: recv loop, day cache, flush
├── src/main.rs           # thin bin: clap, wiring, shutdown order + footgun
└── tests/
    ├── common/mod.rs     # TempLake, in-process relay, canon(), disk reader
    ├── routes.rs         # [E] A1/A2/A3 (oneshot; real hook envelope fixture)
    ├── prop_relay.rs     # [P] A4/A5 (256 cases, spawn-per-request)
    ├── shutdown.rs       # [E] A6a/A6b (CARGO_BIN_EXE child + kill -INT)
    └── cli.rs            # [E] A7 (--help defaults, exit 2 on bad args)
```
