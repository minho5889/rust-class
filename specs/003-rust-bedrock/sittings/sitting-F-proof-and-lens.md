# Sitting F — proof and the payoff

**Builds:** nothing new that computes — this sitting *proves* four claims glake
has been making since Sitting A (std-only, lens-optional, right-on-the-real-lake,
allocates-almost-nothing) and then lets you watch the last one on screen in the
memory lens you built in spec 002.
**Requirements:** R4, R7a, R7b, R10, L6 — tasks 1.9–1.10.
**Ramp you'll use:** step 4 (`&str` vs `String`) and step 8 (lifetimes-lite) —
not to write new code with, but to watch being *true* on a real heap.

## Where you are

Sitting E closed the build: the R9 conservation property went red then green,
`HashMap::entry` tallies both axes, and `glake stats` prints real numbers off
the real lake. Functionally, glake is done — but "done" in this course means
*proven*, and the proofs are exactly the [O]-tagged rows of the requirements
that no `#[test]` can reach: what's in the dependency tree, what's in the
binary, whether the numbers survive a cross-examination by `scan.sh`, and
whether the zero-copy story you've been telling since Sitting C's `<'a>` shows
up as an actual flat line in a trace. Division of labor flips twice today:
task 1.9 is machine checks — **Claude drives, you watch and interrogate** —
and task 1.10 is the lens moment — **you drive, Claude navigates**.

## The build, move by move

All commands from the repo root. Two commits this sitting: one when the lens
wiring lands, one when your observations land in `evidence.md`.

1. **R4, before the lens exists (Claude drives — you predict first).** R4 says:
   with default features, glake's dependency tree is std-only. Before running
   anything, predict what this prints:

   ```console
   cargo tree -p glake
   ```

   It is *not* one line — proptest and its whole subtree appear. Is R4 already
   broken? You answered this in Sitting C when you put proptest under
   `[dev-dependencies]`: R4 governs what glake *ships*, and dev-dependencies
   never enter the shipped binary. The honest spelling of R4 is therefore

   ```console
   cargo tree -p glake -e normal
   ```

   — `-e normal` restricts the tree to normal edges (no dev, no build). Today
   it must print exactly one line: `glake v0.1.0 (…/crates/glake)`. That single
   line *is* R4's baseline. Now we're allowed to add a dependency — as long as
   this command's output never changes.

