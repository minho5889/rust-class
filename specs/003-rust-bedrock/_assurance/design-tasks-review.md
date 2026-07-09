# Assurance review — 003 rust-bedrock: design.md + tasks.md (fast-path combined)

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-09
**Audited:** `design.md` (against approved `requirements.md` rev 2) and `tasks.md`
(against both). Prior gates: intent 90%, requirements 72% → rev 2 → approved.

Both docs meet the readability standard (plain-words opener, detail below,
collapsed audit trail) and the overall shape — six coached sittings after the
ramp, learner writes, properties co-written — is faithful to the constitution's
coached mode. But the audit found two design elements that as specified would
*fail their own requirements' checks*, one latent panic, a constitution-rule
reversal on property-test ordering, and a pedagogy hole (lifetimes arrive
un-ramped, and the design misstates *when* they arrive). Findings ordered by
severity.

---

## MAJOR findings

### M1 — The lens feature pattern, copied as specified, fails R4 (design.md)

design.md: *"The lens hookup is the one exception, behind `--features lens`,
copied from exercise-02's two-line pattern."* But exercise-02's `Cargo.toml` is:

```toml
memlens = { path = "../../crates/memlens" }   # unconditional dependency
[features]
lens = ["memlens/memlens"]                    # only toggles memlens's internal feature
```

`memlens` is in the dependency tree **always**; the feature only turns its
recording on. R4's operational check — "`cargo tree` [with default features] is
std-only; `memlens` appears only under `--features lens`" — would fail, and R7b
(no memlens code in the default binary) is left to LTO luck rather than
construction. **Fix:** design must specify `memlens = { …, optional = true }` and
`lens = ["dep:memlens", "memlens/memlens"]`, with the `#[global_allocator]` block
behind `#[cfg(feature = "lens")]`. Note it explicitly as a *deviation from* the
exercise-02 pattern, since Sitting F (task 1.7) says "wired exercise-02-style."

### M2 — Schema-file dogfooding is undesigned and the scanner can't do it (design.md → tasks 1.5)

The design's flagship decision — read the required-key list from
`datalake/schema/envelope.v1.json` "using your own `get_str`-style scanning" —
doesn't survive contact with the actual file:

- `get_str`/`has_key` are specified for **one-line, top-level JSON objects**;
  the schema file is **pretty-printed multi-line** JSON.
- The required list is a JSON **array of strings** (`"required": [...]`);
  `get_str` returns a top-level **string** value. There is no designed function
  that can extract it.

Sitting D (task 1.5) depends on this mechanism existing, so a beginner hits an
unplanned design problem mid-sitting. **Fix:** design the actual mechanism —
e.g., read the whole file, locate the `"required"` key, and extract the quoted
strings from its `[...]` span (a small `get_str_array` sibling, or even a
documented "find the line, split on quotes" v0 helper), with its own action item
in Sitting D. Alternatively join lines before scanning. Any of these is fine;
*unnamed* is not.

### M3 — Latent panic: `day = &ts[0..10]` (design.md shape table; tasks 1.6)

R8 guarantees the *scanner* never panics, but stats slices the ts value with a
raw byte range. A present-but-short `ts` (e.g. `"ts": "x"`) — which counts as a
*valid* event under rev-2's keys-present-only rule — panics with a byte-index
out of range; a multibyte char straddling index 10 panics on a char boundary.
This contradicts the doc's own "never a panic" ethos, and the R9 generator as
sketched ("mixed valid/blank/broken lines") won't necessarily produce it.
**Fix:** specify `ts.get(0..10)` and decide the bucket for the `None` case
(suggest: an explicit `day = "(bad ts)"` bucket so R9 conservation still holds);
extend the R9 generation strategy to include short and multibyte `ts` values.
This is also a nice teaching beat (`get` vs indexing).

### M4 — Property tests come *after* the code — constitution reversal (tasks.md)

