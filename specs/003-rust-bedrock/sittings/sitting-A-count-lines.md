# Sitting A — a program that reads a file

**Builds:** the first real `glake` — a new workspace crate that takes one `.jsonl`
path on the command line and prints how many lines it has.
**Requirements:** none land here — this sitting is groundwork for **R3b** and
**R6**, both of which land in Sitting B.
**Ramp you'll use:** steps 1–3, 7.

## Where you are

The ramp is behind you: you can run a program (step 1), you know what a move is
and what a borrow is (steps 2–3), and in step 7 you read a file, carried its
`io::Error` out of `main` with `?`, and counted its non-blank lines. This
sitting does one new thing: it moves that exact skill out of the throwaway
`playground/` and into `crates/` — permanent, spec-governed code, reading the
*real* data lake your own hooks have been writing under `datalake/raw-local/`
all along. Dead-simple first, per the tasks doc: count the lines in one file;
everything else (commands, folders, error discipline) is Sittings B onward.

## The build, move by move

1. **Scaffold the crate — it joins the workspace by itself.** From the repo
   root:

   ```console
   cargo new crates/glake
   ```

   Why no workspace surgery is needed: the root `Cargo.toml` declares
   `members = ["crates/*"]`, so anything under `crates/` is already a member.
   Open the generated `crates/glake/Cargo.toml` and notice what cargo wrote:
   `edition.workspace = true` (plus license/repository) — the crate *inherits*
   edition 2024 from the root's `[workspace.package]` instead of restating it.
   One source of truth, same as our docs rule. Compare with
   `crates/memlens/Cargo.toml` — same shape.

2. **Add the safety pledge and prove it runs.** Put `#![forbid(unsafe_code)]`
   as the first line of `src/main.rs` (CLAUDE.md convention: every crate except
   `memlens` forbids unsafe outright — this makes the compiler enforce a
   project rule). Then:

   ```console
   cargo run -p glake
   ```

   `-p glake` matters: the workspace has more than one binary, so a bare
   `cargo run` from the root can't guess which one you mean. Expect
   `Hello, world!`. **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting A — glake crate joins the workspace"
   ```

3. **Take the path from the command line.** `std::env::args()` hands you an
   iterator of `String`s; item 0 is the program's own name, so the path you
   want is item **1** — and it arrives as an `Option`, because the user is
   allowed to type nothing. That `Option` is the whole lesson of this move:
   Rust turned "the argument might be missing" into a type you *must* unpack,
   not a crash you might get. Two shapes work; pick one deliberately:
   collecting into a `Vec<String>` and asking politely with `.get(1)` (this is
   the shape that scales when Sitting B needs *two* arguments), or the terser
   `.nth(1)`. Aim at ramp 7's fallible-main signature:

   ```rust
   fn main() -> Result<(), std::io::Error>
   ```

   When the argument is missing, print one usage line to **stderr**
   (`eprintln!`, not `println!` — stdout is for answers, stderr is for
   complaints) and leave `main` early.

   Two questions to sit with, not answer yet:
   - *Before* writing the polite version, try the rude one once: index with
     `args[1]` and run with no arguments. What happens, and why didn't the
     compiler stop you? Keep that feeling — the exact same
     indexing-vs-`.get()` choice comes back in Sitting E, on timestamps,
     where getting it wrong is a requirements violation.
   - After your usage message prints, run `echo $?`. Is that exit code
     telling the truth? R6 says a usage error must exit `2` — Sitting B
     replaces this whole opening with honest exit codes, so a small lie is
     acceptable *today only*.

4. **Read and count.** Ramp 7, but on real data: `std::fs::read_to_string`
   pulls the file into one owned `String`, `?` carries the `io::Error` out,
   `.lines()` walks it as borrowed `&str` slices, and step 7's optional
   iterator payoff — `.count()` — becomes the real implementation. Print
   **just the number** on stdout (the checkpoint compares it against `wc -l`).
   Memory story, because it's the through-line: the whole run costs *one* heap
   allocation for the file text; counting 134 lines — or a million — allocates
   nothing more, because every `line` is a borrowed view into that same
   buffer. In Sitting F you'll watch your own lens confirm this.

   One more question to park: `.lines().count()` counts blank lines too. Is a
   blank line an event? R5 says no — but *that decision belongs to Sitting C*,
   where "what is this line?" becomes its own function. Today's answer is the
   raw line count, on purpose.

5. **Prove the failure path, then gate and commit.** Misspell the path and
   watch `?` do its job (error printed, nonzero exit, no panic — same as ramp
   7 move 5). Then the pre-commit ritual that every future commit uses:

   ```console
   cargo fmt
   cargo clippy -p glake -- -D warnings
   ```

   **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting A — count lines of one jsonl file"
   ```

## Compiler fights to expect

Every fight below is curriculum, not failure — when one happens, we log it to
the mistake ledger (`learning.*` events) so SKILLS.md can track which error
classes stop recurring.

