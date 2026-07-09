# Sitting K — hello, axum

**Builds:** `crates/relay` exists and answers: a new workspace crate with an
async `main`, one `GET /healthz` route returning static JSON, a clap CLI
(`--port`, `--lake`) proven by the A7 example test, and your own glake library
wired in as a path dependency — compile-checked today, called for real in
Sitting L.
**Requirements:** A7, A3 groundwork — task 1.1 (T1: async/await + tokio).
**Ramp you'll use:** step 12 (async/await + tokio — recapped as the opening
move), with 004's sitting I (clap) and sitting B's `CARGO_BIN_EXE` test
pattern in supporting roles.

## Where you are

004 closed with glake v1: your library has a public trait (`EventParser`),
two backends, a designed error type, and a stranger-ready API. The stranger
arrives today. **relay** is a small HTTP service that does the shell hooks'
job — append envelope events to the lake — but concurrent, validated at the
door, and shaped like every Rust network service you'll ever deploy (spec 006
lifts this exact handler into Lambda). This sitting builds none of that yet:
today is scaffolding — a service that *starts*, *listens*, *answers one URL*,
and *takes arguments* — done properly, because everything in sittings L–N
hangs off this skeleton. The rhythm you know: smallest step that compiles and
teaches, one commit per move.

## The build, move by move

All commands from the repo root. Two commits this sitting: hello axum, then
args + dep + test.

1. **Recap step 12 before touching anything.** Re-read
   `playground/ramp/step-12-async-tokio/my-solution.rs` and answer aloud:
   what does *calling* an async fn do (and what does it not do)? What does
   `.await` yield, and to whom? What does `#[tokio::main]` expand to? You'll
   want all three answers within the hour, because `axum::serve(...)` is just
   a very long-lived future your `main` awaits — every request handler is a
   task the runtime drives while `main` sits parked on that one `.await`,
   which is step 12's whole lesson wearing a server's clothes. And park this
   for Sitting N: step 12's `select!` — "next request OR ctrl-c, whichever
   first" — is exactly how graceful shutdown will work.

2. **Make the crate.** From the repo root:

   ```console
   cargo new crates/relay
   cargo run -p relay
   ```

   Hello, world — but notice what you *didn't* do: no `[workspace]` opt-out
   table, no manifest edits. Steps 12–13 needed that empty `[workspace]`
   table to stay standalone; `crates/relay` is the opposite case — the root
   manifest's `members = ["crates/*"]` glob adopted it the moment the folder
   appeared. Say the difference aloud: the ramp steps were *hiding from* the
   workspace; relay is *joining* it — one shared `Cargo.lock`, one `target/`,
   and the workspace release profile (thin LTO, `panic = "abort"`) applies to
   relay's future release builds for free.

3. **Invite tokio and axum — features by name.** In `crates/relay/Cargo.toml`:

   ```toml
   [dependencies]
   tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
   axum = "0.8"
   ```

   Step 12's Cargo.toml taught the habit: name the slices you use, never
   `features = ["full"]` (when `aws-sdk-*` arrives in 006, this habit is
   binary-size money). Today that's `rt-multi-thread` (the scheduler) and
   `macros` (`#[tokio::main]`). More slices join as sittings need them —
   `fs` in L, `sync` in M, `signal` in N — and each time you'll add the name
   yourself and know why. If you ever reach for a gated module early, the
   compiler's error is a small masterpiece — see the fights section.

