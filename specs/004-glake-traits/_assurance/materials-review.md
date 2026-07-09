# Materials review — Parts 3 & 4 (adversarial, fresh-context)

**Scope:** this single review covers **both spec 004 (glake-traits) and spec 005
(async-relay)** — ramp steps 9–13 + updated ramp README, sittings G–J and K–N,
their rev-2.x requirements/design/tasks, and both `_reference/` builds.
A pointer file lives at `specs/005-async-relay/_assurance/materials-review.md`.

**Reviewer:** materials-critic (blind, fresh context). **Date:** 2026-07-09.
**Method:** every claim checked against the rev-2.x docs and the references;
key checkpoints **executed**, not read — see "What was run and held" at the end.

**Verdict: 88% — the checkpoints are real (everything executed reproduces
verbatim), but two reconciliation misses leave gated docs contradicting each
other at the gate.**

Findings: **2 MAJOR · 3 MODERATE · 7 minor.**

---

## MAJOR

### MAJOR-1 · The generic pipeline is named `fn run<P: EventParser>` in tasks.md and step 10 — the design (rev 2.1) and the reference say `tally_filtered`

- `specs/004-glake-traits/tasks.md:58` — task 1.6: "the generic pipeline
  (`fn run<P: EventParser>`)".
- `playground/ramp/step-10-generics-vs-dyn/README.md:12` — "the spec-004
  pipeline stays *generic* (`fn run<P: EventParser>` — requirement F13)".
- `playground/ramp/step-10-generics-vs-dyn/README.md:22` — "that's why glake
  v1's core is `fn run<P: EventParser>`".

Contradicted by:

- `specs/004-glake-traits/design.md:45` (shape table: the generic pipeline is
  `fn tally_filtered<P: EventParser + ?Sized>(…)`) and `design.md:90` — the
  rev 2.1 changelog *explicitly* records: "generic pipeline named as built
  (`tally_filtered<P: EventParser + ?Sized>`; **`run` stayed the CLI
  dispatcher**)".
- `_reference/glake/src/tally.rs:90` (`pub fn tally_filtered<'a, P>`) and
  `src/main.rs:51` (`fn run(cli: Cli)` — a plain, non-generic bin dispatcher).
- Sitting J itself teaches the correct shape
  (`sitting-J…md:150-159` shows the `tally_filtered` signature verbatim).

Why MAJOR: two gated docs contradict each other **at the combined gate**
(tasks.md vs design.md on the very artifact F13 names), and the worksheet
plants a function name the learner will never find — the F13 tests are
`f13_*` in `tally.rs`, calling `tally_filtered`, and there is no generic `run`
anywhere. The rev-2.1 "materials reconciliation" fixed design.md but missed
tasks.md 1.6 and both step-10 mentions. Three one-line edits.

### MAJOR-2 · Sitting N tells the learner the A8 allow-list "doesn't name" tracing-subscriber — requirements rev 2.2 added it

- `specs/005-async-relay/sittings/sitting-N-drain-flush-prove-observe.md:246-251`
  — "And `tracing-subscriber`: the A8 allow-list says 'tracing' — is the
  subscriber a violation? … **the honest answer is that A8's text doesn't
  *name* it. Record the question in `evidence.md` for the close-out gate**".

Contradicted by:

- `specs/005-async-relay/requirements.md:128-130` — A8 as amended: "`tokio`,
  `axum`, `serde_json`, `clap`, `thiserror`, `anyhow` (bin only), **`tracing` +
  `tracing-subscriber`** allowed".
- `requirements.md:146` — rev 2.2 changelog: "A8 allow-list gains
  `tracing-subscriber` … **the audit question 'does the list match reality'
  answered**".

Why MAJOR: the guide stages a spec-gap discovery exercise around a gap that
the same authoring pass already closed. The learner would dutifully record a
false finding in `evidence.md` and carry it to the close-out gate. The rev 2.2
amendment and this sitting-N paragraph are two halves of one reconciliation;
only one landed. One-paragraph fix (turn it into "read the rev 2.2 changelog —
this exact question was asked and answered before you got here", which is the
same teaching beat sitting J uses for F8).

---

## MODERATE

### MODERATE-1 · `let-else` is handed to the learner as "D's friend" — sitting D (and the entire 1→13 / A→F arc) never teaches it

