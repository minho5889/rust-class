# Sitting I — clap and filters, test-first

**Builds:** `glake stats` learns `--type` and `--since` — real filters built
from closures and iterator adapters (ramp 11, now on payroll) — proven by the
F3 partition property *written before the filter exists*; the hand-rolled arg
matcher retires in favor of `clap` derive (F10); and the day rule gets its
normative v1 definition, with bad-ts events counted, never silently eaten
(F2a/F2b).
**Requirements:** F1, F2a, F2b, F3, F4, F10 (tasks 1.3–1.4; T5).
**Ramp you'll use:** step 11 (closures & adapters — this is the sitting it
was for), step 5 (enums as answers), with step 10's `impl Trait` instincts
from G in the background.

## Where you are

H gave glake one error type and one funnel; the plumbing is done and the tool
still behaves like v0. Today it grows the first *user-visible* v1 behavior:

```console
$ glake stats datalake/raw-local --type gate.approved --since 2026-07-06
5 files · 2 events (filtered from 288)
…
```

Two numbers in that header — kept and unfiltered — because a filter that
hides its denominator is a filter you can't audit. Behind it sits this
sitting's property, F3, third of your career and the first where **you**
drive the whole co-write: for any event set and any filter, *kept + excluded
= total* — per kind and per day too. Filters never invent events, never lose
them, and never, ever touch the malformed count. Same rhythm as C and E: red
commit first, and the failing test is the spec until the code catches up.
Three commits today — red, clap, green.

## The build, move by move

All commands from the repo root.

1. **Say F3 in one breath, then find its edges.** "Whatever the filter, kept
   plus excluded equals the unfiltered total, on every axis." Now the edges,
   out loud before any code — each is a clause in the property or a design
   decision you should be able to defend:

   - What happens to `Blank` and `Malformed` lines under a filter? The design
     ruled: **non-events always pass** — a filter constrains *events*, and
     stats' malformed count must never be silently filtered away. (The
     property will assert `kept.malformed == total.malformed`, exactly.)
   - `--since 2026-07-06` meets an event whose day is `bad-ts` — there is no
     day to compare. Error? Include? The design chose **exclude + count**
     (F2b): silent inclusion lies, erroring punishes old data, and the count
     keeps the conservation visible in the output itself.
   - Why is the filter's answer an *enum* and not a `bool`? Because F2b's
     count needs to know one particular *why*: read the design's filter row —
     `Verdict { Keep, Skip, SkipBadTs }`. A `bool` can say no; it can't say
     which rule said no.

2. **Freeze the stubs.** Create `crates/glake/src/filter.rs` (`pub mod
   filter;` in `lib.rs`) with the design's frozen shapes — `todo!()` bodies,
   underscored params, exactly like C taught:

   ```rust
   #[derive(Debug, Clone, Default, PartialEq, Eq)]
   pub struct Filter {
       pub kind: Option<String>,   // --type <t>, exact match only (F1)
       pub since: Option<String>,  // --since <YYYY-MM-DD> (F2a)
   }

   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum Verdict { Keep, Skip, SkipBadTs }

   impl Filter {
       pub fn new(kind: Option<String>, since: Option<String>) -> Result<Self, GlakeError> { todo!() }
       pub fn is_active(&self) -> bool { todo!() }
       pub fn verdict(&self, line: &Line<'_>) -> Verdict { todo!() }
       pub fn keep(&self, line: &Line<'_>) -> bool {
           matches!(self.verdict(line), Verdict::Keep)
       }
   }
   ```

   (Note who's who: `verdict` answers on a *classified* `Line` — G's `Copy`
   derive is about to earn its keep — and `new` finally gives H's stubbed
   `GlakeError::Usage` its customer.) Then in `tally.rs`, stub the pipeline
   the property will exercise end-to-end:

   ```rust
   #[derive(Debug, Clone, Default, PartialEq, Eq)]
   pub struct FilteredStats {
       pub kept: Stats,          // tally of what survived
       pub total_events: u64,    // F4's M — valid events BEFORE filtering
       pub excluded_bad_ts: u64, // F2b's count
   }

   pub fn tally_filtered<'a>(
       lines: impl IntoIterator<Item = &'a str>,
       filter: &Filter,
   ) -> FilteredStats { todo!() }
   ```

   Check it compiles: `cargo build -p glake`.

