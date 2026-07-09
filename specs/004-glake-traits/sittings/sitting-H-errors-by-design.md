# Sitting H — errors by design

**Builds:** glake's first *designed* failure path — one public `GlakeError`
enum (`thiserror`) that every fallible library function returns, a `main` that
funnels any error into exactly one clear stderr line, and exit codes promoted
from habit to contract: **2** usage/io, **1** validation findings, **0** clean
(F5, normative for v1). Plus the sitting's set-piece: you will try `#[from]`,
hit a wall the requirements were amended over, and come out knowing why
context beats convenience.
**Requirements:** F5, F6 (task 1.2) — and F11 quietly begins (the first
dependency ever to enter glake's shipped tree).
**Ramp you'll use:** step 6 (`Result` + `?` — this is its graduation), step 5
(enums carrying data), step 2 (ownership — the error *owns* its context).

## Where you are

Sitting G left the crate reshaped and provably unchanged. Now the first real
v1 muscle. Look at your `main.rs` honestly: three separate places `match` on
an `io::Result`, and each invents its own `eprintln!` on the spot. It works —
B's tests pin it — but it's *ad-hoc*: the message format lives in three
copies, the exit code is a convention nobody wrote down, and the library
can't fail without `io::Error` leaking its vocabulary to the user. v1's
answer (F5/F6): the library gets one error type that says what glake was
*doing* when it failed, `?` carries it, and one `match` in `main` — the
funnel — turns any error into the single line a script, a CI log, or a
CloudWatch stream can act on. That last audience is the AWS through-line:
a Lambda that panics gives you a stack trace and a cold start; a Lambda that
returns one designed error line gives you an alarm you can route. Same
discipline, learned on a CLI. (The constitution's no-`unwrap()`-in-service-
paths rule — the Cloudflare Nov-2025 outage lesson — is this sitting wearing
work clothes.)

## The build, move by move

All commands from the repo root. Two commits: the enum (with its checklist
tests), then the threading.

1. **Invite `thiserror` — and mark the moment.** In `crates/glake/Cargo.toml`:

   ```toml
   [dependencies]
   thiserror = "2"
   ```

   Under `[dependencies]`, not `[dev-dependencies]` — errors ship. Stop and
   appreciate what just happened: for all of 003, `cargo tree -p glake -e
   normal` printed **one line**, and Sitting F made you prove it. That era
   ends now, *on purpose*: F11 amends the dependency policy to allow exactly
   `clap`, `serde_json`, `thiserror` (still nothing async, lens still
   optional). Run the check and read the new answer:

   ```console
   cargo tree -p glake -e normal --depth 1
   ```

   Two lines now — glake and `thiserror v2.x`. Drop the `--depth 1` and
   you'll see more: `thiserror-impl` marked `(proc-macro)`, hauling in `syn`
   and `quote`. Before you panic about R4's ghost, read the marker again —
   proc-macros are *build-time* code, compiler plugins that run while your
   crate compiles and never enter the shipped binary (Sitting F's
   what-ships-vs-what's-needed lesson, third verse: dev edges, build edges,
   and now proc-macro subtrees). Sitting J's hygiene sweep records the full F11
   evidence; today just know the tree is opening by design, not by drift.

2. **Design the enum on paper first.** Read F6 aloud: fallible library paths
   return `Result<_, GlakeError>`; io errors are `#[source]`-chained and
   *carry the offending path*; the type passes C-GOOD-ERR. Now the design
   question before any macro magic: **what variants exist and what does each
   carry?** The design table answers: two.

   - `Io { path: PathBuf, source: std::io::Error }` — a filesystem operation
     failed, *on this path*, *because of this underlying error*. The path is
     the context the user needs (the walk recurses — the file that failed is
     almost never the path the user typed); the source is the cause a tool
     can still unwrap.
   - `Usage(String)` — arguments that *parsed* but whose values make no sense
     (nothing constructs this today; Sitting I's `--since` validation is its
     customer — stub it now so the F5 exit-code story is complete).

   Resist inventing more (the reference's comment: "resist inventing error
   taxonomy before there are callers who would match on it"). Why does `Io`
   own a `PathBuf` and not borrow a `&Path`? Ramp 2/8 question, sitting-C
   inverted: the error out-lives the function that failed — it's about to
   travel up through `?` after the call frame dies. Borrowing is for results
   that live *inside* the caller's data; **owning is for values that leave**.
   File that sentence away: Sitting J's trait boundary is the same argument
   at pipeline scale.

3. **Write it with `#[from]` — and meet the wall.** Create
   `crates/glake/src/error.rs` (`pub mod error;` in `lib.rs`, alphabetical
   with its neighbors) and write what every thiserror tutorial reaches for
   first — automatic conversion so `?` "just works":

   ```rust
   use std::path::PathBuf;

   #[derive(Debug, thiserror::Error)]
   pub enum GlakeError {
       #[error("cannot read {}: {source}", path.display())]
       Io {
           path: PathBuf,
           #[from]
           source: std::io::Error,
       },
       #[error("{0}")]
       Usage(String),
   }
   ```

   ```console
   cargo build -p glake
   ```

   ```text
   error: deriving From requires no fields other than source and backtrace
   ```

   That's the wall, verbatim, and it isn't thiserror being fussy — it's
   thiserror being *honest*. `#[from]` asks the macro to generate
   `impl From<io::Error> for GlakeError`, and a `From` impl receives exactly
   one value: the `io::Error`. Where would the `path` come from? Nowhere —
   an `io::Error` doesn't know what path it was about (try it:
   `std::fs::read_to_string("/nope")`'s error prints `No such file or
   directory (os error 2)` — *which* file?). You have two exits:

   - **Drop the path.** `Io(#[from] io::Error)` compiles, `?` converts
     automatically… and every failure in a 200-file lake walk reports
     `cannot read: permission denied` with no clue which file. Anonymous
     errors cost a debugging session.
   - **Keep the path, write the conversion yourself.** `#[source]` instead of
     `#[from]` (chain the cause, skip the auto-`From`), plus a helper so call
     sites stay one honest closure long.

   The requirements already ruled — F6 as amended says `#[source]`-chained
   *carrying the offending path* (the changelog entry reads `#[from]` →
   `#[source]`+path, "unimplementable as drafted": this exact wall, hit
   during spec audit). Swap the attribute and add the helper:

   ```rust
   impl GlakeError {
       /// Attach path context to an io error — the `map_err` companion:
       /// `std::fs::read_to_string(p).map_err(|e| GlakeError::io(p, e))`.
       pub fn io(path: &Path, source: std::io::Error) -> Self { /* … */ }
   }
   ```

   Context costs one closure per call site; that's the price, and it's low.

4. **Walk C-GOOD-ERR as tests, not vibes (F6).** The checklist says: messages
   are lowercase with no trailing period; the type is
   `std::error::Error + Send + Sync + 'static`; causes are reachable through
   `source()`. Each clause becomes a `#[test]` in a `#[cfg(test)] mod tests`
   at the bottom of `error.rs` — write all three now:

   - **Send + Sync + 'static, checked at compile time** — the neat trick:
     an empty generic function `fn assert_good_err<T: std::error::Error +
     Send + Sync + 'static>() {}` and one turbofish call
     `assert_good_err::<GlakeError>()`. If the bound ever breaks, this test
     stops *compiling* — the cheapest possible property. (Why do Send/Sync
     matter for a single-threaded CLI? They don't, yet — they matter for the
     `tokio` code 005 will make of you, and for any error that crosses
     `Box<dyn Error + Send + Sync>`. C-GOOD-ERR is about not painting
     yourself into a corner.)
   - **Display style** — build one `Io` (manufacture the inner error with
     `std::io::Error::new(ErrorKind::PermissionDenied, "permission denied")`)
     and one `Usage`; assert each `to_string()` doesn't start uppercase,
     doesn't end with `.`, and the io one starts with
     `cannot read /root/forbidden:` — the requirements' own worked example.
     Why lowercase-no-period? Errors get *composed*: wrapped, prefixed,
     chained after "glake: " — a capitalized sentence mid-chain reads like a
     ransom note.
   - **The chain** — `use std::error::Error as _;` (an anonymous import —
     you only need the trait's methods in scope, ramp 9 move 7's rule), then
     assert `e.source()` is `Some` for `Io` and its `to_string()` still says
     what the inner error said; `Usage("bad").source()` is `None`.

   ```console
   cargo test -p glake error
   cargo fmt && cargo clippy -p glake --all-targets -- -D warnings
   ```

   **Commit point:**

   ```console
   git add crates/glake Cargo.lock && git commit -m "004: sitting H — GlakeError enum + C-GOOD-ERR tests"
   ```

5. **Thread it through the library.** Two homes change, both in `walk.rs`:

   - `jsonl_files` returns `Result<Vec<PathBuf>, GlakeError>` now — every
     `std::fs` call inside (the `metadata`, the `read_dir`, the per-entry
     unwrap) gets `.map_err(|e| GlakeError::io(<the path at hand>, e))`.
     Note *which* path each site attaches: the recursion's current `dir`, not
     the root the user typed — that's the entire value of carrying context.
   - A new public helper, because both commands read files the same way:

     ```rust
     pub fn read_file(path: &Path) -> Result<String, GlakeError>
     ```

     — `read_to_string` with the same `map_err`, in the lib where it belongs.

   Chase the compiler outward: `main.rs`'s `validate` and `stats` stop
   matching on io errors inline and become
   `fn validate(files: &[PathBuf]) -> Result<ExitCode, GlakeError>` (same for
   `stats`), using `read_file(file)?` — watch three inline `match` blocks
   collapse into three `?`s. Ramp 6 said `?` was for propagating; now you
   feel what it's *for*: the library states facts, one place decides tone.

6. **Build the funnel (F5) and make the codes normative.** In `main`: keep
   the hand-rolled usage arm (clap replaces it next sitting — don't polish
   it), then route both commands through one seam:

   ```rust
   match run(cmd, path) {                 // run: dispatch → validate | stats
       Ok(code) => code,
       Err(e) => {
           eprintln!("glake: {e}");
           ExitCode::from(2)
       }
   }
   ```

   That `eprintln!` is now the **only** place a library error becomes user
   text — the funnel. And write the contract into a doc comment at the top of
   `main.rs`, because F5 makes it normative for v1: **2** usage problems and
   io failures · **1** `validate` found findings · **0** clean. Those are
   the same numbers B chose; what's new is that they're *promised* — scripts
   may now depend on them, and I and J's tests will.

   Update `tests/cli.rs`: your bad-path test asserts the new message shape
   and — new teeth — that stderr is exactly **one line**. The validated
   behavior to aim at:

   ```console
   cargo run -p glake -- stats /no/such/path
   # stderr: glake: cannot read /no/such/path: No such file or directory (os error 2)
   # exit:   2
   ```

   One line, glake-prefixed, path named, cause included — F5 and F6 shaking
   hands. Then the full gate and the real lake:

   ```console
   cargo fmt
   cargo clippy -p glake --all-targets -- -D warnings
   cargo test -p glake
   cargo run -p glake -- validate datalake/raw-local; echo $?   # 0 — clean lake, code unchanged
   ```

   **Commit point:**

   ```console
   git add crates/glake && git commit -m "004: sitting H — errors threaded, one stderr funnel"
   ```

## Compiler fights to expect

The headline fight is move 3's wall — a *macro* error, not an E-code, and
this course's first: proc-macro authors write their own diagnostics, and
thiserror's are good. Log them all (`learning.*`) as usual.

- **`error: deriving From requires no fields other than source and
  backtrace`** — the planned one. What it's really saying: `From` is a
  one-argument conversion, and your variant needs *two* facts to exist. Any
  time an error variant carries context beyond its cause, `#[from]` is
  arithmetically impossible — reach for `#[source]` + a constructor. This
  wall is why the requirement was amended before you ever typed it.
- **`error[E0277]: `?` couldn't convert the error to `GlakeError`** — after
  the threading, at any `?` still sitting on a bare `io::Result`. The trait
  the compiler wants (`From<io::Error> for GlakeError`) is exactly the one
  you *declined* to auto-derive at the wall — so this error is your own
  design talking, telling you a call site is missing its
  `.map_err(|e| GlakeError::io(path, e))`. Don't cave and add a blanket
  `From`; the whole point is that no io error enters glake without a path
  stapled to it.
- **`error[E0308]: mismatched types` in `main`** — while the signatures of
  `validate`/`stats` shift from `ExitCode` to `Result<ExitCode, GlakeError>`,
  every `return ExitCode::from(2)` inside them becomes stale (those paths
  now `?` instead). Mechanical, but it forces the good question: which
  failures are *the command's own verdict* (validate's exit 1 — an `Ok`!)
  versus *errors* (io — an `Err`)? Findings are not failures; that
  distinction is F5's whole table.
- **`unused variable` / `dead_code` on `Usage`** — shouldn't fire (`pub`
  items in a lib are API surface, not dead code), but if you made a private
  helper only `Usage` would use, clippy will notice nobody calls it yet.
  Sitting I is the caller; keep the stub minimal today.
- **Display test red on style** — `"Cannot read…"` capitalized or a trailing
  period sneaking into the `#[error("…")]` string. Not a compiler fight; the
  C-GOOD-ERR tests you wrote in move 4 are doing their job. Fix the
  attribute, not the test.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                  # no diff
cargo clippy -p glake --all-targets -- -D warnings # clean
cargo test -p glake                                # green: the three f6_* checklist
                                                   # tests + your whole 003 suite
cargo tree -p glake -e normal --depth 1            # exactly two lines: glake, thiserror
```

```
cargo run -p glake -- stats /no/such/path; echo $?
# → ONE stderr line: glake: cannot read /no/such/path: No such file or directory (os error 2)
# → 2
cargo run -p glake -- validate datalake/raw-local; echo $?
# → all lines well-formed, 0        (findings-vs-errors: still not an Err)
cargo run -p glake -- stats datalake/raw-local
# → same shape and numbers as a pre-sitting run taken back-to-back with this
#   one (errors are plumbing, not behavior — G's diff discipline applies)
```

- `git log --oneline -2` shows the enum commit below the threading commit.
- You can answer aloud: why can't `#[from]` build a variant that carries a
  path? Why does `Io` *own* its `PathBuf` (and which sitting repeats that
  argument at a trait boundary)? Recite the F5 exit-code table and say which
  of the three numbers `validate`'s malformed-lines case uses — and why
  that's an `Ok` in the code.

## Hints (one at a time)

<details><summary>Hint 1 — where each map_err goes in the walk</summary>

There are exactly three fallible io calls in the reference walk: `metadata`
at the top (attach the *user's* path — it's the only path that exists yet),
`read_dir` in the recursive helper (attach `dir`), and the per-entry `?`
inside the loop (also `dir` — the entry itself is what failed to appear).
Plus `read_to_string` inside the new `read_file` (attach its argument). If
your walk's helper returns `io::Result`, change it too — `GlakeError` all
the way down, or the `?`s won't chain.

</details>

<details><summary>Hint 2 — the funnel's shape without giving away main</summary>

`run` is just your existing dispatch `match` with its body's error handling
deleted: build the file list with `jsonl_files(Path::new(path))?`, then call
the command fn and return what it returns. All three functions now return
`Result<ExitCode, GlakeError>`; only `main` itself still returns bare
`ExitCode`, because `main` is where errors stop being values and become an
exit status. If you're fighting `main`'s type, that division is the answer.

</details>

<details><summary>Hint 3 — the Display test keeps failing on the io message</summary>

The `#[error(...)]` attribute is a `format!` in disguise, and `path` is a
`PathBuf` — it has no `Display`. The reference's spelling puts the call in
the format args: `#[error("cannot read {}: {source}", path.display())`.
Inside the attribute you can name fields directly (`path`, `source`) — no
`self.`, no `{path.display()}` inline (that's not format syntax). And check
the inner error you manufacture in the test: `io::Error::new(kind, "…")`
displays your string, while `io::Error::from(kind)` displays the OS text —
either works, but assert against the one you built.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/004-glake-traits/_reference/glake/src/error.rs` — the enum, the
  `io` helper, and the three `f6_*` tests (the module doc comment retells
  this sitting's wall in four lines).
- `specs/004-glake-traits/_reference/glake/src/walk.rs` — where each
  `map_err` attaches which path, and `read_file`.
- `specs/004-glake-traits/_reference/glake/src/main.rs` — the funnel `match`
  in `main` and the exit-code contract in the doc header (ignore everything
  clap/parser/filter — that's I and J).
