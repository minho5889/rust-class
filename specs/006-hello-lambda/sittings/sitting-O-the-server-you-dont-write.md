# Sitting O — the server you don't write

**Builds:** `crates/hello-lambda` — relay's door as a Lambda handler: one
async fn that is the entire HTTP surface, the H4 door-equivalence property
proving hello and relay can never drift (red first, against a stub), honest
per-instance healthz state, and the emit seam that turns "write to the lake"
into "print one line to stdout". No AWS today: everything runs and is proven
locally (H5).
**Requirements:** H1–H5, H7 (+H12 context) — tasks 1.1–1.2 (T1: the Lambda
model; T2: bootstrap; T4: per-instance state).
**Ramp you'll use:** none new — the ramp closed at step 13. Step 12's async
model, sitting K's crate ritual, M's channel-ownership lesson, and N's
stdout-is-protocol discipline are the substrate; the opener is a recall
drill instead (tasks.md section 0).

## Where you are

005 closed with a whole server you wrote by hand: listener, router, shared
counters, single-writer task, graceful shutdown — proven down to a real
SIGINT. 006 moves the door to Lambda, and the lesson is everything that
*disappears*: the platform owns listening, concurrency, and shutdown, and
what's left of your program is one `async fn(Request) -> Response`. You know
this platform better than the course does — you've explained execution
environments and cold starts to paying customers. What's different now is
that the runtime is **your binary**: `provided.al2023` just execs a file
named `bootstrap`, no JVM to warm, no GC to size, and this sitting is where
that stops being a slogan and becomes code you wrote. The Rust is the
teaching target; the AWS side is a conversation between peers.

One piece of forward context (H12): the endpoint this handler eventually
gets is a **public, unauthenticated** Function URL — a written, bounded
decision. You review it in sitting P as the security reviewer; today just
know the code you're writing will stand at an open door, which is one more
reason the handler path bans `unwrap()` outright (H7 — a panicking handler
takes the whole instance down mid-request).

## The build, move by move

All commands from the repo root. Four commits this sitting: scaffold, red,
handler, green — tasks 1.1 and 1.2's pairs.

1. **The recall drill (10 minutes, closed book).** Before any code, write
   the 005 door contract from memory:

   - the three clauses of "valid", in order;
   - the fixed first-problem order and the **three verbatim error strings**;
   - the statuses: what did 202 *promise* in relay (accepted = what,
     exactly)? What does 400 carry? What does a request to no route get?
   - the healthz keys and the quiescent invariant.

   Then open `specs/006-hello-lambda/requirements.md` — "the door check,
   stated in full" — and grade yourself. Ledger the result (`learning.*`:
   cold / needed the doc). The drill isn't busywork: the H4 property is
   about to make drift between two deployments a test failure, and you are
   the person who reviews that property. You can't review drift you can't
   recall. (While you're in requirements.md, reread H1's parenthetical —
   "the door path returns the line through a testable seam" — it's the
   design constraint moves 4–5 build around.)

