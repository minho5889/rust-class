# Assurance audit — 004 glake-traits: design.md + tasks.md (combined)

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-09
**Audited:** `design.md` (vs approved-track `requirements.md`), `tasks.md` (vs `design.md`)
**Also read:** 003 design/tasks (approved), 003 `_reference/glake` source (scan.rs,
classify.rs, lib.rs, Cargo.toml), intent.md, CLAUDE.md conventions.

---

## MAJOR findings

### MAJOR-1 — F8 equivalence is unsatisfiable as designed against the real 003 scanner; the design anticipates the wrong counterexample

Design (Properties table + divergence note) plans for exactly one expected
counterexample: duplicate top-level keys (serde last-wins vs hand first-wins).
Reading the actual reference scanner (`_reference/glake/src/scan.rs`), there are
at least **three divergence classes that will fire before and more often than
duplicate keys**, and the design addresses none of them:

1. **Strict vs lenient parsing.** The hand scanner accepts any line where the
   seven keys appear at depth 1 — including truncated envelopes (missing final
   `}`), trailing garbage after the object, unquoted/odd values elsewhere.
   `serde_json` errors on all of these. The F8 strategy explicitly reuses 003's
   R8 mix "garbage + JSON-ish + valid envelopes" whose JSON-ish arm *includes
   truncations* — so hand→`Event` / serde→parse-error divergences are
   guaranteed within the first runs. `SerdeParser`'s behavior on unparseable
   input is nowhere defined.
2. **Escape handling.** `get_str` documents that escape sequences are returned
   **raw** ("unescaping would require allocating"); `serde_json` unescapes.
   A *fully valid* envelope with `\"`, `\\` or `\uXXXX` in `event_type` or `ts`
   diverges — and the R8 strategy explicitly generates escapes. This also leaks
   outside the property: `--type` (F1) exact-match behaves differently per
   parser on escaped kinds.
3. **Non-string / odd values.** Hand maps non-string `event_type` to the
   `"non-string"` bucket and short/multibyte `ts` to `"bad-ts"` via
   `ts.get(0..10)` on the *raw* slice; `SerdeParser` must replicate both rules
   (including 0..10 byte-slicing of the *unescaped* string — which differs from
   slicing the raw text). Unstated.

**Fix:** the design must define `SerdeParser` semantics precisely (parse-failure
→ what `ClassifiedLine`? unescape → compared how?) and reconcile with F8's "for
any generated line ... classify identically". Realistic options to put in the
Key-decisions table: (a) define equivalence only over serde-parseable lines
(`prop_assume!`/generator precondition) plus [E] tests pinning each parser's
junk behavior — this needs an F8 requirements amendment + re-gate; or (b) define
canonical classification rules both impls must meet (raw-slice day/kind rules
stated as the spec, hand-scanner-compatible), accepting parse-failure fallback
rules. Either way, decide *before* the sitting: this is a pre-authored,
pre-validated course — a reference implementation cannot be "validated" until
these semantics exist.

### MAJOR-2 — the planned duplicate-key triage is not resolvable by the rule the design gives, and the regression-seed plan is incoherent

The design says triage "decides whether hand or serde is 'right' per the
*requirements'* definition (top-level keys-present)". But keys-present
adjudicates *presence* only; the duplicate-key divergence is about which
**value** wins (`get_str` first-occurrence vs serde last-wins). The
requirements are silent on value extraction under duplicates, so by the
constitution's triage taxonomy this is a **spec bug** (amend requirements,
re-gate the amendment) — not something a sitting can settle by pointing at F8.

Coherence problem with the committed seed: per CLAUDE.md, failure seeds go to
`proptest-regressions/` and must replay green after the fix. proptest replays a
seed **through the current strategy**:
- If triage lands "test bug — refine the strategy to avoid duplicates", the old
  seed now generates a *different* input and passes vacuously — the regression
  guard guards nothing, and the semantic question stays open. Not acceptable as
  the plan of record.
