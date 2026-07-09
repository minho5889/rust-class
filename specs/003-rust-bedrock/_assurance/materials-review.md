# Materials review — 003 rust-bedrock worksheets & sitting guides

**Reviewer:** adversarial materials critic (fresh context)
**Date:** 2026-07-09
**Scope:** `playground/ramp/step-01…08` (README + solution.rs), `sittings/sitting-A…F`,
checked against `requirements.md`, `design.md`, `tasks.md`, `playground/ramp/README.md`,
the reference implementation in `_reference/glake/`, and the real repo state.

---

## 1. Compile sweep — ALL PASS

`rustc --edition 2024 solution.rs` + run, per step directory:

| Step | Result |
|---|---|
| 01-hello | PASS |
| 02-ownership-move | PASS |
| 03-borrowing | PASS |
| 04-str-vs-string | PASS |
| 05-enum-match | PASS |
| 06-result | PASS |
| 07-read-file (with sample.jsonl) | PASS |
| 08-lifetimes-lite | PASS |

Also fact-checked and **verified true**: the 134-line count for
`dt=2026-07-05/events.jsonl`; `cargo new` inside this workspace really writes
`edition.workspace = true` (cargo 1.94.1); `scan.sh` line 6 is exactly the
`dt=*/events.jsonl` glob; exercise-02's Cargo.toml has the unconditional memlens
dep as F claims; `memlens::session`, `MEMLENS_PROGRAM`, `MEMLENS_TRACE`, and the
`memlens = ["dep:serde", "dep:serde_json", "dep:libc"]` feature all exist as
described; current lake numbers (S=157, L=64) are consistent with F's
T=221/S=157/L=64 identity; every function name cited in the "If truly stuck"
pointers exists in the reference; trace files carry all 7 envelope keys; the
`../../` vs `../../../../` schema-path arithmetic in D and its reference warning
are both correct; `REQUIRED_KEYS` (7 keys incl. `actor`) matches the schema.

---

## Findings, by severity

### MAJOR

**M1 — Sitting F overrules approved requirements (R10) and design (walk spec) without an upstream amendment.**
- `requirements.md` R10 [O]: *"`glake stats datalake/raw-local` on the real lake **matches** the counts from `datalake/queries/scan.sh`."*
- `design.md` walk row: *"folder → every **`dt=*/`** `*.jsonl`"*; R3a: "walked recursively through `dt=…/`".
- `sitting-F` move 4 instead declares: *"They will **not** match"*, rules "glake counting more than scan.sh is glake being right", reinterprets R10 as the `T − L = S` identity, and asserts R3a *"always meant the whole tree"* — telling a learner whose walk only enters `dt=*` dirs (the design-documented behavior, which sitting B move 4 explicitly left as a free choice) to "fix the walk". The reference `walk.rs` implements the whole-tree version, siding with F against the design text.
- CLAUDE.md change protocol: downstream work contradicting an approved doc must halt, amend upstream, re-gate the amendment. That never happened — R10 and the design's walk row need a rev before F is teachable; otherwise the learner's checkpoint literally fails the requirement as written while the worksheet calls it success.

**M2 — Reference implementation contradicts the design's frozen `classify` interface that sitting D orders the learner to hit "precisely".**
- `design.md` / sitting D move 4 freeze: `pub fn classify<'a>(line: &'a str, required: &[&'a str]) -> Line<'a>` with `Malformed { missing: &'a str }`; D move 5 says "hand it `&REQUIRED_KEYS`".
- `_reference/glake/src/classify.rs:32` implements `pub fn classify(line: &str) -> Line<'_>` — **no `required` parameter** — and `Malformed { missing: &'static str }` (classify.rs:27).
- Sitting D's "If truly stuck" pointer sends the learner to exactly this file; D's deliberate teaching question ("which input does `missing` borrow from — `line`, or `required`?") has **no answer** in the reference, which sidesteps it with `'static`. One of design/reference must change; today the stuck learner is shown a shape the design forbids.

