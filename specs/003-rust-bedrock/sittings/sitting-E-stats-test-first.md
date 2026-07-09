# Sitting E — stats, test-first

**Builds:** the real `stats` command — a pure tally fold (`HashMap::entry` over
classified lines) proven correct by the R9 conservation property *written
before the fold exists*, then output formatting until the numbers reach the
terminal looking like the requirements' worked example.
**Requirements:** R9 (the conservation property), R2 (stats output) — tasks
1.7–1.8.
**Ramp you'll use:** steps 2 (ownership & move), 4 (`&str` vs `String`), 5
(enum + `match`). `HashMap` and iterator chains are **new here** (L5) — this
sitting introduces them (they join SKILLS as 1b items at spec close).

## Where you are

Sitting D finished the reading side: the scanner survives escapes, `classify`
turns every line into one of three honest kinds (`Blank` / `Malformed` /
`Event`), and `validate` is a complete, exit-code-correct command. But `stats`
is still Sitting A's placeholder — it counts non-blank lines and knows nothing
about *what* it counted. Today it grows up: every valid event lands in a
by-type bucket and a by-day bucket, and a property you write first guarantees
that no event is ever dropped or counted twice on either axis. Second time
through the red-then-green rhythm — this time you steer more of the co-write.

## The build, move by move

All commands from the repo root. Two commits this sitting, same shape as C:
red first, green second.

1. **Say R9 in one breath before touching code.** "The by-type totals and the
   by-day totals *each* sum to the grand total of valid events." That's a
   conservation law: every event goes into exactly one kind-bucket and exactly
   one day-bucket — nothing leaks, nothing doubles. Now the question that
   makes it a *property* and not three example tests: an event's day is
   supposed to come from its `ts`. Predict, out loud, what happens to your
   conservation law for each of these three timestamps —
   `"2026-07-09T01:02:03Z"`, `"2026"`, `"🦀🦀🦀🦀"` — if the code takes "the
   first ten bytes of ts" *literally*, with `&ts[0..10]`. All three are legal
   input under keys-present validation (D taught you that). When you have a
   prediction for each, check it against the design's Key-decisions table
   ("day extraction" row) — but don't look up the fix yet; the property you're
   about to write will hunt it down for you.

2. **Freeze the stub.** Create `crates/glake/src/tally.rs`, add `pub mod
   tally;` to `src/lib.rs` (next to `classify`, `scan`, `walk`). The design
   wants the counting to be a **pure fold over lines** — no filesystem, no
   printing — precisely so the property can hammer it ten thousand times
   without touching disk. Aim at:

   ```rust
   pub struct Stats {
       // by_kind:   counts per event_type      (which key type? — move 5)
       // by_day:    counts per day             (same question)
       // events:    grand total of VALID events — R9's conserved quantity
       // malformed: seen and counted, never silently dropped
   }

   pub fn tally(lines: &[&str]) -> Stats {
       todo!()
   }
   ```

   A borrowed slice of borrowed lines — say aloud why neither `&` costs an
   allocation (ramp 3/4). (The reference generalizes this signature with
   `impl IntoIterator`; that's a generic in disguise and 004's territory —
   the slice is the honest v0.) Why does `Stats` carry a `malformed` counter
   when R2 never asks for one? Because R9's generator will produce malformed
   lines, and the property must prove they were *seen and set aside*, not
   lost — "nothing dropped" has to be checkable. Underscore the stub's unused
   parameter (`_lines`) so the red phase survives clippy, like C taught.
   Check: `cargo build -p glake`.

3. **Proptest is already invited.** Sitting C added it; verify by eye that
   `crates/glake/Cargo.toml` still carries

   ```toml
   [dev-dependencies]
   proptest = "1"
   ```

   Nothing to add today — one `[dev-dependencies]` line serves every property
   in the crate, and (same argument as C) it never enters the shipped binary,
   so R4's std-only tree is untouched.

