# Sitting J — the trait, the rival, the measurement

**Builds:** the crown of Part 3. `classify` goes behind an **`EventParser`
trait** with an owned `ClassifiedLine` boundary; `serde_json` arrives as a
rival backend selectable with `--parser hand|serde`; the lib pipeline goes
generic (static dispatch, F13) while the bin spends exactly one
`Box<dyn EventParser>`; a property test proves the backends identical on
well-formed input (F8a) — after you first claim too much, meet a real
counterexample, and triage your way to the amended claim — and a second
property fences *exactly where* they may disagree (F8b). Then the payoff the
whole hand-first curriculum was built for: both backends under the lens, and
a real number on what `serde` costs (F9).
**Requirements:** F7, F8a, F8b, F9, F11, F12, F13 (tasks 1.5–1.9; T1, T2, T6).
**Ramp you'll use:** step 9 (traits — this is `Describe` grown up), step 10
(generics vs `dyn` — glake's layout is that worksheet's closing paragraph,
verbatim), step 11 (the pipeline you'll make generic), steps 2/4 (moves and
`String`s — the boundary's cost is made of them).

## Where you are

I left glake filtering through a pipeline that classifies every kept line
twice — you marked the smell yourself. Today classification becomes a
*value*: one trait, two implementations, and the borrowed `Line<'a>` your
scanner returns meets a boundary it cannot cross. This sitting also carries
the course's most honest moment. You will write the property your gut
believes — *the two parsers agree, full stop* — and proptest will hand you a
shrunk line on which that claim is **false**, with neither parser wrong.
This is not a detour; it is the sitting. The same thing happened while these
materials were being written: spec 004's F8 began life as a single
exact-equivalence requirement, the reference build proved it unsatisfiable,
and the requirement was amended (F8 → F8a/F8b, changelog rev 2) before you
ever saw it. You get to re-live the discovery with the safety net installed
— and to practice the counterexample-triage protocol on a *spec* bug, the
rarest and most valuable kind. Five commits; division of labor swings twice
(Claude drives the hygiene sweep; you drive the measurement).

## The build, move by move

All commands from the repo root.

1. **Argue the boundary before you type it (T1/T6).** Read the design's "In
   plain words" and answer aloud, in your own words: why must the trait
   return an **owned** `ClassifiedLine` when your scanner can hand out
   borrowed `Line<'a>`s for free? Careful — the obvious answer is *wrong*,
   and the design (rev 2) says so explicitly: traits and even `dyn` objects
   can return borrows tied to their inputs
   (`fn classify<'a>(&self, line: &'a str) -> Line<'a>` is perfectly legal).
   The real forcer is the rival: `SerdeParser` parses the line into its own
   *local* `serde_json::Value`, extracts from it — and you cannot return
   borrows of a local (Sitting C's E0515, remember it?). One backend needs
   ownership, so the *shared* boundary must own. That's the T6 lesson in one
   sentence: the abstraction's price is set by its most expensive
   implementer — and move 9 puts a number on it.

2. **Freeze the seam.** Create `crates/glake/src/parser.rs` (`pub mod
   parser;` in `lib.rs`):

   ```rust
   /// The owned verdict on one line — `Line`'s mirror that can cross a
   /// trait boundary (and outlive the input it was parsed from).
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub enum ClassifiedLine {
       Blank,
       Unparseable,                          // strict backends only
       Malformed { missing: String },
       Event { kind: String, day: String },
   }

   pub trait EventParser {
       fn classify(&self, line: &str, required: &[&str]) -> ClassifiedLine;
   }

   #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
   pub struct HandParser;
   #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
   pub struct SerdeParser;

   impl EventParser for HandParser {
       fn classify(&self, _line: &str, _required: &[&str]) -> ClassifiedLine { todo!() }
   }
   impl EventParser for SerdeParser {
       fn classify(&self, _line: &str, _required: &[&str]) -> ClassifiedLine { todo!() }
   }
   ```

   Interrogate every line before moving on (ramp 9 vocabulary): why `&self`
   and not no receiver at all? (Object safety — the bin will hold a
   `Box<dyn EventParser>`; a trait method without `self` can't be called
   through a vtable.) Why one variant more than `Line`? (`Unparseable` —
   your scanner *scans*, it has no concept of failing to parse; a strict
   parser does. G's rubric question: why is `ClassifiedLine` `Clone` but not
   `Copy`, when `Line` is both?) Why do the two backend structs carry no
   fields? (The parser *is* a strategy, not a state — a unit struct that
   exists to pick an impl. `Copy + Default` cost nothing and F12's rubric
   wants them.) Check: `cargo build -p glake`.

3. **Write the agreement property — the one you believe (1.5, red).** Create
   `crates/glake/tests/prop_parsers.rs`. You drive; this is your fourth
   property. The claim to encode is the one your gut has been making since
   move 1: *for any line, both parsers classify identically.* And you
   already own the generator for "any line" — Sitting D's full R8 mixed
   strategy (`json_ish` fragments with escapes and truncations, `any::<String>()`
   garbage, plus a well-formed arm built like your R9 `render`). Bring it
   across and write:

   ```rust
   let hand = HandParser.classify(&line, &REQUIRED_KEYS);
   let serde = SerdeParser.classify(&line, &REQUIRED_KEYS);
   prop_assert_eq!(hand, serde, "backends must agree on: {}", line);
   ```

   512 cases, named `prop_f8_backends_agree` for now — the name will not
   survive the sitting, and that's foreshadowing, not sloppiness. Run it,
   watch `todo!()` panic and shrink, commit the seed (house rule — it's a
   genuine red), ritual, **commit point — the red commit:**

   ```console
   git add crates/glake && git commit -m "004: sitting J — F8 agreement property, red"
   ```

4. **`HandParser` — the boundary allocation, in one visible place (1.6).**
   Implementing the hand backend is a conversion, not a parser: everything
   003 built stays untouched, and one `impl From<Line<'_>> for
   ClassifiedLine` pays the toll — `missing.to_owned()`, `kind.to_owned()`,
   `day.to_owned()`. Write it in `parser.rs`, make `HandParser::classify`
   the one-liner `classify(line, required).into()`, and put a comment on the
   `From` impl saying what it is, because it's the whole sitting in
   miniature: **borrowed in, owned out — two `String`s per event, and this
   exact line is what the F9 lens run measures.** Note what `HandParser`
   can never return: `Unparseable` (junk comes back as "missing the first
   required key" — the scanner answers questions, it doesn't refuse them).
   Now prove the property *machinery* on the one backend you have: point the
   property's second call at `HandParser` too — hand-vs-hand — with a
   `// move 6 swaps the rival in` comment. Green, trivially, and that
   triviality is the point: the harness works, the claim is untested.

5. **The pipeline goes generic (1.6, F13) — and I's smell dies.** Three
   mechanical migrations, each refereed by an existing test:

   - **`tally` lifts to the boundary type:**
     `pub fn tally(lines: impl IntoIterator<Item = ClassifiedLine>) -> Stats`
     — the fold now consumes classified values, and its `Event` arm gets
     *simpler*: the `String`s **move** straight into the maps
     (`entry(kind)`, no `to_owned` — ramp 2: a move is free, and this is why
     the owned boundary won't double your allocation bill; the lens will
     confirm). `Unparseable` folds into `malformed` (to stats both mean "a
     line that isn't a countable event").
   - **`tally_filtered` gains the parser** — the design's generic pipeline:

     ```rust
     pub fn tally_filtered<'a, P>(
         parser: &P,
         required: &[&str],
         lines: impl IntoIterator<Item = &'a str>,
         filter: &Filter,
     ) -> FilteredStats
     where
         P: EventParser + ?Sized,
     ```

     Inside, I's double-classify disappears: `.map(|line|
     parser.classify(line, required))` once, `.inspect` counts events,
     `.filter` applies the verdict, `tally` eats what survives. Say what
     `?Sized` buys before you need it (move 6 will show you the error that
     demands it — or add it now and explain it to Claude instead).
   - **`Filter::verdict` migrates** to `&ClassifiedLine` — same arms, owned
     fields now (`kind: &String` meets `&str` comparisons; `as_str`/deref
     bridges).

   Chase the fallout: `main` calls `tally_filtered(&HandParser,
   &REQUIRED_KEYS, …)` for now (static dispatch in the bin — temporary);
   `validate` classifies via `HandParser` and gains its `Unparseable` arm
   (`not a JSON object` — only strict backends will ever trigger it);
   `prop_tally` and `prop_filter` adapt by classifying through
   `HandParser.classify` (and where the F3 property cloned `Line`s by `Copy`,
   `ClassifiedLine` demands `.cloned()` — say why). Then the F13 unit tests
   in `tally.rs`: one calling `tally_filtered(&HandParser, …)` directly —
   **monomorphized**, no `Box` anywhere — and one threading the same call
   through `&*Box::<dyn EventParser>::new(HandParser)` and asserting the
   results equal: same function, static and dynamic, side by side (ramp 10's
   two ways, now load-bearing). Full gate green. **Commit point:**

   ```console
   git add crates/glake && git commit -m "004: sitting J — EventParser + HandParser, pipeline generic"
   ```

6. **The rival arrives — and the property does its real job (1.7).** Add
   `serde_json = "1"` to `[dependencies]` (the last of F11's three
   permitted arrivals). Implement `SerdeParser::classify`, *deliberately
   mirroring* the scanner's semantics: blank check first (a whitespace line
   is `Blank` on both backends, not `Unparseable`); then strictness —

   ```rust
   let Ok(serde_json::Value::Object(map)) = serde_json::from_str(line) else {
       return ClassifiedLine::Unparseable;
   };
   ```

   — then required keys **in the caller's order** (so both backends name the
   same *first* missing key), then `event_type`/`ts` extracted with
   `as_str` (non-string → the same `"non-string"` / `bad-ts` sentinels) and
   the day through the *same* `day_of_ts` helper (one home, I's rule — the
   backends cannot drift on the day rule because neither owns it). Swap the
   property's second call to `SerdeParser`. Run:

   ```console
   cargo test -p glake --test prop_parsers
   ```

   **Red — and this time nothing is stubbed.** Proptest hands you a shrunk
   line; on the validation build it's a truncated JSON-ish fragment (a lone
   `{` is its purest form): the scanner shrugs and reports
   `Malformed { missing: "event_id" }`; serde reports `Unparseable`. Stop.
   Do not fix anything yet. This is a counterexample, and the pipeline has a
   protocol: **classify before fixing** — spec bug, code bug, or test bug?
   Walk it honestly:

   - Is `HandParser` wrong? It answered the question it has always answered
     — "which required key can't I find?" — exactly as documented since C.
   - Is `SerdeParser` wrong? `{` is not JSON; refusing it is its entire
     value proposition.
   - So the *claim* is wrong: "both parsers classify identically, for any
     line" over-promises — on degenerate input, "identical" isn't even
     well-defined, because the two backends answer *different questions*
     ("what does a byte-scan see?" vs "what does a JSON document mean?").
     **Verdict: spec bug.**

   And now the reveal, which you're owed: this exact triage happened during
   authoring. The original F8 demanded a single exact-equivalence property;
   the reference build met this same class of counterexample; the
   requirement was amended into **F8a** (exact equivalence *on well-formed
   input* — the class real envelope writers produce) and **F8b** (on
   arbitrary input, agreement *or* a divergence inside three documented
   classes), and the amendment was re-gated per the change protocol. Read
   the three classes now — they're the table in
   `specs/004-glake-traits/_reference/NOTES.md`:

   | # | Class | Hand says | Serde says |
   |---|---|---|---|
   | 1 | **Strictness** (truncations, non-object JSON, junk) | scans anyway → `Malformed { missing: "event_id" }` | `Unparseable` |
   | 2 | **Escape normalization** (`a\"b` in a value; a `-` escape inside `ts`) | raw slice, escapes kept (an escaped ts prefix fails the day rule → `bad-ts`) | unescaped (the same ts yields a real day) |
   | 3 | **Duplicate top-level keys** | FIRST occurrence | `Value`'s map keeps LAST |

   Classes 2–3 are declared **unspecified input** for glake — real envelope
   writers never emit them — but F8b *pins* the divergence anyway, so a
   silent behavior change in either backend still fails the suite. Log the
   moment before moving on: a `test.counterexample` + `test.triage` pair in
   the mistake ledger, and a triage entry in
   `specs/004-glake-traits/_assurance/triage-log.md` (verdict: spec bug,
   amendment already gated).

7. **Reshape the property into F8a, and co-write F8b (1.7).** Two properties
   replace your one:

   - **F8a — `prop_f8a_backends_identical_on_wellformed`.** Yours to write.
     The input class does the work: build lines that are valid JSON objects
     *by construction* — assemble a `serde_json::Map` (which physically
     cannot hold duplicate keys) from a `prop::sample::subsequence` of
     `REQUIRED_KEYS` plus a few extras, values drawn from an escape-free
     charset (no quotes, no backslashes, no control chars — multibyte 🦀 is
     fine), `ts` arms covering every day-rule bucket (real days, `"2026"`,
     crab-string, `2026/07/09…`, and a non-string number), then let
     `Value::Object(map).to_string()` do the serializing. Assert serde never
     says `Unparseable` and both backends agree exactly. Yes, the generator
     leans on serde_json — the fox guards *one* henhouse while the property
     guards the other; the hand parser's independent correctness is still
     pinned by C/D's tables and R8.
   - **F8b — `prop_f8b_divergences_fall_in_documented_classes`.** Co-written
     (tricky strategy work — the coached-mode rule applies). The full mixed
     strategy returns — garbage, your `json_ish`, well-formed — plus two
     arms that manufacture classes 2 and 3 *deliberately*, each carrying the
     generator's own knowledge of what each backend must answer (your R9
     tag-the-truth trick, sharpened: divergence isn't merely tolerated
     there, it's **pinned** to the documented behavior, backend by backend).
     The free-range arms assert: agree, or serde said `Unparseable` while
     hand said anything (class 1) — and here's the argument that makes the
     fence airtight (from NOTES, verify it against your own generator): your
     json-ish keys are ≤6 lowercase chars and never spell `event_id`, so no
     free-range line can carry all seven required keys — any line the two
     backends would *classify* differently there is necessarily
     serde-unparseable, i.e. class 1. Anything outside the three classes
     fails the property, and that failure would be a *genuine* bug with
     normal triage.

   One bookkeeping act with a design rationale behind it: your move-3 red
   seed (and the divergence seed from move 6) now reference a test that no
   longer exists, and the design explicitly rules — no expected-failure
   seeds here: *"a seed that a refined strategy would regenerate differently
   guards nothing."* Delete the stale `proptest-regressions` entries for the
   old test; F8a/F8b must pass clean at 512, and `proptest-regressions/`
   stays absent for this file **by design** (the committed-seeds house rule
   is for genuine failures of *living* tests).

   Then make the rivalry user-visible (F7): `ParserChoice` in `cli.rs` —
   `#[derive(ValueEnum)]`, variants `Hand`/`Serde`, `#[default] Hand` — as a
   `--parser` flag on **both** commands (it's not a filter; validate takes
   it too), and its `build()` method:

   ```rust
   pub fn build(self) -> Box<dyn EventParser> { /* match, two Box::new arms */ }
   ```

   That `Box<dyn EventParser>` is the crate's **only** dynamic-dispatch
   point — ramp 10's closing rule ("generic on the inside, one `dyn` at the
   outermost seam where the runtime choice arrives") is now your
   architecture, and F13's tests hold it in place. `main` threads
   `parser.as_ref()` / `&*parser` into the generic pipeline. Add the CLI
   evidence: a `tests/fixtures/junk.jsonl` (one non-JSON line + one valid
   event) and tests pinning the strictness split where users see it —
   `validate junk.jsonl` says `missing key "event_id"` (hand) vs
   `not a JSON object` (`--parser serde`), both exit 1, *and stats still
   agree on the numbers* (both fold junk into malformed) — plus the flag
   spellings test (`--parser nope` → clap's `invalid value … [possible
   values: hand, serde]`, exit 2) and the lake-level parity test: same
   stats output, both parsers, filtered and not. Extend the F13 static test
   to call the pipeline with `&SerdeParser` too. Full gate. **Commit
   points** (two — the rivalry, then the fence):

   ```console
   git add crates/glake Cargo.lock && git commit -m "004: sitting J — SerdeParser + --parser seam, F8a green"
   # …after F8b lands and passes:
   git add crates/glake && git commit -m "004: sitting J — F8b divergence containment"
   ```

8. **The hygiene sweep (1.8) — Claude drives, you interrogate.** Machine
   checks, outputs saved for `evidence.md`:

   ```console
   cargo tree -p glake -e normal --depth 1
   cargo tree -p glake -e normal --depth 1 --features lens
   ```

   The validated answers (your versions may drift by a patch number):

   ```text
   glake v0.2.0 (…/crates/glake)
   ├── clap v4.6.1
   ├── serde_json v1.0.150
   └── thiserror v2.0.18
   ```

   — and with `--features lens`, the same three **plus** `memlens v0.1.0`.
   That pair is F11 verbatim: the three permitted crates present, the lens
   still entering only on request (F's optional-dep work survived v1
   untouched), and a full-depth `cargo tree -p glake -e normal | grep -icE
   "tokio|hyper|async"` printing `0` — still nothing async. Then F12: walk
   the C-COMMON-TRAITS rubric over the whole public API as a table — every
   `pub` type × (`Debug`/`Clone`/`Copy`/`Default`/`PartialEq`) with a
   one-word justification per cell (G's `Copy`-for-borrows,
   `Clone`-for-owners rule decides most rows; make Claude defend any cell
   you doubt). Finally both worlds through the gates:

   ```console
   cargo fmt --check
   cargo clippy -p glake --all-targets -- -D warnings
   cargo clippy -p glake --all-targets --features lens -- -D warnings
   cargo test -p glake
   ```

   One lens-world trap the reference already ate for you: under `--features
   lens` every binary your `tests/cli.rs` spawns opens a memlens session and
   would litter a stray crate-local `datalake/`. The fix is two lines — your
   test helper sets `.env("MEMLENS_TRACE", "/dev/null")` on the spawned
   command, and the crate's `.gitignore` covers `/datalake/` for the test
   harness's own trace. Real traces belong in the repo-root lake; test
   exhaust belongs nowhere.

9. **The measurement (1.9, F9) — you drive.** This is why 003 made you write
   a scanner by hand before letting serde in the building.

   **Predict first, in writing** (the trace grades you): for one `stats`
   pass over the real lake — roughly how many allocations from the hand
   backend, and where? (You know the pieces: the `From` boundary is two
   `String`s per event; tally's maps consume them by *move*; file reads
   staircase. Sitting F measured v0 at ~700 — should v1-hand be near it,
   above it, or double it?) And serde: what does parsing one line into a
   `Value` allocate? (Every key, every value, a map — born and dead per
   line.) Write both guesses down with the ratio you expect.

   **Run traced, fairly.** Redirect the traces so the lake doesn't grow
   between runs (the observer-effect lesson from F — the A/B is only fair if
   both runs scan the *same* lake), same command, once per backend:

   ```console
   cargo build -p glake --features lens
   MEMLENS_TRACE=target/lens-hand.jsonl  MEMLENS_PROGRAM=glake \
     cargo run -p glake --features lens -- stats datalake/raw-local
   MEMLENS_TRACE=target/lens-serde.jsonl MEMLENS_PROGRAM=glake \
     cargo run -p glake --features lens -- stats datalake/raw-local --parser serde
   ```

   (Debug builds, necessarily — memlens *refuses* release builds by design,
   spec 002's rule.) First check: the two stdouts are **identical** — F7 at
   full scale, on real data. Then count what stdout can't show:

   ```console
   grep -c '"event_type":"memlens.alloc"' target/lens-hand.jsonl
   grep -c '"event_type":"memlens.alloc"' target/lens-serde.jsonl
   ls -la target/lens-*.jsonl
   ```

   What you should see — the validation-day numbers, from a 284-event,
   5-file lake (yours will be bigger; the *shape* is the claim):

   | Backend | allocs | deallocs | reallocs | trace size |
   |---|---|---|---|---|
   | `--parser hand` | 737 | 737 | 24 | 336 KB |
   | `--parser serde` | 7,148 | 7,148 | 41 | 3.2 MB |

   **≈ 9.7× the allocations** for identical output — roughly 2.6 allocs per
   event for hand versus 25 for serde. (A re-run hours later, lake at 288
   events: 745 vs 7,236 — same ratio. Yours should hold near 9–10× too.)
   And compare hand-v1 against your sitting-F v0 trace: within a few dozen
   allocations — **the trait boundary was nearly free**, because the
   boundary's `String`s move into the tally instead of being copied (your
   move-5 prediction, settled). Open both traces in `viewer/memlens.html`
   and hunt: the hand trace is F's old friend (staircases, tally churn, the
   scanner's flat line); the serde trace is a *wall* of short-lived
   allocations — click into one line's cluster and name what each bar is (a
   key `String`, a value, the map's spine — a `Value` tree born and dead per
   line). Then say the AWS sentence this course keeps building toward:
   identical output, 9.7× the allocator traffic — on Lambda that's CPU you
   pay for per invocation, at Fargate scale it's latency under load, and
   *you now know how to measure it instead of believing either camp*. The
   honest counterweight goes in the notes too: serde bought strictness
   (`Unparseable` exists), escape correctness, and ~150 lines you didn't
   write — F8b is the map of exactly what you'd give up.

   **Write it down.** Append to `specs/004-glake-traits/evidence.md` (start
   from `specs/_template/` if it's the first entry): the F9 table with
   *your* numbers and lake size, the ratio, the hand-vs-v0 delta, the F11
   tree pair and F12 rubric table from move 8, the property outcomes (F3,
   F8a, F8b — cases run, seeds absent by design for F8), and the move-6
   triage story in three sentences. **Commit point:**

   ```console
   git add crates/glake specs/004-glake-traits && git commit -m "004: sitting J — F9 measured, evidence appended"
   ```

## Compiler fights to expect

The full ledger treatment (`learning.*`) — and note how many of today's
fights are *runtime* property failures; that's what test-first buys.

- **`error[E0515]: cannot return value referencing local variable `map`** —
  if you doubt move 1 and try to make `SerdeParser` return a borrowed
  `Line<'a>`. The `Value` you parsed is a local; borrows of it die at the
  closing brace. This error *is* the design decision — provoke it once on
  purpose, then delete the experiment.
- **`error[E0277]: the size for values of type `dyn EventParser` cannot be
  known at compilation time`** — threading `&*boxed` into `tally_filtered`
  without `?Sized` on `P`. Generic parameters are `Sized` by default;
  `P: EventParser + ?Sized` lifts the requirement so `P = dyn EventParser`
  is legal. Ramp 10's E0277 wearing a bound instead of a `Vec`.
- **`error[E0599]: no method named `cloned`…** / **`the trait `Copy` is not
  implemented for `ClassifiedLine`** — adapting I's F3 property: `Line` was
  `Copy` (borrows and a tag), `ClassifiedLine` owns `String`s and can only
  be `.cloned()` — an explicit, visible cost, which is exactly the G rule
  about who gets which trait.
- **`error[E0308]`s around `&String` vs `&str`** — migrating
  `Filter::verdict` to `ClassifiedLine`: destructuring gives you `&String`
  fields where `Line` gave `&str`. `as_str()`/deref coercion bridges;
  understand which side allocates (neither — that's the point).
- **The divergence failure itself** — not a compiler fight but the sitting's
  centerpiece: `prop_assert_eq!` red with a shrunk line like `{`, hand and
  serde verdicts printed side by side. Your job is the *triage*, not a fix
  — re-read move 6 before touching any code.
- **F8b red inside a pinned arm** — e.g. your duplicate-keys arm expected
  serde to keep the LAST value and your generated line accidentally used the
  same kind twice (first == last, divergence invisible). Constrain the
  generator so the two kinds differ, or the pin isn't pinning. Strategy bugs
  are the third triage category; log it as one.
- **`unexpected argument '--parser'`** at runtime — you added the flag to
  `Stats` but not `Validate` (or vice versa). It's on both, by design —
  it's a backend choice, not a filter; the F1 rejection test only covers
  `--type`/`--since`.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                       # no diff
cargo clippy -p glake --all-targets -- -D warnings      # clean, default world
cargo clippy -p glake --all-targets --features lens -- -D warnings
                                                        # clean, lens world
cargo test -p glake                                     # ALL green (the reference suite
                                                        # is 47: 26 unit + 14 CLI + F3 +
                                                        # F8a/F8b + R8 pair + R9 + drift)
PROPTEST_CASES=2000 cargo test -p glake --test prop_parsers
                                                        # confidence run (reference: <1s)
cargo tree -p glake -e normal --depth 1                 # glake + clap + serde_json + thiserror
cargo tree -p glake -e normal --depth 1 --features lens # …+ memlens (F11)
```

```
cargo run -p glake -- stats datalake/raw-local > target/j-hand.txt
cargo run -p glake -- stats datalake/raw-local --parser serde > target/j-serde.txt
diff target/j-hand.txt target/j-serde.txt               # empty — F7 on the real lake
                                                        # (back-to-back; the lake grows)

cargo run -p glake -- validate crates/glake/tests/fixtures/junk.jsonl; echo $?
# → missing key "event_id" … exit 1        (hand: lenient)
cargo run -p glake -- validate crates/glake/tests/fixtures/junk.jsonl --parser serde; echo $?
# → not a JSON object … exit 1             (serde: strict — the split, on screen)
```

- `git log --oneline -5` shows red → trait+hand → serde/F8a → F8b →
  evidence.
- `crates/glake/proptest-regressions/` holds **no** entries for
  `prop_parsers` (by design — say why, citing the design's seed rationale),
  while I's F3 seed (if it earned one) is still committed.
- `specs/004-glake-traits/evidence.md` holds your F9 table, the F11/F12
  records, and the triage story; the triage is also in
  `_assurance/triage-log.md` and the ledger.
- You can answer aloud: why does the trait boundary own (and what forced
  it)? Where is the crate's one `dyn` point, and what do the F13 tests prove
  about everything below it? Name the three F8b divergence classes from
  memory, and which one your first counterexample belonged to. What did
  serde cost per event on your lake, what did it buy, and which number would
  you quote in a Lambda design review?

## Hints (one at a time)

<details><summary>Hint 1 — the F8a generator won't stay escape-free</summary>

Don't fight escaping — make it impossible. Two rules do all the work: build
the object in a `serde_json::Map` and let serde serialize it (valid JSON by
construction, duplicate keys physically impossible), and draw every string
from a charset with no `"`, no `\`, no control characters — like
`"[a-zA-Z0-9 .:_🦀-]{0,16}"` — so serialization is byte-identical to the
input and the hand scanner's raw slice equals serde's unescaped value. If
F8a still finds a disagreement, print the line and look for a required key
whose *value* your generator made a non-string without covering that case in
both backends' sentinel rules (`"non-string"` / `bad-ts` must match).

</details>

<details><summary>Hint 2 — F8b's Case enum shape</summary>

```rust
enum Case {
    Garbage(String),                     // class 1 divergence allowed
    JsonIsh(String),                     // class 1 allowed
    WellFormed(String),                  // exact agreement REQUIRED
    DupKeys { line: String, first_kind: String, last_kind: String, day: String },
    Escaped { line: String, hand_expects: ClassifiedLine, serde_expects: ClassifiedLine },
}
```

Weight the free-range arms heavier (`2/3/3/1/1` works). The free-range
assertion is two-step: hand is never `Unparseable` (the scanner can't say
it), and *if* the verdicts differ, serde's must be `Unparseable` — that's
class 1, and the failure message should print both verdicts. The `DupKeys`
arm renders `event_type` twice with different kinds and asserts hand ==
first, serde == last. The `Escaped` arm's ts case: put a `-` escape
where a `-` belongs in the ts — hand's raw 10-byte prefix fails `is_day` →
`bad-ts`; serde unescapes → a real day. The generator carries both expected
verdicts; the test just delivers them.

</details>

<details><summary>Hint 3 — the serde trace is unreadable in the viewer</summary>

Shrink the experiment first, like F's Hint 3: trace one fixture file per
backend (`MEMLENS_TRACE=target/lens-small-serde.jsonl … stats
crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl --parser
serde`). Two events' worth of serde allocation is a readable story: for each
line, a burst of same-timestamp allocations (keys, values, map spine) that
all die before the next line starts — per-line churn, the `Value` tree's
lifecycle. Now reload the full-lake trace: that burst times every line in
the lake is your wall, and live-bytes between file reads stays *flat* even
for serde — the tree never outlives its line. Peak bytes barely moves
between backends (say why: what dominates peak, and which backend changed
it? Nothing — file contents did, and both read the same files).

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/004-glake-traits/_reference/glake/src/parser.rs` — the trait, the
  `From` boundary, `SerdeParser` (its module doc is the divergence table in
  prose), and the `strictness_divergence_is_as_documented` test.
- `specs/004-glake-traits/_reference/glake/src/tally.rs` — `tally_filtered`'s
  generic signature and the two `f13_*` tests (the `&*boxed` spelling lives
  there).
- `specs/004-glake-traits/_reference/glake/tests/prop_parsers.rs` — the
  `wellformed_line` strategy, the `Case` enum, and both properties.
- `specs/004-glake-traits/_reference/glake/src/cli.rs` — `ParserChoice` and
  `build()` (the one `dyn` seam).
- `specs/004-glake-traits/_reference/NOTES.md` — the full divergence
  write-up, the ten build decisions, and the validation-day F9 numbers this
  worksheet quotes.
