# Sitting C — the scanner, test-first

**Builds:** the heart of glake — `has_key` / `get_str`, a hand-rolled scanner
that answers questions about one JSON line by handing back **borrowed slices**
(zero copies), with its panic-proof property test written *before* the scanner
exists.
**Requirements:** R8 (the property), R5 (blank-line skip) — tasks 1.3–1.4.
**Ramp you'll use:** step 8 (lifetimes-lite — recapped as the opening move),
with steps 3 (borrowing), 5 (enum + `match`), and 7 (line counting) in
supporting roles.

## Where you are

Sitting B gave glake commands, a walk, and manners: it finds every `.jsonl`
file and fails with clean exit codes instead of panics. But it still can't read
a *line* — it counts them without knowing what any of them says. The design
calls this sitting's function "the heart" because everything downstream
(classify in D, tallies in E) is a thin layer over the two questions you build
today: *does this line have this key?* and *what's its string value?* — and
because we banned `serde`, the answers come from a scanner you write yourself.
New discipline this sitting: the test comes first. You will commit a **failing**
test on purpose, and that commit is the point.

## The build, move by move

All commands from the repo root. Two commits this sitting: one red, one green —
that rhythm (fail first, then earn the pass) is how every property in this
course lands, here and again in Sitting E.

1. **Recap ramp 8 before touching anything.** Re-read your
   `playground/ramp/step-08-lifetimes-lite/` solution and answer these aloud:
   why did `longer(a, b)` refuse to compile without `<'a>` while a
   one-argument function didn't need it? What exactly does `'a` *promise*, and
   what does it cost at runtime? Now look at the design's frozen signature:

   ```rust
   pub fn get_str<'a>(line: &'a str, key: &str) -> Option<&'a str>
   ```

   Before moving on, say why the `<'a>` is on `line` but **not** on `key` —
   what would it mean, and what would it wrongly forbid, if `key` shared `'a`
   too? This signature is `longer()`'s exam, and the whole zero-copy story
   (the slice you return *is* the line's own bytes — nothing allocated, which
   the lens will show you in Sitting F) hangs on it. That story has a name in
   the requirements: **L2**, `&str` vs `String` — borrowed slices where a
   lesser design would allocate owned copies. This sitting is where it lands.

2. **Freeze the stubs.** Create `crates/glake/src/scan.rs`, add `pub mod scan;`
   to `src/lib.rs` (next to `walk`), and write exactly the two public
   signatures the design froze — with `todo!()` bodies:

   ```rust
   pub fn get_str<'a>(line: &'a str, key: &str) -> Option<&'a str> {
       todo!()
   }

   pub fn has_key(line: &str, key: &str) -> bool {
       todo!()
   }
   ```

   Why stubs first: the property test needs something to *call* so it can
   compile — `todo!()` is Rust's honest placeholder, a panic that says "not
   written yet". Note `has_key` carries no `<'a>` at all: it returns a `bool`,
   which borrows nothing. Check it compiles: `cargo build -p glake`.

3. **Invite proptest — as a dev-dependency.** Open `crates/glake/Cargo.toml`
   and add:

   ```toml
   [dev-dependencies]
   proptest = "1"
   ```

   The section name is the lesson: `[dev-dependencies]` exist only while
   testing — they never enter the shipped binary. That's why this doesn't
   violate R4 ("std-only dependency tree"): R4 governs what glake *ships*, and
   Sitting F's `cargo tree` check will confirm proptest isn't in it. Run
   `cargo build -p glake` once so the workspace `Cargo.lock` picks it up.

