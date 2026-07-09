# Sitting M — the one owner, test-first

**Builds:** the spec's heart, in the spec's order: the A4 conservation
property *first* (co-written — the async×proptest pattern), fired at Sitting
L's naive handler under real parallelism with the design's honest-red
protocol; then the fix the whole course has been walking toward — a bounded
mpsc channel and **one writer task that owns every open lake file** — making
A4 green *by construction*, and A5 proving the partitions with your own glake
as the independent witness.
**Requirements:** A4, A5 — tasks 1.3–1.4 (T2: channels as ownership transfer;
T3: Send/Sync, felt; T6: testing async).
**Ramp you'll use:** step 13, all of it — this sitting is its README's move 7
("you are rebuilding this file with a real file where the `Vec` is") made
literal — plus step 12 (spawn) and Sitting E's `entry` idiom on a new payload.

## Where you are

Sitting L closed with a comment in your handler: *what happens when two of
these run at once?* Today you stop guessing. The order of work is the
red-first rhythm from C and E, scaled up: the property comes before the
architecture it justifies. First you and Claude co-write A4 — a test that
fires generated batches of valid and invalid events at the relay
**concurrently**, then checks the lake against the 202s like an accountant:
nothing torn, merged, lost, or invented. Then you run it against L's strawman
and deal honestly with what happens — the design commits, in writing, to two
acceptable outcomes, and *both* teach. Only then do you build the one-owner
design: handlers stop touching files entirely; a valid event becomes a
`String` and is **sent** — ownership and all — through a channel to the one
task holding the open files. Step 13's collector, with a real file where the
`Vec` was. This is the most Rust-shaped idea in the course: where other
stacks reach for a lock, you transfer ownership of the data to the owner of
the sink.

## The build, move by move

All commands from the repo root. Three commits this sitting: the red, the
channel, A5 green.