### MODERATE

**M3 — Sitting B's opening misstates what Sitting A built.** B's "Where you are" claims the A-era program "panics" on wrong arguments or a missing file, and move 2 says "Sitting A probably indexed `args[1]`". But A's own checkpoint *required* `.get(1)` handling, a usage line on stderr with no panic, and a clean `?`-carried `io::Error` (only the exit code was a tolerated lie). A learner who passed A's checkpoint meets a false diagnosis of their own code; B's "prove the three bad shapes" framing ("all three: usage on stderr, then 2") also implies the pre-existing program fails these, when two of the three already behave except for the code.

**M4 — Sitting E move 8's fixture assertion is a trap that will fail as written.** Stats' by-day buckets come from each line's **`ts`** (design's day-extraction row; reference classify). But the fixtures were built in B move 3 by copying *real lake lines* (ts ≈ 2026-07-05…) into partitions *named* `dt=2026-07-01` / `dt=2026-07-02`. E move 8 tells the learner to "assert … both `dt=` days" — the partition names will not appear in the by-day output; the ts-days of the copied lines will (possibly only one distinct day). Nothing tells the learner to align fixture ts values with partition names. Related mismatch: the requirements' approved worked example prints `dt=2026-07-05  214` (with prefix) while sitting E's target look and the reference print `2026-07-05` — the gated example and the built output disagree.