4. **Co-write the R8 property — and watch it fail.** This is your first
   property test, so per the coached-mode rules we write it *together*: you
   drive the keyboard, Claude navigates the proptest API. Create
   `crates/glake/tests/prop_scan.rs` (an integration test — that's why the
   functions are `pub` and live in the library). What the co-write must
   produce, per the design's Properties table:

   - **A strategy for the input line**, mixing two halves with `prop_oneof!`:
     half `any::<String>()` — pure garbage, multibyte included — and half
     "JSON-ish": a small generator that assembles `{"key":value,…}` lines from
     a few random keys and values (string, number, one nested object, one
     array), optionally truncated mid-line at a char boundary. **Sitting C
     uses the simple version**: leave the escape-bearing fragments (`\"`,
     `\\` inside keys and values) out for now, and mark the spot with a
     `// Sitting D: escape fragments land here` comment — task 1.5 turns the
     strategy up to full and your scanner will have to survive it.
   - **The no-panic + in-bounds property**: for any line and any key,
     `has_key` returns without panicking, and if `get_str` returns a slice,
     that slice *physically lives inside the input* — a `&str` knows its
     address (`as_ptr()`) and its length, so you can assert the returned
     span sits within the input's span with two `prop_assert!` lines. That's
     R8's "never reads past the end" made mechanical.
   - **The round-trip property**: build a well-formed line yourself around a
     generated value (no escapes), then assert `get_str` finds *exactly* the
     value you planted. Without this half, a scanner that always returns
     `None` would pass — no-panic alone is too easy.
   - House conventions: name the tests after the REQ (`r8_…`), and set cases
     above the 256 floor — `#![proptest_config(ProptestConfig::with_cases(512))]`.

   Now run it and enjoy the failure:

   ```console
   cargo test -p glake --test prop_scan
   ```

   Both tests panic — `todo!()` — and watch what proptest does next: it
   **shrinks**, hunting for the smallest input that still fails, and reports
   something like `line = ""`. It also writes a seed file under
   `crates/glake/proptest-regressions/` — commit it (house rule: seeds are
   committed whenever a failure occurred; green runs will replay this seed
   first, forever). Then the ritual — `cargo fmt` and

   ```console
   cargo clippy -p glake --all-targets -- -D warnings
   ```

   (`--all-targets` is new: without it, clippy never reads `tests/`. Expect a
   fight here — see below.) **Commit point — the red commit**, the only kind
   of commit in this course that's *supposed* to have failing tests:

   ```console
   git add crates/glake Cargo.lock && git commit -m "003: sitting C — R8 property, red"
   ```

