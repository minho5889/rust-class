# Requirements assurance review — 004 glake-traits

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-09
**Inputs:** `specs/004-glake-traits/requirements.md` (rev of 2026-07-09,
awaiting-review) audited against `specs/004-glake-traits/intent.md`; context:
approved `specs/003-rust-bedrock/requirements.md` (+ its design/tasks for
exit-code provenance) and CLAUDE.md pipeline rules.

**Intent-review gate check:** `_assurance/intent-review.md` verdict = 93%
(≥ 80), so no divergent readings were required to surface as open questions.
None do; consistent. The intent review's one noted elaboration (the
"measured comparison" framing) is carried into F9 — traceable, no action.

---

## Findings (ordered by severity)

### MAJOR

**MAJOR-1 — Static dispatch / generics has no acceptance criterion; F7
forecloses it.**
The intent's learning goal is SKILLS **1c**, which names *generics* and
*static vs dynamic dispatch*; the doc's own T2 promises "generics vs `dyn`,
chosen **and measured**." But the criteria deliver only the dynamic half:
F7 mandates runtime selection via `dyn EventParser`, and F9's lens comparison
measures *hand vs serde*, not *static vs dyn*. No F-line ever requires a
generic (`fn stats_with<P: EventParser>`-style) path to exist, be exercised,
or be measured. As written, a build with zero generics satisfies every
criterion while T2 and the intent's "generics" go untaught.
**Fix:** add one [E] line requiring a generic static-dispatch path alongside
the `dyn` path (e.g. the library API is generic; the binary erases to `dyn`
at the flag boundary), and extend F9 (or add an [O] F9b) to record the
static-vs-dyn measurement T2 claims — or amend T2/intent-traceability with an
explicit descope rationale.