4. **Co-write the R9 property — and watch it fail.** Create
   `crates/glake/tests/prop_tally.rs`. Like C, this is a co-write — you drive,
   Claude navigates — but you've seen the proptest API once now, so you write
   the strategies and Claude only steers. What the co-write must produce, per
   the design's Properties table ("line-set generator emits tagged expected
   counts alongside, so the test knows truth independently"):

   - **A `GenLine` enum, in the test file** — `Blank`, `Malformed`, and
     `Event` carrying a kind, a ts, *and the day the tool must report for
     it*. That third field is the whole trick: the generator **tags each line
     with its own expected answer at construction time**, so the test's truth
     never comes from the code under test.
   - **Strategies.** Kinds: a couple of realistic literals
     (`gate.approved`, `spec.doc_written`) plus a regex strategy like
     `"[a-z]{1,8}\\.[a-z]{1,8}"` for variety. Timestamps, three arms — and
     each arm *knows its own day by fiat*: a well-formed arm that builds the
     ts **from** a generated `yyyy-mm-dd` it keeps as the expected day; a
     deliberately short arm (`"2026"`); a multibyte arm (`"🦀🦀🦀🦀"`). For
     those last two, what day *should* the tool report? R9 forbids dropping
     the event, so it must land somewhere *visible* — the design named that
     bucket in the row you read in move 1. Tag them with it. Mix the three
     line kinds with `prop_oneof!` weights so events dominate (blanks and
     malformed are seasoning), and generate whole line-*sets* with
     `prop::collection::vec(gen_line(), 0..60)`.
   - **A `render` helper** — turns a `GenLine` into the actual JSONL text:
     whitespace for blanks; for events, a line carrying all seven
     `REQUIRED_KEYS` with the generated kind and ts spliced in; for
     malformed, the same line with one required key removed (`actor` has
     history here — requirements audit trail).
   - **The test itself**: fold over the `GenLine` tags to build
     `want_by_kind`, `want_by_day`, `want_events`, `want_malformed` — truth,
     computed without calling glake. Then render every line, call `tally`,
     and `prop_assert_eq!` all four against `Stats`. Finish with the law
     itself: assert `by_kind.values().sum()` **and** `by_day.values().sum()`
     both equal the grand total — both axes, per R9.
   - House conventions: test named `r9_…`; cases above the 256 floor —
     `#![proptest_config(ProptestConfig::with_cases(512))]`.

   Now run it and enjoy the failure:

   ```console
   cargo test -p glake --test prop_tally
   ```

   `todo!()` panics, proptest shrinks (watch it drive the line-set toward
   empty), and a seed lands in `crates/glake/proptest-regressions/` — commit
   it, house rule. Then the ritual — `cargo fmt` and

   ```console
   cargo clippy -p glake --all-targets -- -D warnings
   ```

   **Commit point — the red commit:**

   ```console
   git add crates/glake && git commit -m "003: sitting E — R9 property, red"
   ```