2. **Scaffold (task 1.1, first half).** `cargo new crates/hello-lambda` —
   the workspace glob adopts it the moment the folder exists, exactly as it
   adopted relay in sitting K: shared lock, shared `target/`, and the
   workspace release profile (thin LTO, `panic = "abort"`) waiting for
   sitting P's build. Give it relay's lib+bin shape (`src/lib.rs` with
   `pub mod handler; pub mod state;` and `#![forbid(unsafe_code)]`; a thin
   `src/main.rs`) — required so tests can drive the handler in-process (H5)
   without a network. Then the manifest, and read it as the T1 lesson in
   Cargo.toml form:

   ```toml
   [dependencies]
   lambda_http = { version = "1.3", default-features = false, features = [
       "apigw_http",
       "tracing",
   ] }
   tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
   serde_json = "1"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", default-features = false, features = ["fmt"] }
   glake = { path = "../glake" }

   [dev-dependencies]
   proptest = "1"
   relay = { path = "../relay" }
   tower = { version = "0.5", features = ["util"] }
   axum = "0.8"
   tokio = { version = "1", features = ["sync"] }

   [lints.clippy]
   unwrap_used = "deny"
   expect_used = "deny"
   ```

   Things to say aloud before committing:

   - **lambda_http is feature-picked**, same discipline as relay's tokio
     block: Function-URL payloads are API-Gateway-v2-shaped, so you name
     `apigw_http` and never compile the ALB/REST/WebSocket/VPC-Lattice
     codecs you'll never receive. (`tracing` keeps the runtime's own
     diagnostics flowing into your subscriber instead of vanishing.)
   - **tokio shrank from relay's seven features to two.** Read the diff —
     it *is* T1: no `signal` (the platform owns shutdown), no `sync` (no
     channel, no writer task), no `fs` (no file — stdout is the sink), no
     `net` (no listener to bind). The dev-dependency adds `sync` back for
     tests only: the H4 harness builds relay's channel itself.
   - **glake, third caller** (count them: glake's own bin, relay, now
     hello — the 004 payoff compounding). The door will be *re-derived*
     through the same glake API relay used, not imported from relay — relay
     stays a **dev-dependency**, the H4 oracle. That's deliberate: 006's
     lesson is that the door survives the platform move *because both
     callers use the same library*, and H4 exists to prove non-drift
     instead of assuming it.
   - **The lints table is relay's, verbatim** — sitting N promised "006
     inherits the lint line verbatim"; here's the inheritance. Deployable
     crate, Cloudflare lesson, no `unwrap()`/`expect()` in the handler path.
   - **Deliberately absent: `aws-sdk-*`.** This binary is the no-SDK
     baseline that 007 gets measured against (H7 — the checkpoint greps
     the tree for it).

   Now the stub that gives the red property something to compile against:
   in `handler.rs`, the `Emit` trait (one method, `fn emit(&self, line:
   &str)`, and make the trait `: Sync` — you'll see why in the fights),
   `StdoutEmit` implementing it with a `println!`, and a
   `handle_with(req, emit)` whose match has exactly two shapes: the
   `("POST", "/events")` arm is, for now, one line —
   `todo!("the door: three clauses, fixed order")` — and everything else
   answers 404 through a tiny `empty_response(status)` helper. Writing
   that helper is your first contact with the builder's `Result` (the
   fights section has the two E0308s waiting there). Keep the stub
   import-clean (a `todo!()` body needs no glake imports yet; the clippy
   gate applies at every commit). **Commit point:**

   ```console
   cargo fmt
   cargo clippy -p hello-lambda --all-targets -- -D warnings
   git add crates/hello-lambda Cargo.lock
   git commit -m "006: sitting O — scaffold: crate, deps, stub door"
   ```