CLAUDE.md: each [P] gets "a `proptest` test written **before** the code it
tests"; the tasks template repeats it ("first action items of their sub task").
tasks.md inverts both: 1.4 (R8 property) follows 1.3 (scanner written), "then it
hammers *your* scanner"; 1.6 co-writes R9 after the tally is built in the same
item. Coached mode may justify flexing this — but then it must be a **declared
deviation**, and the doc's own template-deviation note doesn't mention it.
**Fix (preferred):** reorder — write the `get_str`/`has_key` *signatures* stub
first, co-write the R8 property against them at the top of Sitting C, then
implement until green (test-first is also better pedagogy: the property finds
the escape/nesting bugs for the learner). Same for R9 before the tally. Or add
one line declaring and justifying the deviation.

### M5 — Explicit lifetimes arrive un-ramped, and the design misstates when (design.md + tasks.md)

design.md calls `Line<'a>` "the design's one deliberate stretch-goal lesson …
we'll meet it in the **last sitting**." False on both docs' own terms:

- `fn get_str<'a>(line: &'a str, key: &str) -> Option<&'a str>` needs an
  **explicit lifetime annotation** (elision fails with two `&str` params) — that
  is **Sitting C**, the hardest sitting.
- `Line<'a>` + `classify` land in **Sitting D**.

The ramp (steps 1–7) never covers explicit lifetimes; SKILLS 1b has Lifetimes
`[ ]` not started. A complete beginner meets their first lifetime annotation
while simultaneously writing a state-machine scanner. **Fix:** add a ramp step 8
("why elision fails; writing `'a`", 30 minutes, two toy functions) or a short
scripted opener to Sitting C; correct the "last sitting" sentence in design.md.
Secondary nit: `classify<'a>(line: &'a str, required: &'a [String])` unifies two
unrelated lifetimes (`missing` borrows from `required`, `Event` fields from
`line`) — it compiles but teaches a muddy pattern; consider `missing: String`
(one honest allocation) or two lifetime params *as* the stretch lesson.

## MODERATE findings

### D1 — Sittings C, D, E each exceed one concept-cluster for this learner

- **C** (task 1.3): the "~40-line" scanner must handle quotes, escapes, *and*
  top-level-only matching (`payload` is a nested object → brace-depth tracking).
  That's a state machine with three interacting concerns; 40 lines is
  optimistic and one sitting more so. **Fix:** split — C1: flat scanner
  (no escapes, no nesting) + unit tests; C2: hardening (escapes + depth) +
  the R8 property (which per M4 should lead).
- **D** (task 1.5): schema-file reading (see M2) + `Line<'a>` (see M5) +
  validate reporting + fixture tests = four things. Move the schema-read helper
  into its own half-sitting or into C2 where the scanner is fresh.
- **E** (task 1.6): iterators + `HashMap::entry` are *new* concepts (not in the
  ramp; requirements changelog flags them as untracked SKILLS items), plus R6
  error paths, plus co-writing R9. **Fix:** move R6 stderr/exit handling to
  Sitting B (where exit-code plumbing is built anyway) and let E be tally + R9.

### D2 — Scanner correctness has no example tests planned