2. **Wire the lens as an *optional* dependency (Claude drives, you interrogate
   every line).** You've done lens wiring once — exercise 02. But open
   `playground/02-collections-lens/Cargo.toml` and look at its dependency line:
   memlens is *unconditional* there; its `lens` feature only arms the
   recording. Port that pattern into glake and move 1's one-line tree grows a
   permanent memlens subtree — exercise 02's shape **fails R4 by construction**
   (the design's Key-decisions table rejected it for exactly this; audit
   MAJOR-1). glake needs the stronger shape — the dependency itself is opt-in.
   In `crates/glake/Cargo.toml`:

   ```toml
   [dependencies]
   memlens = { path = "../memlens", optional = true }

   [features]
   lens = ["dep:memlens", "memlens/memlens"]
   ```

   Questions to make Claude answer before the commit (this is what "you watch
   and ask" means):

   - Why **two** entries in the feature list? What does `dep:memlens` do that
     `memlens/memlens` doesn't — and what happens if you keep only the first?
     (One of these mistakes is a compile error; the other is far worse. See
     the fights section.)
   - Why the `dep:` prefix at all? Ask what feature Cargo would invent for you
     without it, and why leaking a feature literally named `memlens` into
     glake's public feature set is sloppy.

   Then `src/main.rs`. Two items, and every mention of memlens in this file
   must sit behind the gate, because with default features the crate *does not
   exist* to the compiler:

   ```rust
   #[cfg(feature = "lens")]
   ```

   The two items: the `#[global_allocator]` static (the same two-line pattern
   at the top of exercise 02's `main.rs` — port it, don't reinvent it) and a
   `let _session = memlens::session("glake");` guard as the first line of
   `main` (its `Drop` flushes the trace — spec 002's deterministic-cleanup
   lesson, now in your own tool). Prove both worlds build:

   ```console
   cargo build -p glake
   cargo build -p glake --features lens
   ```

   **Commit point:**

   ```console
   git add crates/glake Cargo.lock && git commit -m "003: sitting F — lens as optional dep behind --features lens"
   ```

3. **R4 after, R7b, and the gates — the claims become commands (Claude
   drives).** Three checks, in this order:

   ```console
   cargo tree -p glake -e normal                     # STILL exactly one line (R4)
   cargo tree -p glake -e normal --features lens     # memlens + libc/serde/serde_json appear
   ```

   The pair is the whole optional-dep lesson in two lines: same crate, same
   `Cargo.toml`, and the dependency is *provably absent* until asked for.
   Then R7b — "no memlens code in the default binary" — checked against the
   actual bytes, not the manifest:

   ```console
   cargo build -p glake
   strings target/debug/glake | grep -ci memlens     # must print 0
   cargo build -p glake --features lens
   strings target/debug/glake | grep -ci memlens     # hundreds (the reference build shows ~360)
   ```

   `strings` dumps every printable run in the binary; a compiled-in memlens
   can't hide, because its very name is embedded in paths, panic messages, and
   the `memlens.*` event-type literals. Note the order matters — both builds
   write the *same* `target/debug/glake`, so check the default build first (or
   rebuild before each grep). Ask Claude: why check the debug binary and not
   release? (Hint: what does the workspace release profile do to symbols —
   and would that hide string literals too?) Finally, the gates, both worlds:

   ```console
   cargo fmt --check
   cargo clippy -p glake --all-targets -- -D warnings
   cargo clippy -p glake --all-targets --features lens -- -D warnings
   cargo test -p glake
   ```

   Write the four numbers down (tree lines, grep counts) — they go into
   `evidence.md` in move 5.

4. **R10 — the cross-check that refuses to match (Claude drives the commands;
   *you* rule on the discrepancy).** R10 says `glake stats datalake/raw-local`
   matches `datalake/queries/scan.sh`. Run both:

   ```console
   cargo run -p glake -- stats datalake/raw-local        # default features — no lens, on purpose
   bash datalake/queries/scan.sh                          # read the "Event inventory" section
   ```

   They will **not** match, and before reading on you must rule: whose bug is
   it? Two pieces of evidence to examine: line 6 of `scan.sh` (look at exactly
   which files its `EVENTS` glob can see) and your own `walk.rs` (which
   directories does it descend into?). Also compare the two by-type tables —
   which *rows* does glake have that scan.sh's inventory is missing entirely,
   and what do their names have in common?

   The ruling, and the arithmetic that makes it a proof rather than a shrug:
   scan.sh's glob is `dt=*/events.jsonl` — it never descends into
   `datalake/raw-local/traces/`, where spec 002's memlens traces live (also
   `.jsonl`, also valid envelope events — the lake dogfoods its own schema).
   Your walk recurses through *every* directory, so glake sees the whole lake.
   **glake counting more than scan.sh is glake being right** — your first tool
   just exposed a blind spot in the standing queries. The reconciliation must
   be exact, though, so collect three numbers back-to-back (the lake grows as
   you work — your own session's hooks are appending to `dt=<today>` right
   now):

   ```console
   cargo run -p glake -- stats datalake/raw-local | head -1    # glake's total,   T
   jq -s 'length' datalake/raw-local/dt=*/events.jsonl         # scan.sh's world, S
   cat datalake/raw-local/traces/dt=*/*.jsonl | wc -l          # trace lines,     L
   ```

   The identity that must hold **to the event**: `T − L = S`. Two more checks
   that pin it per-row: every `memlens.*` row in glake's by-type table must
   sum to exactly `L`, and every *non*-memlens row must equal scan.sh's
   inventory count for that type exactly. The day this worksheet was
   validated (2026-07-09): `T = 221`, `S = 157`, `L = 64`, and
   `221 − 64 = 157` — exact. Your numbers will be bigger; your identity must
   be just as exact. If instead your `T` equals `S` and glake shows no
   `memlens.*` rows at all: your Sitting B walk only enters directories named
   `dt=*` and shares scan.sh's blind spot — Sitting D's checkpoint warned this
   question was coming. Fix the walk (it's an amendment to B's work, one
   commit), and note that R3a's "walked recursively … into every `*.jsonl`"
   always meant the whole tree. Record `T`, `S`, `L`, and the ruling — this
   discrepancy is insight-card material for close-out.