3. **H4 red-first (task 1.1, second half — co-written).** Property
   harnesses are co-written per the constitution, and this one earns it:
   it has three subtleties that would each fail the property *for the
   wrong reason*, and the design names all three (design.md, Properties
   row). Build `tests/prop_door.rs` together:

   - **relay is the oracle, and it must be alive.** Build relay's router
     exactly the way relay's own tests do: make the channel, wrap the
     sender in `AppState`, `oneshot` requests at the router. And hold the
     receiver — `let (tx, _rx) = tokio::sync::mpsc::channel::<String>(256);`
     — where `_rx` is a *real binding*. You know this fight from N's
     footgun, inverted: a bare `_` pattern drops the receiver immediately,
     and relay's handler only 202s while the receiver lives. Get it wrong
     and the property fails with verdicts that look like drift but are
     harness damage (verified, receiver deliberately dropped):

     ```
     Test failed: assertion failed: `(left == right)`
       left: `(202, "")`,
      right: `(500, "{\"error\":\"writer unavailable\"}")`: doors drifted on body "…"
     ```

     Triage discipline: that's a **test bug**, not a code bug — classify
     before fixing, log it if you hit it.
   - **The domain bound.** Every generated body stays ≤ 64 KiB, asserted
     in the harness (`prop_assert!(body.len() <= 64 * 1024, …)`). Above
     the bound the two shapes legitimately diverge for reasons that are
     *not the door*: relay's axum answers oversized bodies with its stock
     413 before your handler runs, and Lambda's platform rejects payloads
     over ~6 MB before the handler ever sees them. H4 (as amended) scopes
     the property under the bound; the divergence gets documented in the
     test's module docs, not reconciled.
   - **Capacity vs batch.** Channel capacity 256, and
     `prop_assert!(bodies.len() < 256, …)` — with nobody draining, a
     `send().await` that hits capacity would hang the property, so the
     harness asserts the precondition instead of just believing it.

   The rest is your sitting-M toolkit: proptest drives a **sync** body
   that builds a current-thread runtime and `block_on`s (equivalence is
   per-request; nothing needs to race — generation stays deterministic),
   and the generator is the A4 message strategy you wrote in sitting M,
   ported: valid envelopes over a multi-day domain (sometimes
   pretty-printed, so bodies span lines), the three malformed arms
   (non-objects, one required key removed, uncomparable ts), occasional
   exact duplicates — the door must be stateless, same body → same
   verdict, no dedup "helpfulness". Two riders make the property carry
   H1/H2 as well: drive hello through a Vec sink and assert **exactly one
   emitted line iff 202**; compare `(status, body text)` pairwise against
   relay. `#![proptest_config(ProptestConfig::with_cases(512))]` — the
   design floor is 256; batches make each case several bodies, so a run
   compares a few thousand verdict pairs.

   Run it and savor the red (reference transcript, door stubbed):

   ```
   running 1 test
   test prop_h4_door_equivalence ... FAILED

   ---- prop_h4_door_equivalence stdout ----
   proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
   thread 'prop_h4_door_equivalence' panicked at src/handler.rs:…:
   not yet implemented: the door: three clauses, fixed order
   …
   Test failed: not yet implemented: the door: three clauses, fixed order.
   minimal failing input: bodies = [
       "{\"actor\":\"proptest\",\"event_id\":\"0a00aa\",\"event_type\":\"session.start\",…}",
   ]
   test result: FAILED. 0 passed; 1 failed
   ```

   Two reading notes: proptest *shrank* the failing batch to a single
   minimal valid envelope — the machinery works before the code exists —
   and the `FileFailurePersistence` warning is proptest failing to park a
   regression seed for an integration test; expected-red is not a genuine
   counterexample, so there's nothing to commit (the ops checklist's
   "seed committed on genuine failure" stays dormant unless H4 catches
   real drift later). **Commit point:**

   ```console
   git add crates/hello-lambda
   git commit -m "006: sitting O — H4 red against the stub door"
   ```