5. **Think before you type.** The fold is yours alone (the co-writing was for
   the test). Settle these on paper first:

   - **The key-type decision, this sitting's heart (ramp 4, lessons L1 + L2).**
     `classify` hands you `kind: &'a str` and `day: &'a str` — borrowed from
     the line. Do your maps key on `&str` (zero-copy, the design's instinct)
     or on owned `String`? Work out what each costs and what each *forces*:
     borrowed keys mean `Stats` itself borrows — the struct grows a lifetime
     parameter, and something in `main` must keep every file's contents alive
     for as long as the maps exist. Owned keys mean a conscious allocation
     per *distinct-ish* insertion. Either choice can go green; the gate is
     being able to defend yours.
   - **One event, two maps.** The same event updates `by_kind` *and*
     `by_day`. If your keys are owned, can one `String` serve both maps?
     What does ramp 2 say happens to it after the first insertion?
   - **The counting idiom (L5).** Look up `HashMap::entry` in the std docs —
     specifically what `entry(k).or_insert(0)` *returns*. It's not a number.
     What do you have to do to it before `+= 1` works?
   - **Your move-1 prediction.** Wherever the day extraction lives after
     Sitting D (`classify`'s `Event` arm), the property's short and multibyte
     arms are about to judge it. Which `str` method answers "give me bytes
     0..10" with an `Option` instead of a panic?

6. **Implement the fold until R9 goes green.** In `tally.rs`: one `match` on
   `classify(line, &REQUIRED_KEYS)` with three arms (ramp 5) — `Blank` does nothing,
   `Malformed` bumps one counter, `Event` bumps three things. Work in the
   rhythm:

   ```console
   cargo test -p glake --test prop_tally   # the property
   cargo test -p glake                     # plus D's suite, C's scanner, the tables
   ```

   Every failure arrives shrunk — expect the 🦀 arm to be the last one
   standing if move 1's prediction was "it's fine". Fix where the *design*
   says the fix lives (the decision row you read — one method call, one
   visible bucket), keep every seed, log every fight to the ledger.

7. **Wire the real `stats` command — output formatting (R2).** Replace the
   placeholder in `main.rs`: read each file (a read that fails mid-walk gets
   B's manners — clear stderr, exit 2, no panic), collect every line, hand
   them to `tally`, then print. The requirements' worked example is the
   target look:

   ```
   3 files · 214 events

   by type
     spec.doc_written      86
     ...

   by day
     2026-07-05           214
   ```

   Two things to figure out, both new:
   - **Alignment** — `format!`'s width specifiers (`{name:<24}` pads right,
     `{n:>6}` pads left). Look them up in the `std::fmt` docs; this is the
     lesson where `println!` stops being magic.
   - **Order.** Run `stats` twice on the same input. Is the output identical?
     Why not — what did you learn about `HashMap` just now, and what's the
     cheapest way to make the printout deterministic before a test asserts
     on it? (Highest-count-first with an alphabetical tiebreak reads best;
     any *documented* total order passes.)

   Consider also printing the malformed count when it's non-zero ("N
   malformed line(s) — run validate") — honest output, and it advertises the
   other command.

8. **The R2 fixture test.** Add one test to `crates/glake/tests/cli.rs`
   (same `CARGO_BIN_EXE_glake` + fixture-path pattern as B): run `stats` on
   `tests/fixtures/lake`, and assert against truth you compute **by hand** —
   open your fixture files and count: valid events, events per type, events
   per day, remembering C's planted blanks and D's malformed line count
   nowhere. Days come from each line's `ts` — and B move 3 made you align
   every fixture line's `ts` with its partition, so the two days the output
   must show are `2026-07-01` and `2026-07-02`. Assert the grand-total line,
   at least one by-type line, both of those days **as bare days** (that's how
   the output prints them — no `dt=` prefix), and exit code 0. If a day
   assertion fails, check the fixture lines' `ts` values before suspecting
   the fold. B's advice still stands: assert with `contains` on the
   load-bearing fragments, not whole-output equality — your formatting may
   still evolve.

9. **Gate and close green.** The full ritual:

   ```console
   cargo fmt
   cargo clippy -p glake --all-targets -- -D warnings
   cargo test -p glake
   cargo run -p glake -- stats datalake/raw-local   # the real lake, real numbers
   ```

   **Commit point — the green commit:**

   ```console
   git add crates/glake && git commit -m "003: sitting E — stats green"
   ```

## Compiler fights to expect

Same contract as C: every fight is curriculum, logged to the mistake ledger
(`learning.*`) as SKILLS evidence — and again the best one isn't a compile
error but a runtime panic the property will shrink for you.

- **`error[E0308]: mismatched types — expected `String`, found `&str`**` (or
  its mirror) — at `.entry(kind)`, the instant your key-type decision from
  move 5 meets reality. If you chose owned keys, the fix is a *conscious*
  allocation (`to_owned()`) — L2 in one line: you can now point at the exact
  place stats pays for ownership and say why. The same error reappears in the
  test if your `want_*` truth maps' key type doesn't match `Stats`'s — make
  the test agree with your decision.
- **`error[E0382]: use of moved value: `key`** — you built one owned `String`
  and fed it to `by_kind`, then reached for it again for `by_day`. Ramp 2,
  verbatim: insertion *moves* the key into the map. Each map needs its own
  copy — and noticing that "one event costs two owned keys" is exactly the
  observation Sitting F's lens will let you verify.
- **`error[E0106]: missing lifetime specifier`** — if you chose borrowed keys:
  a struct holding `HashMap<&str, u64>` must declare whose lifetime that is
  (`Stats<'a>`), and then **`error[E0597]: `content` does not live long
  enough`** follows in `main` if a file's `String` drops while the maps still
  borrow from it. This pair *is* the borrowed-vs-owned trade-off (L1), spoken
  in compiler; wrestle it honestly before deciding whether to switch camps.
- **`error[E0368]: binary assignment operation `+=` cannot be applied to type
  `&mut u64`** — `entry(k).or_insert(0)` hands back a mutable *reference* to
  the counter, not the counter. One `*` fixes it; understanding why (you're
  writing through the reference into the map's own storage — no lookup twice,
  no copy out) is the entire Entry-API lesson.
- **`error[E0282]: type annotations needed`** — `.values().sum()` in the
  conservation assertion. `sum` can produce many numeric types and the
  compiler won't guess; annotate or turbofish (`sum::<u64>()`).
- **Runtime panic, property-caught: `byte index 10 is not a char boundary`**
  (the 🦀 arm) and its sibling **`byte index 10 is out of bounds`** (the
  `"2026"` arm) — if the day extraction still slices `ts` with `[0..10]`.
  Proptest will shrink the whole line-set down to the one poisoned timestamp
  and leave a seed. This is your move-1 prediction settling its account, and
  the seed file is the design's MAJOR-3 audit finding made permanent — commit
  it.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                  # no diff
cargo clippy -p glake --all-targets -- -D warnings # clean (tests included)
cargo test -p glake                                # ALL green: r9_*, the r8_* suite,
                                                   # both correctness tables, the full
                                                   # cli.rs suite incl. the new R2 test
PROPTEST_CASES=2000 cargo test -p glake --test prop_tally
                                                   # confidence run: still green at ~4x cases
```

```
cargo run -p glake -- stats crates/glake/tests/fixtures/lake
# → grand total, by-type block, by-day block; the numbers match your
#   hand-count of the fixture files (blanks nowhere, malformed not an event)

cargo run -p glake -- stats datalake/raw-local
cargo run -p glake -- stats datalake/raw-local
# → run twice: byte-identical output (deterministic ordering), and adding
#   the by-type column by hand equals the grand total — R9, eyeballed on
#   real data. (The formal scan.sh cross-check is Sitting F's R10.)
```

- `git log --oneline -2` shows `003: sitting E — stats green` above
  `003: sitting E — R9 property, red` — the rhythm, preserved in history a
  second time.
- If proptest ever failed along the way, `crates/glake/proptest-regressions/`
  is committed, not gitignored.
- You can answer aloud: what does the `bad-ts` bucket protect — what would R9
  report if short-ts events were silently skipped instead? Why do your maps'
  keys cost what they cost (two allocations per event, or a lifetime
  parameter — whichever you chose, defend it)? And how many allocations does
  the *scanner* contribute to a `stats` run versus the tally? Sitting F's
  lens will grade that last answer.

## Hints (one at a time)

<details><summary>Hint 1 — a nudge: the fold, and who owns what</summary>

The whole of `tally` is: start from an empty `Stats`, loop over the lines,
`match classify(line, &REQUIRED_KEYS)` — three arms, three behaviors (nothing / one counter /
three counters). If the key-type decision is what's blocking you, ask it as a
lifetimes question: the maps outlive the loop and get returned out of the
function — do the *lines* outlive the maps? Inside the property test they
happen to (the rendered `Vec` lives to the end), but write the answer for
`main`, where a file's contents `String` is born and dies inside the reading
loop unless you deliberately keep it. Whichever way you land, land there on
purpose.

</details>

<details><summary>Hint 2 — the shape: Stats and the entry idiom</summary>

The owned-keys version of the struct (swap in `&'a str` and a lifetime
parameter if you took the other road):

```rust
pub struct Stats {
    pub by_kind: HashMap<String, u64>,
    pub by_day: HashMap<String, u64>,
    pub events: u64,
    pub malformed: u64,
}
```

and the counting idiom, once per axis:

```rust
*stats.by_kind.entry(/* this event's kind, as your key type */).or_insert(0) += 1;
```

Note the leading `*` — `or_insert` returns `&mut u64`, and you write straight
through it. Derive `Debug` (proptest wants to print `Stats` on failure) and
`Default` (an empty `Stats` for free via `Stats::default()`).

</details>

<details><summary>Hint 3 — the shape: deterministic printing</summary>

A `HashMap` refuses to have an order, so borrow its pairs into something that
can hold one:

```rust
fn sorted(map: &HashMap<String, u64>) -> Vec<(&String, &u64)>
```

```rust
// collect map.iter() into a Vec
// sort_by: compare counts DESCENDING (b against a), and
//          .then(...) an alphabetical tiebreak on the keys —
//          without the tiebreak, equal counts still shuffle between runs
// caller loops the Vec and prints "  {key:<24} {n:>6}"
```

Both commands' output paths can share this one helper — by-type and by-day
are the same shape.

</details>

## If truly stuck

Read, don't copy — then close it and write yours:

- `specs/003-rust-bedrock/_reference/glake/src/tally.rs` — function `tally`
  (and `Stats::is_conserved`, its packaging of the R9 law). Note the reference
  chose owned `String` keys; if yours borrows instead, that's not wrong — be
  able to say what each choice costs.
- `specs/003-rust-bedrock/_reference/glake/tests/prop_tally.rs` — strategy
  `gen_line` and test `r9_totals_conserved_on_both_axes`.
- `specs/003-rust-bedrock/_reference/glake/src/main.rs` — functions `stats`
  and `sorted` (the formatting and the ordering).