5. **The lens moment — L6, and now *you* drive.** Everything before this was
   bookkeeping; this is why spec 002 was built first. Three parts.

   **Predict first, in writing** (coached-mode rule — the trace grades you):
   for one pass of `stats` over the whole lake, roughly how many heap
   allocations does *the scanner* contribute — every `has_key`, every
   `get_str`, across every line? Which line of your `tally.rs` allocates, and
   how many times does it run? And which single number in the viewer will
   reveal whether your `stats` reads files one-at-a-time or hoards them (you
   made that architecture call somewhere around D/E — the peak-bytes tile
   knows which way you went)?

   **Run traced.** The trace filename comes from the `MEMLENS_PROGRAM` env var
   (the `session("glake")` string only labels the meta header), and the path
   is cwd-relative — so from the repo root:

   ```console
   MEMLENS_PROGRAM=glake cargo run -p glake --features lens -- stats datalake/raw-local
   ```

   Look at the output *before* the viewer: it disagrees with move 4's run —
   more files, more events, possibly a `malformed line(s)` complaint your
   validate has never once made about the real lake. Stop and explain it
   before reading on; the answer is in *when* the trace file is created versus
   when your walk runs. … Ruled? Here's the mechanism: the lens's sink opens
   at the program's *first allocation* — before your walk ever runs — so the
   walk finds glake's own newborn trace in `traces/dt=<today>/` and reads its
   partially-flushed prefix; a buffer boundary can even tear one line in half
   mid-write. You are watching the observer effect: the instrument is part of
   the experiment. That's also why R10 in move 4 was measured with the lens
   *off*, and why any torn line heals once the process exits and flushes
   (`cargo run -p glake -- validate datalake/raw-local` — default features —
   must still exit 0 afterward; prove it).

   **Open the lens.** Open `viewer/memlens.html` in your browser and drop
   `datalake/raw-local/traces/dt=<today>/glake-<pid>.jsonl` onto it. The
   validation-day trace, for scale: ~1,500 lines, ~700 allocations, ~135 KB
   allocated over the whole run — and not one byte of it from `scan.rs`. Your
   numbers will differ; the shape shouldn't. Four hunts, scrubbing with
   `←`/`→`:

   - **The staircases.** The collections-growth panel shows a few tall realloc
     lineages — one per file your walk found. That's `read_to_string` growing
     its `String` by amortized doubling: exercise 02's Lesson-1 staircase,
     wild-caught in your own tool. Tallest staircase = biggest file.
   - **The churn.** Hundreds of tiny bars on the heap timeline, born and dead
     almost at the same `t`. Match them to the one line of your `tally.rs`
     that allocates per event line — and explain why most of those `String`s
     die instantly (what does `entry` do with the key you hand it when the map
     has seen that kind before?). Could a cleverer API spend less? Park that
     thought; it's 004's opening argument.
   - **The flat line.** Between file-read cliffs, watch live-bytes while
     thousands of `has_key`/`get_str` calls execute: nothing. This is Sitting
     C's promise kept — the slices are the line's own bytes — and the
     compile-time proof is two characters long. Say which two.
   - **The peak.** Read the peak-bytes tile and derive it from your own code:
     exactly which `String`s are alive at that moment? The answer tells you —
     without re-reading `main.rs` — whether your stats hoards file contents or
     streams them, and what the memory bill of that D-era architecture choice
     is. (Lambda footnote for the road: this number is the kind that sizes
     memory, and memory is what you pay for.)

   **Write it down.** Append your observations to
   `specs/003-rust-bedrock/evidence.md` (start it from `specs/_template/` if
   this is its first entry): the four machine-check numbers from move 3, the
   `T`/`S`/`L` reconciliation and ruling from move 4, and at least three
   concrete lens observations *with numbers* (peak bytes and what it equals,
   the churn count and its guilty line, the scanner's contribution). Evidence
   is append-only, plain, and yours — this file is what "the learner can
   explain why it's efficient" looks like on paper. **Commit point** (the
   trace is telemetry; it commits with the work):

   ```console
   git add specs/003-rust-bedrock/evidence.md datalake/raw-local/traces
   git commit -m "003: sitting F — R4/R7/R10 proven, L6 lens notes to evidence"
   ```

