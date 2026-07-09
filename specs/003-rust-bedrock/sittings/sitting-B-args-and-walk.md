# Sitting B — Commands, folders, and failing well

**Builds:** glake learns its two commands (`validate`/`stats`), finds every
`*.jsonl` under a lake folder (or takes one file as-is), and fails like a real
Unix tool — clear stderr, exit 2, never a panic.
**Requirements:** R3a, R3b, R6 (task 1.2).
**Ramp you'll use:** steps 4 (`&str` vs `String`), 5 (`match`), 6 (`Result` + `?` (L4)) — step 7's file reading carries over from Sitting A.

## Where you are

In Sitting A you made `crates/glake` a real workspace member: it takes one
path from `std::env::args` with a polite `.get(1)`, reads the file, and prints
a line count. It already fails without panicking — a missing argument prints a
usage line on stderr, a missing file surfaces a clean `?`-carried `io::Error`
— but the exit *codes* still lie: the usage path returns `Ok(())` (exit 0, the
lie A's move 3 told you to tolerate), and the I/O path exits with whatever
`main`'s `Err` default is, not a code you chose. This sitting keeps A's
manners and finishes the contract: a command vocabulary (`validate`/`stats`)
and honest exit codes — from here on, every way glake can fail is a *designed*
behavior with a test, not a default.

## The build, move by move

Each command below runs from the repo root. Both commands will do the same
placeholder thing this sitting (count files and lines) — B is about **routing
and failing well**; `validate` and `stats` get their real bodies in Sittings D
and E.

1. **Make main own the process exit code.** Change the signature you aim at to:

   ```rust
   fn main() -> std::process::ExitCode
   ```

   Why: R6 says "exit non-zero, no panic", and returning an `ExitCode` from
   `main` is the idiomatic way to do that — unlike `std::process::exit`, it
   lets destructors run (that will matter when the lens session flushes its
   trace in Sitting F). Reserve **2** for usage and I/O errors; **1** stays
   free because Sitting D needs it to mean "the lake has malformed lines"
   (R1b). Two different failures, two different numbers — that's the contract
   scripts rely on.

2. **Parse `<validate|stats> <path>` with one `match`.** Collect the args into
   a `Vec<String>`, then match on the *pair* (command, path) so that every bad
   shape — no args, a lone command, an unknown command — falls into a single
   `_` arm that prints a usage line **to stderr** (`eprintln!`, not
   `println!`: stdout is for answers, stderr is for complaints) and returns
   `ExitCode::from(2)`.

   Before you write it, look at what you already have: Sitting A's `.get(1)`
   match already turns "no arguments" into a usage line on stderr — that
   behavior is right, and only its exit code (`Ok(())` → 0) is the tolerated
   lie. What's new is the grammar: two words instead of one means two shapes A
   never had to consider — a lone command and an unknown command — and the
   pair-match lets all three bad shapes share one usage arm instead of each
   needing its own check. (Hold onto A's get-vs-index instinct — the exact
   same choice comes back in Sitting E on a timestamp, and the design already
   made a call about it.)

   Prove the three bad shapes by hand:

   ```
   cargo run -p glake; echo $?
   cargo run -p glake -- stats; echo $?
   cargo run -p glake -- frobnicate x; echo $?
   ```

   All three: usage on stderr, then `2` — the stderr half you've had since A;
   the honest `2` is what this move adds.

   **Commit point** (after `cargo fmt` and
   `cargo clippy -p glake -- -D warnings` pass, as before every commit):
   `003: sitting B — validate/stats commands, usage on stderr, exit 2`

3. **Build a miniature lake to test against.** Create
   `crates/glake/tests/fixtures/lake/` with two partitions, mirroring the real
   lake's shape:

   ```
   mkdir -p crates/glake/tests/fixtures/lake/dt=2026-07-01
   mkdir -p crates/glake/tests/fixtures/lake/dt=2026-07-02
   ```

   Put an `events.jsonl` in each (a couple of real lines copied from
   `datalake/raw-local/dt=*/events.jsonl` are perfect — Sittings D and E will
   grow these fixtures into malformed/blank cases, so real envelope lines age
   well). **One edit per copied line:** change its `ts` so the day agrees with
   the partition it lands in — lines in `dt=2026-07-01/` get a `ts` starting
   `2026-07-01`, lines in `dt=2026-07-02/` a `ts` starting `2026-07-02`. Why
   this matters: `stats`' by-day buckets come from each line's **`ts`**, not
   from the folder name, and Sitting E's checkpoint hand-counts these fixtures
   expecting partition name and `ts` to agree — lines copied as-is would all
   report their *original* day and E's by-day assertions would fail. Also drop
   a decoy `notes.txt` into one partition: your walk must *not* pick it up,
   and a test should prove that.

4. **Write the walk in its own room.** Give the crate a library: create
   `src/lib.rs` declaring a `walk` module, and in `src/walk.rs` aim at the
   design's frozen signature:

   ```rust
   pub fn jsonl_files(path: &Path) -> std::io::Result<Vec<PathBuf>>
   ```

   Why a library next to the binary: Sitting C's property tests need to call
   your pure functions directly, and `main.rs` should stay a thin shell that
   only translates `Result`s into exit codes. The behavior to achieve: a
   **file** path → a one-element `Vec` with just it (R3b); a **folder** →
   every `*.jsonl` beneath it, however deep — `dt=` partitions and the memlens
   `traces/` folder alike (R3a, rev 4); any
   I/O failure → `Err` bubbled up with `?` (ramp step 6 — no printing, no
   exiting inside the library; that's main's job). You'll want a private
   recursive helper — decide for yourself what it should borrow and what it
   should return.

   Three questions to settle *before* the compiler gets a vote:

   - Rust gives you two `ends_with`-shaped ways to ask "is this a `.jsonl`
     file?" — one on `Path`, one on `str` — plus `Path::extension()`. One of
     those compiles happily and never matches anything. Which, and why?
   - In what order does `read_dir` hand back entries? If you run
     `glake stats` twice, must the output match? What's the cheapest way to
     make it deterministic?
   - You *could* check that folder names look like `dt=…` before recursing,
     or recurse into any directory and let the `.jsonl` filter do the work.
     This is not a free choice: requirements rev 4 fixed R3a's scope — the
     walk finds **every `.jsonl` beneath the path**, `dt=` partitions and
     `traces/` alike — so recurse into every directory. Still worth writing
     down: what would the dt=-only version have silently missed? The real
     lake makes it concrete in move 6.

5. **Prove R3a and R3b with example tests.** In a `#[cfg(test)] mod tests` at
   the bottom of `walk.rs`, write two tests against your fixture lake: the
   folder walk finds *exactly* the two `events.jsonl` files (decoy excluded),
   and a single-file path returns just that file. To find the fixtures no
   matter where the test runs from, build the path off
   `env!("CARGO_MANIFEST_DIR")`. Run:

   ```
   cargo test -p glake
   ```

   **Commit point:** `003: sitting B — jsonl_files walk (R3a/R3b) with fixture tests`

6. **Wire main to the walk and make failure a report, not a crash.** Replace
   Sitting A's direct file read: hand the path to `jsonl_files`, `match` on
   the `Result`, and on `Err` print **one clear stderr line naming the path
   and the OS's reason** (aim at the shape `glake: <path>: <why>` — the `Err`
   value already knows why and `Display`s it), then `ExitCode::from(2)`.
   Notice you *can't* use `?` here — see the fights below for why that
   restriction is the lesson. On `Ok`, keep the placeholder: print how many
   files were found and the total line count across them.

   Then take it to the real lake:

   ```
   cargo run -p glake -- stats datalake/raw-local
   cargo run -p glake -- stats /no/such/path; echo $?
   ```

   Look closely at the first run's file count versus
   `ls datalake/raw-local/dt=*/events.jsonl`. Your walk finds something
   *extra* — the memlens `traces/` files. That's not a bug: it's R3a (rev 4)
   working as specified, every `.jsonl` beneath the path. Write what you
   observe in a comment near `jsonl_files` — Sitting F's reconciliation
   against `scan.sh` (R10: glake's total = scan.sh's event count + the trace
   lines) builds on exactly this note, and you'll be glad past-you left it.

