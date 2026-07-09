# Sitting D — harden the scanner, then judge every line

**Builds:** the scanner survives escaped quotes, backslashes, and multibyte text
under the *full* R8 strategy; then every line becomes a `Line<'a>` (the design's
stretch lesson), and `glake validate` earns its first real body — `file:line
missing key "…"`, exit 1 iff anything was malformed.
**Requirements:** R1a, R1b (tasks 1.5–1.6) — plus R8 turned up to full strength
and R5 finally given a type of its own.
**Ramp you'll use:** steps 5 (`match`, enums) and 8 (lifetimes-lite — this is
the sitting it was for), with step 4's `&str`-vs-`String` instincts everywhere.

## Where you are

Sitting C left you with the heart beating: `get_str<'a>`/`has_key` pass the R8
property on the *simple* strategy, your correctness table proves top-level-only
behavior, and the red-commit-then-green-commit rhythm is in your hands. But the
scanner is still flat — it believes every `"` it meets. This sitting breaks that
belief with the strategy the design always intended, then spends the trust you
earn: once the scanner is unbreakable, you can build `classify` and the first
command whose output a script could stake an exit code on.

## The build, move by move

All commands run from the repo root. No new dependencies this sitting —
`proptest` arrived in C, and the schema drift test is std-only *on purpose*
(R4's `cargo tree` audit in Sitting F must stay boring).

1. **Turn the R8 strategy up to full — and go red on purpose.** The design's
   Properties table promised the JSON-ish half of the generator would include
   "escapes `\"` `\\`" and truncations, and the garbage half already carries
   multibyte. Extend your generator in `crates/glake/tests/prop_scan.rs` to
   deliver on that: values and keys that contain `\"` and `\\`, a 🦀 or two,
   and lines truncated mid-thought (careful *where* you cut a `String` — the
   generator itself must not panic; there's a method that asks whether an index
   is safe to cut at).

   Same hour, add rows to the correctness table in `scan.rs` — these are the
   assertions the property can't make (R8 proves *no panic*, not *right
   answers*):

   - a value containing an escaped quote, e.g. built from
     `r#"{"msg":"he said \"hi\"","tag":"x"}"# ` — and here's the question you
     must answer before you can even write the assertion: should
     `get_str(line, "msg")` return the text with the `\"` still in it, or
     unescaped? Think about what unescaping would force your *return type* to
     become, and what that costs. Your answer **is** the assertion; defend it
     in a `///` comment on `get_str`.
   - a value that *ends* in a backslash: `r#"{"path":"C:\\","x":"y"}"# `. The
     discriminating assertion is not about `path` — it's that `"x"` **must
     still be found**. If your escape logic misreads `\\"`, the string never
     closes and `x` vanishes.
   - a multibyte value (`🦀` inside), asserting the exact slice comes back.

   Write the rows with raw strings (`r#"…"#`) so the backslashes on your screen
   are the bytes in the data — half of all escape-test confusion is the *test
   file's own* escaping.

   ```console
   cargo test -p glake
   ```

   Red — the escaped-quote rows must fail against the flat scanner. If
   everything passes, don't celebrate: your generator or your rows aren't
   actually producing escapes. Prove it with a deliberately wrong assertion,
   then fix the generator. Same rhythm as Sitting C — the history proves
   test-first (`cargo fmt` + `cargo clippy -p glake -- -D warnings` still gate
   every commit, red or not). **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting D — R8 strategy to full + escape rows, red"
   ```

2. **Harden the scanner until it's green again.** One new fact of JSON is all
   this takes: *inside a string, a backslash makes the next character literal.*
   Where you scan for a closing quote, you now need a tiny bit of state that
   remembers "the character I'm looking at was escaped." Questions to settle
   before typing:

   - The `"C:\\"` row from move 1: after two backslashes, is the next quote
     escaped or closing? Whatever state you keep must get this right — a check
     like "was the *previous* byte a backslash?" reads plausibly and fails
     exactly here. (Think about what the state should *do to itself* after it
     fires.)
   - When your scanner meets a `"`, it decides key-or-value. What two
     conditions make it a key? (Where in the *nesting* is it, and what's the
     next meaningful character after it?) Your C-era top-level rows pinned
     this; re-read them and make sure the hardened rewrite still honors both
     conditions.
   - Your scanner slices `&line[start..end]`. With 🦀 in the line, why is
     slicing at a quote's position always safe — *if* your positions are byte
     positions? Can any byte of a multibyte UTF-8 character ever equal `b'"'`?
     Look up how UTF-8 marks continuation bytes; the answer is the reason the
     `as_bytes()` approach is sound and a `chars().enumerate()` count is not.
   - An unterminated string (the truncation arm will make them) means the whole
     scan should give up cleanly. You already know the one-character operator
     that turns "no closing quote" into "key not found" — it works on `Option`
     too.

   Iterate `cargo test -p glake` until property and table are both green. If
   the property itself caught a panic along the way, proptest wrote a seed file
   into a `proptest-regressions/` directory beside the test — **commit it**
   (CLAUDE.md rule) and classify the counterexample in
   `specs/003-rust-bedrock/_assurance/triage-log.md` before fixing: spec bug,
   code bug, or test bug. **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting D — scanner hardened, R8 full green"
   ```

3. **`REQUIRED_KEYS`, kept honest by a drift test.** Create
   `crates/glake/src/classify.rs` (declare it in `lib.rs`) and put the
   rev-3 design decision in it:

   ```rust
   pub const REQUIRED_KEYS: [&str; 7] = [ /* copy from the schema's required[] */ ];
   ```

   The seven names come from `datalake/schema/envelope.v1.json`. The tool
   never reads that file at runtime — a **test** guarantees the constant can't
   drift from it. Write `crates/glake/tests/schema_drift.rs`: read the schema
   with the path built from

   ```rust
   concat!(env!("CARGO_MANIFEST_DIR"), "/../../datalake/schema/envelope.v1.json")
   ```

   (your crate is two levels below the repo root — count them yourself, don't
   trust anyone's `../` chain), then extract the quoted names inside the
   `"required": [ … ]` array and assert they match the constant. Two questions
   shape the test:

   - Can you reuse your scanner for this? Check its contract from C:
     one line, top level, *string values*. What is `required`'s value, and how
     many lines does the pretty-printed schema put it on? Conclude, then write
     the few lines of std string-searching the test actually needs — test
     code is allowed to be plain.
   - If a future schema rev *reorders* the required list without changing the
     set, should this test fail? Decide, and make the code say your decision
     out loud.

   Then earn the test: a test you've never seen fail proves nothing. Delete
   one key from the constant, run `cargo test -p glake`, watch the drift test
   name the drift, restore it. (Poetic choice for the drill: `actor` — rev 1
   of the requirements famously forgot it.) **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting D — REQUIRED_KEYS drift-tested against the schema"
   ```