**MAJOR-2 — F2 is untestable as written: "day" and "bad-ts" are undefined —
and 003 explicitly left this hole for 004 to close.**
003's approved definition of a proper record says v0 checks only that `ts` is
*present*, "not that `ts` is a real date (**that can come in 004**)." F2 now
depends on exactly that deferred definition: it compares an event's "day" to
a date and excludes "bad-ts" events, but nothing states how a day is derived
(prefix of `ts`? which formats parse? what about events in `traces/` files
with no `dt=` partition?) or what makes a `ts` "bad." A tester cannot decide
whether `2026-7-6`, `2026-07-06T09:00:00Z`, or a numeric `ts` is kept,
excluded, or an error. F2 is also compound: (a) the keep rule, (b) the
bad-ts exclusion rule, (c) the one-line exclusion note — three testable facts
in one line, despite the changelog's claim that "no compound EARS" was
pre-applied.
**Fix:** define "event day" in the criteria preamble (e.g. "the first 10
chars of `ts` iff they match `YYYY-MM-DD`; anything else is bad-ts"), then
split into F2a (keep rule), F2b (bad-ts excluded), F2c (exclusion note).

### MODERATE

**MODERATE-3 — F5's "exit codes unchanged from v0 (2 usage/io, 1 validation
findings)" cites a contract 003's approved requirements never froze.**
003's gated requirements say only "exits **non-zero**" (R1b, R6); the
specific 2/1 codes live one layer down, in 003's approved *design/tasks*
(and 003's sitting A even mis-attributes them to R6). Promoting the codes to
normative status here is the right move — but "unchanged from v0" is a false
provenance claim at the requirements layer, and F5 is also compound: (a) all
fallible lib paths return `Result<_, GlakeError>` (an [O] code-review/clippy
fact, not an [E]), (b) binary maps errors to one stderr line [E], (c) exit
codes 2 usage/io, 1 validation findings [E].
**Fix:** split into F5a [O] (lib error type discipline), F5b [E] (one stderr
line), F5c [E] (exit codes, stated as *normative for v1*, with a note that
they codify v0's built behavior rather than an approved v0 requirement).

**MODERATE-4 — F8 is weaker than the promise the worked example makes.**
The plain-words console shows `--parser serde` producing "(same numbers as
the hand parser — proven by a property test)", but F8 only requires that both
parsers "**classify** identically." Two parsers can agree a line is a valid
envelope yet extract different `event_type`/day values (e.g. escape handling,
duplicate keys, whitespace) — identical classification, different stats, and
F8 still passes while the console claim is false.
**Fix:** strengthen F8 to full observable equivalence: identical
classification *and* identical extracted fields used by stats/filters
(`event_type`, day) for every generated line.

**MODERATE-5 — Filter × `validate` interaction is unspecified.**
F1 applies `--type` to "both commands," but malformed lines — `validate`'s
entire subject matter — have no reliable `event_type` to filter on. Does
`glake validate --type X` still report malformed lines (unfilterable), drop
them, or error? And F2 is silent on whether `--since` applies to `validate`
at all (F1 says "both commands"; F2 doesn't). Whichever answer, it also
feeds F3's partition property (which bucket do malformed lines land in?).
**Fix:** one [E] line pinning filter semantics for `validate` (recommended:
malformed lines are always reported regardless of filters, since filters
predicate on fields malformed lines don't have), and state F2's command scope.

**MODERATE-6 — T4 (modules & lib/bin split, "a public API you'd let a
stranger call") is only fractionally covered.**
The intent's plan text is "glake → **lib+bin** with query traits…", and F12
silently presupposes a lib exists — but no criterion requires the lib/bin
boundary or its shape (what's public: parser trait, filters, stats? does the
bin stay a thin exit-code shell per 003's rev-3 layout?). F12's
C-COMMON-TRAITS review is one facet of T4, not the split itself. If v0's
reference shape already is lib+bin (per intent assumption 3), then say what
v1 must *change* (public API surface, module reorganization).
**Fix:** add an [O] line: the library crate exposes the parse/filter/stats
API (named), the binary contains only arg-parsing + error-to-exit-code
mapping, reviewed in design.

**MODERATE-7 — F7 freezes design shape at the requirements layer and is
compound.**
`dyn EventParser` and "selects at runtime" are *how it works*, not *what must
be true* — design.md's decision to make (and, per MAJOR-1, a decision whose
premature freezing costs a lesson). F7 also bundles four facts: trait exists,
two impls, flag selects, default `hand`.
**Fix:** require behavior ("`--parser hand|serde` selects the backend;
default `hand`; both produce output per F8") and leave the dispatch mechanism
to design — where the T2 static-vs-dyn choice must be a Key-decisions row.

### MINOR

**MINOR-8 — F3 is conservation-only.** kept + excluded = total holds even if
the filter keeps exactly the wrong events (F1/F2 examples only spot-check).
Cheap strengthening: every kept event satisfies the predicate ∧ every
excluded event fails it (or is bad-ts under `--since`).

**MINOR-9 — File-count semantics under filters undefined.** The worked
example prints "2 files · 9 events (filtered from 221)" but F4 pins only the
event numbers. Is "2 files" files-scanned or files-containing-kept-events?
One clause in F4 settles it.

**MINOR-10 — Status/ordering.** Header says `awaiting-review` with
**Assurance: —**; the constitution's assurance-before-attention rule means
the doc should sit in `drafting` until this verdict is embedded. Also the
changelog's "no compound EARS … pre-applied" claim is falsified by F2/F5/F7
— drop or qualify it after the splits.

**MINOR-11 — ID scheme drift.** 003 used `R`-IDs and the constitution's
property-naming example is `prop_r1_…`; 004 switches to `F`-IDs (→
`prop_f3_…`, `prop_f8_…`). Legal but say so once, so test names and
cross-spec references don't confuse (F3 here vs any future R3 citation).

---

## Coverage matrix (intent → criteria)

| Intent element | Covered by | Verdict |
|---|---|---|
| Filters (`--type`, `--since`) | F1–F4 | ✅ (F2 undefined terms — MAJOR-2) |
| thiserror chain / error design | F5, F6 | ✅ (split needed — MODERATE-3) |
| Query traits | F7, F8 | ✅ (design leakage — MODERATE-7) |
| Generics / static-vs-dyn (SKILLS 1c, T2) | — | ❌ MAJOR-1 |
| lib+bin, modules, public API (T4) | F12 partial | ⚠ MODERATE-6 |
| proptest suite | F3, F8 | ✅ |
| API rubric (C-GOOD-ERR, C-COMMON-TRAITS) | F6, F12 | ✅ |
| Measured comparison under the lens | F9 | ✅ (hand-vs-serde only) |
| clap replaces hand args | F10 | ✅ |
| Out-of-scope guard (no async/AWS/new commands) | F11 + "not building" | ✅ |

Readability standard: met — opens with plain words + a worked console
example, criteria clearly fenced below, changelog collapsed. The worked
example itself overstates F8 (MODERATE-4).

---

VERDICT: 70% — Readable, well-scoped, and mostly traceable, but the generics/static-dispatch learning goal has zero acceptance coverage, F2's day/bad-ts semantics are undefined (the exact hole 003 deferred to 004), and three compound lines plus a mis-cited v0 exit-code contract need splitting before a human should spend review time on it.