- `specs/004-glake-traits/sittings/sitting-I-clap-and-filters.md:421-422` —
  Hint 2: "`let Line::Event { kind, day } = line else { return Verdict::Keep };`
  — non-events flow through (**let-else, D's friend**)".
- Verified: `grep` over `specs/003-rust-bedrock/sittings/` and
  `playground/ramp/step-0*` finds **zero** let-else occurrences; sitting D's
  only `else` is a comment (`sitting-D…md:397`). The reference's
  `filter.rs:78` uses let-else, so a learner who peeks meets it cold too.

Concept-before-taught plus a false continuity attribution. Fix: either a
two-line syntax introduction in the hint ("new syntax, read it as…") or
respell the hint with `match`/`if let` (step 5 material).

### MODERATE-2 · A5 is written after the writer exists; 005 design.md promises both properties "written before the writer task"

- `specs/005-async-relay/design.md:92` — "A4/A5 proptests (≥256 cases, seeds
  committed) **written before the writer task**."
- `specs/005-async-relay/tasks.md:59-65` — 1.4 builds the writer, then "**A5
  partition property** (glake-agrees check) green".
- `specs/005-async-relay/sittings/sitting-M-the-one-owner.md:338-372` — A5 is
  move 7, after the writer (moves 5–6), and the guide is candid: "Likely green
  on the first run — and that's honest: the red-first rhythm spent its red on
  A4."