4. **Hello, axum.** Replace `src/main.rs` with the smallest server that
   answers. You write it; the shape is three ideas —

   - an async handler: `async fn healthz() -> &'static str` returning the
     static JSON literal `{"received":0,"accepted":0,"rejected":0}` — a
     borrowed string baked into the binary, zero allocations per request
     (ramp 4 would approve). This is **A3 groundwork**: the *shape* is
     frozen today (three counters, these names); Sitting L makes the numbers
     true. Static JSON is an honest placeholder — the counts really are zero,
     the service has done nothing;
   - a router: `axum::Router::new().route("/healthz", axum::routing::get(healthz))`;
   - a listener + serve, inside `#[tokio::main] async fn main()`:

   ```rust
   let listener =
       tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 7311))).await?;
   println!("relay: listening on 127.0.0.1:{}", listener.local_addr()?.port());
   axum::serve(listener, app).await?;
   ```

   (`use std::net::{Ipv4Addr, SocketAddr};` up top; hardcode 7311 for now —
   clap takes over in move 5. The `?`s mean `main` returns a `Result`; use
   `anyhow::Result<()>` and add `anyhow = "1"` — the constitution's split:
   binaries speak `anyhow`, libraries will speak `thiserror` when Sitting L
   builds one.) Two deliberate details to say aloud before running:
   `Ipv4Addr::LOCALHOST`, never `0.0.0.0` — a lab tool has no business on
   the network (that's the bind half of **A7**); and the listening line
   reports `local_addr()`, not the number you asked for — the *actual* bound
   port. They're the same today; the moment `--port 0` exists ("OS, pick a
   free one"), only `local_addr()` tells the truth, and Sitting N's shutdown
   test will parse this exact line to find the port. Print the truth from
   day one. Now:

   ```console
   cargo run -p relay
   # → relay: listening on 127.0.0.1:7311      (and then... nothing. it's serving.)
   ```

   From a second terminal:

   ```console
   curl -s localhost:7311/healthz
   # → {"received":0,"accepted":0,"rejected":0}
   curl -si localhost:7311/healthz | head -1
   # → HTTP/1.1 200 OK
   ```

   Ctrl-c the server and *watch how it dies*: instantly, mid-anything, no
   goodbye. That abruptness is fine for a server holding nothing — and will
   be the whole problem once it holds a queue of your events. Sitting N
   earns the goodbye. **Commit point:**

   ```console
   cargo fmt
   cargo clippy -p relay --all-targets -- -D warnings
   git add crates/relay Cargo.lock && git commit -m "005: sitting K — hello axum"
   ```

5. **Args (A7).** You built a clap derive in 004's sitting I; this one is
   smaller. Add `clap = { version = "4", features = ["derive"] }` and give
   relay its contract:

   - `--lake <dir>` — a `PathBuf`, default `datalake/raw-local`;
   - `--port <n>` — a `u16`, default `7311` (and note what the *type*
     already promises: clap rejects `--port 99999` for you, exit 2, no code
     written);
   - doc comments on the struct and both fields — clap prints `///` lines as
     help text, so write them for the reader: say where events land, say
     `127.0.0.1 only`, say `0 = let the OS pick`.

   Wire `args.port` into the bind and keep `args.lake` unused for now —
   underscore it or pass it along to nothing; L gives it a job. Check the
   surface:

   ```console
   cargo run -p relay -- --help
   ```

   The reference's help, for the target shape (yours will differ in wording,
   not in contract):

   ```
   Usage: relay [OPTIONS]

   Options:
         --lake <LAKE>  Lake root directory (events land in <lake>/dt=<day>/events.jsonl) [default: datalake/raw-local]
         --port <PORT>  Port to listen on (127.0.0.1 only; 0 = let the OS pick) [default: 7311]
     -h, --help         Print help
     -V, --version      Print version
   ```

   Try `--port 0` once and read the listening line — the OS picked, and
   `local_addr()` told you which. That's move 4's discipline paying off
   early.