The fixture [E] tests cover R1a–R6 only; the R8 property proves *no panic*, not
that `get_str` returns the *right* slice. Nothing anywhere asserts
`get_str(r#"{"a":"b"}"#, "a") == Some("b")`, or the escape/nesting cases Claude
is supposed to "coach through." **Fix:** an explicit action item in Sitting C:
learner-written `#[test]`s for get_str/has_key happy path + escapes + nested
key must NOT match (this last one guards the R1a false-positive risk via
`payload`'s inner keys).

### D3 — design.md is missing two template sections it actually needs

- **No Key decisions table.** "Two design choices worth knowing" gives choices
  but no *options considered / why-not*. At least three decisions deserve rows:
  schema-file-at-startup vs hardcoded list (rejected by requirements — cite it),
  char-scanner vs regex-free line splitting, `&str`-borrowing enum vs owned
  fields. Alternatives make the human gate real.
- **No formal Properties table** (REQ / ∀-statement / generation strategy). R8's
  strategy is concrete in prose (50/50 `any::<String>()` + JSON-fragment
  generator, one-level nesting, seeds committed — good). R9's is one clause
  ("generate mixed valid/blank/broken lines") — no weights, no statement of
  *what unit* the property targets (the tally function over `Vec<Line>`? the
  whole binary?), no type/day pools small enough to force collisions. **Fix:**
  add the table; for R9: target the pure tally function; strategy = vec of
  lines drawn from {valid event with kind ∈ 3-element pool, day ∈ 2-element
  pool, short-ts event (see M3), blank, whitespace, malformed} with stated
  weights.

### D4 — Per-element REQ citations are missing (template rule)

The template requires each design element to cite its REQ IDs ("### Handler
(R1, R2)"). The five-part shape table and the interfaces block cite none;
traceability currently lives only in the verification section, so the
args/walk/scan/classify/report → REQ mapping is reconstructible but not
*visible*. All of R1a–R10 are in fact covered (verified: R1a/b→classify+report,
R2→stats, R3a/b→walk, R4/R7a/b→features, R5→scan, R6→args/walk errors,
R8/R9→properties, R10→ops check) — so this is a labeling fix, not a coverage
hole: add a REQ column to the shape table.

## MINOR findings

- **L-IDs are cited but never defined.** design.md cites "L6" and tasks 2.1
  cites "L1–L6", but requirements.md's learning goals are *unnumbered* bullets.
  The mapping is guessable (bullet order) but an auditor shouldn't have to
  guess. Fix: a one-line legend in design.md ("L1=ownership … L6=lens"), since
  requirements.md is approved and shouldn't be edited for this.
- **"Six sittings, one commit each" is contradicted two paragraphs later:**
  Sitting C has two commits (C, C2) and task 1.7 has no commit of its own
  (folded into 1.8). Harmless — CLAUDE.md's real rule is one commit per learner
  step — but fix the opener's claim ("six sittings, seven commits") so the
  reviewer isn't approving an inaccurate summary.
- **Unknown-command → usage + exit 2** (task 1.2) cites no REQ — technically
  uncited behavior. It's obviously right; suggest tucking it under R6's
  umbrella in the design ("bad invocation" alongside "bad path") rather than
  leaving it orphaned.
- **Task 1.8 driver is implicit.** Sittings A–E state who writes; 1.8 ("run …,
  open the trace, notes → evidence.md") doesn't say the learner drives. One
  word fixes it — the lens *moment* should be the learner's hands.
- **Positive notes** (so the human knows what *not* to re-review): ramp
  correctly precedes construction and matches steps 1–7 exactly; "steps" not
  "rungs"; commit message format matches CLAUDE.md; teardown-line omission is a
  properly declared template deviation; the Rust-concepts column maps cleanly
  onto SKILLS 1a/1b including the flagged-at-requirements iterators/HashMap
  additions; ops checklist includes the property-auditor pre-close run;
  no scope creep beyond std-only intent found.

---

## Suggested revision order

1. M1 (`optional = true` dep) and M3 (`ts.get(0..10)`) — two-line design fixes.
2. M2 — design the schema-read mechanism (one new paragraph + one interface line).
3. M4 — reorder properties test-first in tasks.md (or declare the deviation).
4. M5 + D1 — add ramp step 8; resplit sittings C/D/E; fix "last sitting" claim.
5. D2–D4, minors — table/labeling passes.

VERDICT: 68% — Readable, well-shaped, and fully requirement-covered, but two design elements would fail their own requirements' checks (lens dep pattern, schema dogfooding), a latent `&ts[0..10]` panic, property-tests-after-code contradicting the constitution, and un-ramped lifetimes mean the human would be approving known landmines; one focused revision pass should clear it.