1. **Predictions on paper (the property grades them).** Three questions,
   written answers, before any code:

   - Your L handler under sixteen truly concurrent POSTs: *can* lines tear?
     Recall exactly what your append does — open in append mode, then how
     many `write_all` calls per event? One (line and newline in one buffer)
     or two (line, then `b"\n"`)? Your answer to "can it tear" hinges on
     that detail, and you probably didn't know it mattered when you wrote it.
   - The counters under the same load: does `received = accepted + rejected`
     still hold *after* every request finishes? (Different answer than
     "during" — L's quiescence lesson.)
   - A request gets its 202 and the process is killed one millisecond later.
     Is the event on disk? For *your* strawman, today, honestly — and what
     in your handler's code makes you sure?

2. **The async×proptest pattern, co-written (T6 — you drive, Claude
   navigates).** Add `proptest = "1"` to relay's `[dev-dependencies]` and
   create `crates/relay/tests/prop_relay.rs`. This is the course's hard-bit
   rule in action (properties, tricky concurrency — written together), and
   the pattern deserves its one-breath statement before the code: **proptest
   drives a sync test body; the body builds its own multi-thread tokio
   `Runtime` and `block_on`s the scenario** — generation stays deterministic
   (proptest's RNG never crosses an `.await`), only the scenario is async.
   Inside the scenario, **every request gets its own `tokio::spawn`** against
   a cloned router. Two traps this shape exists to dodge, both worth saying
   aloud:

   - `#[tokio::test]` + proptest don't compose: the macro wants an async fn,
     proptest wants a sync closure it can call thousands of times — and if
     you build a `Runtime` while already inside one, tokio panics
     (`Cannot start a runtime from within a runtime`). Sync body, own
     runtime: the test controls its world.
   - The tempting shortcut — `join_all` over sixteen un-spawned `oneshot`
     futures — polls them all **on one task**: interleaved, never parallel,
     and A4 would be testing nothing. `tokio::spawn` per request, on a
     `new_multi_thread` runtime (give it `worker_threads(4)`), is what makes
     handlers actually race on OS threads. (Add `sync` to relay's tokio
     features now — the channel type arrives in move 5, and the test file
     compiles against it from the start if you write `spawn_relay`'s final
     signature first; or add it when the compiler asks. Your call.)

   What the co-write must produce, per the design's Properties table:

   - **A multi-day ts domain.** A `DAYS` array of five days — three sensible
     ones, plus `1999-01-01` and something far future. Why multi-day is
     *load-bearing*: a writer that dumps every event into one folder would
     pass a single-day A5 by luck. Why weird days: the door accepts what it
     can partition, not what a calendar blesses (L's lesson, now generating
     test cases). A `ts()` strategy formats `<day>T<hh>:<mm>:<ss>Z` from
     generated components.
   - **`valid_body()`** — a full envelope (all seven `REQUIRED_KEYS`, a
     generated id, a kind drawn from a few realistic literals) — and a coin
     flip: half the time, emit it `to_string_pretty`, **spanning lines**.
     A raw body may legally span lines; A4 rev 2.1's re-serialization rule
     is exactly what makes the lake stay one-line-per-event, and the
     generator must exercise it.
   - **`invalid_body()`** — one arm per door clause, mirroring 004's
     malformed strategies: non-objects (junk, `[1,2,3]`, `123`, `"quoted"`,
     `null`, empty), an envelope with one `REQUIRED_KEYS` entry removed, an
     envelope whose ts has no comparable day.
   - **The oracle travels with the input** (E's `GenLine` trick, second
     time): a two-variant enum `Msg { Valid(String), Invalid(String) }` —
     the generator *tags* each body with what the door must say, so the
     test's truth never comes from the code under test.
   - **`batch()`** — `prop::collection::vec` of 1..16 messages, weighted
     ~3:1 valid:invalid, and — the subtle one — **occasional exact
     duplicates** (repeat a message, weight about 1-in-6). The requirements'
     no-dedup policy says duplicate submissions yield duplicate lines;
     *multiset* equality is what catches a writer that "helpfully" dedups —
     or a test that compares `HashSet`s and can't see doubles.
   - **The scenario** (a `run_scenario(msgs) -> Result<Outcome, TestCaseError>`
     helper — A4 and A5 share it verbatim): build the runtime; inside
     `block_on`: `TempLake`, build the relay (today: L's router over the
     temp lake), spawn one task per message (each owning a `Router` clone —
     cheap, and the whole point of `AppState: Clone`), join every handle
     checking each against its oracle (valid → 202, invalid → 400,
     `prop_assert_eq!`), snapshot `/healthz` (quiescent by construction —
     every request joined), then read the disk. Collect it all into an
     `Outcome` struct that *holds the `TempLake`* so the evidence outlives
     the runtime.
   - **The A4 assertions**: canonicalize both sides (`canon` from L's
     common module — parse + compact re-serialize, sorted keys), sort both
     `Vec`s, `prop_assert_eq!` — multiset equality, executable form. Plus
     the healthz triple: `received == msgs.len()`,
     `accepted == 202-count`, `rejected == 400-count` — L's A3 invariant,
     observed for the first time after *real* concurrency.
   - House conventions: `#![proptest_config(ProptestConfig::with_cases(256))]`
     — the floor, not C/E's 512, and say why aloud: every case builds a
     runtime, spins four threads, and touches a real filesystem; 256 cases
     of *that* buys more confidence than 512 of a pure fold. The checkpoint
     compensates with a `PROPTEST_CASES` confidence run. Test named
     `prop_a4_conservation`.

3. **The red run — the honest-red protocol.** The design commits to this in
   writing (read its "Honesty about red" paragraph now — it's short), so run
   it and find out which world you're in:

   ```console
   cargo test -p relay --test prop_relay
   ```

   - **Outcome one: it tears.** Proptest hands you a shrunk batch — likely
     two or three valid bodies — and the disk multiset disagrees: a line
     with another line's tail welded onto it, or two events sharing a line,
     or a bare fragment. Open the temp-lake path from the failure output if
     it's still there; more likely, read the canon mismatch. This is a real
     red: your two-write append (line, then newline) gave the scheduler a
     seam, and under four worker threads another request's bytes landed in
     it. On validation day, the two-write shape under 200 truly parallel
     appends tore roughly 170 of 200 lines, run after run. Commit the seed
     — `crates/relay/proptest-regressions/` is house-rule permanent.
   - **Outcome two: it holds.** All 256 cases green. Your handler is not
     correct — it got *saved*, and you should be able to name the savior:
     you (perhaps unknowingly) built the full record in one buffer, so each
     event was **one `write_all`, one `write(2)` syscall, on a file opened
     with `O_APPEND`** — and Linux makes small O_APPEND writes atomic. The
     OS quietly serialized you. Now say why that's no foundation: it
     depends on single-syscall formatting (one refactor away from breaking —
     the two-write spelling tears, and you now know how close you came to
     writing it), on the write not coming back short (a `write_all` that
     loops is multiple syscalls again), on Linux and a local filesystem
     (NFS makes no such promise), and on nobody ever adding a retry. Green
     by OS mercy, and mercy is not in the requirements.

   Either way, one more question before the fix, because it's the one no
   careful formatting answers: **your strawman survived shutdown questions
   only by being slow** — every request paid full disk latency before its
   202, so a finished request was a durable request. That coupling *is* the
   strawman's whole safety story, and it's exactly what a real service
   can't afford (and what you're about to remove — a 202 that means
   "enqueued" creates, for the first time, a moment where accepted events
   exist only in memory). Hold that thought; move 6 makes it a red you can
   see. **Commit point — the red commit** (adjust the message to your
   outcome; both are honest):

   ```console
   git add crates/relay Cargo.lock
   git commit -m "005: sitting M — A4 vs the strawman, red (torn lines)"
   # or: "005: sitting M — A4 vs the strawman, held by O_APPEND (no guarantee)"
   ```

   (If it tore, `git add` the `proptest-regressions/` seed too. And log the
   outcome to the ledger either way — `learning.*`; which world you landed
   in is SKILLS evidence about your own I/O instincts.)

4. **Think before you type — the writer's design.** The fix is step 13's
   collector: handlers send, one task owns the files. Settle these on paper;
   the middle one is a genuine design tension, not a quiz:

   - **What crosses the channel, and what's its license?** The design pins
     `mpsc::Sender<String>` — each message one formatted JSONL line, moved.
     Say the T3 sentence before the compiler makes you: `tx.send(line)`
     compiles because `String` is `Send` — it may change tasks/threads —
     and after the `.await`, the handler provably cannot touch those bytes
     again. No lock exists because no sharing exists.
   - **The tension: `Sender<String>` dropped the day on the floor.** Your
     door hands back `Accepted { day, line }` — but a `String` channel
     carries only the line, so the writer must *re-derive* the day: parse
     the line (it's compact JSON by construction), read `ts`, apply
     **glake's `day_of_ts`** — the same one rule the door's classifier used,
     so door and writer can't drift. That's a third parse per accepted
     event, and it creates a branch that can't happen (a queued line with no
     day) which A9 says you may not `unwrap()` away and conservation says
     you may not silently drop. So: the impossible branch writes to a
     visible `dt=bad-ts` quarantine and logs an error — better an event in a
     wrong-looking folder you can see than one silently gone. Now the
     question worth five minutes: *would you amend the design instead?* A
     channel item of `{ day, line }` removes the parse and the unreachable
     branch at the cost of one small struct. That's not a rebellion — it's
     the change protocol (halt the item, amend upstream, re-gate the
     amendment). The reference implementation follows the design's letter
     (String, re-derive, quarantine); this guide does too. Decide what *you*
     think, say it aloud, and log it — spec-close collects exactly this kind
     of finding.
   - **Bounded, 256 — why?** When the writer falls behind, a full channel
     makes `send().await` *wait* — the handler slows, the client sees
     latency: backpressure for free, and the 202 stays honest
     ("enqueued", really enqueued). Unbounded hides overload until OOM. AWS
     through-line, since 006 deploys this handler: a Lambda's memory is its
     bill *and* its CPU — an unbounded queue converts overload into a
     bigger bill and then a dead function; backpressure converts it into
     visible latency. Choose visible.
   - **Per-day handle cache.** Events arrive in any day order (your DAYS
     domain guarantees it), and opening a file per line is waste. A
     `HashMap<String, File>` with the `entry` API — Sitting E's exact idiom,
     new payload: `Occupied` → reuse, `Vacant` → `create_dir_all` +
     open-append + insert. Bounded in practice by distinct days seen — a
     handful, not a leak.

5. **Build the writer (you write; T2 lands here).** New module
   `crates/relay/src/writer.rs` (`pub mod writer;`):

   ```rust
   pub async fn run(mut rx: mpsc::Receiver<String>, lake: PathBuf) -> u64
   ```

   The loop of the spec: `while let Some(line) = rx.recv().await { … }`.
   Everything you settled in move 4 goes inside — an `append` helper taking
   the cache, the lake root, and the owned line; day via glake; the entry
   cache; **one `write_all`** for line + newline (push `b'\n'` onto
   `line.into_bytes()` — you own the `String`, so the newline lands in
   place; and after move 3 you know exactly why building the full record
   before writing matters); `flush` after every line (tokio's `File`
   buffers internally — flush hands bytes to the kernel; that's durability
   enough for a localhost lab lake — the paranoid tier is `sync_all()`
   /fsync, deliberately not paid here); on I/O error past this point the
   202 already went out, so log loudly (`tracing::error!` — add `tracing`)
   and **keep draining** — one bad day-dir must not lose every other day's
   events, and a panicking writer is A9's nightmare. After the loop (which
   ends when — say it before reading on — *every sender is dropped*): a
   final flush of every cached handle, then return `written`, the count of
   lines that hit disk. The count is not decoration: `main` will print it
   in Sitting N's goodbye line, and your tests get a second conservation
   witness. Add a small unit test pinning day-derivation to the door's day
   rule (a line with a good ts, a bad ts, a non-string ts, junk, no ts —
   `Some("2026-07-09")` / `None`s).

   Notice what the module *is*: the `File` handles live in this function's
   locals. No other code in the crate **can** write to the lake — not
   "shouldn't": *can't*, because Rust has no way to name another task's
   locals. That sentence is the whole concurrency story.

6. **Rewire the routes, adapt the harness — and meet the queue's red.**
   Three edits, then a run you should predict first:

   - `AppState` drops `lake` and gains `tx: mpsc::Sender<String>` (counters
     stay). The handler's `Ok(accepted)` arm becomes
     `state.tx.send(accepted.line).await` — the ownership transfer the
     design pivots on (and `accepted.day` dies here unused, per move 4's
     tension — leave a one-line comment saying the writer re-derives it).
     `Ok(())` → bump accepted, 202. `Err(_)` → the writer is gone, which is
     unreachable by construction once N orders the shutdown — but A9 says
     no unwrap, so: bump *rejected* (the quiescent invariant must survive
     even the impossible path) and answer 500 with a one-line error. Three
     lines for a branch that can't fire; a panicking handler would cost the
     service.
   - `main.rs`: build `let (tx, rx) = mpsc::channel::<String>(256);`, spawn
     the writer (`tokio::spawn(writer::run(rx, args.lake.clone()))` — `rx`
     and the path both *move*; nothing shared), pass `tx` to the state.
     Keep the `JoinHandle` in a variable even though nothing awaits it yet —
     Sitting N's entire opening question is about this exact handle, and
     today's binary still dies rudely at ctrl-c (one sitting more).
   - `tests/common/mod.rs`: your router-builder helper grows into the
     reference's shape — `spawn_relay(lake) -> (Router, JoinHandle<u64>)`:
     channel, spawn writer, router over `AppState::new(tx)`.

   Now update `run_scenario` the *obvious* way — requests join, healthz
   snapshots, read the disk — and run A4:

   ```console
   cargo test -p relay --test prop_relay
   ```

   **Expect red — and this one is the design's promised red, not OS
   roulette: shutdown-mid-queue.** The multiset comes up short: bodies that
   got their 202 are missing from disk. Nothing tore — they're sitting in
   the channel, or in the writer's hands, at the instant your test looked.
   The strawman never had this failure because it never had a queue; you
   built the queue on purpose (that's where the handler's disk latency
   went), and a queue at shutdown is lost **unless somebody drains it**.
   (If the scheduler happens to hand you green, rerun — the race is real
   and 256 cases rarely miss it; witness the red before fixing it, because
   watching accepted events vanish is the lesson N's ctrl-c wiring exists
   to prevent.) The fix is the drain choreography, and it's four lines at
   the end of the scenario:

   ```rust
   drop(app);                       // the LAST router clone dies → the last tx dies
   let written = writer.await?;     // recv() yields None → drain → flush → count
   ```

   — then assert `written == accepted.len()` alongside the multiset. Say
   the causal chain out loud, in order, because it *is* graceful shutdown
   in miniature and step 13 move 4 made you narrate it once already: last
   sender drops → `recv()` returns `None` (after yielding every queued
   line — close is not discard) → loop ends → final flush → the
   `JoinHandle` yields the count. No flags, no sleeps: channel close is the
   shutdown signal.

   ```console
   cargo test -p relay --test prop_relay
   # → prop_a4_conservation ... ok        (256 cases, and now say WHY it can't fail:
   #    can't tear — one writer, whole-record writes; can't lose — drain-then-flush;
   #    can't invent — only door-passed lines enter the channel; can't dedup — nothing
   #    anywhere compares two lines)
   ```

   Green **by construction**, not by mercy. Also rerun L's routes tests —
   they read the disk right after a 202, and that luxury is gone; give them
   the same `drop(app)` + `writer.await` choreography before their disk
   asserts (the guide warned you in L; the fix is mechanical and each test
   gets a comment saying "on disk no later than shutdown"). **Commit point:**

   ```console
   cargo fmt && cargo clippy -p relay --all-targets -- -D warnings
   git add crates/relay Cargo.lock
   git commit -m "005: sitting M — the one owner: channel + writer, A4 green"
   ```

7. **A5 — partitions, with your own tool as the witness.** One prediction
   first: could a writer that ignores `day` entirely and appends everything
   to `dt=2026-01-01/events.jsonl` pass A4? … Yes — conservation counts
   lines, not addresses. That's why A5 exists, and why the DAYS domain is
   multi-day. Second property in `prop_relay.rs`, `prop_a5_partitions`,
   same `run_scenario`, different assertions:

   - **Line by line**: for every `(folder_day, line)` on disk, parse the
     line, read its `ts`, and assert
     `glake::classify::day_of_ts(ts) == Some(folder_day)` — every event
     lives in the folder its *own* ts names.
   - **The glake-agrees check** — the 004 payoff, in full: walk the temp
     lake with your library's own walker
     (`glake::walk::{jsonl_files, read_file}`), run your own tally pipeline
     over the lines (`glake::tally::tally_filtered` with `&SerdeParser`,
     `&REQUIRED_KEYS`, `&Filter::default()` — module paths per *your* 004
     crate; if your names drifted from the worksheets, use yours), and
     assert three ways: glake's event total equals the relay's accepted
     count; glake saw **zero** malformed lines (the strict door kept them
     out — L's asymmetry, now measurable); and glake's by-day map equals
     the map you build from the 202'd bodies' own ts days. Two programs you
     wrote, independent pipelines, agreeing about everything.

   ```console
   cargo test -p relay --test prop_relay
   # → running 2 tests
   #   test prop_a4_conservation ... ok
   #   test prop_a5_partitions ... ok
   ```

   Likely green on the first run — and that's honest: the red-first rhythm
   spent its red on A4, where the architecture was actually wrong; A5's
   teeth are the multi-day domain, and your writer was born day-aware. If
   it *does* go red, triage before fixing (spec bug / code bug / test bug —
   the `_assurance/triage-log.md` discipline), and commit the seed.
   **Commit point:**

   ```console
   git add crates/relay && git commit -m "005: sitting M — A5 partitions + glake agreement green"
   ```

## Compiler fights to expect

The richest crop since C — async trait bounds, moves across `spawn`, and two
fights with no diagnostic at all. Ledger (`learning.*`) takes them all.

- **panic: `Cannot start a runtime from within a runtime`** — you wrote the
  property inside `#[tokio::test]`, or built the `Runtime` inside an async
  context. The pattern exists to prevent this: proptest owns a **sync** fn;
  the sync fn owns the runtime; only the scenario is async.
- **`error[E0382]: use of moved value: `app``** — spawning requests in a
  loop that moves the router in whole. Step 13's `tx` fight, verbatim, with
  a `Router` in the title role — and the same fix, for the same reason:
  clone per iteration (`let app = app.clone();` first line of the loop
  body); a `Router` clone is a cheap handle onto shared innards, and
  `AppState: Clone` is *why* the design derives it.
- **`error[E0373]/E0521: closure may outlive the current function` /
  `borrowed data escapes`** — the spawned block borrows `msg` (or the batch
  `Vec`) instead of owning it. `tokio::spawn` demands `'static`: the task
  may outlive your stack frame, so it must own its world — move the
  message in (consume the `Vec` with `into_iter()`, not `iter()`).
- **`error[E0277]: `?` couldn't convert the error …`** — `prop_assert!`
  inside the scenario, where the enclosing closure returns the wrong type.
  `prop_assert_*!` doesn't panic — it *returns* `Err(TestCaseError)` — so
  every function it appears in must return `Result<_, TestCaseError>`;
  that's the real reason `run_scenario` has the signature it has.
- **`error[E0382]: borrow of moved value: `accepted.line``** — logging or
  re-reading the line after `send(...)`. Step 13 move 5, now with real
  stakes: the send *moved* it; the handler no longer has the data. If you
  need something after the send (the day for a log line), read it before,
  or log the copy you kept on purpose — every `clone()` here is a decision.
- **The hang, once more, in test clothing** — the suite prints
  `test prop_a4_conservation ...` and sits forever. Not slow: deadlocked.
  Some sender is still alive when `writer.await` runs — almost always a
  router clone you kept (`drop(app)` must kill the *last* one; check you
  didn't stash a clone in the `Outcome` or a variable that outlives the
  await). No diagnostic exists for this; step 13 move 4 is the training,
  and Sitting N asks you to predict this exact shape cold, in `main`.
- **`error[E0433]: … configured out` — `sync` (or `fs`)** — tokio slices
  again; by now you fix this in ten seconds flat, which is the habit paying
  rent.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                   # no diff
cargo clippy -p relay --all-targets -- -D warnings  # clean (tests included)
cargo test -p relay                                 # ALL green: door table + writer's day test,
                                                    # A1 x2 / A2 / A3 (drain choreography added),
                                                    # BOTH properties, A7 pair
PROPTEST_CASES=1024 cargo test -p relay --test prop_relay
                                                    # confidence run at 4x: still green
                                                    # (reference: ~4 s — each case is a whole
                                                    # runtime + temp lake; that's normal)
ls /tmp | grep relay-test                           # prints NOTHING — every TempLake dropped,
                                                    # 1024 cases and no litter (Drop, proven)
```

- If proptest ever failed along the way, `crates/relay/proptest-regressions/`
  is committed, not gitignored. (If the strawman *held* in move 3, there may
  be no seed at all — the reference suite never produced one either. A
  seedless history plus a written held-by-O_APPEND note is a legitimate
  outcome of the honest-red protocol.)
- `git log --oneline -3` shows the rhythm: strawman verdict → one owner →
  A5 green.
- You can answer aloud: why can't a `Mutex<File>` in every handler be the
  house answer, even though it also serializes? (Two reasons: every
  request's latency re-couples to the disk, and shared-mutable-state is the
  thing the language keeps steering you away from — transfer beats guard.)
  What exactly crosses the channel, and which auto trait is its passport?
  Where did the handler's disk latency go, and who pays it now? Why 256 and
  not unbounded — and what does a full channel do to a caller? And the
  writer's `HashMap<String, File>`: what bounds its size, and which sitting
  taught you the `entry` call inside it?

## Hints (one at a time)

<details><summary>Hint 1 — a nudge: run_scenario's skeleton</summary>

Shape it as data-in, evidence-out, no assertions inside (A4 and A5 assert
differently on the same evidence):

```text
fn run_scenario(msgs: Vec<Msg>) -> Result<Outcome, TestCaseError>
    build multi-thread Runtime (worker_threads 4, enable_all)
    block_on(async {
        TempLake; (app, writer) = spawn_relay(lake.path())
        for msg in msgs: spawn a task { app.clone().oneshot(post_event(...)).await; (msg, status) }
        join all handles: valid→assert 202, push canon(body); invalid→assert 400, count
        healthz snapshot (quiescent — everything joined)
        drop(app); written = writer.await
        Outcome { lake, accepted, rejected, written, health }
    })
```

The `Outcome` holds the `TempLake` — if it didn't, the lake would be deleted
(Drop) before the property reads the disk. Ownership as test correctness.

</details>

<details><summary>Hint 2 — the shape: the writer's append</summary>

```rust
let file = match files.entry(day) {
    Entry::Occupied(entry) => entry.into_mut(),
    Entry::Vacant(entry) => {
        let dir = lake.join(format!("dt={}", entry.key()));
        tokio::fs::create_dir_all(&dir).await?;
        let file = OpenOptions::new().create(true).append(true)
            .open(dir.join("events.jsonl")).await?;
        entry.insert(file)
    }
};
let mut record = line.into_bytes();
record.push(b'\n');
file.write_all(&record).await?;
file.flush().await
```

`entry.into_mut()` vs `entry.insert(file)` — both hand back `&mut File`, so
the match arms unify. `append(true)` still matters even with one owner: the
shell hooks may be appending to the same *live* lake file in another process
(requirements' coexistence note) — your channel serializes relay's writes;
O_APPEND keeps relay and the hooks from clobbering each other's offsets.

</details>

<details><summary>Hint 3 — the A5 expected-by-day map</summary>

Truth from the oracle side, never from the code under test: fold over the
**accepted bodies** (the canon strings you collected at 202 time), parse
each, `day_of_ts` its ts, and `*map.entry(day.to_owned()).or_insert(0) += 1`
— Sitting E's idiom, fourth appearance. Then `prop_assert_eq!` against
glake's `by_day`. If they disagree, the failing day names the file family
one side isn't seeing — the per-type-table debugging move from Sitting F,
one spec later.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/005-async-relay/_reference/relay/tests/prop_relay.rs` — the
  strategies (`DAYS`, `valid_body`, `invalid_body`, `batch`), `run_scenario`,
  and both properties. Its module docs restate the whole async×proptest
  argument; read them even if you copy nothing.
- `specs/005-async-relay/_reference/relay/src/writer.rs` — `run`, `append`,
  `day_of_line`, and the quarantine branch, each with its reasoning in
  comments.
- `specs/005-async-relay/_reference/relay/src/routes.rs` — the finished
  `AppState` and the send-based handler (including the impossible-500 arm).