The materials' sequencing is defensible (and honestly argued), but the design's
"How we verify" line was not reconciled to it — the same class of miss as
MAJOR-1, lower stakes because no learner-visible instruction contradicts
another; only the design's verification sentence is stale. Fix: amend
design.md:92 ("A4 written before the writer; A5 immediately after, per the
honest-red protocol") in the rev-2.x pass.

### MODERATE-3 · F11's allowed-crate list: requirements say four crates, both sittings say "exactly three"

- `specs/004-glake-traits/requirements.md:88` — F11: "`clap`, **`serde`**,
  `serde_json`, `thiserror` now allowed".
- `specs/004-glake-traits/sittings/sitting-H-errors-by-design.md:47-49` —
  "F11 amends the dependency policy to allow **exactly** `clap`, `serde_json`,
  `thiserror`".
- `specs/004-glake-traits/sittings/sitting-J…md:188-189` — serde_json is "the
  last of **F11's three** permitted arrivals".
- Reality (`_reference/NOTES.md:100-123`, verified by running `cargo tree`):
  the tree has exactly clap + serde_json + thiserror; `serde` is never a
  direct dep.

The sittings match reality; the requirement text lists a crate nothing uses.
A learner running H's move-1 check against F11's letter would expect a
four-name allow-list. Fix requirements.md F11 (drop `serde` or mark it
"transitively via serde_json") — it's the normative doc.

---

## minor

1. **"the ten build decisions" — there are eleven.**
   `sitting-J…md:594` says NOTES holds "the ten build decisions";
   `specs/004-glake-traits/_reference/NOTES.md:58-98` numbers 1–11. (005 gets
   its equivalent right: sitting-N:573 says "fourteen", NOTES has 14.)

2. **004 requirements console sketch is internally implausible.**
   `requirements.md:13-14` shows `… --type gate.approved --since 2026-07-06` →
   `2 files · 9 events (filtered from 221)`; sitting I (`sitting-I…md:313-314`)
   records the same command on validation day as `5 files · 2 events (filtered
   from 288)`. The file count is never filtered (verified against the binary),
   so the same lake showing 2 files at 221 events and 5 files at 288 events on
   the same date doesn't cohere. Illustrative only, but the sketch sits in the
   gate-facing "In plain words" block.

3. **`.inspect` appears untaught.** `sitting-J…md:161-162` — "the pipeline …
   `.map(…)` once, **`.inspect` counts events**, `.filter` applies the
   verdict". Ramp 11 taught filter/map/collect/filter_map; `inspect` is never
   introduced anywhere in 1→13/A→N. One parenthetical ("`inspect`: peek
   without consuming — `map` that returns the item") would close it.

4. **Step 9 overclaims what the property proves.**
   `playground/ramp/step-09-traits/README.md:12` — "a property test will prove
   both honor the contract **identically**". F8a/F8b (and sitting J's whole
   dramaturgy) exist because "identically, full stop" is false. If the
   overclaim is deliberate foreshadowing of J's move-6 counterexample, it
   works — but the worksheet states it as fact, not as the gut belief J will
   demolish. Suggest hedging ("…prove exactly where they agree — and fence
   where they may not").

5. **Step 10 "same body" vs different label.**
   `step-10…README.md:16` — move 3 says `announce_dyn` has the "*same body*"
   as `announce_generic`, but the checkpoint (line 31) and Hint 2 print
   `[dynamic]` vs `[static]`. Trivially confusing for a literal reader.

6. **Seed-committing philosophy has a wrinkle across I and J.**
   `sitting-I…md:139-141` commits the `todo!()`-red F3 seed as "house rule —
   it's a genuine red"; `sitting-J…md:286-292` defines the rule as for
   "genuine failures of **living** tests" and deletes J's own stub-red seed as
   guarding nothing. The distinction (F3 survived under its name; J's test was
   renamed) is real but never stated — one sentence in J's move 7 would
   pre-empt the learner asking why I's stub seed stays.

7. **Ambiguous fixture inventory wording.** `sitting-I…md:280-281` — "the
   reference's fixture lake (5 valid events, one bad-ts, one malformed, one
   blank…)" reads as 5+1 events; actually 5 events total *of which* one is
   bad-ts (verified against the fixtures and the quoted `filtered from 5`
   outputs). The quoted outputs disambiguate, the prose doesn't.

---

## What was run and held (verification log)

Everything below was executed, not eyeballed; all matched the materials.

- **004 reference**: `cargo test` → **47/47 green** (26 unit + 14 CLI + 1 F3 +
  2 F8 + 2 R8 + 1 R9 + 1 drift — exactly sitting J's checkpoint breakdown);
  `clippy --all-targets --features lens -- -D warnings` clean; F3/F8a/F8b at
  512 cases as claimed; `PROPTEST_CASES=2000` confidence runs: prop_filter
  3.2s ("~3s" claimed), prop_parsers 0.4s ("<1s" claimed).
- **004 CLI outputs** (fixture lake + junk.jsonl): all six quoted transcripts
  in sitting I move 7 / checkpoint and sitting J checkpoint reproduce
  **verbatim** — `3 files · 3 events (filtered from 5)`, the bad-ts note, the
  no-note compose case, `--type gate` → 0, clap's `unexpected argument
  '--type'` exit 2, `--since 2026/07/02` funnel line exit 2,
  `missing key "event_id"` vs `not a JSON object` both exit 1,
  `glake: cannot read /no/such/path: … (os error 2)` exit 2.
- **005 reference**: `cargo test` → **14/14 green** with exactly NOTES'
  breakdown (5 unit / 2 CLI / 4 routes / 2 props @256 / 1 shutdown, real
  `kill -INT`); lens clippy clean; A1 fixture verified **byte-identical** to a
  real live-lake envelope (`datalake/raw-local/dt=2026-07-09`, `spec_id`
  present as promised); test names match the guides
  (`a6_sigint_drains_flushes_and_exits_zero`, `a7_*`, `prop_a4/a5`).
- **Ramp solutions 9–11**: compile under `rustc --edition 2024` and print
  exactly the checkpoint outputs (four lines / `8 vs 16` + batch lines / the
  six-entry-then-five-entry kinds lists, `5 non-blank`, `2 hits`, `1 blank`).
- **Ramp solutions 12–13** (isolated copies): step 12 prints ≈402/≈201 ms and
  `hare wins`; step 13 prints `collector owns 12 lines:` with shuffling order.
- **Claimed-error variants, all provoked verbatim**: E0046, E0186, E0599
  (+"implemented but not in scope" help) for step 9; E0308 mixed-vec and E0277
  with the exact "you could box the found value" help for step 10; E0502
  (three arrows), E0382 (+clone-before-moving help), E0631 for step 11;
  E0308 "expected `u64`, found future", E0728 "this is not `async`", and the
  `unused implementer of Future` warning for step 12; E0382
  "in previous iteration of loop", and the codeless
  `future cannot be sent between threads safely` + `Rc<String>` help +
  `required by a bound in tokio::spawn` note for step 13; sitting H's macro
  wall `deriving From requires no fields other than source and backtrace`
  reproduced against thiserror 2 verbatim.
- **Continuity anchors that held**: sitting E's `impl IntoIterator` promissory
  note exists (`sitting-E…md:64`); 003 reference suite is **17 tests** as
  sitting G claims; `HashMap::entry` taught in E; E0515 in C; E0004 in step 5;
  step 7 move 7 is the first `.filter` closure; root `Cargo.toml` is
  `members = ["crates/*"]` with the thin-LTO/panic-abort profile step 12/K
  describe; F9 numbers (737 vs 7,148, ≈9.7×; ≈103 allocs/event) consistent
  across NOTES and both sittings; MEMORY confirms the learner hasn't started,
  so the ramp README's "step 1 ready" status is accurate, not stale.
- **Terminology/conventions**: zero occurrences of "rung" anywhere in scope;
  all commit messages match the house shapes (`ramp: step N — …`,
  `004:/005: sitting X — …`); every worksheet/sitting has plain-English
  opener, hint ladder, checkpoint, commit lines, and the don't-peek
  convention (including 12–13's save-first ritual); all ramp README relative
  links resolve; relay's stray `_reference/relay/datalake/` trace files are
  untracked and covered by the crate `.gitignore` (working as designed).
