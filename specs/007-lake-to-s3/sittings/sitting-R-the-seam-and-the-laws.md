# Sitting R — the seam and the laws

**Builds:** `crates/lake-store` — the second trait of your life: `ObjectStore`
+ `ObjectMeta` + `StoreError`, spelled the desugared way after you walk into
the Send wall on purpose — plus `FakeStore`, an in-memory S3 with a ledger;
and `crates/lake-sync`'s pure core (`plan` + `run`), proven by the S3
conservation and S4 idempotence properties at 512 generated lakes each, red
first. No AWS today: the laws run entirely against the fake — that's the
point (T2).
**Requirements:** S1–S4 (+S7 groundwork) — tasks 1.1–1.3 (T1: a trait as a
seam; T2: property-testing side effects; T5: idempotence as design).
**Ramp you'll use:** none new — the ramp closed at step 13. Step 9's traits,
step 10's generics-vs-`dyn` rule, sitting J's trait boundary and F13 tests,
sitting E's R9 conservation generator, and 003 R3b's "a file is walked as
itself" are the substrate; the opener is a re-read of your own code instead
(tasks.md section 0).

## Where you are

Q ended with the stack destroyed and one lesson still warm: your accepted
events were living in a CloudWatch log group — "the lake in exile," one
`println!` away from durability. 007 gives the lake its real home, and the
architecture is one trait. In 004 you designed `EventParser` because two
*parsers* had to be swappable; today you design `ObjectStore` because the
thing behind it — S3 — cannot be in the room while you test. That's the
promotion this sitting hands your 004 lesson: **put the seam where the
un-testable thing starts**, then prove the laws against a fake on your side
of it. Everything interesting in lake-sync — key derivation, the
size/etag compare, conservation, idempotence — will run in code that has
never heard of AWS. The real S3 module arrives next sitting and will be thin
enough to read whole.

One thing to know before you type: the trait's exact spelling is a planned
fight. You'll write it the way your gut says (`async fn`), predict where it
breaks, meet the compiler's wall, and desugar your way through. The error
you're about to earn is the async-fn-in-trait lesson of this whole phase —
we take it on purpose, not by accident.

## The build, move by move

All commands from the repo root. Five commits this sitting: the seam, the
fake, red, plan, laws green — tasks 1.1–1.3.

1. **The opener (tasks.md §0): re-read your own 004 trait, closed book
   first.** Before opening the file, write down from memory: what does
   `EventParser::classify` return, and why did the boundary have to *own*
   (what forced it — sitting J move 1)? Where is glake's one `dyn` point,
   and what rule put it there? Now open
   `crates/glake/src/parser.rs` and grade yourself. Then the question that
   sets up today (answer aloud before reading on): glake needed
   `Box<dyn EventParser>` because `--parser` is a **runtime** choice. Who
   chooses the `ObjectStore` implementation, and when? (Tests name
   `FakeStore`, binaries name `S3Store` — the choice is made at **compile
   time**, so nobody needs a vtable: consumers stay generic over
   `S: ObjectStore` and monomorphize. Step 10's closing rule, second
   application — and this time there's no outermost `dyn` at all. Move 3
   adds a second, harder reason the trait *couldn't* be `dyn` even if you
   wanted it.) Ledger the recall result (`learning.*`: cold / needed the
   file).