**M5 — `for` loops are never taught, then load-bearing.** No ramp step teaches `for` (step 6's *solution* uses `for input in [...]` + `{input:>12}` unasked). Step 7 then uses `for line in contents.lines()` as a given and its "You can already" line claims "loop while borrowing instead of taking (**step 3**)" — step 3 contains no loop of any kind. First-loop-ever arriving inside "read a file" doubles the new-concept load in violation of one-concept-per-sitting.

**M6 — Recurring false back-references in the ramp.** (a) Step 2 move 1: "Use `String::from(...)` — you saw this shape in step 1" — step 1 used only a `&str` literal. (b) Expression-return ("last expression, no semicolon") is attributed to **step 1** in three places — step 4 hint 2, step 6 hint 2, step 8 move 4 — but step 1 never teaches it; it is first actually explained in step 5 move 4 (and used silently in step 3's solution). A beginner told to "remember step 1" will go re-read step 1 and find nothing.

**M7 — The ramp commit protocol is unfulfillable as written.** `tasks.md` 0.1 requires one commit per step (`ramp: step N — <concept>`), and CLAUDE.md requires one commit per learner step — but no ramp worksheet ever mentions committing, and steps 1–6 run at play.rust-lang.org, so there is nothing in the repo to commit unless the learner is told to copy their code back (they aren't). Steps 7–8 run locally but also never mention the commit.

### MINOR

**m8 — Playground affordances misdescribed.** Step 3 move 1: "type `s.` in the playground and look for the one about case" — play.rust-lang.org has no autocomplete/method popup. Step 1 move 1: the playground does not give "an empty `fn main() { }`"; its default snippet already contains `println!("Hello, world!");`.

**m9 — "point" vs "move" drift inside the ramp.** Steps 2, 3, 5 call their numbered items "points" ("point-7 experiment", "point 3"); steps 4, 6, 7, 8 and all six sittings say "move". Cosmetic, but a learner-facing course should pick one. (No occurrence of "rung" anywhere — good; "step"/"sitting" otherwise consistent, and all commit messages match the `003: sitting X — …` / `ramp: step N — …` formats.)

**m10 — Counter-type mismatch guide vs reference.** Sitting E's hints and fights say `u32` (`HashMap<String, u32>`, `sum::<u32>()`); the reference uses `u64` throughout. A stuck learner comparing sees an unexplained discrepancy (and, if they mix, an E0308 the fights section doesn't list).

**m11 — L-label traceability gap.** Requirements label the learning goals L1–L6 and task 2.1 demands "L1–L6 notes" in evidence.md, but the sittings cite only **L2** (E move 5) and **L6** (F) by label. L1/L3/L4/L5 are exercised in substance (A/E ownership, D enums, B `Result`/`?`, E iterators+HashMap) but never anchored, so close-out will have to reverse-engineer where each lesson "showed up in the code".

**m12 — Step 7 checkpoint demands output the exercise never specified.** Checkpoint: prints "exactly `3 non-blank lines`"; move 3 only said "print the count". The format exists only in solution.rs.

**m13 — Module-layout drift from the design's Detailed interfaces.** design.md names `stats.rs / validate.rs — folds over (file, line_no, Line<'a>)`; the sittings and reference instead build `tally.rs` (fold over lines only) with `validate`/`stats` as functions in `main.rs`. Reasonable engineering, but it deviates from the approved design without a changelog entry — same protocol issue as M1/M2, smaller stakes.

**m14 — Byte/char wording.** Step 4 move 2 says "slice out the first 10 **characters**" for `.get(0..10)`, which is byte-indexed; the solution's comments get it right, the README wording doesn't.

---

## 2. Coverage matrix

- **Ramp concepts (README table steps 1–8):** each taught by exactly one worksheet, titles and concepts aligned. ✓ (Gap: `for` loops and closures belong to no step — see M5; closures arrive inline in step 6 with a one-line gloss, tolerable.)
- **Requirements:** R1a→D, R1b→D, R2→E, R3a→B, R3b→B, R4→F, R5→C(+D), R6→B, R7a→F, R7b→F, R8→C(+D), R9→E, R10→F. All exercised. ✓ (But see M1: F's R10 treatment contradicts the requirement's text.)
- **Learning goals:** L1–L6 all exercised in substance; only L2/L6 cited by label (m11).
- **Tasks:** 1.1→A, 1.2→B, 1.3/1.4→C, 1.5/1.6→D, 1.7/1.8→E, 1.9/1.10→F. All covered; commit grain per sitting matches tasks.md (A:2, B:3, C:2 red/green, D:5, E:2, F:2). ✓ Task 0.1's per-step ramp commits are uncovered by any worksheet (M7).

## 3. Solution-leak check — PASS (two near-misses)

No sitting guide pastes a multi-line function body from the reference. Stubs
(`todo!()`), type definitions, and commented skeletons only. Near-misses, within
the letter of the rule and behind progressive-disclosure hints: B hint 1
reproduces `main`'s pivotal tuple-match line verbatim
(`match (args.get(1).map(String::as_str), args.get(2))` = reference main.rs:22),
and E hint 2 hands over both the sitting's central decision (owned `String`
keys) and the exact entry idiom line. Defensible as final-tier hints.

## 4. Consistency — see M1–M4, m9, m10, m13

Everything else checked clean: no "rung"; commit formats consistent; all
commands valid for the repo layout (`-p glake`, fixture paths,
`CARGO_BIN_EXE_glake`, `cargo tree -e normal`, `strings`/`grep -c` caveat is
even documented); `ts.get(0..10)`+`bad-ts` consistently taught (E) and
implemented (reference); optional-dep lens shape consistent across design,
tasks, F, and reference.

## 5. Beginner realism — see M5, M6, m8

Otherwise strong: every new API in the sittings is either introduced with its
type spelled out, co-written (proptest), or Claude-driven (F). The Option-match
in A is covered by step 6; raw strings are explained at first use in D;
`std::fmt` width specifiers are flagged as new in E. Sitting B's tuple-match
and `Option::map`/`String::as_str` arrive untaught but only inside Hint 1.

---

VERDICT: 78% — Compile-clean, fully covered, and rigorously fact-checked, but Sitting F re-rules R10 and the walk against the approved specs, the reference classify breaks the design's frozen interface, Sitting E's fixture assertion fails as written, and the ramp threads three false back-references plus an untaught `for` loop.

---

# Re-verification pass — 2026-07-09

**Reviewer:** adversarial materials critic (fresh context, post-fix)
**Inputs:** requirements.md rev 4, design.md rev 3, all six sittings, all eight
ramp steps + solutions, `_reference/glake` (classify signature change), fixtures.

## 1. Compile sweep — ALL PASS (8/8)

`rustc --edition 2024 solution.rs` + run, per step directory: steps 01–08 all
compile and run clean (step 07 verified against its `sample.jsonl`, prints
`3 non-blank lines`, exit 0).

## 2. Reference tests — ALL GREEN

`cargo test` in `_reference/glake`: 7 suites, 17 tests total
(7 unit + 6 cli + 2 prop_scan + 1 prop_tally + 1 schema_drift), 0 failed.

## 3. Finding-by-finding

- **M1 — FIXED.** requirements.md rev 4: R3a now *"walked recursively into every
  `*.jsonl` file beneath it — `dt=` partitions and `traces/` alike"* (line 118);
  R10 now the reconciliation identity *"glake's total = scan.sh's process-event
  count + the lines of memlens trace files … 221 = 157 + 64 (amended rev 4 from
  'matches')"* (line 132), changelog entry present. design.md rev 3 walk row:
  *"every `*.jsonl` beneath it, recursively (`dt=` partitions and `traces/`
  alike — rev 3)"* + changelog row citing the change protocol. Sitting F move 4
  now opens *"R10 (requirements rev 4) is an identity, not an eyeball match"*;
  Hint 2(d) cites *"R3a (rev 4) scopes the walk to every `.jsonl` beneath the
  path"*. Zero overruling language remains (grep for "will not match" / "being
  right" / "always meant": no hits). Sitting B move 4 now says the walk scope
  *"is not a free choice: requirements rev 4 fixed R3a's scope … so recurse
  into every directory"* — the old free-choice contradiction is gone.
- **M2 — FIXED.** `_reference/glake/src/classify.rs:38`:
  `pub fn classify<'a>(line: &'a str, required: &[&'a str]) -> Line<'a>`;
  line 31: `Malformed { missing: &'a str }`; tests call
  `classify(GOOD, &REQUIRED_KEYS)`. The module doc now *answers* D's teaching
  question: *"`missing` borrows from `required` — the key names live in the
  caller's slice, not in the line."* Sitting D (lines 162, 380) and design.md
  (line 79) carry the identical signature; the "If truly stuck" pointer now
  lands on a reference that matches the frozen interface.
- **M3 — FIXED.** Sitting B "Where you are" now correctly credits A: *"It
  already fails without panicking — a missing argument prints a usage line on
  stderr, a missing file surfaces a clean `?`-carried `io::Error` — but the
  exit *codes* still lie."* Move 2: *"Sitting A's `.get(1)` match already turns
  'no arguments' into a usage line on stderr — that behavior is right"* and
  *"the stderr half you've had since A; the honest `2` is what this move adds."*
- **M4 — FIXED.** B move 3 now mandates *"One edit per copied line: change its
  `ts` so the day agrees with the partition"* and explains why; the reference
  fixtures comply (dt=2026-07-01 lines carry ts 2026-07-01, dt=2026-07-02 carry
  2026-07-02 — verified by grep). E move 8 asserts *"the two days the output
  must show are `2026-07-01` and `2026-07-02` … **as bare days** … no `dt=`
  prefix"* plus a debugging pointer back to fixture ts. Requirements worked
  example now prints the bare day (`2026-07-05           214`, line 30; rev 4
  changelog notes the correction).
- **M5 — FIXED.** Step 7 move 3 introduces the `for` loop on its own before it
  does real work (*"One new piece of syntax, met on its own … `for item in
  collection { ... }`"* with a warm-up); step 6's solution no longer contains
  any `for` loop; step 7's "You can already" line no longer attributes looping
  to step 3.
- **M6 — FIXED.** (a) Step 2 move 1: *"Use `String::from(...)` — new here"*.
  (b) Expression-return: step 4 hint 2 introduces it as *"new trick"*; step 5
  move 4 teaches it in the main flow (*"no `return`, no semicolon … the `match`
  **is** the function body"*); step 6 hint 2 and step 8 move 4 now back-reference
  **step 5**. No remaining attribution to step 1.
- **M7 — FIXED.** All eight step checkpoints carry a "Save what YOU wrote"
  line naming `my-solution.rs` and the exact commit (`ramp: step N — <concept>`),
  e.g. step 1: *"paste your playground code into `my-solution.rs` next to this
  README, then commit it — `ramp: step 1 — hello`"*; steps 7–8 use the local-file
  variant (*"your working file already lives in this folder — name it
  `my-solution.rs`, then commit"*). Format matches tasks.md 0.1.
- **m8 — FIXED.** Step 1 move 1: *"It greets you with a `fn main` that already
  contains `println!(\"Hello, world!\");` — delete that starter line"*. Step 3
  move 1: *"the playground has no autocomplete, so open the std `String` docs
  and scan the method list"*.
- **m9 — PARTIAL.** All READMEs now say "move" (step 2 "move 7", step 3
  "move-7 experiment", step 5 "move-3"/"move 6"). Residue in solution-file
  comments only: `step-03-borrowing/solution.rs:14` ("point-7 experiment"),
  `step-05-enum-match/solution.rs:19` ("point-6 punchline") and `:36`
  ("point-6 addition") still say "point".
- **m10 — FIXED.** Sitting E is `u64` throughout (`HashMap<String, u64>`,
  `sum::<u64>()`, `&mut u64`); zero `u32` occurrences remain in the sitting.
- **m11 — PARTIAL.** L-labels now anchored in the sittings: L1 (A moves 4/fights,
  E fights), L2 (C moves+fights, E), L3 (D move 4), L5 (E intro + move 5), L6 (F
  throughout). **L4 is still cited by label in no sitting** — only in
  requirements.md and a reference `main.rs` comment; sitting B (its natural
  home: `Result` + `?` + ExitCode) never says "L4".
- **m12 — FIXED.** Step 7 move 4 specifies the exact output —
  *"print in exactly this shape: `println!(\"{count} non-blank lines\");` — the
  checkpoint checks that wording."*
- **m13 — FIXED.** design.md rev 3 Detailed interfaces: *"tally.rs — pure fold
  over lines (property-testable); validate/stats are thin functions in main.rs
  that own I/O and exit codes (rev 3 layout)"*, with a changelog row citing the
  change protocol; matches the reference (`src/tally.rs` exists, no
  stats.rs/validate.rs) and the sittings.
- **m14 — FIXED.** Step 4 move 2: *"Slice out the first 10 **bytes** by calling
  `.get(0..10)` … (string ranges index bytes, not characters)"*.

## 4. Regression check — CLEAN

- **Solution leaks:** no sitting or step guide pastes a reference function
  body. Sitting D hint 3 remains a commented skeleton; the two previously
  accepted near-misses (B hint 1 tuple-match line, E's entry idiom in hint
  code) are unchanged, nothing new leaked. The new classify body (the
  `required` loop) appears nowhere outside the reference.
- **Terminology:** zero occurrences of "rung" in any learner-facing file.
- **Commit formats:** all 13 sitting commit points still match
  `003: sitting X — …`; all 8 ramp checkpoints match `ramp: step N — <concept>`.
- **New-content consistency:** sitting F's R10 numbers (T=221, S=157, L=64,
  221−64=157) match requirements rev 4's authored identity; reference `walk.rs`
  recurses every directory (R3a rev 4); reference fixtures' ts↔partition
  alignment holds; compile sweep and full reference test suite green post-change.

## 5. Remaining items (both minor, no teaching impact)

1. m9 residue: three "point-N" comments in step 3/5 solution files.
2. m11 residue: L4 unanchored in sitting B.

RE-VERDICT: 96% — All seven majors and five of seven minors verified fixed with specs amended per the change protocol and everything compiling and testing green; only two cosmetic residues remain (three "point-N" solution comments in steps 3/5, and L4 still lacking a by-label anchor in sitting B).