3. **Write the F3 property — you drive, and watch it fail.** Create
   `crates/glake/tests/prop_filter.rs`. Task 1.3 says it plainly: *reuse your
   R9 generator* — open your `prop_tally.rs` and bring `GenLine` and
   `render` across (a test-local copy is fine; the reference does the same).
   Two adaptations, one new trick:

   - **One new ts arm**: `"2026/07/09T00:00:00Z"` — ten characters that are
     *not* a day. Tag its expected day `bad-ts`. Under v0's rule (any 10-char
     prefix) that's a lie; under v1's day rule (move 5) it's the truth. Your
     generator is now ahead of your code — properties get to do that.
   - **The filter must come *from* the data**, or it never bites: a random
     `--type` almost never matches a random kind, and a property that only
     tests the everything-passes case proves nothing. The new proptest trick
     is `prop_flat_map` — generate the line set first, then *derive* the
     filter's candidates from it: every kind the set actually contains (plus
     `None` and one absent kind), every tagged day (plus `None` and the
     extremes `0000-01-01` / `9999-12-31`), and `prop::sample::select` to
     pick. This is the one genuinely new API today — spend your Claude
     questions here.
   - **The assertions**, all against public API: classify every rendered
     line once (`classify(line, &REQUIRED_KEYS)`); build three tallies —
     `total` (everything), `kept` (lines where `filter.keep(&classified)`),
     `excluded` (the complement); then assert the partition:
     `kept.events + excluded.events == total.events`; per-key on *both* axes
     (kept[k] + excluded[k] == total[k], and neither side holds a key total
     lacks — filters don't invent, either); `kept.malformed ==
     total.malformed` and `excluded.malformed == 0`; both sides still
     `is_conserved()`. Finally the pipeline consistency check: 
     `tally_filtered(rendered lines, &filter)` reports the same `kept`, the
     right `total_events`, and an `excluded_bad_ts` you compute from your
     tags (an event whose tagged day is `bad-ts`, that passes `--type`,
     while `--since` is active — the *reserved* meaning of `SkipBadTs`).

   House conventions: test named `prop_f3_…`, 512 cases. Run and enjoy:

   ```console
   cargo test -p glake --test prop_filter
   ```

   `todo!()` panics, proptest shrinks toward the empty set, a seed lands in
   `proptest-regressions/` — commit it, house rule. Ritual (`cargo fmt`,
   clippy with `--all-targets`), then **commit point — the red commit:**

   ```console
   git add crates/glake && git commit -m "004: sitting I — F3 partition property, red"
   ```

4. **Retire the hand-rolled args — `clap` derive (F10).** Add to
   `crates/glake/Cargo.toml`:

   ```toml
   clap = { version = "4", features = ["derive"] }
   ```

   (Forget `features = ["derive"]` and meet the first fight below.) Create
   `crates/glake/src/cli.rs`: a `Cli` struct with `#[derive(Parser)]` and
   `#[command(subcommand)]`, and a `Command` enum — `Validate { path }` and
   `Stats { path, kind, since }`. The struct *is* the interface: clap
   generates parsing, `--help`, and error handling from it. Three details
   that are each a lesson:

   - You cannot name a field `type` — it's a keyword. Name it `kind` and
     tell clap the user-facing spelling: `#[arg(long = "type", value_name =
     "EVENT_TYPE")]`. (`--since` needs only `#[arg(long, value_name =
     "YYYY-MM-DD")]`.)
   - The filter flags live on `Stats` **only**. That's F1 as amended:
     `validate` has nothing to filter (malformed lines have no `event_type`
     or day), so `glake validate lake --type x` must die at the *definition
     level* — clap rejects an argument the subcommand never declared, exit 2,
     no code of yours involved.
   - In `main`, `Cli::parse()` fails with **E0599** until you `use
     clap::Parser;` — `parse` is a trait method, and ramp 9 move 7 taught
     you the rule the hard way so this moment would feel like recognition,
     not mystery.

   Replace the hand-rolled arg `match` in `main` (a moment of silence — it
   served honestly since Sitting B) with `Cli::parse()` and a `match` on
   `cli.command`; thread `kind`/`since` as far as a `Filter::default()` for
   now. Update `tests/cli.rs`: bad usage still exits **2** (clap's own
   default — the F5 contract survives the regime change), but the wording is
   clap's now; assert case-insensitively on `usage` / `possible values`
   rather than your old string. Add the two new F10 tests: `--help` exits 0
   and names both commands; and inside `cli.rs`, clap's own self-check —

   ```rust
   Cli::command().debug_assert();   // needs: use clap::CommandFactory as _;
   ```

   — plus the F1 definition-level test: `Cli::try_parse_from(["glake",
   "validate", "lake", "--type", "x"])` is an `Err`. Gate, then **commit
   point:**

   ```console
   git add crates/glake Cargo.lock && git commit -m "004: sitting I — clap derive CLI, filters stats-only"
   ```

5. **The day rule becomes law (F2).** The requirements define it once,
   normatively: an event's day is the first 10 characters of its `ts` **iff**
   they match `YYYY-MM-DD` (4 digits, dash, 2, dash, 2); otherwise the event
   is bad-ts. This *tightens* v0 — E's `.get(0..10)` accepted any 10-char
   prefix, so `2026/07/09T…` bucketed as a phantom day; under v1 it's
   visibly `bad-ts`. Why now? Because `--since` compares days as strings,
   and string-compare is only chronological if every day has the same shape
   — one malformed "day" in a bucket and F2a's `≥` silently lies. Build it
   in `classify.rs` as two public helpers with **one home**:

   ```rust
   pub fn is_day(s: &str) -> bool          // shape check: 9999-99-99 passes, on purpose
   pub fn day_of_ts(ts: &str) -> Option<&str>  // .get(0..10).filter(is_day) — E's lesson kept
   ```

   Point `classify`'s `Event` arm at `day_of_ts`, add the unit rows
   (`2026-07-09` yes; `9999-99-99` yes — shape, not calendar; `2026-7-9`,
   `2026/07/09`, a 🦀 string — no), and update `prop_tally.rs`'s truth: the
   expected-day tag comes from the same rule now (and add the
   `2026/07/09T…` arm there too, tagged `bad-ts`). Run the R9 property —
   if your truth and your code disagree about the new arm, one of them is
   still living in v0.

6. **Implement: verdict, then the pipeline.** The filter first —
   `Filter::verdict` is Option-combinator country (ramp 11's `filter_map`
   sensibility, applied to logic):

   - Non-events: `Keep`, unconditionally (move 1's decision).
   - `--type` check **first**: a wrong-type event is a plain `Skip` even if
     its day is also bad-ts — `SkipBadTs` is *reserved* for events the
     since-rule alone excluded, so F2b's printed count means exactly one
     thing. (`self.kind.as_deref().is_some_and(|want| want != kind)` — meet
     `as_deref`, the `Option<String>` → `Option<&str>` bridge; the fights
     section explains why you'll want it.)
   - Then `--since`: no constraint → `Keep`; day is `bad-ts` → `SkipBadTs`;
     otherwise plain string `>=` — which the day rule just made
     chronological.

   `Filter::new` validates the `--since` value with the *same* `is_day`
   (clap can't know our date grammar): a bad value becomes
   `GlakeError::Usage("--since expects YYYY-MM-DD, got …")` — H's funnel
   turns it into one stderr line and exit 2, no new plumbing. `is_active` is
   two `is_some()`s. Then `tally_filtered` — one adapter chain, ramp 11 at
   work:

   ```rust
   let mut total_events = 0u64;
   let mut excluded_bad_ts = 0u64;
   let kept_lines = lines.into_iter().filter(|line| {
       // classify; if it's an Event, total_events += 1;
       // match filter.verdict(&classified): Keep → true, Skip → false,
       // SkipBadTs → count it, false
   });
   let kept = tally(kept_lines);
   ```

   Two things to notice and say aloud. The closure *captures* both counters
   by `&mut` (ramp 11 move 5's E0502 lesson, now production code — watch the
   borrow end before `kept` is used). And yes: this classifies each kept
   line **twice** — once in the closure to judge it, once inside `tally` to
   count it. With the zero-copy scanner that's cheap (F proved the scanner
   allocates *nothing*), but it's a smell — mark it with a
   `// Sitting J: classification becomes a value that travels` comment.
   J's trait makes the classification an owned value produced exactly once,
   and *measures* what that costs instead of guessing.

7. **Wire the output (F4) and extend the fixtures.** In `main.rs`, `stats`
   builds `Filter::new(kind, since)?`, calls `tally_filtered`, and prints:

   - header: `N files · K events (filtered from M)` **only when
     `filter.is_active()`** — an unfiltered v1 run stays byte-identical to
     v0 (G's diff discipline still holds);
   - `(N bad-ts event(s) excluded by --since)` only when the count is
     non-zero;
   - the malformed note and both sorted tables, unchanged.

   Then grow `tests/fixtures/lake/` a third partition — `dt=2026-07-03/`
   with one good event and one whose `ts` can't yield a day (the reference
   uses `"ts":"soon"`) — and write the [E] tests in `tests/cli.rs`, asserting
   against **your own hand-count** (E's rule: `contains` on load-bearing
   fragments). For calibration, the reference's fixture lake (5 valid events,
   one bad-ts, one malformed, one blank, three partitions) prints, verbatim:

   ```console
   glake stats tests/fixtures/lake --type gate.approved
   # 3 files · 3 events (filtered from 5)
   glake stats tests/fixtures/lake --since 2026-07-02
   # 3 files · 2 events (filtered from 5)
   # (1 bad-ts event(s) excluded by --since)
   glake stats tests/fixtures/lake --type gate.approved --since 2026-07-03
   # 3 files · 1 events (filtered from 5)    ← and NO bad-ts note: the bad-ts
   #                                            event is session.start — --type
   #                                            excluded it first (move 6's order)
   ```

   Cover at minimum: F1 exactness (`--type gate` matches nothing —
   `0 events (filtered from 5)`); F2a+F2b (the note appears, old days
   vanish); the compose case above; F5's bad value
   (`--since 2026/07/02` → one stderr line `glake: --since expects
   YYYY-MM-DD, got "2026/07/02"`, exit 2); and F1's rejection
   (`validate … --type x` → clap's `error: unexpected argument '--type'
   found`, exit 2).

8. **Gate and close green.**

   ```console
   cargo fmt
   cargo clippy -p glake --all-targets -- -D warnings
   cargo test -p glake
   cargo run -p glake -- stats datalake/raw-local --type gate.approved --since 2026-07-06
   ```

   That last command against the real lake printed, on validation day:
   `5 files · 2 events (filtered from 288)` — your numbers will differ (the
   lake grows); the *shape* must match. **Commit point — the green commit:**

   ```console
   git add crates/glake && git commit -m "004: sitting I — filters green, F3 holds"
   ```

## Compiler fights to expect

A crowded sitting, so a crowded ledger (`learning.*` for each, as always).

- **`cannot find derive macro `Parser` in this scope`** — clap without
  `features = ["derive"]`. The derive API is feature-gated; the fix is one
  Cargo.toml edit away. (While you're there: this is your second shipped
  dependency — F11 allows it; J's sweep will record the tree.)
- **`error: expected identifier, found keyword `type`** — naming the field
  after the flag. Keywords are not identifiers; `kind` + `#[arg(long =
  "type")]` separates the Rust name from the user's spelling. (There is a
  raw-identifier escape hatch, `r#type` — know it exists, don't use it here.)
- **`error[E0599]: no function or associated item named `parse` found`** —
  with clap's generous `help: items from traits can only be used if the
  trait is in scope` and the exact `use clap::Parser;` line. Ramp 9 move 7,
  in the wild, five days later. Same fight again in the test module with
  `CommandFactory` for `Cli::command()`.
- **`error[E0308]: mismatched types — expected `&str`, found `&String`** (or
  mirror) — comparing `self.kind: Option<String>` against `Line`'s
  `kind: &str` inside `verdict`. `as_deref` is the designed bridge:
  `Option<String>` → `Option<&str>`, no allocation, then `is_some_and`
  closes the match. If you fought `Option::map` + `==` into submission
  instead, it works — but read the reference's spelling and say which is
  clearer.
- **`error[E0502]: cannot borrow … as immutable because it is also borrowed
  as mutable`** — if you try to read `total_events` (say, to build
  `FilteredStats`) while the `kept_lines` iterator — whose closure holds the
  `&mut` capture — is still alive. Adapters are *lazy*: the closure's borrow
  lasts until the chain is consumed. Consume it (`tally(kept_lines)`)
  *before* touching the counters, exactly the ordering the skeleton in move
  6 shows. Ramp 11 move 5's lesson with the roles reversed.
- **The property fails with a shrunk case naming your `SkipBadTs` count** —
  not a compiler fight, and the best failure of the sitting: your verdict
  checks `--since` before `--type`, so a wrong-type bad-ts event lands in
  `SkipBadTs` and inflates the note. The design's reserved-meaning rule
  (move 6) is the fix; the seed goes in `proptest-regressions/`, committed —
  it earned it.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                  # no diff
cargo clippy -p glake --all-targets -- -D warnings # clean
cargo test -p glake                                # ALL green: prop_f3_*, R8, R9
                                                   # (with the tightened day truth),
                                                   # f10/f1/f2 CLI tests, H's f6_*
PROPTEST_CASES=2000 cargo test -p glake --test prop_filter
                                                   # confidence run (reference: green in ~3s)
```

```
cargo run -p glake -- stats crates/glake/tests/fixtures/lake --since 2026-07-02
# → (filtered from M) header + the bad-ts note, numbers matching your hand-count

cargo run -p glake -- validate crates/glake/tests/fixtures/lake --type x; echo $?
# → clap: "error: unexpected argument '--type' found", exit 2 (F1: stats-only)

cargo run -p glake -- stats datalake/raw-local --since 2026/07/06; echo $?
# → glake: --since expects YYYY-MM-DD, got "2026/07/06" — ONE line, exit 2 (F5 funnel)

cargo run -p glake -- stats datalake/raw-local
# → byte-identical to an unfiltered v0 run: no "(filtered from", no note (F4)
```

- `git log --oneline -3` shows red → clap → green, the rhythm's third
  performance.
- You can answer aloud: why is `Verdict` three-valued (what question can't a
  `bool` answer)? Why must `--type` be checked before `--since`? Why does
  `9999-99-99` pass `is_day` — and why is that *correct*? And where exactly
  does your pipeline classify twice, and which sitting deletes that?

## Hints (one at a time)

<details><summary>Hint 1 — the lines_and_filter strategy shape</summary>

```rust
fn lines_and_filter() -> impl Strategy<Value = (Vec<GenLine>, Filter)> {
    prop::collection::vec(gen_line(), 0..60).prop_flat_map(|lines| {
        // build Vec<Option<String>> of --type candidates: None, one absent,
        //   plus every Event's kind in `lines`
        // build Vec<Option<String>> of --since candidates: None, extremes,
        //   plus every comparable tagged day in `lines`
        let filter = (prop::sample::select(kinds), prop::sample::select(sinces))
            .prop_map(|(kind, since)| Filter { kind, since });
        (Just(lines), filter)
    })
}
```

`prop_flat_map` is "generate, then generate *from* what you generated" — the
lines must be decided before their kinds can be candidates. `Just(lines)`
re-wraps the decided value as a strategy so the tuple works. Shrinking gets
odd across `flat_map` (the filter can jump as the lines shrink) — that's
expected; the partition law holds for *every* (lines, filter) pair anyway.

</details>

<details><summary>Hint 2 — verdict, arm by arm</summary>

The reference's order, as prose: (1) `let Line::Event { kind, day } = line
else { return Verdict::Keep };` — non-events flow through (let-else, D's
friend). (2) If a wanted kind is set and doesn't equal `kind` → `Skip`,
immediately — this is what reserves `SkipBadTs`. (3) Match on
`self.since.as_deref()`: `None` → `Keep`; `Some(_)` while `day == "bad-ts"`
→ `SkipBadTs`; `Some(since)` → `Keep` iff `day >= since` else `Skip`. Five
outcomes, no allocation anywhere — the filter reads, it never copies.

</details>

<details><summary>Hint 3 — the tally_filtered closure keeps fighting you</summary>

The filter closure must do three jobs in one pass: classify, count events
into `total_events`, and return the keep/skip bool (counting `SkipBadTs` on
the way). Shape:

```rust
.filter(|line| {
    let classified = classify(line, &REQUIRED_KEYS);
    if let Line::Event { .. } = classified {
        total_events += 1;
    }
    match filter.verdict(&classified) {
        Verdict::Keep => true,
        Verdict::Skip => false,
        Verdict::SkipBadTs => { excluded_bad_ts += 1; false }
    }
})
```

`classified` is a `Line<'_>` — G made it `Copy`, so the `if let` peek doesn't
move it and the `verdict` call still has it. If the counters trip E0502 at
the end of the function, you built `FilteredStats` before the chain was
consumed — `tally(...)` first, struct literal second.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/004-glake-traits/_reference/glake/src/filter.rs` — `Filter`,
  `Verdict`, `new`, `verdict` (its doc comments carry this sitting's design
  decisions). **One warning:** the reference's `verdict` already takes the
  owned `ClassifiedLine` from Sitting J — yours today takes your borrowed
  `Line<'_>`; same arms, different parameter.
- `specs/004-glake-traits/_reference/glake/src/cli.rs` — the derive structs
  and the `f10`/`f1` unit tests (skip `ParserChoice`/`--parser`; that's J).
- `specs/004-glake-traits/_reference/glake/src/classify.rs` — `is_day`,
  `day_of_ts`, and the `f2_day_rule_table` test.
- `specs/004-glake-traits/_reference/glake/tests/prop_filter.rs` — the
  strategy and `assert_axis_partition` (again: it pipes through `HandParser`
  and owned lines — J's shapes; your truth flows through `classify`).