2. **Scaffold two crates (task 1.1 begins).** The design's three-crate
   layout starts with two (the evolved hello-lambda is sitting T):

   ```console
   cargo new crates/lake-store --lib
   cargo new crates/lake-sync
   ```

   The workspace glob adopts both on sight, as ever. Give lake-sync the
   lib+bin shape (`src/lib.rs`, and leave `src/main.rs` an empty
   `fn main() {}` with a `// sitting S wires clap here` comment — the CLI is
   next sitting's move). `#![forbid(unsafe_code)]` on both lib roots, and
   the lint table both crates inherit from relay/hello, verbatim:

   ```toml
   [lints.clippy]
   unwrap_used = "deny"
   expect_used = "deny"
   ```

   `crates/lake-store/Cargo.toml`, the part that *is* a design decision —
   read it aloud before committing:

   ```toml
   [features]
   # The fake NEVER rides by default — consumers turn it on in
   # [dev-dependencies] only, so it can't leak into a deployable artifact.
   # (Sitting S adds: default = ["s3"], s3 = ["dep:aws-sdk-s3"] — the real
   # impl. Today the SDK stays out of the building entirely.)
   fake = ["dep:md5", "dep:tokio"]

   [dependencies]
   thiserror = "2"
   md5 = { version = "0.8", optional = true }
   tokio = { version = "1", features = ["rt"], optional = true }

   [dev-dependencies]
   tokio = { version = "1", features = ["macros", "rt"] }
   ```

   Things to say aloud: **why is the fake a feature and not a
   `#[cfg(test)]` module?** (Other crates' tests need it — lake-sync's
   properties, hello-lambda's S6 tests. `#[cfg(test)]` is invisible outside
   the crate; a feature that only dev-dependencies enable is the visible-
   but-never-shipped shape, and it's what keeps sitting S's size bill
   honest.) **Why md5, and why is it optional?** (The fake must speak the
   same etag spelling reality will — bare-hex md5 — and only the fake
   computes it today. It's a *compatibility checksum, not security*; pure
   Rust, so the ARM64 story stays trivial — S7's Graviton checklist will be
   n/a by construction.) **Why does the fake need tokio at all?** (Move 5's
   dwell — hold the question.)

   And `crates/lake-sync/Cargo.toml` for today (clap and aws-config are
   sitting S):

   ```toml
   [dependencies]
   lake-store = { path = "../lake-store" }
   glake = { path = "../glake" }
   md5 = "0.8"
   thiserror = "2"
   tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }

   [dev-dependencies]
   lake-store = { path = "../lake-store", features = ["fake"] }
   proptest = "1"
   tempfile = "3"
   ```

   glake, **fifth caller** (its own bin, relay, hello, the H4 harness, now
   lake-sync): the walker that already knows what a lake looks like
   (`jsonl_files` — recursive, sorted, path-context errors) is your own
   library; lake-sync adds no walking logic of its own. That's 004 still
   compounding.

3. **The trait — write the wrong thing first (task 1.1, the Send moment).**
   In `lake-store/src/lib.rs`, spell the seam the way every instinct says:

   ```rust
   pub trait ObjectStore: Send + Sync + 'static {
       async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>, StoreError>;
       async fn put(&self, key: String, body: Vec<u8>) -> Result<(), StoreError>;
   }
   ```

   (Interrogate the shape while it's in front of you: why does `put` take
   an **owned** `String` and `Vec<u8>`? Because the caller is going to move
   them into a spawned task — a borrow can't make the trip, the same
   ownership lesson as relay's channel sends in M. Why the
   `Send + Sync + 'static` supertraits? The store itself will cross threads
   inside an `Arc` and outlive every task it's handed to.)

   This compiles. (Give `ObjectMeta` and `StoreError` one-line placeholder
   definitions for now so it can — move 4 makes them real.) Now **predict
   before you provoke**: lake-sync's `run` will spawn each upload onto a
   `tokio::task::JoinSet`. Write the prediction down — *what will the
   compiler demand of the future that `store.put(...)` returns, and can
   this trait promise it?* Then write the probe (this is `run`'s real
   skeleton — it stays):

   ```rust
   // lake-sync/src/run.rs — the bones
   pub async fn run<S: ObjectStore>(store: Arc<S>, uploads: Vec<(String, Vec<u8>)>) {
       let mut tasks: JoinSet<Result<(), StoreError>> = JoinSet::new();
       for (key, body) in uploads {
           let store = Arc::clone(&store);
           tasks.spawn(async move { store.put(key, body).await });
       }
       while let Some(_joined) = tasks.join_next().await {}
   }
   ```

   `cargo build -p lake-sync` — and meet the wall (verified, verbatim):

   ```
   error: future cannot be sent between threads safely
      --> src/run.rs:…
       |
       |         tasks.spawn(async move { store.put(key, body).await });
       |               ^^^^^ future created by async block is not `Send`
       |
       = help: within `{async block…}`, the trait `Send` is not implemented
               for `impl Future<Output = Result<(), StoreError>>`
   note: future is not `Send` as it awaits another future which is not `Send`
       |
       |         tasks.spawn(async move { store.put(key, body).await });
       |                                  ^^^^^^^^^^^^^^^^^^^^ await occurs here on type
       |                                  `impl Future<Output = Result<(), StoreError>>`, which is not `Send`
   note: required by a bound in `JoinSet::<T>::spawn`
       |
   142 |     pub fn spawn<F>(&mut self, task: F) -> AbortHandle
   ...
   145 |         F: Send + 'static,
       |            ^^^^ required by this bound in `JoinSet::<T>::spawn`
   help: `Send` can be made part of the associated future's guarantees for all implementations of `ObjectStore::put`
       |
    20 -     async fn put(&self, key: String, body: Vec<u8>) -> Result<(), StoreError>;
    20 +     fn put(&self, key: String, body: Vec<u8>) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
       |
   ```

   Read it bottom-up, slowly — this is the best async diagnostic of the
   phase, and its `help:` block literally *writes the design decision for
   you*:

   - `JoinSet::spawn` demands `F: Send` — the runtime may migrate a task
     between worker threads (you knew this from O's `Emit: Sync` chain).
   - Inside a **generic** function, `store.put(...)`'s future is whatever
     the trait promises — an opaque `impl Future` — and the sugar `async
     fn` promises *nothing about Send*. For a concrete type the auto-trait
     would leak through; across a trait boundary, the trait's written
     word is all the compiler has. The sugar has no place to write `+ Send`
     — on stable Rust, this trait cannot say it.
   - So the fix is to stop using the sugar **in the trait** and write what
     it desugars to, plus the one bound the sugar couldn't carry:

   ```rust
   use std::future::Future;

   pub trait ObjectStore: Send + Sync + 'static {
       fn list(&self, prefix: &str)
           -> impl Future<Output = Result<Vec<ObjectMeta>, StoreError>> + Send;
       fn put(&self, key: String, body: Vec<u8>)
           -> impl Future<Output = Result<(), StoreError>> + Send;
   }
   ```

   That return type — `impl Future` in a trait method — is RPITIT
   (return-position impl-Trait-in-trait), and writing it by hand IS the
   async-fn-in-trait lesson; a proc-macro (`trait_variant::make(Send)`)
   would do the same thing while hiding it. The probe now compiles. One
   more consequence to say aloud, because it closes move 1's question for
   good: a trait with RPITIT methods **isn't `dyn`-compatible** — provoke
   it once if you like (`&dyn ObjectStore` anywhere) and the compiler
   answers with E0038, "`ObjectStore` is not dyn compatible … because
   method `list` references an `impl Trait` type in its return type" (the
   fights section has the full text). Generic-over-`S` isn't just the
   honest F13 answer here; it's the only one on offer.

4. **`ObjectMeta` + `StoreError` — the S7 groundwork.** Two data types
   finish the seam. `ObjectMeta { key: String, size: u64, etag: String }`
   is what `list` answers — the compare side of next move's idempotence
   check. Run G's rubric over the derives yourself before reading the
   reference's answer: `Debug`/`Clone`/`PartialEq`/`Eq`/`Hash` — why each?
   Why *not* `Copy` (it owns `String`s), why not `Ord` (no natural total
   order worth promising)? And one doc-comment contract that the whole
   spec leans on: **`etag` is bare lowercase hex, no quotes.** Both
   implementations will promise that exact spelling; sitting S shows you
   the line in the real store where that promise has to be *manufactured*
   (and what silently dies if it isn't).

   `StoreError`: thiserror, two variants — `List { prefix, source }` and
   `Put { key, source }` — because C-GOOD-ERR says an error carries the
   *name the caller needs* (S5 will print the failed object's key from
   this very field). The sources are
   `Box<dyn std::error::Error + Send + Sync>`: the fake fails with a plain
   string, the real store with a many-generic SDK error, and a boxed
   source is the honest common shape (nobody matches on an S3 error's
   internals at this seam). Write the three C-GOOD-ERR unit tests —
   Send+Sync+'static (a compile-time assertion in a test's clothing),
   display style (lowercase, no trailing period, context present), source
   chain reaching the cause. **Commit point:**

   ```console
   cargo fmt
   cargo clippy -p lake-store --all-targets -- -D warnings
   git add crates/lake-store crates/lake-sync Cargo.lock
   git commit -m "007: sitting R — the ObjectStore seam, desugared through the Send wall"
   ```

5. **`FakeStore` — an S3 you can interrogate (task 1.1 closes).** Create
   `lake-store/src/fake.rs` behind the feature
   (`#[cfg(feature = "fake")] pub mod fake;`). Three jobs, and each one is
   a requirement's witness:

   - **Model S3 where the laws lean on it.** The bucket is a
     `Mutex<BTreeMap<String, Vec<u8>>>` — and `insert` *replacing* the old
     value at a key is not an implementation accident, it's the S3
     semantic S3/S4 stand on: **one object per key; a re-put overwrites**.
     (`BTreeMap`, not `HashMap`: `list` comes back sorted like S3's
     lexicographic listings — determinism for free.) `list` computes etags
     *at list time* from the stored bytes: `format!("{:x}",
     md5::compute(body))` — the same bare-hex spelling move 4 pinned, so
     fake and reality can never drift on format.
   - **Keep a ledger.** `puts()` (count), `put_log()` (which keys, in
     order — S4 will diff this across passes to prove "exactly the changed
     set"), and an in-flight gauge with a high-water mark whose
     increment/decrement live **inside `put` itself** — S9's bound (next
     sitting) must be measured where the work happens, not where it was
     scheduled. Interior mutability throughout, because the trait's
     methods take `&self` and many tasks will share one store through an
     `Arc` — relay's counters pattern, re-employed as a test instrument.
   - **Inject failure.** `fail_on(key)`: the chosen key's put returns
     `Err(StoreError::put(key, …))`, stores nothing, counts nothing —
     exactly like a PUT that died on the wire. S5's mid-run-failure test
     (sitting S) needs a store that can fail on cue; the gauge must
     decrement on *both* paths (a failed put was still in flight).

   Two subtleties you'll hit while writing it — the first is the answer to
   move 2's held question:

   - **The dwell.** `put` awaits `tokio::task::yield_now()` twice — once
     before the insert, once after. Without an await inside `put`, a put
     starts and finishes within one poll, tasks can never overlap, and
     next sitting's S9 gauge would read 1 forever — a vacuous bound. A
     real network PUT parks its task at an `.await` exactly like this;
     `yield_now` is the deterministic lab version of that parking.
   - **The lock and the unwrap ban.** `Mutex::lock()` returns a `Result`
     (poison), and this crate denies `unwrap()`. The honest recovery for
     test bookkeeping is `unwrap_or_else(PoisonError::into_inner)` — a
     poisoned mutex means another test thread panicked while holding it;
     the data is still fine to read, and propagating the poison would only
     turn one failure into two. Write it once as a `fn lock<T>` helper.

   Now the impls — and notice what you're *allowed* to write here:

   ```rust
   impl ObjectStore for FakeStore {
       async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>, StoreError> { … }
       async fn put(&self, key: String, body: Vec<u8>) -> Result<(), StoreError> { … }
   }
   ```

   **Sugar in the impl, desugared in the trait.** The compiler proves each
   concrete future here *is* `Send` (auto-traits leak through `async fn`
   for concrete types) and checks it against the trait's written promise —
   if `put` ever held a `MutexGuard` across one of its `.await`s, this
   impl would stop compiling. And if you try to "match the trait" by
   desugaring the impl too, clippy objects under our `-D warnings`
   (verified):

   ```
   error: this function can be simplified using the `async fn` syntax
     --> src/fake.rs:…
      = note: `-D clippy::manual-async-fn` implied by `-D warnings`
   ```

   The rule, worth a comment at the impl because it's subtle: **desugar
   where the desugaring says something** (the trait — it's where `+ Send`
   lives), sugar where it doesn't (the impls).

   Unit-test the fake itself (`#[tokio::test]`, module-level
   `#![allow(clippy::unwrap_used)]`): put-replaces-key (one object per
   key, both puts counted), bare-hex etags pinned to the famous
   empty-input constant `d41d8cd98f00b204e9800998ecf8427e` (a format
   change can't hide from a pinned constant), and fail_on (named error,
   nothing stored, nothing counted, gauge balanced). Green transcript
   (verified — 6 tests with the feature on):

   ```
   running 6 tests
   test fake::tests::fail_on_fails_exactly_the_chosen_key ... ok
   test tests::s7_display_is_lowercase_no_period_with_context ... ok
   test fake::tests::put_replaces_the_object_at_a_key ... ok
   test fake::tests::list_computes_bare_hex_md5_etags ... ok
   test tests::s7_error_is_send_sync_static ... ok
   test tests::s7_source_chain_reaches_the_cause ... ok
   ```

   **Commit point:**

   ```console
   cargo fmt && cargo clippy -p lake-store --all-targets --all-features -- -D warnings
   git add crates/lake-store
   git commit -m "007: sitting R — FakeStore: put-replaces-key, ledger, gauge inside put"
   ```

6. **The laws, red first (task 1.2 — co-written).** Property harnesses are
   co-written per the constitution; this one generates *filesystems*.
   Before the strategy work, stub the core so red has something to compile
   against: in `lake-sync`, `plan.rs` gets `Plan { upload: Vec<Upload>,
   skip: Vec<String> }`, `Upload { key, path, size }`, and
   `pub fn plan(root, prefix, remote) -> Result<Plan, SyncError>` whose
   body is `todo!("walk + compare: move 7")`; `run.rs` keeps move 3's
   skeleton but grows its real signature —
   `pub async fn run<S: ObjectStore>(store: Arc<S>, plan: Plan) ->
   Result<Outcome, SyncError>`, `Outcome { uploaded, failed, skipped }`,
   body `todo!()`. (`SyncError` in `lib.rs`: `#[error(transparent)]
   Walk(#[from] glake::error::GlakeError)`, an `Io { path, source }`
   variant, `#[error(transparent)] Store(#[from] StoreError)` — resist
   inventing more taxonomy before there are callers, 004's lesson. One
   design choice to notice: `plan` takes the remote listing as a
   *parameter* instead of calling `store.list()` itself. Handing the
   listing IN is what keeps the module pure — the properties feed it a
   fake's listing, `--dry-run` will feed it an empty one, the real path
   feeds it S3's. Decisions and effects never share a function.)

   Now `tests/prop_sync.rs`. The generator is your E-sitting R9 generator
   graduating from lines to lakes, and its **domain is S1's real domain,
   not a sanitized one** — this is where most property suites quietly go
   wrong, so build it deliberately:

   - **Dirs:** plain `dt=` days, **a `dt=bad-ts` arm** (relay's quarantine
     partition is legal lake content and must sync like any other — S1
     says verbatim), and **`traces/dt=…` arms** (the volume workload's
     real shape). Keep one day out of the pool (the reference reserves
     `dt=2026-07-07`) — the S4 new-file mutation will claim it,
     collision-free.
   - **Lines:** mostly valid envelopes, plus blanks, whitespace, junk,
     truncated JSON — because **sync moves bytes, not judgments**. A
     malformed line is exactly as worth conserving as a valid one; glake
     judges, lake-sync carries.
   - **Files:** 1–8 of them, 0–40 lines each, trailing newline sometimes
     absent (that last one will matter in move 7 — leave it in even if it
     feels fussy). Generate `(dir, name, lines, trailing)` tuples and fold
     them into a `BTreeMap<String, Vec<u8>>` — collisions on (dir, name)
     overwrite, mirroring a filesystem.

   The two properties, at `ProptestConfig::with_cases(512)` (design floor
   256):

   - **S3 — conservation.** Write the lake to a tempdir, sync against a
     fresh fake (list → plan → run, exactly the shape main.rs will wear),
     then assert *two* equalities: the store's key set equals
     `raw/<relative path>` over the local files (right content at wrong
     keys is also a failure), and the **multiset of lines** across store
     bodies equals the multiset across local files — nothing lost,
     duplicated, or invented. (Order deliberately ignored: the law is
     about content survival, not layout.)
   - **S4 — idempotence.** Same lake: sync once (everything uploads),
     sync again — **the ledger must show zero new puts**; then mutate ONE
     file and sync a third time — the put-log's tail must be **exactly
     the changed key**, and conservation must hold *again* against disk
     as it is now (that re-check is what put-replaces-key exists for).
     The mutation strategy has three arms: append a line, add a new file
     (in the reserved day), and the one that earns its place —
     **SameSize**: swap one alphanumeric byte for a different one. Same
     length, same newline structure; *only a checksum can see this edit*.
     Without this arm, move 7's etag branch could rot into dead code
     behind the size check and no test would ever notice.

   Runtime pattern is M's, unchanged: proptest drives a sync body that
   builds a current-thread runtime and `block_on`s. Run it, savor the red
   (verified transcript, core stubbed):

   ```
   running 2 tests
   test prop_s3_sync_conservation ... FAILED
   test prop_s4_idempotence ... FAILED

   ---- prop_s3_sync_conservation stdout ----
   proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
   thread 'prop_s3_sync_conservation' panicked at src/plan.rs:…:
   not yet implemented: walk + compare: move 7
   …
   Test failed: not yet implemented: walk + compare: move 7.
   minimal failing input: lake = GenLake {
       files: {
           "dt=2026-07-05/events.jsonl": [],
       },
   }
   test result: FAILED. 0 passed; 2 failed
   ```

   Proptest shrank the failing lake to **one empty file** — the machinery
   works before the code exists, same as O's red. (The
   `FileFailurePersistence` warning is O's old friend: proptest failing to
   park a regression seed for an integration test — and expected-red is
   not a genuine counterexample anyway; nothing to commit under the seeds
   rule.) **Commit point:**

   ```console
   git add crates/lake-sync
   git commit -m "007: sitting R — S3/S4 laws red against the stubbed core"
   ```

7. **`plan` — the pure core (task 1.3).** Three pieces, and a war story.

   - **The key rule (S1, normative):** `key = prefix + path relative to
     the lake root`, forward slashes, **nothing special-cased** — your
     walker's output maps mechanically, so `dt=2026-07-09/events.jsonl` →
     `raw/dt=2026-07-09/events.jsonl`, `traces/dt=…/run.jsonl` →
     `raw/traces/dt=…/run.jsonl`, and `dt=bad-ts/…` flows through
     verbatim. Normalize the prefix once (`raw` and `raw/` both mean
     `raw/…`) so the concatenation can be dumb. Build the relative path
     from `Path::components()` joined with `'/'` — components() already
     ate the separators, and an S3 key must never see a Windows `\`.
     Refuse non-UTF-8 components with a named error rather than upload a
     lossily-renamed object.
   - **The compare (T5):** size first — it's free, already in the
     metadata, and a mismatch settles it. Etag second, **only when sizes
     tie**: md5 the local bytes and compare against the remote etag. This
     two-step is the whole idempotence design: the *store* is the source
     of truth (no manifest file to corrupt), and "safe to run twice" falls
     out of key derivation + compare instead of being a flag someone can
     forget. For the local md5, `std::fs::read` + `md5::compute` is fine
     *today* — sitting S replaces it with the streaming discipline and
     explains exactly what that buys.
   - **The war story, before you write `relative_key`.** While these
     materials were being validated, the red-first pass caught a real bug
     in the reference's own first spelling of this function —
     `_reference/NOTES.md` decision 14, kept as a lesson.
     `path.strip_prefix(root)` looks obviously right, and it *is* — for
     directory roots. But your walker honors 003 R3b: **a single file
     passed as the root is walked as itself** — and stripping a path from
     *itself* succeeds with an **empty remainder**. The key silently became
     a bare `raw/`: an object named like a directory, invisible to every
     query. The S1 example table (below) is what caught it before it
     shipped — this is J's F8 story again, one spec later: the tests you
     write before the code are the ones that get to catch the bug *during
     authoring* instead of during deploy day. Write the single-file-root
     row into your test table **first**, then implement: empty remainder →
     fall back to the file's name, with a comment saying why.

   The S1/S2 examples (`plan.rs` unit tests + one lib-level test): the key
   table over a fixture lake with a plain day, `dt=bad-ts`, and a
   `traces/` file (assert all three keys verbatim, sorted walker order,
   everything under `raw/`); **the single-file-root + prefix-normalization
   row**; the compare table (absent → upload; size differs → upload, etag
   never read; size ties + etag differs → upload; both tie → skip);
   missing root → `Err` with path context (S5's exit-2 discipline arrives
   with the CLI next sitting, but the error's shape is pinned now). And
   S2's ledger half as a lib test: pre-populate a fake with one matching
   object, `list` + `plan` (never `run`) — the plan shows the skip and
   **the ledger shows zero puts**: planning is free of side effects by
   construction. **Commit point:**

   ```console
   cargo fmt && cargo clippy -p lake-sync --all-targets -- -D warnings
   git add crates/lake-sync
   git commit -m "007: sitting R — plan: S1 keys + size/etag compare, examples green"
   ```

8. **`run`, and the laws go green (task 1.3 closes).** Flesh out move 3's
   skeleton: for each `Upload`, read the body, spawn
   `store.put(key, body)` onto the `JoinSet` (owned key, owned body, an
   `Arc::clone` of the store per task — say why each of the three must be
   owned before you write it: the task may outlive every borrow the
   compiler can see), then drain `join_next`, sorting successes into
   `outcome.uploaded` and failures into `outcome.failed` *as data* — a
   failed put is a reportable outcome, not an early return (S5's
   partial-progress rule, which sitting S wires to the CLI). Sort both
   lists before returning: completion order is scheduler noise, and
   diffable output is the house style. No semaphore yet — the bound is
   sitting S's move, and S9's gauge test will prove it; today's version is
   dead-simple-first, correct before bounded.

   ```console
   cargo test -p lake-sync
   ```

   Green, the full ritual — reference transcript for the property target
   (yours should match in shape; the reference's full suite finishes this
   file's 2 properties in under 4 s):

   ```
   running 2 tests
   test prop_s3_sync_conservation ... ok
   test prop_s4_idempotence ... ok

   test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.70s
   ```

   Sit with what that means for one minute, because it's the sitting's
   whole argument: **1,024 generated lakes — blanks, junk, quarantine
   partitions, trailing-newline chaos, same-size edits — synced,
   re-synced, mutated, and re-proven, in less time than one real S3
   round-trip.** That's what putting the seam in the right place bought.
   No counterexample? Then `proptest-regressions/` stays absent — the
   committed-seeds rule is for genuine failures. If S4 *did* hand you a
   shrunk lake (a blank-only file whose same-size arm had nothing to
   swap is the classic), triage before touching code: spec, code, or
   test bug? (That particular one is a strategy bug — the arm must
   degrade to append when no swappable byte exists. Log the triage.)
   **Commit point:**

   ```console
   cargo fmt && cargo clippy -p lake-sync --all-targets -- -D warnings
   git add crates/lake-sync
   git commit -m "007: sitting R — run over Arc<S>, S3/S4 laws green at 512"
   ```

## Compiler fights to expect

Ledger them (`learning.*`) — today's are the async-trait canon, and you
met the big one on purpose.

- **The Send wall itself** — move 3, in full. The thing to internalize
  from the error's anatomy: the failure is at the *use site*
  (`tasks.spawn`), the cause is in the *trait*, and rustc's `help:`
  bridges them by proposing the exact desugared signature. When a
  trait-shaped async error confuses you, scroll to the `help:` first.
- **`error[E0038]: the trait `ObjectStore` is not dyn compatible`** — if
  you reach for `&dyn ObjectStore` or `Box<dyn ObjectStore>` out of 004
  habit (verified):

  ```
  note: for a trait to be dyn compatible it needs to allow building a vtable
     |
     |     fn list(&self, prefix: &str)
     |     -> impl Future<Output = Result<Vec<ObjectMeta>, StoreError>> + Send;
     |        ^^^^^^^^^^^ …because method `list` references an `impl Trait`
     |                    type in its return type
  ```

  Every impl returns a *different* concrete future type, so no single
  vtable entry can exist. Generic `S: ObjectStore` isn't a style choice
  here; it's the only shape the trait supports — which is fine, because
  the choice was compile-time all along (move 1).
- **clippy: `this function can be simplified using the `async fn`
  syntax`** (`manual_async_fn`, denied) — you desugared an *impl* to
  match the trait's spelling. Move 5's rule: the desugaring says
  something only in the trait.
- **`error[E0507]`/"borrowed data escapes" flavors in `run`** — spawning
  with a borrowed key/body/store instead of moved-and-cloned ones. The
  spawn boundary is an ownership boundary; the fix is always "give the
  task its own" (`Arc::clone`, owned `String`, owned `Vec<u8>`), never a
  lifetime annotation.
- **clippy: `used `unwrap()`…, denied** — in `fake.rs` at the mutex.
  The `PoisonError::into_inner` recovery (move 5) is the sanctioned
  spelling; in test modules, the usual module-level allow.

## Checkpoint

From the repo root — the sitting counts as done only when all of these
hold:

```
cargo fmt --check                                            # no diff
cargo clippy -p lake-store --all-targets --all-features -- -D warnings   # clean
cargo clippy -p lake-sync --all-targets -- -D warnings       # clean
cargo test -p lake-store --features fake                     # 6 green (3 StoreError + 3 fake)
cargo test -p lake-sync                                      # green — at this sitting's close:
                                                             # 4 unit + 1 ledger example + 2 properties @ 512
                                                             # (the full reference suite is 15; sitting S adds the rest)
cargo tree -p lake-sync -e normal | grep -c aws              # 0 — the laws passed and AWS was
                                                             # never in the building (F's paper
                                                             # cut: prints 0 AND exits 1)
```

- `git log --oneline -5` shows seam → fake → red → plan → laws green.
- `proptest-regressions/` is absent from lake-sync (or holds a genuinely
  triaged seed with its `_assurance/triage-log.md` entry).
- You can answer aloud: why can't `async fn` in a trait promise `Send`,
  and who demanded it? (Walk the chain: JoinSet::spawn → F: Send → the
  opaque future → the trait's written word.) Why is the trait desugared
  but the impls sugared? Why does the fake's gauge live *inside* `put`,
  and what would a scheduler-side gauge measure instead? Why must the
  fake's `insert` replace — which law dies if it appends? What does the
  SameSize mutation arm keep alive? And the war story: what does your
  walker do with a single-file root, and what key did that almost mint?

## Hints (one at a time)

<details><summary>Hint 1 — the FakeStore ledger's shape</summary>

Every mutable thing sits in a `Mutex` or an atomic, because `put` takes
`&self`:

```rust
#[derive(Debug, Default)]
pub struct FakeStore {
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
    fail_on: Mutex<Option<String>>,
    put_log: Mutex<Vec<String>>,
    puts: AtomicU64,
    in_flight: AtomicU64,
    high_water: AtomicU64,
}
```

`put`'s choreography, in order: `in_flight.fetch_add` then
`high_water.fetch_max` (the gauge brackets the WHOLE call);
`yield_now().await`; the fail_on check (fail = named error, store
nothing, count nothing) or the insert + log + count; `yield_now().await`;
`in_flight.fetch_sub` **on both paths**. Accessors clone out of the locks
(`put_log()`, `objects()`) so tests never hold your mutex across their
own awaits.

</details>

<details><summary>Hint 2 — the property harness plumbing</summary>

Three helpers carry both properties:

```rust
/// One full sync pass: LIST → plan → run — main.rs's real path with the
/// fake in the store seat.
async fn sync(root: &Path, store: &Arc<FakeStore>) -> Result<(Plan, Outcome), TestCaseError> {
    let remote = store.list("raw/").await.map_err(|e| TestCaseError::fail(e.to_string()))?;
    let the_plan = plan(root, "raw/", &remote).map_err(|e| TestCaseError::fail(e.to_string()))?;
    let outcome = run(Arc::clone(store), the_plan.clone()).await
        .map_err(|e| TestCaseError::fail(e.to_string()))?;
    Ok((the_plan, outcome))
}
```

plus `line_multiset(bodies) -> BTreeMap<String, usize>` (count every
line across a set of bodies — `str::lines`, order ignored) and, for S4's
re-check, a `disk_files(root)` that reads the lake back **off disk with
the real walker**, not from the generator's map — after a mutation, disk
is the truth the law must hold against. The S4 skeleton: sync, assert
puts == file count; sync, assert puts unchanged; `apply_mutation` (returns
the changed relative path); sync, assert `put_log()[before..]` ==
exactly `{ raw/<changed> }`; then `assert_conservation(&disk_files(…), …)`
one last time.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/007-lake-to-s3/_reference/lake-store/src/lib.rs` — the desugared
  trait with its why-not-sugar doc block, the `ObjectMeta` etag contract,
  and the C-GOOD-ERR tests. Path warning as ever: the reference lives
  outside the workspace, so its deps point at other `_reference` crates —
  from `crates/lake-sync`, your glake is `../glake` and your lake-store is
  `../lake-store`.
- `specs/007-lake-to-s3/_reference/lake-store/src/fake.rs` — the ledger,
  the dwell comment, the poison recovery, and the sugar-in-impls teaching
  comment.
- `specs/007-lake-to-s3/_reference/lake-sync/src/plan.rs` — the key rule,
  the compare, and the `relative_key` fallback with the bug-story comment
  at the site.
- `specs/007-lake-to-s3/_reference/lake-sync/tests/prop_sync.rs` — the
  full generator (DIRS/NAMES/line/GenLake), the three mutation arms, and
  both properties.
- `specs/007-lake-to-s3/_reference/NOTES.md` §2 and §4 — the validated
  test census and the fourteen build decisions (read decision 14 *after*
  your own single-file-root test passes — compare notes).