6. **The A7 example test.** Same pattern as glake's CLI suite since sitting
   B: an integration test that runs the *real binary*. Create
   `crates/relay/tests/cli.rs` with two `#[test]`s (plain, not
   `#[tokio::test]` — you're spawning a process, not awaiting a future):

   - **help shows the contract**: run `env!("CARGO_BIN_EXE_relay")` with
     `--help`; assert exit 0 and that stdout contains `--lake`, `--port`,
     both defaults (`datalake/raw-local`, `7311`), and `127.0.0.1` — a user
     can discover the whole A7 contract without reading the spec;
   - **bad args exit 2**: for a few broken invocations (`--nope`,
     `--port not-a-number`, a bare `--port`, an unexpected positional),
     assert exit code 2 and a non-empty stderr — clap explains itself, no
     panic. Exit 2 for usage errors has been the house contract since 004's
     F5; relay inherits it by using the same tool.

   ```console
   cargo test -p relay --test cli
   # → running 2 tests ... test result: ok. 2 passed
   ```

   (Reference test names, if you want the convention: `a7_help_shows_flags_and_defaults`,
   `a7_bad_args_exit_2` — tests named after the REQ they witness, as always.)

7. **Wire your glake in (compile check only).** The 004 payoff starts here.
   In `crates/relay/Cargo.toml`:

   ```toml
   glake = { path = "../glake" }
   ```

   That's your library — the one with `EventParser`, `SerdeParser`, and the
   day rule — now a dependency of a second program. Nothing calls it yet
   (Sitting L's door is the first real call); today is the compile check and
   a look at the tree:

   ```console
   cargo build -p relay
   cargo tree -p relay -e normal --depth 1
   # → relay v0.1.0 (…/crates/relay)
   #   ├── anyhow v1.…
   #   ├── axum v0.8.…
   #   ├── clap v4.…
   #   ├── glake v0.2.0 (…/crates/glake)   ← yours, by path
   #   └── tokio v1.…
   ```

   Read the glake line slowly: `(path)` means relay builds against the
   source in your own repo — when L calls `SerdeParser`, the door's verdicts
   come from code *you* wrote and property-tested. Drift between validator
   and lake reader becomes impossible, because they are the same function.
   That tree is also **A8 groundwork**: relay's direct dependencies stay on
   a short allow-list, and Sitting N's hygiene sweep will hold this exact
   command against it. **Commit point:**

   ```console
   cargo fmt
   cargo clippy -p relay --all-targets -- -D warnings
   git add crates/relay Cargo.lock && git commit -m "005: sitting K — args, A7 test, glake dep"
   ```

## Compiler fights to expect

Same contract as every sitting: each fight is curriculum, logged to the
mistake ledger (`learning.*`) as SKILLS evidence.

- **`error[E0752]: `main` function is not allowed to be `async``** — if you
  write `async fn main()` and forget `#[tokio::main]`. Step 12 move 3's
  lesson from the other side: something ordinary has to sit at the bottom
  driving futures, and plain `main` is where the buck stops. The macro
  builds the runtime `main` can't be.
- **`error[E0433]: failed to resolve` … with `note: found an item that was
  configured out` … `the item is gated behind the `net` feature`** — if a
  tokio module you reach for isn't in your feature list. Read that note
  twice; it's one of rustc's best: the code *exists* in tokio's source, but
  your feature selection compiled it away. (You may not hit this today —
  axum turns on tokio's `net` slice for its own use, and cargo unifies
  features across the graph, so `TcpListener` arrives whether you name it or
  not. Fine print worth saying aloud: your build works because *axum* asked.
  When L reaches for `tokio::fs`, nobody else has asked — you'll meet this
  error for real, and the fix is one word in your own feature list.)
- **`error[E0277]: the trait bound … `Handler<_, _>` is not satisfied`** —
  axum's most famous error, pages of it, pointing at your `.route(...)`
  line. Ninety percent of the time the cause is tiny: the handler isn't
  `async`, or its return type isn't something axum knows how to turn into a
  response. Don't read all forty lines — check those two things first. (L
  adds the third classic cause: extractor order. Not today's problem.)
- **`error: cannot find derive macro `Parser``** — clap without
  `features = ["derive"]`. Same lesson as tokio's slices: the derive is an
  opt-in feature, and 004's sitting I already made you pay it once —
  this is the rerun you should catch in seconds.
- **`error[E0308]: mismatched types` at the bind** — if you hand
  `TcpListener::bind` a bare string tuple and types don't line up, or forget
  `.await` on it (step 12's E0308: expected listener, found future — the
  main event of the ramp, back within the week).

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                   # no diff
cargo clippy -p relay --all-targets -- -D warnings  # clean
cargo test -p relay                                 # 2 passed (the A7 pair)
cargo tree -p relay -e normal --depth 1             # tokio, axum, clap, anyhow, glake (path) —
                                                    # and nothing else
```

```
cargo run -p relay
# → relay: listening on 127.0.0.1:7311

curl -s localhost:7311/healthz
# → {"received":0,"accepted":0,"rejected":0}

cargo run -p relay -- --port 0
# → relay: listening on 127.0.0.1:<whatever the OS picked> — the REAL port

cargo run -p relay -- --help          # both flags, both defaults, "127.0.0.1"
cargo run -p relay -- --port oops; echo $?
# → clap usage error on stderr, exit 2
```

- `git log --oneline -2` shows `005: sitting K — args, A7 test, glake dep`
  above `005: sitting K — hello axum`.
- You can answer aloud: what is `main` *doing* while requests are being
  answered (hint: it's parked on one `.await` — whose)? Why print
  `local_addr()` instead of `args.port`? Why does the glake line in the tree
  say `path`, and what does that buy Sitting L? And what happens to a
  request that's mid-flight when you ctrl-c — today, honestly? (You watched
  it. Sitting N fixes it.)

## Hints (one at a time)

<details><summary>Hint 1 — a nudge: the wiring order in main</summary>

Four statements, in dependency order: parse args → build the router → bind
the listener (`.await`, and `?` — the bind can genuinely fail: try running
two relays on the same port and read the error) → print the listening line →
`axum::serve(listener, app).await?`. The router is plain data until `serve`
drives it; nothing handles requests before that last line, and nothing after
it runs until the server stops. If your `println!` never appears, it's below
the `serve` — parked behind a future that doesn't finish.

</details>

<details><summary>Hint 2 — the shape: the A7 test file</summary>

```rust
use std::process::Command;

#[test]
fn a7_help_shows_flags_and_defaults() {
    let out = Command::new(env!("CARGO_BIN_EXE_relay"))
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    // assert contains: --lake, --port, datalake/raw-local, 7311, 127.0.0.1
}
```

`CARGO_BIN_EXE_relay` is cargo's own env var naming the freshly built
binary — the same trick glake's `cli.rs` has used since sitting B, so steal
your own pattern. `unwrap()` is fine in tests (a panicking test is a failing
test, which is the point); Sitting N's lint sweep will formalize where it's
banned.

</details>

## If truly stuck

Read, don't copy — then close it and write yours:

- `specs/005-async-relay/_reference/relay/src/main.rs` — the `Args` struct
  and the first half of `main` (ignore everything about channels, writers,
  and shutdown: that's sittings M–N material sitting in the finished file).
  One warning: the reference's glake path dep points at
  `../../../004-glake-traits/_reference/glake` because the reference lives
  four levels deep — from `crates/relay`, *yours* is `../glake`.
- `specs/005-async-relay/_reference/relay/tests/cli.rs` — both A7 tests,
  including the bad-args table.
