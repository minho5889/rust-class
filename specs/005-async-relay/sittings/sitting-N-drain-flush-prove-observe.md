# Sitting N — drain, flush, prove, observe

**Builds:** the ending relay deserves: graceful shutdown (ctrl-c → refuse new
work → finish in-flight → drain the queue → flush → goodbye → exit 0), proven
by an A6 test that SIGINTs the *real binary*; then the hygiene sweep (A8
dependency tree, A9 lints mechanized) and the payoff — a live relay and your
own glake agreeing to the event, and the memory lens pointed at a running
service for the first time.
**Requirements:** A6a/A6b, A8, A9, A10 — tasks 1.5–1.7 (T5: graceful
shutdown).
**Ramp you'll use:** step 13 move 4 (the forgotten-sender hang — this sitting
opens by collecting the debt that step announced), step 12 (`select!` — "next
request OR ctrl-c" — and `JoinHandle`s), and Sitting F's whole toolkit
(optional deps, `cargo tree`, the lens ritual).

## Where you are

Sitting M made the relay correct under fire: one owner, conservation by
construction, partitions witnessed by glake. But the *binary* still dies like
sitting K's hello world — ctrl-c kills it mid-anything, and now there's a
queue to kill it mid-*of*. M's tests already perform the drain choreography
(`drop(app)`, then `writer.await`); today `main` learns the same moves, wired
to a real signal. The design named this sitting's footgun before any code
existed, and step 13's worksheet promised you'd be asked to predict it cold.
That's move 1. No peeking.

## The build, move by move

All commands from the repo root. Four commits: shutdown, the A6 test, the
hygiene sweep, evidence.

1. **The checkpoint question — predict before running (closed book).** Here
   is the natural first draft of a graceful `main`. Read it; don't type it:

   ```rust
   let (tx, rx) = mpsc::channel::<String>(256);
   let writer = tokio::spawn(writer::run(rx, args.lake.clone()));
   let state = AppState::new(tx.clone());          // ← keep tx around, just in case
   let app = routes::router(state);
   axum::serve(listener, app)
       .with_graceful_shutdown(shutdown_signal())
       .await?;
   let written = writer.await?;                    // ← what happens HERE?
   println!("relay: drained, {written} events on disk. bye.");
   ```

   Ctrl-c arrives. The listener closes, in-flight requests finish, `serve`
   returns, the router — and the `tx` clone inside it — drops. Then `main`
   reaches `writer.await`. **Write down, in full sentences, exactly what
   happens next and why**, mechanism included. When you're done — and only
   then — check yourself against step 13 move 4's narration and the
   design's "one footgun, named up front" paragraph. If your answer names
   the surviving sender, where it lives, what `recv()` is waiting to see,
   and why that wait can never end, you understand channel ownership; T5 is
   mostly downhill from here. (Grade honestly and ledger it: predicted cold
   / needed the step-13 rerun / needed the design doc.)

2. **Wire the real shutdown.** Additions to `main.rs`, in the order the
   design's shape table gives them:

   - tokio's `signal` feature, and a five-line helper:

     ```rust
     async fn shutdown_signal() {
         if let Err(e) = tokio::signal::ctrl_c().await {
             tracing::error!("cannot listen for ctrl-c ({e}); shutting down");
         }
     }
     ```

     Two things said aloud: this future *completing* is the shutdown
     signal — `with_graceful_shutdown` takes any future and acts when it
     finishes (step 12's `select!` is underneath: axum races "next
     connection" against your future); and the error arm honors A9 without
     `unwrap()` — if the process can't hear ctrl-c (it practically can't
     fail), the least-wrong service is one that shuts down *now* rather
     than one that can never be stopped. Returning IS the signal.
   - **Move `tx` in — no clone.** `AppState::new(tx)` takes the sender by
     value; `main` keeps nothing. After the state moves into the router and
     the router into `serve`, the drain trigger belongs entirely to the
     router's lifetime: serve returns → router drops → *last* sender drops
     → the writer's `recv()` sees `None` → drain begins. Your move-1
     prediction, inverted into the fix.
   - `writer.await` **last**, after `serve` returns — and notice what the
     `?` on it means: the `JoinHandle` yields `Err` only if the writer task
     *panicked*, and anyhow turns that into a nonzero exit. A crashed
     writer must not masquerade as a clean drain.
   - The goodbye: `println!("relay: drained, {written} events on disk. bye.")`
     — M's returned count, spent. And a discipline that makes the A6 test
     (and any script) possible: **stdout is protocol, stderr is logs.**
     Exactly two lines ever print to stdout — `listening on` (K) and
     `drained` (now). Route tracing to stderr:

     ```rust
     tracing_subscriber::fmt().with_writer(std::io::stderr).init();
     ```

     (`tracing-subscriber = { version = "0.3", default-features = false, features = ["fmt"] }`
     — the fmt half only; you'll meet this crate's full weight in 006.)

   Now feel it work, twice. First the honest run:

   ```console
   cargo run -p relay -- --lake /tmp/n-lake
   # POST an event or two from another terminal, then ctrl-c:
   # → relay: drained, 2 events on disk. bye.        exit code 0
   ```

   Then **witness the footgun you predicted** — change one line to
   `AppState::new(tx.clone())`, rerun, POST one event, ctrl-c: the listener
   dies (a new curl can't connect)… and nothing else happens. No goodbye,
   no exit, no error — the exact silence you wrote down in move 1, live.
   `kill -9` it from another terminal, revert the line, and log the
   prediction-vs-observed note to the ledger. **Commit point:**

   ```console
   cargo fmt && cargo clippy -p relay --all-targets -- -D warnings
   git add crates/relay Cargo.lock
   git commit -m "005: sitting N — graceful shutdown: drain wired, footgun dodged"
   ```

3. **The A6 test — a real child, a real SIGINT.** Why not inject a fake
   shutdown future into an in-process server? Because that never exercises
   the `tokio::signal::ctrl_c` wiring, the drop-tx choreography in `main`,
   or the process exit code — which are the three things A6 is *about*.
   `crates/relay/tests/shutdown.rs`, one `#[test]` (plain — you're running
   a process), built from parts you mostly own already:

   - **Spawn the child**: `Command::new(env!("CARGO_BIN_EXE_relay"))` with
     `--port 0` (the OS picks — parallel test runs can't collide; K's
     `local_addr()` discipline is about to pay) and `--lake` at a
     `TempLake`, stdout piped. Read the first stdout line, assert it starts
     with `relay: listening on 127.0.0.1:`, parse the real port off the
     end.
   - **Speak HTTP by hand** — a teaching detour worth having: a request is
     four header lines, a blank line, and the body, written straight down a
     `std::net::TcpStream`:

     ```text
     POST /events HTTP/1.1
     Host: 127.0.0.1:<port>
     Content-Type: application/json
     Content-Length: <body.len()>
     Connection: close

     <body>
     ```

     (`\r\n` line endings; `Connection: close` makes the server end the
     stream after responding, so read-to-EOF is your complete-response
     detector — no client crate, no framing code.) Parse the status from
     the first line's second word.
   - **The traffic**: three valid envelopes across *two different days*
     (each must get 202), one junk body (400, and assert the error text).
   - **The signal**: `Command::new("kill").arg("-INT").arg(child.id().to_string())`
     — the real thing a terminal ^C delivers; no signal crate needed.
   - **A6b, the exit**: poll `child.try_wait()` against a deadline (~10 s)
     instead of a bare `wait()` — a bare wait would hang the whole suite
     forever *precisely when shutdown regresses*, and you want a loud
     failure with a kill, not a stuck CI lane. Assert exit code **0**, and
     that the remaining stdout contains
     `relay: drained, 3 events on disk. bye.` — the writer's own count, as
     protocol.
   - **A6a, the refusal**: after exit, `TcpStream::connect` to the port
     must fail. (Refusal *during* the graceful window is real — axum closes
     the listener on the signal — but racy to catch deterministically; the
     post-exit form still proves the listener died with the process. Pin
     the pragmatism in a comment.)
   - **A6b, the disk**: `disk_lines` multiset equals the three 202'd bodies
     (canon both sides), the reject is nowhere, and each line sits in the
     folder its own ts names.

   One forward-looking line while you're here: give the child
   `.env("MEMLENS_TRACE", "/dev/null")` — move 5 wires the lens feature,
   and a lens build of this test must not drop a trace in the repo (glake's
   cli tests set the same convention).

   ```console
   cargo test -p relay --test shutdown
   # → running 1 test
   #   test a6_sigint_drains_flushes_and_exits_zero ... ok
   ```

   **Commit point:**

   ```console
   git add crates/relay && git commit -m "005: sitting N — A6 real-SIGINT test green"
   ```

4. **Hygiene sweep — A8 and A9 (Claude drives, you interrogate; F's
   division of labor).** Four checks, mechanized where possible:

   - **A9, made law.** In `crates/relay/Cargo.toml`:

     ```toml
     [lints.clippy]
     unwrap_used = "deny"
     expect_used = "deny"
     ```

     Question to make Claude answer before committing: A9's text says "no
     `unwrap()`/`expect()`" — why are there *two* lines here, and what
     would `unwrap_used` alone miss? (Exactly what it says: `expect()` is a
     separate lint, and a lint gate that half-enforces its rule teaches the
     wrong lesson.) Then run clippy and deal with the fallout: your
     *handler and writer paths* should already be clean (you've been living
     the rule since L), but the lint now bites **tests**, where unwrap is
     fine — a panicking test is a failing test. The house move: a file-level
     `#![allow(clippy::unwrap_used)]` at the top of each test file (and
     `#[allow(...)]` on in-module test mods). Relay is the first
     deployable-shaped crate, so this is the first crate where the
     Cloudflare lesson is *enforced by the build* — 006 inherits the lint
     line verbatim.
   - **The lens, wired the F way.** Port your glake pattern — this should
     take five minutes because you interrogated every line of it in F:
     optional dep + feature (`memlens = { path = "../memlens", optional = true }`;
     `lens = ["dep:memlens", "memlens/memlens"]` — say again what each half
     does, and which failure is silent), the `#[global_allocator]` static
     and `let _session = memlens::session("relay");` at the top of `main`,
     both behind `#[cfg(feature = "lens")]`. Build both worlds; clippy both
     worlds.
   - **A8, the tree.** The allow-list is the requirements' A8 line; hold
     the tree against it:

     ```console
     cargo tree -p relay -e normal --depth 1
     # → relay v0.1.0 (…/crates/relay)
     #   ├── anyhow v1.…            (bin only — check: main.rs is its only user)
     #   ├── axum v0.8.…
     #   ├── clap v4.…
     #   ├── glake v0.2.0 (…/crates/glake)   ← the validation logic, NOT reimplemented
     #   ├── serde_json v1.…
     #   ├── thiserror v2.…
     #   ├── tokio v1.…
     #   ├── tracing v0.1.…
     #   └── tracing-subscriber v0.3.…
     cargo tree -p relay -e normal | grep -ci memlens     # → 0   (default world: absent)
     cargo tree -p relay -e normal --features lens | grep -ci memlens
     # → nonzero (the lens exists only when asked for)
     ```

     Two things to interrogate: `tower` and `proptest` are missing — where
     are they? (`-e normal` again: dev-dependencies never ship — F's
     false-alarm lesson.) And `tracing-subscriber`: the A8 allow-list says
     "tracing" — is the subscriber a violation? It's tracing's output half,
     bin-side only, and the reference ships it too — but the honest answer
     is that A8's text doesn't *name* it. Record the question in
     `evidence.md` for the close-out gate rather than hand-waving it; specs
     get amended by exactly this kind of noticing. (F's paper cut, recalled
     before it bites: `grep -c` prints `0` *and exits 1* — fine at the
     prompt, fatal in a future `set -e` script.)
   - **The full gate, both configs** (lens test runs get the trace
     redirect):

     ```console
     cargo fmt --check
     cargo clippy -p relay --all-targets -- -D warnings
     cargo clippy -p relay --all-targets --features lens -- -D warnings
     cargo test -p relay
     MEMLENS_TRACE=/dev/null cargo test -p relay --features lens
     ```

     Same suite, both worlds, green twice. (The reference's count for
     scale: 14 tests — 5 unit, 2 CLI, 4 routes, 2 properties, 1 shutdown —
     identical in both configs. Your numbers may differ; both-configs-green
     may not.) **Commit point:**

     ```console
     git add crates/relay Cargo.lock
     git commit -m "005: sitting N — hygiene: lints denied, lens wired, tree checked"
     ```

5. **The payoff, part one — the live agreement (you drive).** Everything so
   far was tests watching the relay; now *you* watch it. Scratch lake, real
   binary, real curl (write the three envelopes yourself — distinct
   `event_id`s, two or three distinct days, realistic `event_type`s):

   ```console
   cargo run -p relay -- --lake /tmp/goldeneye-smoke &
   # → relay: listening on 127.0.0.1:7311

   # three valid POSTs (two days), one junk:
   curl -s -o /dev/null -w "%{http_code}\n" -X POST localhost:7311/events -d @event1.json   # 202
   #   … ×3, then:
   curl -s -X POST localhost:7311/events -d '{"not":"an envelope"}'
   # → {"error":"missing key: event_id"}
   curl -s localhost:7311/healthz
   # → {"accepted":3,"received":4,"rejected":1}

   kill -INT %1
   # → relay: drained, 3 events on disk. bye.          (and exit 0 — check with: wait %1; echo $?)
   ```

   The validation-day transcript, for comparison — yours should differ only
   in days and ids:

   ```
   POST /events (3 valid, distinct days)      → 202, 202, 202
   POST /events '{"not":"an envelope"}'       → 400 {"error":"missing key: event_id"}
   GET  /healthz                              → {"accepted":3,"received":4,"rejected":1}
   kill -INT <pid>                            → exit 0
   relay: drained, 3 events on disk. bye.
   lake: dt=2026-07-05/events.jsonl (1 line) · dt=2026-07-09/events.jsonl (2 lines)
   ```

   Now the cross-examination — your 004 tool reads what your 005 tool wrote:

   ```console
   cargo run -p glake -- stats /tmp/goldeneye-smoke
   ```

   Validation day, that printed:

   ```
   2 files · 3 events

   by type
     bolt.done                     1
     gate.approved                 1
     session.start                 1

   by day
     2026-07-09                    2
     2026-07-05                    1
   ```

   Read the agreements off, one by one, into your notes: glake's grand
   total = relay's `accepted` counter (3 = 3); glake's by-day map = the
   `dt=` dirs = the days the events' own `ts` claimed; relay's quiescent
   invariant 4 = 3 + 1. Two programs, two specs, one lake, zero
   disagreement — this paragraph is what "every deployed artifact has a
   spec trail" looks like when it works.

6. **The payoff, part two — the lens on a *service* (A10, you drive).** F
   pointed the lens at a CLI: one run, one exit, one trace. A service is a
   different animal — a long-lived runtime, per-request churn, a writer
   ticking in the background — and *this* is the observation that matters
   for AWS, because 006 deploys this exact handler and Lambda bills for
   the memory that holds it. **Predict first, in writing**: for one
   accepted POST, end to end, how many heap allocations — order of
   magnitude? Where do they happen — axum's request machinery, your door
   (count its parses: the classifier's, the re-serialization's, and the
   writer's re-derive — move 4 of M priced this), the channel, the writer?
   And which of those happen *once per process* versus *once per request*
   — the distinction that separates a cold-start cost from a
   traffic-multiplied cost?

   Then run traced — **redirect the trace out of the lake**: the relay is
   *writing* a lake this time, and F taught you what happens when the
   instrument's output lands inside the experiment:

   ```console
   MEMLENS_TRACE=target/relay-lens.jsonl cargo run -p relay --features lens -- --lake /tmp/lens-lake
   # ~36 POSTs across 3 days — a shell loop; vary the ts day every third event
   # then ctrl-c:
   # → relay: drained, 36 events on disk. bye.
   ```

   Open `viewer/memlens.html`, drop `target/relay-lens.jsonl` on it. The
   validation-day trace, for scale (36 accepted events, debug build):
   ~7,400 trace lines — ~3,700 allocs / ~3,600 deallocs / 98 reallocs;
   ~1.26 MB allocated across the whole session; net live at exit ≈73 blocks
   (runtime globals and the per-day file cache, freed past the last flush).
   The headline number: **≈103 allocations per accepted event**, end to
   end. Yours will differ; the shape shouldn't. Four hunts:

   - **The baseline.** Scrub to before your first POST: the runtime,
     listener, and router allocate a working set *once*. That block is the
     cold-start bill — on Lambda it's paid at init, and it's why Rust's
     no-GC, no-warmup story matters there: what you see is all there is,
     no JIT warming up behind it.
   - **The per-request cluster.** Find one POST's burst and pick it apart
     against your prediction: request buffers (the validation trace shows a
     632 B allocation ×~217 — one-ish per request plus change), the door's
     two `serde_json` parses and the compact re-serialized `String` (M's
     two-parse decision, now with a price tag you can read), the writer's
     re-derive parse, small-string noise (5–48 B) from JSON keys and
     values. This cluster × your traffic is the number that scales; the
     baseline isn't.
   - **The writer's rhythm.** Allocations from the file cache happen once
     per *day*, not per event (the `entry` API earning its keep, visibly),
     and the per-line work is nearly flat — you moved the line's `String`
     in, pushed one byte, wrote. Compare its footprint to a handler's.
   - **The contrast with F.** glake's whole-lake `stats` run traced ~700
     allocations *total*; the relay spends ~103 *per event*. Neither is
     wrong — a CLI amortizes over a batch, a service pays per request —
     but say which number Lambda multiplies by traffic, and which one
     `mem`-sizing (and therefore billing) follows. That sentence, in your
     own words, is A10's deliverable.

   **Write it down.** Append to `specs/005-async-relay/evidence.md` (start
   from `specs/_template/` if it's the first entry): the smoke transcript
   numbers and the glake agreement (move 5), the A8 tree facts and the
   tracing-subscriber question (move 4), and ≥3 lens observations *with
   numbers* — your per-event allocation count, the baseline-vs-per-request
   split, the writer's once-per-day pattern. Evidence is append-only,
   plain, and yours. **Commit point:**

   ```console
   git add crates/relay specs/005-async-relay/evidence.md
   git commit -m "005: sitting N — payoff: live glake agreement + lens notes to evidence"
   ```

   (Close-out — evidence polish, property-auditor, SKILLS 1d updates,
   MEMORY, the `spec-close/005-async-relay` marker — is task 2.1, Claude's
   machine work, next session.)

## Compiler fights to expect

Lighter on compile errors, heavier on the kind with no diagnostic — which is
this sitting's theme. Ledger (`learning.*`) as always.

- **The hang, in production clothes** — the footgun itself. No error, no
  warning, no panic: a process that closed its listener and will wait
  forever at `writer.await`. You predicted it (move 1), witnessed it
  (move 2), and wrote the test that would catch its return (move 3's
  timeout poll — which turns "hangs forever" into "fails in ten seconds").
  The compiler proves memory safety, not liveness; past that border,
  correctness is on your design. This is the most important non-error in
  the course.
- **`error[E0382]: use of moved value: `tx``** — if anything in `main`
  touches `tx` after `AppState::new(tx)`. Read it warmly: the signature
  took the sender by value *so that* this error exists — the API making the
  footgun hard to write. (It can't stop `tx.clone()`, which is why move 1
  is a prediction exercise and not a compiler exercise.)
- **`error[E0433]: … use of unresolved … `memlens``** — on the *default*
  build, if any lens mention escapes its `#[cfg(feature = "lens")]` gate.
  F's defining optional-dep fight: with the feature off, the crate wasn't
  compiled, wasn't linked, doesn't exist. Every mention carries the gate.
- **clippy: `used `unwrap()` on a `Result``, denied** — the new lint biting
  a test file you forgot to `#![allow(clippy::unwrap_used)]`, or — better
  catch — a real straggler in library code that predates the law. The lint
  sweep exists to find exactly these; fix the code path, not the lint
  config, when it's a handler.
- **`error[E0308]: mismatched types` at `with_graceful_shutdown`** — passing
  `shutdown_signal` (a function) instead of `shutdown_signal()` (its
  future). Step 12's inert-future lesson from a new angle: axum wants the
  state machine, not the recipe for one.
- **The A6 test's own failure modes** (test bugs, not relay bugs — classify
  before fixing, per the triage discipline): reading stdout without
  `Stdio::piped()` (nothing to read); asserting on stdout that contains log
  noise because tracing went to stdout (your move-2 stderr discipline is
  what the test is standing on); a child leaked past a failed assert (the
  timeout helper kills on its way out — check yours does).

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                        # no diff
cargo clippy -p relay --all-targets -- -D warnings       # clean, default world
cargo clippy -p relay --all-targets --features lens -- -D warnings
                                                         # clean, lens world
cargo test -p relay                                      # ALL green (reference scale: 14)
MEMLENS_TRACE=/dev/null cargo test -p relay --features lens
                                                         # same suite, lens world, green
cargo tree -p relay -e normal | grep -ci memlens         # 0
cargo tree -p relay -e normal --features lens | grep -ci memlens   # nonzero
```

```
# the live ritual, end to end, against a scratch lake:
cargo run -p relay -- --lake /tmp/goldeneye-smoke        # 3 x 202 + 1 x 400 + healthz + ctrl-c
# → {"accepted":3,"received":4,"rejected":1}             # quiescent: 4 = 3 + 1
# → relay: drained, 3 events on disk. bye.               # exit 0
cargo run -p glake -- stats /tmp/goldeneye-smoke         # glake total == relay accepted,
                                                         # by-day == the dt= dirs exactly
```

- `crates/relay/Cargo.toml` carries `unwrap_used = "deny"` **and**
  `expect_used = "deny"`; the A6 test passes with a *real* `kill -INT` and
  asserts exit code 0, the drained line, the refused connection, and the
  disk multiset.
- `target/relay-lens.jsonl` exists (and is *not* in the lake), has been
  opened in `viewer/memlens.html`, and `specs/005-async-relay/evidence.md`
  holds the smoke numbers, the glake agreement, the A8 notes, and ≥3 lens
  observations with real numbers — including your per-accepted-event
  allocation count (validation calibration: ≈103).
- `git log --oneline -4` shows shutdown → A6 test → hygiene → evidence.
- You can answer aloud, cold: the footgun — a `tx` clone survives in `main`;
  narrate the exact mechanism of the hang and the one-line fix. Why must
  the A6 test use a real child and a real signal? Why is exit code 0 part
  of the *requirement* and not a nicety? (Think: systemd on EC2, Lambda's
  runtime — supervisors act on exit codes; a drain that needs SIGKILL is
  data loss on a schedule.) And the A10 sentence: which allocations does
  traffic multiply, which does a cold start pay once, and what does that
  distinction do to how you'd size — and pay for — this handler on Lambda
  next spec?

## Hints (one at a time)

<details><summary>Hint 1 — the shutdown order, as a diagram to keep</summary>

```text
1. (tx, rx) = mpsc::channel(256)            bounded — 202 means "enqueued"
2. writer = tokio::spawn(writer::run(rx))   the owner starts first
3. state  = AppState::new(tx)               tx MOVES in; main keeps NO clone
4. axum::serve(...).with_graceful_shutdown(ctrl_c).await
      └─ ctrl-c → listener closes (A6a), in-flight finish,
         serve returns, router+state drop
           └─ last Sender drops → writer's recv() → None
5. writer.await                             drain + flush finish (A6b)
6. goodbye line, exit 0
```

If your relay hangs at step 5, some sender survived step 4 — and you know
the usual suspect by name now. If it exits but events are missing, look at
step 4½: did the writer get a chance to drain, or did you print the goodbye
before `writer.await`?

</details>

<details><summary>Hint 2 — the timeout poll for the child</summary>

```rust
fn wait_with_timeout(child: &mut Child, limit: Duration) -> ExitStatus {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("relay did not exit within {limit:?} after SIGINT");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
```

Read the `panic!` as the feature: a shutdown regression becomes a named,
ten-second failure with a cleaned-up child — instead of a test suite that
sits until CI's global timeout reaps it and tells you nothing.

</details>

<details><summary>Hint 3 — no trace file / wrong numbers in the lens run</summary>

F's Hint-1 checklist still rules: the feature line needs *both* halves
(`dep:memlens` + `memlens/memlens` — the second one arms it; without it the
run is silently traceless), and you must actually build `--features lens`.
New to the service setting: use `MEMLENS_TRACE` (the full-path override),
not `MEMLENS_PROGRAM` — the relay's cwd-relative default would drop the
trace next to wherever you launched from, and if that's inside the lake the
relay is writing, your dt= dirs grow a surprise (the observer effect, F
move 5 — this time you're the lake's *writer*, so the pollution would be
permanent, not just a skewed reading). And take the healthz/`glake stats`
numbers with the lens *off* if you want them comparable to move 5's — a
traced debug binary is slower, and a big curl loop against it can back the
channel up, which is backpressure working, but makes timing-sensitive
observations noisier.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/005-async-relay/_reference/relay/src/main.rs` — the wiring order
  (its module docs are the Hint-1 diagram with commentary) and
  `shutdown_signal`.
- `specs/005-async-relay/_reference/relay/tests/shutdown.rs` —
  `http_post_events` (the hand-rolled request), `wait_with_timeout`, and
  the single A6 test's assertion order.
- `specs/005-async-relay/_reference/relay/Cargo.toml` — the `[lints.clippy]`
  table and the lens feature. Path warning as ever: the reference's
  `memlens` path is `../../../../crates/memlens`; from `crates/relay`,
  yours is `../memlens`.
- `specs/005-async-relay/_reference/NOTES.md` — the validation numbers this
  guide quotes (smoke transcript, 14/14, the lens trace breakdown), plus
  fourteen semantic decisions worth reading *after* your build, as a
  compare-notes exercise.