## Compiler fights to expect

Lighter than C–E — you write little code today — but the optional-dep gate has
teeth, and two of this sitting's fights throw no error at all. All of them go
to the mistake ledger (`learning.*`) as usual.

- **`error[E0433]: failed to resolve: use of unresolved module or unlinked
  crate `memlens`** — on the *default* build, the moment any mention of
  memlens escapes its `#[cfg(feature = "lens")]` gate (a stray `use`, the
  static, the session line). This is the fight that defines optional deps:
  with the feature off, the crate isn't merely unused — it was never compiled,
  never linked, doesn't exist. Every mention must carry the gate; that
  discipline *is* R7b.
- **`unexpected_cfgs` — unexpected `cfg` condition value** — if the string in
  `#[cfg(feature = "…")]` doesn't exactly match the feature name in
  `Cargo.toml` (a typo, a stray capital). Just a warning — until our
  `-D warnings` gate promotes it. Appreciate this lint: a misspelled cfg is
  otherwise *silently false forever*, and this class of bug used to ship.
- **The silent one:** `lens = ["dep:memlens"]` *without* `memlens/memlens`.
  Compiles both ways, runs, exits 0 — and no trace file ever appears, because
  memlens without its own `memlens` feature is compiled as an inert shell (its
  `session` is a no-op). No error code exists for "you configured the
  instrument but never armed it"; the checkpoint's *the-trace-file-exists*
  line is the only tripwire. This is why move 2 made you ask what each half of
  the feature list does.
- **The false alarm:** running plain `cargo tree -p glake`, seeing proptest's
  subtree, and declaring R4 dead. Not a compiler fight — a *reading* fight.
  `-e normal` is the claim R4 actually makes; know the difference between what
  a crate ships and what it needs while being tested.
- **A tooling paper cut:** `grep -c` prints `0` *and exits 1* when nothing
  matches — harmless at the prompt, but if move 3's R7b check ever lands in a
  `set -e` script, the "good" result kills the script. (File it away; it will
  bite some future CI lane instead.)
- **A runtime surprise, not a bug:** the lens run's stats disagreeing with the
  default run, sometimes flagging a malformed line — the observer effect from
  move 5. Your code is fine; the snapshot was of a lake that contained a
  half-written file. If the *default* run ever flags a malformed line, that's
  different — then your tool found something real; bring it to session.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo tree -p glake -e normal                          # exactly one line (R4)
cargo tree -p glake -e normal --features lens          # memlens subtree present
cargo build -p glake && strings target/debug/glake | grep -ci memlens
                                                       # 0  (R7b)
cargo build -p glake --features lens && strings target/debug/glake | grep -ci memlens
                                                       # large (hundreds — R7a wiring is really in there)
cargo fmt --check                                      # no diff
cargo clippy -p glake --all-targets -- -D warnings     # clean, default world
cargo clippy -p glake --all-targets --features lens -- -D warnings
                                                       # clean, lens world
cargo test -p glake                                    # entire suite green: R8 + R9 properties,
                                                       # correctness table, drift test, fixtures, CLI
