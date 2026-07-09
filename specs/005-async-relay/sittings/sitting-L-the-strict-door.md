# Sitting L — the strict door

**Builds:** `POST /events` — the door: your glake library's verdicts mapped to
202/400 in the requirements' fixed error order, a naive direct append behind
it (deliberately — Sitting M puts it on trial), live counters behind
`/healthz` via `Arc<Counters>` atomics, and the A1/A2/A3 example tests driven
in-process with a real hook-emitted envelope as the fixture.
**Requirements:** A1, A2, A3 — task 1.2 (T4: shared state done right).
**Ramp you'll use:** step 13 move 6 (`Arc`, the license to share) and step 2
(ownership) — plus 004's sittings H (`thiserror`) and J (`SerdeParser`) doing
most of the lifting.

> **Pacing:** two ideas share this sitting — the door check (moves 1–5) and
> the counters (moves 6–8) — and the guide paces them as two halves with a
> marked break point after move 5. Split across two sessions freely if the
> first half runs long; each half ends on a green commit.

## Where you are

Sitting K left a service that listens and answers one static URL. Today it
earns its keep: events arrive over `POST /events`, and relay becomes the
first *external caller* of the glake API you built in 004 — the "stranger"
that public API design kept talking about is you, one crate over. The policy
is deliberately asymmetric and worth saying before any code: **glake is a
tolerant reader** (malformed lines get counted, never crash a scan) but
**relay is a strict gatekeeper** (what fails the door gets a 400 and never
touches the lake) — a writer that accepts what it can't partition corrupts
the lake for every future reader. One honest confession up front: today's
handler will append to the lake *directly*, the obvious way, as if requests
arrived one at a time. That single-request-world assumption is this sitting's
strawman — real and deliberate; Sitting M fires two hundred concurrent
requests at it and you'll watch what happens. Build it well anyway: everything
except the append survives M untouched.

## The build, move by move

All commands from the repo root. Two commits this sitting: the door, then
counters + tests.