5. **Promise the right answers too — the correctness table.** R8 proves your
   scanner *never panics*; it says nothing about being *right* (a scanner
   returning `None` for everything passes the no-panic half). So before
   implementing, write the answer sheet: a `#[cfg(test)] mod tests` at the
   bottom of `scan.rs` with ordinary `#[test]`s over known lines. Rows to
   cover — write the assertions now, while they're all red:

   - a realistic event line: `get_str(line, "event_type")` returns exactly the
     value you wrote into it;
   - a key whose value is a *number*: `has_key` says true, `get_str` says
     `None` (present, but not a string — R8's "wrong type" answer);
   - a key that appears **only inside the `payload` object**: both functions
     must say *no* — the design's scanner is top-level only, and this row is
     the one that enforces it;
   - a key that isn't there at all;
   - a short list of garbage lines (`""`, `"{"`, a lone quote, `"🦀🦀🦀"`)
     where you only assert "calling this doesn't explode".

   Deliberately absent: any row with escaped quotes (`\"`) inside a value.
   That's Sitting D's opening move — today's scanner is allowed to be naive
   about escapes, and your table must not promise otherwise.

6. **Think before you type.** The scanner is yours alone (the co-writing was
   for the test). Settle these on paper first — each one is a trap the
   property will find if you guess wrong:

   - You'll walk the line one position at a time. What *state* do you need to
     carry? Hint: strings are the complication — a `{` inside `"…"` is text,
     not structure. Cheapest plan: when you meet a quote, find the string's
     end and jump past it whole, so the main loop never looks inside one.
   - Two kinds of string will pass under your scanner: **keys** and
     **values**. `{"a":"b"}` has one of each and they look identical. What
     distinguishes a key, mechanically, from where you're standing? And what
     stops `"actor"` inside `{"payload":{"actor":…}}` from matching — how do
     you know how *deep* you are? (Your table's payload row is waiting for
     this answer.)
   - If you count depth, what happens when garbage hands you `}` as the very
     first character? What must your counter's *type* let it do that `usize`
     won't? The property's garbage half will put exactly this line in front
     of you.
   - Bytes or chars? `line.as_bytes()` gives you `u8`s and byte indices —
     fast, but you may only slice `&line[a..b]` at char boundaries, so every
     index you keep must sit next to an ASCII delimiter (a UTF-8 guarantee
     worth saying aloud). `line.char_indices()` sidesteps that — but
     `.chars().enumerate()` does *not* (it counts chars, not bytes, and
     slicing with those numbers is the classic 🦀 panic). Pick one and know
     why.

7. **Write the flat scanner, and let the property drive.** Implement in
   `scan.rs` until everything passes. The design's shape (and the reason both
   publics can't disagree): one shared private walk that both wrap — see Hint
   2 when you want its signature. Work in the red-green rhythm:

   ```console
   cargo test -p glake --test prop_scan   # property
   cargo test -p glake                    # property + table + Sitting B's tests
   ```

   Every failure proptest finds arrives *shrunk* — a two-character line
   instead of a forty-character one — and leaves a seed in
   `proptest-regressions/`. Keep the seeds, log the fights (each is a
   `learning.*` ledger entry), fix, rerun. When both properties, the whole
   table, and B's suite are green, the scanner is done — resist the urge to
   handle escapes "while you're in there"; D exists for a reason.

8. **Blank lines stop counting (R5).** Sitting A left you a parked question:
   `.lines().count()` counts blank lines — is a blank line an event? R5 says
   no: *skipped, not counted, not flagged*. Plant the evidence first: add one
   empty line and one whitespace-only line (spaces + a tab) into
   `crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl`. Then make
   the placeholder count in `main.rs` skip them — you already wrote this
   exact filter in ramp step 7; the question is only "what does
   whitespace-only mean", and `str::trim` answers it. Prove it mechanically:
   one new test in `tests/cli.rs` (same `CARGO_BIN_EXE_glake` pattern as B)
   that runs `stats` on that fixture file and asserts the printed count is
   the non-blank line count. (The *durable* home for this decision is
   Sitting D's `Line::Blank` arm — today's filter is the down payment.)

9. **Gate and close green.** The full ritual, then the second commit:

   ```console
   cargo fmt
   cargo clippy -p glake --all-targets -- -D warnings
   cargo test -p glake
   cargo run -p glake -- stats datalake/raw-local   # still runs clean on the real lake
   ```

   **Commit point — the green commit:**

   ```console
   git add crates/glake && git commit -m "003: sitting C — flat scanner green"
   ```

## Compiler fights to expect

Same deal as A and B: every fight is curriculum, logged to the mistake ledger
(`learning.*`) as SKILLS evidence. This sitting adds a twist — half the fights
below aren't compiler errors at all, but **runtime panics the property test
will catch for you**. That's the R8 payoff: proptest is a sparring partner
that punches exactly where you're weakest, then shrinks the punch to its
smallest reproducible form.

- **`error[E0106]: missing lifetime specifier`** — if you write the helper (or
  retype `get_str`) as `fn …(line: &str, key: &str) -> Option<&str>`. Ramp 8's
  `longer()` verbatim: two input references, one borrowed output — elision has
  no rule, so *you* must say which input the output borrows from. The `<'a>`
  on `line` alone is the answer *and* a promise: the result lives exactly as
  long as the line, and the compiler will hold every caller to it.
- **`warning: unused variable: `line`` under `-D warnings`** — at the red
  commit, because `todo!()` bodies never touch their parameters, and our
  clippy gate denies warnings. The idiomatic red-phase move: underscore the
  stub's parameter names (`_line`, `_key`) and drop the underscores when you
  implement. A lint, not an error code — but the ledger takes lints too.
- **`error[E0308]: mismatched types — expected `u8`, found `char`**` — if you
  scan `as_bytes()` but compare against `'"'`. Bytes and chars are different
  types with different literals; the byte-sized quote is `b'"'`. (If you chose
  `char_indices` instead, you'll meet the mirror image comparing to `b'"'`.)
- **`error[E0515]: cannot return value referencing local variable`** — if you
  build a cleaned-up `String` inside the scanner and try to return a `&str`
  view of it. The local dies at the closing brace; a borrow of it can't leave.
  This error is the design talking: the *whole point* (L2) is returning
  borrowed `&str` slices of the caller's line, not owned `String` copies you
  manufactured.
- **Runtime panic: `attempt to subtract with overflow`** — the property's
  garbage half will feed you a line starting with `}`, and if your depth
  counter can't go below zero, debug builds abort. You answered this in move
  6; if you skipped it, proptest didn't.
- **Runtime panic: `byte index N is not a char boundary`** — slicing
  `&line[a..b]` with indices that landed inside a multibyte char. If you kept
  the move-6 discipline (indices only ever adjacent to ASCII delimiters) this
  never fires; if it does, the shrunk counterexample will contain a 🦀, and
  the seed file that catches it is worth committing forever.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                  # no diff
cargo clippy -p glake --all-targets -- -D warnings # clean (tests included)
cargo test -p glake                                # ALL green: both r8_* properties,
                                                   # the correctness table, R5 CLI test,
                                                   # and Sitting B's suite still passing
PROPTEST_CASES=2000 cargo test -p glake --test prop_scan
                                                   # confidence run: still green at ~4x cases
```

```
cargo run -p glake -- stats crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl
# → printed line count is LESS than
wc -l < crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl
# by exactly the number of blank/whitespace lines you planted (R5)

cargo run -p glake -- stats datalake/raw-local
# → still runs clean on the real lake
```

- `git log --oneline -2` shows `003: sitting C — flat scanner green` sitting
  *above* `003: sitting C — R8 property, red` — the rhythm, preserved in
  history.
- If proptest ever failed along the way, `crates/glake/proptest-regressions/`
  is committed, not gitignored.
- You can answer aloud: why does `get_str` need `<'a>` and `has_key` doesn't?
  How many heap allocations does scanning one line cost — and how do you
  *know*? (Sitting F's lens will check your answer.) And which sitting makes
  `\"` your problem?

## Hints (one at a time)

<details><summary>Hint 1 — a nudge: what the walk carries</summary>

Treat strings as opaque tokens: the moment you see a quote, find where that
string *ends* and jump past it whole — then the main loop only ever sees
structure, never string innards. With that plan, the state you carry shrinks
to two things: your position, and how deeply nested you currently are (opening
brackets go down a level, closing ones come back up). A key is then just "a
string at the right depth whose next non-space character is a colon" — and
each of those three conditions kills one wrong match: the depth check kills
`payload`'s insides, the colon check kills value strings, and the comparison
kills every other key.

</details>

<details><summary>Hint 2 — the shape: one walk, two wrappers</summary>

Both public functions are asking the same question with different endings, so
give them one shared private walk they can't disagree through:

```rust
enum Value<'a> {
    Str(&'a str),
    Other,
}

fn find_value<'a>(line: &'a str, key: &str) -> Option<Value<'a>>
```

`has_key` is then one line (`is_some()`), and `get_str` is a two-arm `match`
(ramp 5). Inside the walk, a second tiny helper keeps the string-skipping
honest:

```rust
fn string_end(bytes: &[u8], start: usize) -> Option<usize>
```

— given the position just past an opening quote, return the closing quote's
index; `None` means the string never closes (truncated line), and `?` on that
`Option` lets the whole walk give up cleanly — R8's "not found" instead of a
crash. Today's `string_end` is one honest loop looking for `b'"'`. Sitting D
will make it wiser without changing its signature.

</details>

<details><summary>Hint 3 — the skeleton, as comments</summary>

```rust
fn find_value<'a>(line: &'a str, key: &str) -> Option<Value<'a>> {
    // bytes + position + signed depth
    // loop over positions, match on the byte:
    //   opening brace/bracket  -> deeper, step on
    //   closing brace/bracket  -> shallower, step on
    //   quote -> a string starts just past it; string_end(...)? finds its close
    //            key test: right depth AND next non-space byte after it is b':'
    //            if it's a key and the slice between the quotes == key:
    //                skip the colon and any spaces;
    //                next byte a quote? -> the value string's bounds via
    //                    string_end again -> Some(Value::Str(that slice))
    //                anything else -> Some(Value::Other)
    //            either way, not our key: jump to just past the string's close
    //   anything else -> step on
    // fell off the end -> None
}
```

Every index you keep here is adjacent to an ASCII byte (`"`, `:`), which is
why the slices can't hit a char boundary. Fill the comments in; the property
and the table will referee.

</details>

## If truly stuck

Read, don't copy — then close it and write yours:

- `specs/003-rust-bedrock/_reference/glake/src/scan.rs` — function
  `find_value`. One warning: the `string_end` next to it already contains
  Sitting D's escape hardening — yours today is the simpler flat version.
- `specs/003-rust-bedrock/_reference/glake/tests/prop_scan.rs` — functions
  `r8_never_panics_and_slices_stay_in_bounds` and `r8_wellformed_roundtrip`
  (and note its `json_ish` strategy is the *full* Sitting-D version — escapes
  included — so yours is allowed to be smaller today).