7. **Pin every R6 door shut with CLI example tests.** Create
   `crates/glake/tests/cli.rs` — an integration test that runs your real
   binary. Cargo hands you the compiled binary's path in an env var; the
   pattern to aim at:

   ```rust
   std::process::Command::new(env!("CARGO_BIN_EXE_glake"))
   ```

   Cover: (a) each bad-arg shape from move 2 → exit code `Some(2)` and
   `"usage"` somewhere on stderr; (b) a path that doesn't exist → exit
   `Some(2)`, stderr *names the path*, and stderr does **not** contain
   `"panicked"` — that last assertion is R6's "no panic" made mechanical.
   (The *unreadable*-path case stays a manual checkpoint below: `chmod`-based
   tests lie under root and in CI, so we check that door by hand.)

   ```
   cargo test -p glake
   cargo fmt && cargo clippy -p glake -- -D warnings
   ```

   **Commit point:** `003: sitting B — R6 error paths and CLI example tests`

## Compiler fights to expect

Every one of these is on the syllabus — when we hit one, we log it
(`learning.*` mistake ledger) and it becomes SKILLS evidence. Losing to the
compiler here *is* the coursework.

- **E0308 — mismatched types: expected `String`, found `&str`.** You'll meet
  it matching an arg against the literal `"validate"`. Ramp step 4 in the
  wild: literals are `&str`, `env::args` gives owned `String`s, and `match`
  won't compare across that line for free. The fix is a one-word borrow
  conversion — find the method on `String` (or on `Option<String>`) that
  lends you a `&str`.