4. **The door (task 1.2 begins).** Replace the `todo!()` with the real
   thing: `fn door(body: &str) -> Result<String, String>` — `Ok` carries
   the compact line to emit, `Err` the first problem's message. One match
   on your own `SerdeParser.classify(body, &REQUIRED_KEYS)`:

   - `Blank | Unparseable` → `"not a json object"` — a whitespace-only
     body and un-JSON junk get the same words at this door. Same asymmetry
     as 005: glake-the-reader *counts* what it can't parse; a door turns
     it away.
   - `Malformed { missing }` → `format!("missing key: {missing}")` — glake
     reports the first missing key in `REQUIRED_KEYS` order, which is what
     makes the fixed order fall out of the library instead of your code.
   - `Event { day, .. } if day == "bad-ts"` → `"no comparable day in ts"`
     — glake never rejects on ts, it *buckets*; the sentinel comes back
     and the strict door refuses it. (relay's exact move.)
   - `Event { .. }` → parse to `serde_json::Value`, return
     `value.to_string()`. Two parses per accepted event — the price
     sitting M put a number on — because a pretty-printed body legally
     spans lines and must land as ONE compact line (A4 rev 2.1's rule,
     alive in its third crate). One pleasant difference from relay: this
     path uses `Value`'s `Display` (`to_string()`), which is infallible —
     no impossible error branch to map away, where relay's
     `serde_json::to_string` made you handle one.

   The three error strings are relay's `Rejection` Display texts
   **verbatim**. They are the contract (H2, H4) — and the property you
   just wrote is the tripwire that fires if either side ever rewords one.

5. **The handler around it.** `handle_with` grows real arms — and notice
   what the match *is*: what axum's router did with a radix tree, three
   match arms do here. At this size, **the match is the router** (T1).
   Three mechanical fights live in this move; the fights section has the
   full error texts:

   - **Body in**: `req.body()` is `&lambda_http::Body` — an *enum*
     (`Empty` / `Text` / `Binary`), because the platform hands you the
     payload already read, possibly base64-decoded. It's also
     `#[non_exhaustive]`, so the compiler demands a wildcard arm (E0004 —
     the crate author reserving the right to add variants). Fold
     everything that can't be a JSON object — empty, non-UTF-8 binary,
     unknown variants — to `""`, which the door already answers with its
     first words. (Documented divergence, outside H4's domain: axum would
     answer a non-UTF-8 body with its own rejection text; proptest
     generates Rust `String`s, which are UTF-8 by construction.)
   - **Response out**: `Response::builder()…body(…)` returns `Result` —
     a builder can be fed garbage (status 9999, an invalid header), so
     the type says so. Your inputs are constants and can't fail, but H7
     bans the `unwrap()` you'd reach for — and `?` converts `http::Error`
     into `lambda_http::Error` (a `Box<dyn Error>`) for free. You wrote
     `empty_response(status)` for the stub; add `json_response(status,
     value)` (compact body, `content-type: application/json`) and route
     everything through the pair. One byte-level detail that matters to
     H4: `Value::to_string()` prints the same compact form axum's `Json`
     writes — that's what keeps your 400 bodies byte-identical to
     relay's.
   - **The seam**: the door *returns* the line; `println!` lives only in
     `StdoutEmit::emit` at the outermost edge (H1's parenthetical, and
     relay's writer-seam lesson recycled — there tests drained a channel,
     here they'll hand in a Vec sink). Cost: one dyn dispatch per
     *accepted* event, nothing on the reject or healthz paths. This is
     also 007's swap point — an S3-backed sink replaces `StdoutEmit`
     without touching the door or the routing. And the trait needs
     `: Sync`, because `&dyn Emit` rides inside a future the runtime may
     move between worker threads — if you dropped the bound in move 2,
     now's when the compiler collects (see the fights: it's the best
     Send/Sync diagnostic you'll read this phase).

   On the reject path: count it, answer
   `{"error": …}` as 400, and send the diagnostic to `tracing::debug!` —
   **stderr, never stdout** (H2: no event line for rejected bodies).

6. **state.rs + main.rs — per-instance, eagerly (H3a).** The state module
   is relay's counters kept *verbatim on purpose*, because their meaning
   is about to change underneath them (that's move 7's conversation).
   The mechanics:

   - a `static INSTANCE: OnceLock<Instance>` with
     `instance() -> &'static Instance` doing `get_or_init`. `OnceLock` is
     the standard library's answer to "a global initialized exactly once,
     thread-safely, no `unsafe`, no macro crate" — in relay this job was
     done by `Arc<Counters>` threaded through axum state; here there's no
     router to carry state for you, and a process-global is the *honest*
     shape: the state really is "whatever this one process remembers".
   - identity, two sources in order: `$AWS_LAMBDA_LOG_STREAM_NAME`
     verbatim when present — the platform mints one log stream per
     instance, so it's a unique identity that costs nothing and lets you
     jump from a healthz answer straight to that instance's CloudWatch
     stream on deploy day — else `i-` + 16 hex from init-nanos ⊕ pid
     through one round of SplitMix64. No `rand` dependency: the entropy of
     "when did this process start, at nanosecond grain" is plenty for a
     lab id that only has to make two instances *visibly* different.
   - `started`: seconds-since-epoch formatted by a hand-rolled,
     unit-pinned civil-from-days (Hinnant's algorithm — the one every date
     library uses underneath). "RFC3339-ish", no leap seconds — exactly as
     honest as the envelope `ts` the hooks emit. A date crate was judged
     not worth its binary weight for one string formatted once per cold
     start; sitting P's bloat read will show you the ledger that judgment
     was made against. Pin it with a unit test against known instants
     (epoch, a leap day, one real timestamp you cross-check with
     `date -u -d @…`).
   - `main.rs` is four lines of wiring you still own, in order:
     subscriber → `std::io::stderr` FIRST (stdout is protocol — sitting
     N's discipline with a new reader: both streams land in the same
     CloudWatch stream, but only stdout stays greppably pure event
     lines); **eager state touch** — call `state::instance()` and log a
     `cold start` line *before* handing control to the platform, so
     `started` means cold-start time, never first-request time (H3a's
     explicit demand — and that stderr line doubles as your cold-start
     marker for the H10 experiment in Q); then
     `run(service_fn(handler::handler)).await`. That last line is the
     server you don't write: long-poll the Runtime API, deserialize the
     payload into an `http::Request`, call you, post the answer back,
     repeat. One event at a time per instance; the fleet, not the
     process, is the concurrency unit. **Commit point:**

   ```console
   cargo fmt && cargo clippy -p hello-lambda --all-targets -- -D warnings
   git add crates/hello-lambda Cargo.lock
   git commit -m "006: sitting O — handler: the door goes to the cloud"
   ```

7. **The example tests, the T4 conversation, and green (task 1.2
   closes).** `tests/handler.rs`: H1–H3b as `#[tokio::test]`s driving
   `handle_with` with hand-built `lambda_http` Requests and a Vec sink
   behind the `Emit` trait — no AWS, no network, no stdout-capture
   gymnastics (H5). Use a real hook-emitted envelope from the live lake
   as the H1 fixture (the same line relay's A1 test uses — two specs,
   provably the same traffic; its extra `spec_id` key is part of the
   point: the door checks required keys are *present*, it doesn't forbid
   more). The suite's one structural subtlety: the counters are a process
   global, and the test harness runs `#[tokio::test]`s on parallel
   threads *in one process* — so every handler-driving test holds a
   shared `static SERIAL: tokio::sync::Mutex<()>` (tokio's lock, not
   std's: the guard lives across `.await`s, which is exactly the mistake
   `clippy::await_holding_lock` exists to catch in the std version), and
   H3a is asserted as **before/after deltas**, not absolutes.

   That awkwardness is not a test smell — it's the T4 lesson leaking into
   the harness, and it's where the conversation happens. Answer these in
   writing, from your support-engineer years, before running the suite:

   - Where do these three `AtomicU64`s live, in platform terms? Who else
     can ever see them?
   - relay's `received=41` described *the service*. What does
     `received=41` describe here? Write the one-sentence reply you'd have
     sent on a ticket that read: "we POSTed 200 events but healthz says
     accepted=3 — where did they go?"
   - When do the counters reset, and who decides? You've read the
     execution-environment lifecycle docs more times than anyone should —
     now it's *your* `OnceLock` being reaped. What does that do to every
     "just keep it in memory" plan you've watched a customer ship?
   - The atomics are now strictly heavier than needed — one request at a
     time means they're never contended. Why keep them? (Two answers:
     it's the same code you wrote in 005, kept so the *meaning* change is
     visible; and `Relaxed` ops compile to plain loads/adds on an
     uncontended path — you pay only when contended, and here you never
     are. Zero-cost the way Rust likes it.)
   - The one that pays on deploy day: why do `instance` and `started`
     exist in the healthz body at all? (Sitting Q watches two concurrent
     curls land on two different instances — T4 made visible with two
     shell commands.)

   Last arm: the **404 fallthrough** (H3b). Anything that isn't exactly
   `POST /events` or `GET /healthz` — including right-path-wrong-method.
   Here the two servers *documentedly diverge*: relay's axum answers
   `GET /events` with 405 (its router knows the path exists), your match
   doesn't make that distinction, and H3b's letter says 404. H4 can't see
   the difference — its generator only speaks `POST /events` — so the
   divergence is outside the property's domain and pinned by example
   instead: the reference's 404 test drives `GET /events`,
   `POST /healthz`, `DELETE /events`, a trailing slash, and root, and
   also asserts 404s happen *before* the door — no counter moves, nothing
   emitted. Yours should too.

   Green, the full ritual:

   ```console
   cargo test -p hello-lambda
   ```

   Reference transcript (names and counts; yours should match in shape —
   10 tests across three targets):

   ```
   running 4 tests
   test handler::tests::door_reserializes_multiline_bodies_to_one_line ... ok
   test state::tests::instance_is_initialized_exactly_once ... ok
   test state::tests::rfc3339_utc_known_instants ... ok
   test handler::tests::door_verdicts_in_fixed_order ... ok

   running 5 tests
   test h1_multiline_body_emitted_as_one_compact_line ... ok
   test h1_valid_event_202_empty_body_one_line_emitted ... ok
   test h3_everything_else_is_404 ... ok
   test h2_invalid_bodies_400_in_fixed_order_and_emit_nothing ... ok
   test h3_healthz_shape_and_counter_deltas ... ok

   running 1 test
   test prop_h4_door_equivalence ... ok
   test result: ok. 1 passed; … finished in 0.52s
   ```

   Read the last line's time once more: 512 cases, a few thousand verdict
   pairs against a live relay router, half a second. **Commit point:**

   ```console
   cargo fmt && cargo clippy -p hello-lambda --all-targets -- -D warnings
   git add crates/hello-lambda
   git commit -m "006: sitting O — examples green, H4 green, lints hold"
   ```

## Compiler fights to expect

Ledger them (`learning.*`) as always — two are E0308s you've met in other
clothes, two are new to this platform's types.

- **`error[E0308]` at a response builder — `expected `Response<Body>`,
  found `Response<String>``** — the signature fight of the day. The
  builder's `body(T)` is generic; hand it a `String` and you've built a
  `Response<String>`, but the handler promises `Response<Body>`:

  ```
  error[E0308]: `?` operator has incompatible types
      = note: expected struct `lambda_http::Response<lambda_http::Body>`
                 found struct `lambda_http::Response<std::string::String>`
  ```

  The fix is one wrap: `Body::Text(value.to_string())`. Body is an enum
  because the platform speaks more than text — the type is telling you
  responses can be binary/base64 too.
- **`error[E0308]: mismatched types` — `expected `Response<Body>`, found
  `Result<Response<Body>, Error>``** — you wrapped the builder's `Result`
  in `Ok(…)` without the `?`. rustc's `help:` line even draws the arrow:
  `the type constructed contains … due to the type of the argument
  passed`. The `?` isn't optional ceremony: it's the conversion from
  `http::Error` to `lambda_http::Error`, and it's how H7's no-unwrap rule
  costs you zero lines here.
- **`error[E0277]: `dyn Emit` cannot be shared between threads safely`**
  — if the `Emit` trait lacks `: Sync`. Read the whole note chain slowly;
  it is the entire Send/Sync story of this phase in one diagnostic:

  ```
  = help: the trait `Sync` is not implemented for `dyn Emit`
  = note: required for `&dyn Emit` to implement `Send`
  note: required because it's used within this `async` fn body
     --> src/handler.rs:…
      |  pub async fn handle_with(req: Request, emit: &dyn Emit) -> …
  ```

  Bottom-up: the runtime may move your future between threads, so the
  future must be `Send`; the future captures `&dyn Emit`; `&T` is `Send`
  only when `T: Sync`. The error points at `main.rs`'s `run(service_fn(…))`
  line — the *call site* that demands the bound — but names the exact
  capture that breaks it. relay taught you this with `Arc`; here it
  arrives through a trait object.
- **`error[E0004]: non-exhaustive patterns: `&_` not covered`** — a match
  on `req.body()` without a wildcard arm. The note says it plainly:
  ``lambda_http::Body` is marked as non-exhaustive, so a wildcard `_` is
  necessary` — the upstream crate reserving the right to add variants
  without a semver break. Anything you don't recognize can't be a JSON
  object; fold it to `""`.
- **The one with no diagnostic: the dead oracle.** `let (tx, _) = …` in
  the H4 harness and every relay 202 degrades to
  `{"error":"writer unavailable"}` 500 — the property "catches drift"
  that is really harness damage (move 3 shows the exact output).
  Sitting N's footgun, mirror-imaged: there a *surviving* clone hung the
  drain; here a *dropped* receiver poisons the oracle. Triage it as a
  test bug and remember `_rx` binds, `_` drops.
- **clippy: `used `unwrap()`…`, denied** — in `tests/` files you forgot to
  head with `#![allow(clippy::unwrap_used)]`. Same rerun as N: a
  panicking test is a failing test, which is fine; the ban is for the
  handler path.

## Checkpoint

From the repo root — the sitting counts as done only when all of these
hold:

```
cargo fmt --check                                          # no diff
cargo clippy -p hello-lambda --all-targets -- -D warnings  # clean
cargo test -p hello-lambda                                 # all green — reference scale: 10
                                                           # (4 unit, 5 examples, 1 property @ 512 cases)
cargo tree -p hello-lambda -e normal --depth 1             # exactly: glake (path), lambda_http,
                                                           # serde_json, tokio, tracing,
                                                           # tracing-subscriber — nothing else
cargo tree -p hello-lambda -e normal | grep -c aws-sdk     # 0   (H7 — and recall F's paper cut:
                                                           #      grep -c prints 0 AND exits 1)
```

Reference tree, for the target shape (paths will be yours):

```
hello-lambda v0.1.0 (…/crates/hello-lambda)
├── glake v0.2.0 (…/crates/glake)
├── lambda_http v1.3.0
├── serde_json v1.0.150
├── tokio v1.52.3
├── tracing v0.1.44
└── tracing-subscriber v0.3.23
```

- `git log --oneline -4` shows scaffold → H4 red → handler → green.
- The T4 answers from move 7 are written down (they seed the evidence.md
  notes Q will finish).
- You can answer aloud: what are the three things the platform now owns
  that relay's `main.rs` did by hand? Why is relay a *dev*-dependency and
  what would importing its door as a normal dep have cost the lesson? Why
  must `Emit` be `Sync` — walk the chain from `run()`'s bound to the
  `&dyn`. Where exactly does the 64 KiB domain bound come from, and what
  do the two platforms each do just above their own limits? And the T4
  one-liner: what does `received` count, *per what*, since *when*?

## Hints (one at a time)

<details><summary>Hint 1 — the H4 harness skeleton</summary>

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]
    #[test]
    fn prop_h4_door_equivalence(bodies in batch()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all().build()
            .map_err(|e| TestCaseError::fail(format!("runtime: {e}")))?;
        rt.block_on(async move {
            let (tx, _rx) = tokio::sync::mpsc::channel::<String>(256); // _rx LIVES
            prop_assert!(bodies.len() < 256, "batch must fit the channel");
            let app = relay::routes::router(relay::routes::AppState::new(tx));
            for body in &bodies {
                prop_assert!(body.len() <= 64 * 1024, "over the H4 domain bound");
                let (relay_status, relay_text) = relay_verdict(&app, body).await?;
                let (hello_status, hello_text, emitted) = hello_verdict(body).await?;
                prop_assert_eq!((hello_status, &hello_text), (relay_status, &relay_text),
                    "doors drifted on body {:?}", body);
                prop_assert_eq!(emitted, usize::from(hello_status == 202),
                    "emit count wrong on {:?}", body);
            }
            Ok(())
        })?;
    }
}
```

`relay_verdict` is your sitting-M oneshot pattern (clone the router per
request, `to_bytes` the body); `hello_verdict` calls `handle_with` with a
fresh Vec sink and returns the emitted-line count as the third element.
Both convert failures to `TestCaseError::fail` instead of unwrapping —
the file-level `#![allow(clippy::unwrap_used)]` covers tests, but a
failure *message* beats a bare panic when proptest is shrinking.