4. **`Line<'a>` — the stretch lesson.** Every line of the lake becomes exactly
   one of three things. The design froze the shape; aim at it precisely:

   ```rust
   pub enum Line<'a> {
       Blank,
       Malformed { missing: &'a str },
       Event { kind: &'a str, day: &'a str },
   }

   pub fn classify<'a>(line: &'a str, required: &[&'a str]) -> Line<'a>
   ```

   This is ramp 8 graduating: in C, one *function* promised "my return borrows
   from my input." Now a whole *type* carries that promise — a `Line<'a>` is
   proof, checked at compile time, that classifying a million lines allocates
   nothing. Sitting F's lens will show you that flat allocation line for real.

   The behavior: whitespace-only → `Blank` (R5 stops being an `if` scattered
   through `main` and becomes a value); any required key absent → `Malformed`
   naming it; otherwise `Event`. Decisions to make deliberately, not by
   accident:

   - `Malformed` carries **one** `&str`. A line could be missing three keys.
     What did the enum's shape just decide for you, and does R1a's worked
     example in `requirements.md` bless it?
   - Trace the lifetime: when you return `Malformed { missing }`, which input
     does `missing` actually borrow from — `line`, or `required`? Both are
     `'a` in the signature; know which one you're using.
   - `kind` wants `get_str(line, "event_type")` — but every required key being
     *present* (that's `has_key`) doesn't mean `get_str` returns `Some`. When
     can it be `None` here, and what `&str` will you put in `kind` then? It
     must be something a stats table can *display*, never a crash — and
     remember `clippy::unwrap_used` guards this crate.
   - `day` is the first ten characters of `ts`. Third time now: Sitting A's
     `args[1]`-vs-`.get(1)` choice, back on a string slice. One spelling is a
     promise that panics on legal-but-strange input; one is a question. Choose
     deliberately and write a one-line comment defending it — Sitting E's
     property generator includes timestamps built specifically to judge this,
     and the design's Key-decisions table has already ruled. You can peek, or
     you can let the counterexample find you (more instructive, and it feeds
     the triage log).

   Unit tests in a `#[cfg(test)]` mod: one known-good envelope line → the
   `Event` you expect; whitespace → `Blank`; and derive the broken case from
   the good one with `.replace(…)` — deleting the `actor` pair — asserting
   `Malformed { missing: "actor" }`. `assert_eq!` on your enum will demand two
   derives; let the compiler name them (and note them against the
   C-COMMON-TRAITS rubric from CLAUDE.md). **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting D — classify: every line becomes a Line<'a>"
   ```

5. **`validate` gets its real body — and fixtures to prove it.** First grow
   Sitting B's miniature lake: in
   `crates/glake/tests/fixtures/lake/dt=2026-07-02/events.jsonl`, add a blank
   line and a line missing one required key (take a real line and delete its
   `actor` pair by hand). Leave `dt=2026-07-01` clean — you need a known-good
   fixture too, and you're about to see why. Re-run `cargo test -p glake`
   right after: nothing from B should have silently depended on fixture line
   counts.

   Now replace the placeholder in `main.rs`'s `validate` arm: for each file
   from `jsonl_files`, read it, walk `content.lines()` with line numbers,
   classify each line (hand it `&REQUIRED_KEYS`), and report every `Malformed`
   as `file:line  missing key "…"` — then exit **1 iff at least one line was
   malformed** (R1b), keeping 2 meaning what B made it mean. Questions that
   *are* the design of this function:

   - Which stream do the reports belong on? The worked example in
     `requirements.md` shows a script-friendly answer — think about who pipes
     `validate`'s findings and why that settles stdout-vs-stderr.
   - `.enumerate()` starts at 0. The worked example reports `:41` the way an
     editor counts. Your fixture test should assert the *exact* line number of
     the planted bad line — that assertion is your off-by-one tripwire.
   - Will you collect all the `Line`s from all files and print at the end, or
     report as you go? Try the hoarding version if you're curious — the borrow
     checker will explain, in its own words, why a `Line<'a>` cannot outlive
     the file content it borrows from. Read that error properly (fights,
     below), then pick the architecture that doesn't fight the truth.
   - A file that fails to *read* mid-run is still R6 territory: one clear
     stderr line, exit 2, no panic — and B already taught you why `?` is not
     available in a `main` that returns `ExitCode`.

   Extend `crates/glake/tests/cli.rs` with the R1 pair — "exits non-zero
   *exactly when*" is two tests, one per direction:

   - `validate` on `tests/fixtures/lake` → stdout names the missing key *and*
     the right `file:line`; exit code `Some(1)`.
   - `validate` on the clean `dt=2026-07-01/events.jsonl` → exit `Some(0)`.

   ```console
   cargo test -p glake
   cargo fmt && cargo clippy -p glake -- -D warnings
   ```

   **Commit point:**

   ```console
   git add crates/glake && git commit -m "003: sitting D — validate reports file:line, exits 1 iff malformed (R1a/R1b)"
   ```

## Compiler fights to expect

Same deal as A and B: each fight is syllabus, and each one we hit gets logged
to the mistake ledger (`learning.*` events) as SKILLS evidence.

- **E0106 — missing lifetime specifier.** The moment you write
  `Malformed { missing: &str }` in the enum. Ramp 8's whole point arrives:
  a type that stores a borrow must *name* the borrow, because whoever holds a
  `Line` needs the compiler to know how long its insides live. Add `<'a>` and
  watch the requirement ripple to `classify`'s signature — that ripple is the
  design's frozen interface writing itself.
- **E0716 — temporary value dropped while borrowed.** In your classify tests,
  the shape `let l = classify(&good.replace(…, …), …);` builds a `String`,
  borrows a `Line` out of it, and drops the `String` at the semicolon. The
  `Line` is now a promise with nothing behind it, and the compiler refuses.
  Bind the replaced `String` to its own `let` first — naming a value is what
  gives it a lifetime worth borrowing against.
- **E0597 — borrowed value does not live long enough.** The hoarding version
  of `validate` from move 5: `Line`s collected into a `Vec` that outlives the
  per-file `content`. There is no lifetime annotation that fixes this — the
  *architecture* is wrong, not the syntax. Report as you go (or keep every
  file's content alive as long as its `Line`s). This error teaching you to
  restructure rather than annotate is the single most valuable fight in the
  sitting.
- **E0277 / E0369 — `Line<'_>` doesn't satisfy `Debug` / can't be compared.**
  `assert_eq!` needs to print and compare your enum. Two `#[derive(…)]`s fix
  it; the deeper lesson is that traits are *opt-in* — Rust won't guess that
  equality even makes sense for your type (C-COMMON-TRAITS says decide it
  deliberately for every public type).
- **E0603 — constant `REQUIRED_KEYS` is private.** The drift test lives in
  `tests/`, which is a *separate crate* — it sees only what `lib.rs` and your
  modules mark `pub`. Integration tests experiencing your library exactly as
  a stranger would is a feature, not friction.
- **A runtime panic, not a compiler error:** `byte index N is not a char
  boundary` under the full R8 strategy — the classic sign that positions from
  `chars().enumerate()` (character counts) were used to slice bytes. The
  property is doing precisely its job: commit the seed proptest wrote, triage
  it in the log (this one's a code bug), then move the scanner onto
  `as_bytes()` ground.
- **The fight the compiler won't pick:** escape logic that asks "was the
  previous byte a backslash?" compiles cleanly, passes the `\"` row, and fails
  only on `"C:\\"` — where the backslash is itself escaped, so the quote
  *does* close. No error code; only your move-1 correctness row stands between
  this bug and the lake. (This is why the row exists.)

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                          # no diff
cargo clippy -p glake -- -D warnings       # clean
cargo test -p glake                        # ALL green: R8 property at full strategy
                                           # (≥256 cases, the house default), escape +
                                           # multibyte correctness rows, schema drift
                                           # test, classify unit tests, both R1 CLI tests
```

```
cargo run -p glake -- validate crates/glake/tests/fixtures/lake; echo $?
# → the planted line reported as <path>:<line>  missing key "…" with the RIGHT
#   line number; the blank line NOT flagged (R5); a summary; exit 1

cargo run -p glake -- validate crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl; echo $?
# → clean report, exit 0  (R1b is "iff" — this direction counts as much as the other)

cargo run -p glake -- validate datalake/raw-local; echo $?
# → the REAL lake: every line well-formed, exit 0. Whether your total includes
#   the traces/ file your walk met in Sitting B depends on the dt=-recursion
#   decision you wrote down there — both answers validate clean today (even the
#   memlens trace dogfoods the envelope), and Sitting F's R10 cross-check will
#   force the question properly. Note the numbers drift: dt=2026-07-09 grows as
#   we work, because your own session's hooks are appending to it right now.
#   If validate ever flags a real lake line, don't assume you're wrong — read
#   the line. You may have just found the lake's first genuine defect, which is
#   the tool doing its job. Bring it to session either way.
```

- Drift drill re-armed: remove one key from `REQUIRED_KEYS`, `cargo test -p
  glake` fails naming the schema file, restore, green again.
- `git log --oneline -5` shows the five sitting-D commits, red before green.
- You can answer aloud: after the one `String` per file that reading costs,
  how many heap allocations did classifying every line in the real lake add —
  and which two characters in the `Line` type are the compile-time proof?

## Hints (one at a time)

<details>
<summary>Hint 1 — the escape logic won't come right</summary>

One boolean of state is enough: "is the character I'm looking at escaped?"
The discipline is in how it changes: seeing a backslash *while not escaped*
sets it; **every** character seen while escaped clears it — including a second
backslash. Walk `C : \ \ "` through your rule by hand, one byte at a time,
saying the flag's value out loud; the `\\` case works exactly when the flag
consumes itself. And do all of this over `line.as_bytes()` with byte indices —
`"` and `\` are ASCII, so they can never appear inside a multibyte character's
bytes, which is why slicing at those positions is always boundary-safe.

</details>

<details>
<summary>Hint 2 — the helper shape, and the drift test's extraction</summary>

Hardening gets much easier if "find the end of this string" becomes its own
function — something shaped like
`fn string_end(bytes: &[u8], start: usize) -> Option<usize>` (content starts
at `start`, returns the closing quote's index). Then every place a string
opens, you `?` it: an unterminated string cleanly becomes "nothing found,"
which is exactly what the truncation arm of the strategy demands.

For the drift test: `read_to_string` the whole schema, `find` the text
`"required"`, then `find` the `[` after it and the `]` after that. Inside
that slice, `split('"')` alternates unquoted/quoted pieces — the key names are
every second piece starting from the second. Collect both sides into `Vec`s,
sort them (or don't — but make it your documented decision), `assert_eq!` with
a message that tells future-you *which file to go edit*.

</details>

<details>
<summary>Hint 3 — the shape of classify and validate</summary>

```rust
// classify.rs
pub fn classify<'a>(line: &'a str, required: &[&'a str]) -> Line<'a> {
    // 1. trimmed-empty check       -> Blank
    // 2. loop over `required`; first key that fails has_key -> Malformed { missing: key }
    // 3. kind: get_str "event_type", with your chosen fallback &str
    //    day:  get_str "ts", then your chosen first-ten-chars spelling
    //    -> Event { kind, day }
}
```

```rust
// main.rs — the validate arm calls something like:
fn validate(files: &[std::path::PathBuf]) -> std::process::ExitCode {
    // counter of malformed lines
    // for each file:
    //     read_to_string, match: Err -> stderr line, exit 2 (R6 again)
    //     for (n, line) in content.lines().enumerate():
    //         if classify(...) is Malformed { missing } -> report with n+1, bump counter
    // counter == 0 -> success; else summary + exit 1
}
```

Every line of both skeletons is something you've already written once — in
this sitting's earlier moves, or in B's error boundary.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours from memory:

- `specs/003-rust-bedrock/_reference/glake/src/scan.rs` — `string_end`, and
  `find_value` for how the hardened walk uses it
- `specs/003-rust-bedrock/_reference/glake/src/classify.rs` — `classify` (and
  `REQUIRED_KEYS` just above it)
- `specs/003-rust-bedrock/_reference/glake/tests/schema_drift.rs` —
  `required_keys_match_schema_registry` — **but** the reference crate sits four
  levels deep, so its `../../../../` schema path is wrong for yours; take the
  idea, recount your own `..`s
- `specs/003-rust-bedrock/_reference/glake/src/main.rs` — `validate` only;
  `stats` below it is Sitting E's material and reading it now spoils E's
  red-then-green