- **E0277 — "the `?` operator can only be used in a function that returns
  `Result`".** You'll hit it if you try `?` inside `main` once it returns
  `ExitCode`. This is the whole architecture in one error: `?` *bubbles*
  errors upward, but main is the top — the place errors stop being values and
  become an exit code and a stderr line. Inside `walk.rs`, `?` everywhere;
  inside `main`, `match`.
- **E0599 — no method `path` on `Result<DirEntry, …>`.** `read_dir` doesn't
  yield entries, it yields `Result`s of entries — the directory can fail
  mid-iteration. Ramp step 6 again: acknowledge the `Result` (one `?`) before
  touching what's inside.
- **E0277 — can't compare `Option<&OsStr>` with `&str`.** `Path::extension()`
  returns an `Option` around an *OS* string, not a `str`. Three nested worlds
  to unwrap; look for an `Option` combinator that tests the inside without
  `unwrap()` (which R6-adjacent discipline bans from this crate anyway).
- **E0382 — use of moved value.** Classic shape: you `push(path)` into your
  `Vec` and then try to print or inspect `path` afterward. Ramp step 2: the
  push *moved* it. Reorder, or borrow before you give it away.
- **The fight the compiler won't pick:** `Path::ends_with(".jsonl")` compiles
  cleanly and is silently always-false — `Path`'s version compares whole path
  *components*, not string suffixes. No error code, just a walk that finds
  nothing. If your R3a test fails with an empty `Vec`, start here. (This is
  why move 4 made you choose consciously.)

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                  # no diff
cargo clippy -p glake -- -D warnings               # clean
cargo test -p glake                                # all green: R3a, R3b walk tests + R6 CLI tests
```

```
cargo run -p glake -- stats crates/glake/tests/fixtures/lake
# → reports exactly 2 files (decoy notes.txt excluded), correct total line count

cargo run -p glake -- stats crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl
# → reports exactly 1 file (R3b: no walking when given a file)

cargo run -p glake -- frobnicate x; echo $?
# → usage line on stderr, exit 2   (same for zero args and a lone command)

cargo run -p glake -- stats /no/such/path; echo $?
# → ONE line on stderr naming /no/such/path and the reason; exit 2; no backtrace

chmod 000 crates/glake/tests/fixtures/lake/dt=2026-07-02
cargo run -p glake -- stats crates/glake/tests/fixtures/lake; echo $?
chmod 755 crates/glake/tests/fixtures/lake/dt=2026-07-02
# → middle command: clear stderr line, exit 2, no panic (the unreadable half of R6)

cargo run -p glake -- stats datalake/raw-local
# → runs clean on the real lake; you can say OUT LOUD which .jsonl files it
#   found and why the count is what it is
```

And `git log --oneline -3` shows the three sitting-B commits.

## Hints (one at a time)

<details>
<summary>Hint 1 — the argument match won't come together</summary>

Collect once: `let args: Vec<String> = std::env::args().collect();`. Then
match on a *tuple* so one expression decides everything:
`match (args.get(1).map(String::as_str), args.get(2))`. Now `args.get(1)` is
an `Option<&str>` you can compare against literals, and every invalid shape —
`(None, _)`, `(Some("stats"), None)`, `(Some(anything-else), _)` — falls
naturally into the `_` arm. Count the arms you actually need: it's two.

</details>

<details>
<summary>Hint 2 — the recursive walk won't take shape</summary>

Split it in two. `jsonl_files` itself only decides *file or folder*: ask
`std::fs::metadata(path)?` (a missing or unreadable path becomes your R6 `Err`
right there, for free) — if it's a file, return a `Vec` containing
`path.to_path_buf()`; if it's a folder, delegate to a private helper shaped
like `fn collect(dir: &Path, found: &mut Vec<PathBuf>) -> io::Result<()>`.
The helper loops over `std::fs::read_dir(dir)?`, `?`s each entry, recurses
when `entry.path()` is a directory, and pushes when
`path.extension().is_some_and(|e| e == "jsonl")`. Sort the `Vec` before
returning so output order never depends on the filesystem's mood.

</details>

<details>
<summary>Hint 3 — the CLI tests won't assert</summary>

The pattern per test: build
`Command::new(env!("CARGO_BIN_EXE_glake")).args(["stats", "/no/such/path"])`,
call `.output()`, and assert on two things:
`out.status.code() == Some(2)` and
`String::from_utf8_lossy(&out.stderr)` containing (or, for `"panicked"`,
NOT containing) the text you promised. For the fixture-path tests, build
absolute paths with `Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/...")`
so the tests pass regardless of the runner's working directory.

</details>

## If truly stuck

Read, don't copy — then close it and write yours:

- `specs/003-rust-bedrock/_reference/glake/src/walk.rs` — `jsonl_files` and `collect`
- `specs/003-rust-bedrock/_reference/glake/src/main.rs` — `main` (the arg match and the error boundary)
- `specs/003-rust-bedrock/_reference/glake/tests/cli.rs` — `r6_usage_on_bad_args` and `r6_bad_path_stderr_exit2_no_panic`