- **`error[E0282]: type annotations needed`** — if you chose the `collect()`
  shape and wrote `let args = std::env::args().collect();`. `collect` can
  build a `Vec`, a `HashMap`, a `String`… the compiler refuses to guess which.
  You must name the target: annotate the binding as `Vec<String>` (or
  turbofish it). New error class for your ledger — first time an *inference*
  limit, rather than an ownership rule, pushes back.
- **`error[E0277]: the trait bound `Option<String>: AsRef<Path>` is not
  satisfied`** — if you pass the `Option` straight into `read_to_string`.
  Translation for *this* program: you skipped deciding what happens when the
  user typed no path. Unpack the `Option` first; the decision is the code.
  (Note the code: you met E0277 in ramp 7 about `?` — same code, entirely
  different lesson. Error codes are *classes*, not single mistakes.)
- **`error[E0308]: `match` arms have incompatible types`** — if your `Some`
  arm produces the path but the `None` arm just prints and falls through. A
  `match` used as an expression must give one type from every arm; the `None`
  arm has to *diverge* — leave `main` — instead of producing a value.
- **`error[E0382]: borrow of moved value: `path``** — if you pass `path` to
  `read_to_string` by value and then try to use it again (say, printing it in
  the output). Ramp 2 resurfacing in real code: passing by value is a move.
  Lend it (`&path`) instead — `read_to_string` only needs to look.
- **A runtime panic, not a compiler error:** `index out of bounds` from
  `args[1]` with no arguments — the deliberate experiment from move 3. The
  compiler was silent because indexing is a *promise* ("this exists") while
  `.get()` is a *question* ("does this exist?"). Same family as ramp 7's
  count-of-4: past the type system's border, correctness is on you and your
  tests.

## Checkpoint

All commands from the repo root. The sitting counts as done when:

- ```console
  cargo run -p glake -- datalake/raw-local/dt=2026-07-05/events.jsonl
  ```
  prints a single number on stdout that **matches**
  `wc -l < datalake/raw-local/dt=2026-07-05/events.jsonl` (134 as of this
  writing — that day is closed, so it won't drift). If you're ever off by
  one against `wc -l`: `wc -l` counts newline characters, `.lines()` counts
  lines — they only agree when the file ends with a trailing newline (this
  one does).
- `cargo run -p glake` (no path) prints one usage line on **stderr** and
  nothing on stdout — prove the stream with
  `cargo run -p glake 2>/dev/null`, which must show no output.
- `cargo run -p glake -- nope.jsonl` prints an `io::Error` (something like
  `No such file or directory`), `echo $?` is nonzero, and no panic/backtrace
  appears.
- `cargo fmt` changes nothing and `cargo clippy -p glake -- -D warnings`
  exits clean.
- Both commits exist (`003: sitting A — glake crate joins the workspace`,
  `003: sitting A — count lines of one jsonl file`).
- You can answer aloud: how many heap allocations did counting all 134 lines
  cost, and which single expression paid for it?

## Hints (one at a time)

<details><summary>Hint 1 — a nudge</summary>

`std::env::args()` is an iterator you can drive like any other. Item 0 is the
program name — you never want it. The thing you want is item 1, and both ways
of asking for it (`.get(1)` on a collected `Vec`, or `.nth(1)` on the iterator)
return an `Option`, for the same reason `read_to_string` returns a `Result`:
the world outside your program gets a vote. `eprintln!` works exactly like
`println!` but targets stderr.

</details>

<details><summary>Hint 2 — the pieces</summary>

The opening line, if you take the shape that scales to Sitting B:

```rust
let args: Vec<String> = std::env::args().collect();
```

Then `args.get(1)` gives you an `Option<&String>` to `match` on. In the `None`
arm: one `eprintln!`, then leave `main` with `return Ok(());` (yes, `Ok` — the
exit-code lie move 3 told you to tolerate until Sitting B). In the `Some` arm:
the path. After that it's ramp 7 verbatim: `read_to_string(...)?`, then
`.lines()` chained into the counting adapter from step 7's move 6.

</details>

<details><summary>Hint 3 — the skeleton</summary>

```rust
#![forbid(unsafe_code)]

fn main() -> Result<(), std::io::Error> {
    // 1. collect args; match on .get(1):
    //      None      -> usage to stderr, return early
    //      Some(...) -> the path
    // 2. read_to_string with `?`  (lend the path if you print it later)
    // 3. println! the .lines() count
    Ok(())
}
```

Fill each comment with the one or two lines it describes — every line in it is
something you already wrote once in ramp 3, 5, 6, or 7.

</details>

## If truly stuck

Read function **`main`** in
`specs/003-rust-bedrock/_reference/glake/src/main.rs` — **only the
args-handling at the top of it**. Everything below (two commands, `ExitCode`,
feature gates, `validate`/`stats`) is Sittings B–F material and reading it now
spoils those sittings. The reference `main` is the *final* form, so yours won't
match it today — it shouldn't. Take the shape of the args handling, close the
file, and write your version from memory.