1. **Read the door's definition, then predict.** Open
   `specs/005-async-relay/requirements.md` and read two things: the boxed
   paragraph "The door check, defined once" and the A2 line's fixed error
   order (not-a-JSON-object → first missing key in `REQUIRED_KEYS` order →
   no comparable day). Now predict the door's verdict for each of these,
   out loud, before writing any code:

   - a valid envelope, but pretty-printed across 12 lines;
   - a body missing `actor` *and* carrying `"ts":"nope"` — which error wins?
   - a valid envelope with `"ts":"1999-01-01T00:00:00Z"`;
   - a body that is only whitespace;
   - a valid envelope with an extra key glake never heard of (`spec_id`).

   Check yourself against the requirements text (the answers are all in it:
   accepted-and-flattened; missing key wins — order is fixed; accepted —
   partitions mean what the event claims; not-a-json-object; accepted — the
   door checks keys are *present*, it doesn't forbid more). If any answer
   surprised you, that's the reading this sitting exists for.

2. **The door as a pure function.** New module: `crates/relay/src/validate.rs`
   — and a decision first. K built relay as a plain binary; give it the
   lib/bin shape now (a `src/lib.rs` declaring `pub mod validate;`, `main.rs`
   using `relay::validate`), for exactly the reason glake has it: sitting
   B built that shape so C's tests could reach the pure core, and your
   oneshot tests in move 5 need the same reach. Add the door's two
   dependencies — `serde_json = "1"` and `thiserror = "2"` — and build two
   types plus one function:

   - **`Rejection`** — an enum, one variant per clause of the door check, in
     the fixed A2 order: not a JSON object (carrying nothing), missing key
     (carrying the key's name), no comparable day. Derive `thiserror::Error`
     and write the `#[error("…")]` messages yourself — they ARE the
     user-visible `{"error":"…"}` strings, so 004 sitting H's rubric applies:
     lowercase, no trailing period, name the thing (`missing key: {0}`).
   - **`Accepted`** — a struct: the event's `day` (its `dt=` bucket) and the
     one compact `line` the lake will store.
   - **`pub fn check(body: &str) -> Result<Accepted, Rejection>`** — one
     `match` on `SerdeParser.classify(body, &REQUIRED_KEYS)` (your library;
     `use glake::classify::REQUIRED_KEYS; use glake::parser::{ClassifiedLine, EventParser, SerdeParser};`).
     Map the verdicts: `Blank | Unparseable` → not a JSON object (glake
     distinguishes them because a *reader* must skip blanks silently; a
     *door* turns both away with the same words); `Malformed { missing }` →
     missing key; `Event` whose day is the `"bad-ts"` sentinel → no
     comparable day (glake never rejects on ts — it *buckets*; the strict
     door is exactly this one extra rule on top of the tolerant reader);
     any other `Event` → accepted.

   Two design points to settle while writing, both worth saying aloud:

   - **Why the fixed order costs no effort.** You are not sequencing careful
     `if`s — the order falls out of `ClassifiedLine`'s shape, because your
     classifier can only report a missing key on something that *parsed*,
     and only reports a day on something that *had its keys*. Make bad
     states unrepresentable and the error order stops being a bug you can
     write.
   - **The re-serialization rule (A4 rev 2.1) and the second parse.** A raw
     body may legally span lines — pretty-printed JSON is still JSON — but
     the lake is JSONL: one event, one line. So the accepted `line` is the
     body *re-serialized compact*: `serde_json::from_str::<Value>(body)`,
     then `to_string`. Notice you're parsing twice — `SerdeParser` parsed
     internally but its trait boundary hands back only the owned verdict,
     never the `Value`. The alternative is extending glake's public API,
     and 004 is closed; A8 says *reuse*. Two parses per accepted event is
     the price of leaving your published API untouched — a lab relay pays
     it happily, and Sitting N's lens will show you exactly what it costs.
     (Both `from_str` and `to_string` return `Result`; the failure arms are
     near-impossible here, but the house rule is no `unwrap()` in handler
     paths — `map_err` them to a rejection. One line each; a panicking
     handler costs the whole service, which is the Cloudflare lesson the
     constitution keeps.)

3. **The door table.** `#[cfg(test)] mod tests` at the bottom of
   `validate.rs` — put `#[allow(clippy::unwrap_used)]` on the module while
   you're there: tests may unwrap (a panicking test is a failing test, which
   is fine), and Sitting N's lint sweep makes that allowance load-bearing.
   Rows to cover, written from your move-1 predictions:

   - clause 1, several ways: `"not json"`, `"[1,2]"`, `"123"`, `"\"str\""`,
     `"null"`, `""`, whitespace-only;
   - clause 2: an envelope with `actor` removed → `missing key: actor`;
   - **the order check**: missing key AND bad ts in one body → the missing
     key wins (this row pins A2's "first problem" promise);
   - clause 3: all keys present, `ts` values `"nope"`, `"2026"`,
     `"2026/07/09T01:02:03Z"` — and one *non-string* ts (`"ts":1234`), just
     as uncomparable;
   - the happy path: day extracted, line parses back to the same JSON value;
   - the multiline row: pretty-print a good envelope, check the accepted
     `line` contains no `'\n'` and round-trips to the same value;
   - weird-but-comparable days pass: `1999-01-01`, even `9999-99-99` — the
     door does a shape check, not a calendar check.

   ```console
   cargo test -p relay
   ```

   Table green, A7 pair still green. **Commit point:**

   ```console
   cargo fmt
   cargo clippy -p relay --all-targets -- -D warnings
   git add crates/relay Cargo.lock && git commit -m "005: sitting L — the strict door"
   ```

4. **The handler — naive append, on purpose.** New module `routes.rs`
   (`pub mod routes;` in lib.rs). The state first: handlers need to know
   where the lake is, so build a small `AppState` (for now just the lake's
   `PathBuf`), derive `Clone`, and change K's router to
   `Router::new().route("/events", post(post_events)).route("/healthz", get(healthz)).with_state(state)`
   — `main` passes `args.lake` in, which finally gives `--lake` its job.
   Then the handler:

   ```rust
   async fn post_events(State(state): State<AppState>, body: String) -> Response
   ```

   Note the parameter order — it is load-bearing (see the fights). The body:
   run `validate::check(&body)`; on `Err(rejection)`, answer
   `(StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": rejection.to_string() })))`
   — A2's one-line JSON error; on `Ok(accepted)`, append and answer
   `StatusCode::ACCEPTED` with an empty body (A1 shows only the status
   line; 202 over 200/201 is deliberate — "accepted for processing", which
   becomes honest-to-the-letter in Sitting M). The append, single-request
   world, written the obvious way with `tokio::fs` (add `fs` and `io-util`
   to your tokio features — `io-util` carries `AsyncWriteExt`, the trait
   that gives `File` its `write_all`/`flush`):

   - build the dir path `<lake>/dt=<accepted.day>` and `create_dir_all` it;
   - `OpenOptions::new().create(true).append(true).open(…/events.jsonl)`;
   - write the line and a newline, `flush`, return the 202 (`flush` matters
     even here: tokio's `File` buffers internally, and a 202 for bytes
     sitting in a user-space buffer would be a small lie).

   Every one of those returns `Result` — no `unwrap()`: map I/O failure to a
   500 with a one-line JSON error, three honest lines. And as you write it,
   *feel* the single-request assumption: open-per-request, write, flush,
   all inside the handler, every request paying disk latency before its 202.
   It works. Sitting M asks what "works" means when two hundred of these run
   at once — write down your guess now, in a comment above the append:
   `// M's question: what happens when two of these run at once?`

5. **The A1/A2 example tests — with a real envelope.** Dev-dependency first:
   `tower = { version = "0.5", features = ["util"] }` — its
   `ServiceExt::oneshot` calls your `Router` like a plain async function: no
   port, no socket, no running binary. Two files:

   **`tests/common/mod.rs`** — shared plumbing, worth ten minutes of care
   because every remaining test in the spec uses it:

   - **`TempLake`** — a unique dir under `std::env::temp_dir()` (pid + a
     per-process `AtomicU32` counter in the name), created in `new()`,
     **removed in `Drop`**. Two lessons in one small type: property runs and
     examples must never touch the live lake (the shell hooks are appending
     to it *right now*, concurrently — requirements, "what we're not
     building": no cross-process locking), and `Drop` is how Rust spells
     cleanup-that-cannot-be-forgotten — the dir disappears pass or fail,
     because unwinding runs destructors. (We could add the `tempfile` crate;
     hand-rolling is one less dependency and a better lesson.)
   - **request builders** — `post_event(body)` (`Request::builder()`, POST,
     `/events`, `content-type: application/json`) and `get_healthz()`;
   - **`canon(json_text)`** — parse + re-serialize: the executable form of
     A1/A4's "compared as parsed JSON values" (serde_json's map sorts keys,
     so pretty and compact spellings of one value canonicalize identically);
   - **`disk_lines(lake)`** — walk the `dt=*` dirs, return `(day, line)`
     pairs;
   - **`body_json(body)`** — collect a response body to a `serde_json::Value`.

   **`tests/routes.rs`** — the examples, as `#[tokio::test]`s (each test
   gets a runtime; inside, `app.clone().oneshot(request).await`):

   - **A1**: the fixture must be a *real hook-emitted envelope from the live
     lake* — the requirements insist on it (the example proves the door
     admits what the hooks actually produce, not a hand-crafted lookalike).
     Harvest one: `head -1 datalake/raw-local/dt=<a recent day>/events.jsonl`,
     paste it verbatim as a `const` raw string. Read your specimen before
     using it — notice the extra `spec_id` key (the door checks required
     keys are *present*; it doesn't forbid more — your move-1 prediction),
     and note which day its `ts` claims, because the test asserts the line
     landed in exactly `dt=<that day>` with `canon(line) == canon(fixture)`.
   - **A1, multiline corollary**: `to_string_pretty` the same fixture, POST
     it, assert 202 — and that the lake got exactly *one* line, same value.
   - **A2**: a table of the three clauses (junk / `[1,2,3]` / empty →
     `not a json object`; fixture minus `actor` → `missing key: actor`;
     `{"not":"an envelope"}` → `missing key: event_id`; both-problems body →
     the missing key wins; fixture with ts broken → `no comparable day in
     ts`), each asserting status 400 and the exact
     `{"error":"…"}` body — then assert the lake is **empty**: rejected
     bodies appear nowhere.

   One choreography note to say aloud: your A1 test reads the disk *right
   after* the 202, and that works because the strawman writes before
   answering. Enjoy it while it lasts — Sitting M moves the write off the
   request path, and this test will learn the drain choreography then.

   ```console
   cargo test -p relay --test routes
   ```

   > **Break point.** Door green, examples green — a fine place to stand up.
   > If you're stopping: commit what you have as
   > `005: sitting L — door wired, A1/A2 green` and take the counters fresh.

6. **Counters — the other kind of sharing (T4).** Now `/healthz` stops
   lying. In `routes.rs`, a `Counters` struct: three `AtomicU64`s —
   `received`, `accepted`, `rejected` — and `Arc<Counters>` as a second
   field on `AppState`. Before wiring, the design question this sitting is
   named for, posed properly — many concurrent handlers, two things to
   share:

   - **numbers** — increments *compose*: any interleaving of "add one"s
     lands on the same total. Hardware has an instruction for it. So:
     atomics, shared freely via `Arc`, no lock, no waiting — `clone` on
     `AppState` copies a pointer and bumps a refcount (step 13 move 6's
     `Arc`, now in a struct axum clones per request);
   - **a file** — byte writes do *not* compose: interleave two multi-byte
     appends and you get a torn line no atomic can prevent. Today the file
     is still in the handler (the strawman); M's whole job is moving it to
     the one place sharing can't reach.

   Same struct, both answers side by side — that contrast is the lesson.
   Wire the bumps into `post_events` (`fetch_add(1, Ordering::Relaxed)` —
   received on entry, accepted after the write lands, rejected on 400 *and*
   on the 500 path, so the books balance on every exit) and rewrite
   `healthz` to load all three into
   `Json(serde_json::json!({ "received": …, "accepted": …, "rejected": … }))`.
   Two things to say aloud, both from the requirements as amended:

   - **Why `Relaxed` is enough**: an occurrence counter needs each increment
     to happen exactly once (atomicity); it does not need to order *other*
     memory around itself — no reader derives anything from cross-counter
     ordering. When in doubt elsewhere, stronger orderings are the safe
     default; here Relaxed is the textbook case.
   - **Quiescent consistency (A3)**: the three counters are *independent*
     atomics — a reader can catch `received` bumped while the same request's
     `accepted` hasn't landed. `received = accepted + rejected` is promised
     **once no requests are in flight**, and that's what A3 says — the
     honest cost of lock-free counters, stated, not hidden.

   Run it and look at the wire:

   ```console
   cargo run -p relay -- --lake /tmp/l-lake &
   curl -s localhost:7311/healthz
   # → {"accepted":0,"received":0,"rejected":0}
   ```

   The *keys moved*. You wrote `received` first; serde_json's map sorts
   keys, so the wire says `accepted` first. The requirements sketch shows
   `{"received":…,…}` — semantically identical (a JSON object has no key
   order), but pin the fact now: any test asserting on healthz should
   compare parsed values, never grep the literal. (K's static string was in
   requirements order for exactly this reason: it was a string.)

7. **The A3 example test.** In `tests/routes.rs`: two accepted POSTs (the
   fixture, then the fixture with a minute changed — two distinct events),
   three rejected (junk, `{"not":"an envelope"}`, empty), all awaited
   *sequentially* — quiescent by construction — then `GET /healthz` and
   assert the parsed value equals
   `{"received": 5, "accepted": 2, "rejected": 3}`. Sequential matters and
   the test should say so in a comment: this example verifies the counters
   count; what happens to the invariant *under real concurrency* is a
   property, and it's Sitting M's A4 run that will observe it after the
   dust settles.

8. **Gate and close green.** The ritual, plus a live smoke because this is
   the first sitting where relay *does* something:

   ```console
   cargo fmt
   cargo clippy -p relay --all-targets -- -D warnings
   cargo test -p relay
   ```

   Then, in one terminal `cargo run -p relay -- --lake /tmp/l-lake --port 7311`,
   and in another (using your real-envelope fixture, saved to a file if you
   like):

   ```console
   curl -si -X POST localhost:7311/events -d @fixture.json | head -1
   # → HTTP/1.1 202 Accepted                        (and content-length: 0 — empty body)
   curl -s -X POST localhost:7311/events -d '{"not":"an envelope"}'
   # → {"error":"missing key: event_id"}
   curl -s localhost:7311/healthz
   # → {"accepted":1,"received":2,"rejected":1}
   cat /tmp/l-lake/dt=*/events.jsonl | wc -l
   # → 1
   ```

   Ctrl-c the relay (still abrupt — N's job), then **commit point:**

   ```console
   git add crates/relay Cargo.lock && git commit -m "005: sitting L — counters live, A1–A3 green"
   ```

## Compiler fights to expect

All to the ledger (`learning.*`), as ever. Axum contributes a genuinely new
fight class this sitting: trait-bound errors where the *diagnostic* is huge
and the *cause* is one token.

- **`error[E0277]: the trait bound … `Handler<_, _>` is not satisfied`, take
  two — extractor order.** If you write
  `post_events(body: String, State(state): State<AppState>)`, axum refuses
  with pages of generics. The rule underneath: extractors run in order, and
  the body is a *stream* that can be consumed exactly once — so the
  body-consuming extractor (`String` here) must come **last**. State first,
  body last. Tape that to the monitor; it is the most-hit axum error in
  existence.
- **`error[E0277]: the trait bound `AppState: Clone` is not satisfied`** —
  at `with_state`. Axum clones your state once per request; the derive is
  one line, but say *why* it's cheap before adding it: a `PathBuf` clone
  today (an allocation, actually — noted; M replaces it with a channel
  handle), an `Arc` bump after move 6. What `Clone` costs is always worth
  knowing before deriving it.
- **`error[E0433]: … `fs` … configured out`** — reaching for `tokio::fs`
  before adding the feature. K's fights section promised you'd meet this
  for real; the fix is one word, and now you know nobody else in the graph
  had asked for async file I/O.
- **`error[E0368]: binary assignment operation `+=` cannot be applied to
  type `AtomicU64``** — writing `counters.received += 1`. An atomic is not
  a number; it's a cell you *ask* to add — `fetch_add(1, Ordering::…)`, and
  being forced to name an ordering is the point (you then get to say why
  Relaxed).
- **`warning: unused implementer of `Future` that must be used`** — a
  `write_all(...)` or `flush()` without `.await`. Step 12 move 5's sneakiest
  shape, now with stakes: the 202 goes out, the write never happens. Our
  `-D warnings` gate turns it fatal, which is exactly what you want.
- **`error[E0382]: borrow of moved value: `body``** — if you try to use
  `body` after handing it somewhere by value (say, building the 400 message
  after moving the body into a helper). Ramp 2's law in a handler; borrow
  for the check (`check(&body)`), and the question of who finally *owns*
  the accepted line gets its real answer in M.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                   # no diff
cargo clippy -p relay --all-targets -- -D warnings  # clean
cargo test -p relay                                 # ALL green: door table (incl. order row,
                                                    # weird-days row, multiline row), A1 x2,
                                                    # A2 table, A3 counters, A7 pair
```

```
# live, against a scratch lake (never datalake/raw-local for experiments):
curl -si -X POST localhost:7311/events -d @fixture.json | head -1   # HTTP/1.1 202 Accepted
curl -s  -X POST localhost:7311/events -d 'junk'                    # {"error":"not a json object"}
curl -s  localhost:7311/healthz                                     # counters, keys sorted
```

- The A1 fixture in `tests/routes.rs` is a verbatim line from
  `datalake/raw-local` — it has a `spec_id` key you never asked for, and
  your door admitted it anyway, on purpose.
- You can answer aloud: why 202 and not 200 or 201 — and what will make
  that choice *literally* true in M? Why is `received = accepted + rejected`
  only promised at quiescence, and which design choice bought that caveat?
  Why does the door reject what glake-the-reader would merely bucket? And
  the comment you left in the handler: what *do* you think happens when two
  of these run at once — torn bytes, lost lines, or nothing? (Write the
  guess down. M grades it.)

## Hints (one at a time)

<details><summary>Hint 1 — a nudge: the handler's shape</summary>

The whole handler is a `match` on `validate::check(&body)` with two arms,
and the `Ok` arm is the only place I/O lives. Build the response values with
`(StatusCode::…, Json(json!({ … }))).into_response()` — the tuple form is
axum's "status plus body" idiom, and `.into_response()` unifies the arms'
types (they'd otherwise be two different opaque types and E0308 you). The
bare `StatusCode::ACCEPTED` is itself a response — `.into_response()` on it
and the types line up.

</details>

<details><summary>Hint 2 — the shape: TempLake's Drop</summary>

```rust
pub struct TempLake { path: PathBuf }

static NEXT: AtomicU32 = AtomicU32::new(0);

impl TempLake {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "relay-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        TempLake { path }
    }
}

impl Drop for TempLake {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
```

pid + counter: parallel tests in one binary get distinct counters, parallel
test binaries get distinct pids. The `let _ =` on removal is deliberate —
cleanup failure shouldn't panic a passing test. After M's property runs,
`ls /tmp | grep relay-test` should print nothing; if it doesn't, some test
leaked its lake past `Drop`, which is worth understanding, not ignoring.

</details>

<details><summary>Hint 3 — the shape: one oneshot test</summary>

```rust
#[tokio::test]
async fn a1_valid_event_202_and_lands_in_its_day() {
    let lake = TempLake::new();
    let app = /* build router with a state pointing at lake.path() */;

    let response = app.clone().oneshot(post_event(FIXTURE)).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let lines = disk_lines(lake.path());
    assert_eq!(lines.len(), 1);
    // day == the day the fixture's ts claims; canon(line) == canon(FIXTURE)
}
```

You'll want a small helper that builds the state+router for a given lake
path — three tests use it today and every M/N test after; put it in
`common/mod.rs` next to `TempLake` and let M evolve its signature.

</details>

## If truly stuck

Read, don't copy — then close it and write yours:

- `specs/005-async-relay/_reference/relay/src/validate.rs` — `Rejection`,
  `Accepted`, `check`, and the full door table in its tests. (Its doc
  comments carry the why; read them even if you don't need the code.)
- `specs/005-async-relay/_reference/relay/src/routes.rs` — `Counters` and
  the two handlers. **Warning:** the reference is the *finished* crate — its
  `AppState` already holds a channel sender and its handler sends instead of
  writing. Take the counters and the response shapes; the strawman append is
  yours alone (the reference never had it — you'll replace it in M anyway).
- `specs/005-async-relay/_reference/relay/tests/common/mod.rs` and
  `tests/routes.rs` — `TempLake`, `canon`, `disk_lines`, and the A1/A2/A3
  tests (same warning: `spawn_relay` there is M's shape).