</details>

<details><summary>Hint 2 — the door as one match</summary>

```rust
fn door(body: &str) -> Result<String, String> {
    match SerdeParser.classify(body, &REQUIRED_KEYS) {
        ClassifiedLine::Blank | ClassifiedLine::Unparseable =>
            Err("not a json object".to_owned()),
        ClassifiedLine::Malformed { missing } =>
            Err(format!("missing key: {missing}")),
        ClassifiedLine::Event { day, .. } if day == "bad-ts" =>
            Err("no comparable day in ts".to_owned()),
        ClassifiedLine::Event { .. } => {
            let value: serde_json::Value = serde_json::from_str(body)
                .map_err(|_| "not a json object".to_owned())?;
            Ok(value.to_string())
        }
    }
}
```

The fixed order isn't re-implemented — it falls out of `ClassifiedLine`'s
shape (glake can only report a missing key on something that parsed, and
only reports a day on something with all its keys), exactly as it did in
relay's `validate.rs`. If your error strings differ from relay's by one
character, H4 will tell you which character.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/006-hello-lambda/_reference/hello-lambda/src/handler.rs` — the
  `Emit` seam, the match-as-router, and the door. Path warning as ever:
  the reference lives outside the workspace, so its glake dep points at
  `../../../004-glake-traits/_reference/glake` — from `crates/hello-lambda`,
  yours is `../glake` (and relay is `../relay`).
- `specs/006-hello-lambda/_reference/hello-lambda/src/state.rs` — the
  `OnceLock`, both id sources, and the hand-rolled clock with its pinned
  unit tests.
- `specs/006-hello-lambda/_reference/hello-lambda/tests/prop_door.rs` —
  the full harness: generators, the live-receiver comment block, both
  verdict functions, the domain-bound asserts.
- `specs/006-hello-lambda/_reference/hello-lambda/tests/handler.rs` — the
  `SERIAL` mutex, the delta assertions, and the 404 table.
- `specs/006-hello-lambda/_reference/NOTES.md` §4 — twelve semantic
  decisions (the emit-seam trait, the re-derived door, the 404/405
  divergence, the identity rule) worth reading *after* your build as a
  compare-notes exercise.