- If triage lands "code bug — fix one parser", the seed replays the same input
  and must pass. That's the coherent path, but it forces a choice the design
  dodges: make `HandParser` last-wins (changes 003's frozen scanner semantics —
  change-protocol ripple into 003's correctness table), or make `SerdeParser`
  first-wins (requires bypassing `Value`'s map, i.e. not really "serde parses
  it" anymore).

**Fix:** pre-commit the resolution in design.md now (recommended: amend
requirements to define duplicate-key lines' classification explicitly, then
implement both parsers to that definition; keep the *staged red* as the lesson,
with the amendment as its scripted ending). Also note: nothing guarantees the
strategy even *generates* duplicate keys in 256–512 cases — if this
counterexample is "planned", the generator must deliberately produce
duplicate-key lines (or an [E] fixture must pin it deterministically).

### MAJOR-3 — tasks.md asserts materials exist that do not, and no task creates them

Tasks: "All materials pre-authored and validated against the 004 reference" and
0.2's "Worksheets + validated solutions". Reality: `specs/004-glake-traits/`
contains only the four docs — no `sittings/`, no `_reference/`, and
`playground/ramp/` has no step-09..11 (only README.md). Unlike 003, where the
equivalent claim was true at gate time (six sitting guides + a 17/17-green
reference exist on disk), here the claim is false and **no task item authors or
validates the Part-3 materials or the 004 reference**. Since MAJOR-1/2 can only
be settled by building that reference, this is the missing first main task.

**Fix:** add a task 0.1 "author + validate ramp steps 9–11, sittings G–J
worksheets, and the 004 reference implementation (all checkpoints compile/pass;
materials-critic review)" — or gate tasks.md only after the materials exist.

---

## MODERATE findings

### MODERATE-1 — `ClassifiedLine`, the design's central type, is never defined
The trait returns it, `Filter::keep` takes it, F8 asserts equality on it — but
design.md never gives its variants/fields. It must mirror
`Blank | Malformed { missing: String } | Event { kind: String, day: String }`
(F8 covers blanks and malformed too, so all three variants must survive the
boundary), and must derive at least `Debug + PartialEq` (the F8
`prop_assert_eq!` won't compile without them) — which is also exactly the F12
C-COMMON-TRAITS conversation. 003's approved design had a "Detailed interfaces"
section; 004 needs the same, especially for pre-authored worksheets.
**Fix:** add the interfaces block: `ClassifiedLine` with derives, `Filter`,
the `--parser` → `Box<dyn EventParser>` mapping (clap `ValueEnum`).

### MODERATE-2 — F11 has no design element and no verifying task
No design row cites F11; "How we verify" covers F3/F8/[E]/F6/F12/F9 and skips
it; tasks/ops have no dependency-policy check (003 had the R4 `cargo tree`
check as task 1.9). "Nothing async" and "lens stays optional-dep" are exactly
the kind of silent drift a one-line `cargo tree` assertion catches.
**Fix:** add F11 to the verify section and an ops-checklist line
(`cargo tree` inspection: only clap/serde/serde_json/thiserror added, no
tokio/async-*, memlens still optional).

### MODERATE-3 — the filter interface cannot produce the F2/F4 numbers as shaped
`fn keep(&self, e: &ClassifiedLine) -> bool` applied as `.filter(|e| f.keep(e))`
yields kept events only. F4's `M` (pre-filter total) is recoverable, but F2's
"one-line note says how many [bad-ts] were excluded" needs the *reason* for
exclusion, which a bool predicate erases. The design says "excluded counted,
incl. the bad-ts rule" without saying how.
**Fix:** name the mechanism — e.g. `keep` returns `Keep | Excluded(Reason)`
(nicer enum lesson), or a documented pre-pass counting
`since.is_some() && day == "bad-ts"`.

### MODERATE-4 — F5's `#[from]` is unimplementable against the design's error shape
F5 says "(`thiserror`, `#[from]` io source chain)"; the design's
`Io { path, #[source] source }` cannot use `#[from]` (thiserror forbids `#[from]`
on variants with extra fields). The design's shape is the *better* one
(C-GOOD-ERR wants the path context) but it silently contradicts the
requirement's letter while both docs sit in the same gate.
**Fix:** amend F5's parenthetical to "source chain (`#[source]`)" in the
combined gate, or note the deviation in design's Key decisions.

### MODERATE-5 — the "In plain words" dyn/borrow justification teaches a false rule
"the borrowed `Line<'a>` can't cross a `dyn` boundary that outlives its input"
is wrong as a general claim — borrowed returns cross `dyn` boundaries fine
(`fn classify<'a>(&self, line: &'a str, ...) -> Line<'a>` is object-safe and
works for `HandParser`). The *actual* forcer is per-impl: `SerdeParser`'s
strings live in a locally-owned `serde_json::Value` dropped at return. The
Key-decisions row states this correctly; the plain-words paragraph — the part
the learner absorbs — does not. In a spec whose whole point is this lesson,
the imprecision is a teaching hazard.
**Fix:** reword plain-words to match the Key-decisions row.

### MODERATE-6 — F12 review and validate-under-filter semantics are unowned
(a) The F12 API-rubric review appears only in the ops checklist; no sitting
task performs it, though "a public API you'd let a stranger call" is the
intent's stated learning goal — it should be a named activity in sitting G or J
(learner walks the checklist, Claude coaches). (b) F1 says filters apply to
"both commands", but the design never says what `validate --type t` does with
`Malformed` lines (they have no `event_type`; are they still reported?).
**Fix:** add an F12 review step to a sitting; add one sentence defining
malformed-line behavior under filters (and cover it in an [E] fixture).

---

## MINOR findings

- **512 vs ≥256 case-count drift:** tasks "Done means" says 512-case property;
  design and the ops checklist say ≥256. Pick one number.
- **Unvalidated `--since`:** `Option<String>` + lexicographic ≥ means
  `--since garbage` silently filters everything. Decide: clap value-parser
  validation (exit 2, feeds F10) or document the behavior.
- **`GlakeError::Usage(String)` may be vestigial** once clap owns
  usage/exit-2; if nothing constructs it, clippy/dead-code will complain.
  State what still produces it (or drop it).
- **Task numbering starts at 0.2** with no 0.1 in this doc (continuation from
  003's 0.1); confusing standalone — renumber or add the missing 0.1 (which
  MAJOR-3 wants anyway).
- **Traceability polish:** the Properties table and shape table never cite T1–T6
  against sittings the way tasks do; harmless, but the F9/T6 linkage lives only
  in prose.

## What's good (keep it)

- Both [P]s have Properties rows with genuinely concrete strategies that reuse
  validated 003 generators; test-first ordering is explicit in tasks (1.3, 1.5
  red before implementation) — the 003 MAJOR-4 lesson stuck.
- Key decisions table has real alternatives and real reasons; the
  exclude+count `--since`/bad-ts decision is well argued and conservation-aware.
- Coached-mode discipline is clean: learner writes G–J, Claude confined to
  close-out machine work, "you drive" named on the lens run.
- Every design element cites REQ IDs; no scope creep found.
- The owned-boundary-measured-by-the-lens arc (F9/T6) is a genuinely good
  design move — it just needs the F8 semantics underneath it to be real.

## Coverage matrix

| REQ | Design element | Task | OK? |
|---|---|---|---|
| F1 | cli/filter rows | 1.4 | ✔ (validate semantics gap, MOD-6b) |
| F2 | filter row + decision | 1.4 | ✔ (count mechanism gap, MOD-3) |
| F3 | Properties row | 1.3 | ✔ |
| F4 | filter row | 1.4 | ✔ |
| F5 | error.rs | 1.2 | ✔ (`#[from]` conflict, MOD-4) |
| F6 | error.rs + verify | 1.2 | ✔ |
| F7 | parser.rs | 1.6 | ✔ |
| F8 | Properties row | 1.5/1.6 | ✖ MAJOR-1/2 |
| F9 | verify | 1.7 | ✔ |
| F10 | cli row + verify | 1.4 | ✔ |
| F11 | — | — | ✖ MOD-2 |
| F12 | lib/bin row + verify | ops only | ◐ MOD-6a |

---

VERDICT: 62% — Well-shaped, traceable, test-first docs undermined by an F8 equivalence plan that misidentifies its counterexamples (strict-parse and escape divergences will fire before the "planned" duplicate-key seed, whose triage rule can't actually resolve it), plus a false pre-authored-materials claim and an uncovered F11.