```

```
# R10, freshly collected back-to-back — the identity must hold to the event:
cargo run -p glake -- stats datalake/raw-local | head -1     # T
jq -s 'length' datalake/raw-local/dt=*/events.jsonl          # S
cat datalake/raw-local/traces/dt=*/*.jsonl | wc -l           # L
# T − L = S, every memlens.* row sums to L, every other row equals
# scan.sh's inventory exactly (validation day: 221 − 64 = 157)

cargo run -p glake -- validate datalake/raw-local; echo $?   # still exit 0, default features,
                                                             # even after the lens run grew the lake
```

- The L6 trace exists at `datalake/raw-local/traces/dt=<today>/glake-<pid>.jsonl`
  (named `glake-…`, not `unnamed-…`) and has been opened in `viewer/memlens.html`.
- `specs/003-rust-bedrock/evidence.md` holds the machine-check numbers, the
  `T`/`S`/`L` reconciliation with your ruling, and ≥3 lens observations with
  real numbers in them.
- `git log --oneline -2` shows both sitting-F commits.
- You can answer aloud: why does glake count more events than `scan.sh`, and
  which of the two is wrong? (Trick question — defend it.) Across the entire
  lake, how many heap allocations did `get_str` make, what in the trace is the
  runtime witness, and which two characters in your code are the compile-time
  proof?

## Hints (one at a time)

<details><summary>Hint 1 — no trace file appears (or it's called unnamed-*.jsonl)</summary>

Three separate knobs, three separate failure smells. (1) The feature: if
`lens = [...]` in `Cargo.toml` lacks the `memlens/memlens` half, memlens is
present but *inert* — no file, no error; check the feature line first. (2) The
name: the filename comes from the `MEMLENS_PROGRAM` env var, not from
`session("glake")` — forget the env prefix and you get `unnamed-<pid>.jsonl`
(which works, but a lake full of `unnamed` files is a lake nobody can query).
(3) The place: the path is relative to the *current directory* — run from the
repo root or the trace lands somewhere surprising. And make sure you actually
ran with `--features lens` at all; the default build is designed to leave no
trace, and it's very good at it.

</details>

<details><summary>Hint 2 — the R10 reconciliation won't balance</summary>

Collect all three numbers within seconds of each other — your own session's
hooks append to `dt=<today>/events.jsonl` as you work, so a `T` from five
minutes ago and an `S` from now can differ legitimately. Balance still off?
Check: (a) you ran glake with *default* features — a lens run writes a new
trace mid-scan and inflates `T` by a moving amount (the observer effect from
move 5); (b) `L` must count *every* file under `traces/`, including a new
`glake-*.jsonl` if you've already done the L6 run — re-run the `wc -l` and
look at which files it listed; (c) blank lines — `jq -s 'length'` counts JSON
documents while glake counts events and skips blanks (R5), so a stray blank
line in an `events.jsonl` would split those by one. Today's lake has none,
which is itself checkable: `glake validate` + a `wc -l` comparison. The
per-type tables are your debugger of last resort: find the *one row* that
disagrees between glake and scan.sh's inventory, and it will name the file
family one of them isn't seeing.

</details>

<details><summary>Hint 3 — the viewer is a wall of bars and you can't find "nothing"</summary>

Absence needs contrast. First shrink the experiment: point the lens at one
small file and keep the junk trace out of the lake by redirecting it —

```console
MEMLENS_TRACE=target/lens-small.jsonl cargo run -p glake --features lens -- stats crates/glake/tests/fixtures/lake/dt=2026-07-01/events.jsonl
```

(`MEMLENS_TRACE` overrides the whole path; `target/` is gitignored.) Drop
*that* into the viewer: a dozen-line file gives a trace you can read event by
event — one `read_to_string` staircase, then the tally's little alloc/dealloc
pairs, and in between, while the scanner chews every line: no marks at all.
That gap *is* the scanner. Now reload the full-lake trace and re-find the same
three shapes at scale: scrub between the file-read cliffs and watch live-bytes
hold flat; the churn pairs are `tally.rs`'s `to_owned` calls (born and dead at
the same `t` whenever `entry` meets a key the map already has); peak bytes is
whichever `String`s your architecture keeps alive at once. If your timeline
shows *more* than these three shapes, click the odd bars — the story panel
names sizes and sequence, and whatever you find goes straight into
`evidence.md` either way.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/003-rust-bedrock/_reference/glake/Cargo.toml` — the `[features]`
  table and the optional `memlens` dependency line. **But** the reference sits
  four levels deep, so its `../../../../crates/memlens` path is wrong for
  yours — from `crates/glake` the lens is one `..` away.
- `specs/003-rust-bedrock/_reference/glake/src/main.rs` — the `LENS` static
  above `main` and the `_session` guard on `main`'s first line: the only two
  `#[cfg(feature = "lens")]` items in the whole crate, which is the point.
