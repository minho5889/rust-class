# Design — 004 glake-traits (glake v1)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

v0's straight pipeline gets one new joint and two new muscles:

```
 clap CLI ──► walk ──► EventParser (trait) ──► filter(closures) ──► tally/report
                        ├─ HandParser  (your 003 scanner)              │
                        └─ SerdeParser (serde_json)          GlakeError everywhere
```

The big design move: `classify` goes behind a **trait**, and the trait
boundary returns an **owned** value:

```rust
pub enum ClassifiedLine {                 // derives Debug, Clone, PartialEq, Eq
    Blank,
    Unparseable,                          // serde-only: body isn't a JSON object
    Malformed { missing: String },        // first REQUIRED_KEY not found
    Event { kind: String, day: String },  // day per the requirements' day rule
}
```

Why owned? Not because traits forbid borrows (they don't — a
`fn classify<'a>(&self, line: &'a str) -> Line<'a>` is legal `dyn` or not).
The forcer is `SerdeParser`: it parses into its own local `serde_json::Value`,
and you cannot return borrows of a local. One backend needs ownership, so the
*shared* boundary needs ownership. That's the honest lesson: v0's zero-copy
pipeline vs v1's owned trait boundary is exactly the allocation difference the
lens run (F9) makes visible. Abstraction has a price; this spec puts a number
on it.

## The shape

| Part | What changes | Rust you learn | REQs |
|---|---|---|---|
| **cli.rs** | `clap` derive struct: command, path, `--type`, `--since`, `--parser`; filter flags on `validate` → usage exit 2 | derive macros, how clap owns usage/exit-2 | F1, F2a/b, F10 |
| **parser.rs** | `trait EventParser { fn classify(&self, line: &str, required: &[&str]) -> ClassifiedLine }` + `HandParser` (wraps the v0 scanner; never returns `Unparseable` — the scanner has no parse-failure concept; allocates only at the boundary) + `SerdeParser` (parse to a local `Value`, extract; non-object/invalid JSON → `Unparseable`, tallied as malformed) | traits, why the boundary owns | F7, F8a/b |
| **pipeline (lib)** | the core walk→classify→tally path is `fn run<P: EventParser>(…)` — **generic, monomorphized**; unit tests call it with each parser directly; only `main` wraps the choice in `Box<dyn EventParser>` | static vs dynamic dispatch, side by side | F13, F7 |
| **filter.rs** | `Filter { kind: Option<String>, since: Option<String> }` → `fn verdict(&self, e: &ClassifiedLine) -> Verdict { Keep, Skip, SkipBadTs }` (a bare `bool` can't feed F2b's excluded-bad-ts count); tallied with iterator adapters + `match` | closures, iterator adapters, Option combinators | F1–F4 |
| **error.rs** | `#[derive(thiserror::Error)] enum GlakeError { Io{path, #[source] source}, Usage(String) }`; io errors wrapped via a small `map_err` helper that attaches the path (`#[from]` can't — multi-field variant); lib returns `Result<_, GlakeError>`, `main` maps to stderr + exit code | error design, source chains, C-GOOD-ERR | F5, F6 |
| **lib/bin split** | already the reference shape; the learner's own crate refactors here (sitting G) | modules, `pub` discipline, API rubric | F12 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Trait return type | `Line<'a>` (borrowed) · **owned `ClassifiedLine`** | owned | `SerdeParser` parses into a local `Value` and can't return borrows of it; one backend needing ownership forces the shared boundary to own — the T6 lesson, measured in F9 |
| Dispatch | generics everywhere · `dyn` everywhere · **generic lib pipeline + one `Box<dyn>` at the CLI seam** | both, deliberately | F13 makes the pairing testable; one `dyn` at the outermost seam is idiomatic CLI shape |
| Backend disagreement | force exact equivalence · **equivalence (F8a) + contained divergence (F8b)** | split | exact equivalence is *unsatisfiable*: the hand scanner is lenient (truncated/trailing-junk lines) and keeps escapes raw; serde is strict and unescapes. Denying that teaches a lie; fencing it with a property teaches a boundary |
| `--since` vs `bad-ts` | error · include · **exclude + count** | exclude + count | no comparable day exists; silent inclusion lies, erroring punishes old data — count keeps R9-style conservation visible |
| Filter composition | trait objects · **plain struct + verdict method** | struct + verdict | smallest thing that teaches closures/adapters and still yields F2b's count |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| F3 | ∀ event sets, ∀ filters: kept + excluded = total (per kind and per day too) | reuse 003's line-set generator; filter drawn from {none, type∈generated kinds ∪ absent, since∈generated days ∪ extremes} |
| F8a | ∀ well-formed lines: `HandParser ≡ SerdeParser` (verdict + `event_type` + day) | constrained generator: valid JSON objects, unique top-level keys, escape-free extracted values; equality on `ClassifiedLine` |
| F8b | ∀ lines: parsers agree **or** the divergence ∈ {serde-`Unparseable` vs lenient classify, escape normalization in extracted values, duplicate top-level keys} | 003's R8 full mixed strategy (garbage + JSON-ish + valid); the test *classifies* each divergence and fails on any unclassifiable one |

No "expected failing seed" here (rev 2): duplicate keys and escape divergences
are **documented classes inside F8b**, not counterexamples to commit — a seed
that a refined strategy would regenerate differently guards nothing. Any
*unclassifiable* divergence F8b finds is a genuine bug and gets normal triage.

## How we verify

- F3/F8a/F8b proptests (≥256, seeds committed on genuine failure) written
  before their code; F13 monomorphized unit tests.
- [E] fixtures extend 003's; clap usage (incl. `validate --type` → exit 2)
  tested via `CARGO_BIN_EXE`.
- [O] F6/F12 rubric review and the F11 `cargo tree` check recorded in
  evidence (owned by a named task); F9 lens comparison with both parsers on
  the real lake, numbers in evidence.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-3 directive | pending combined ack |
| 2026-07-09 | Rev 2 per design+tasks audit (62%): `ClassifiedLine` defined (variants + derives F8 needs); the dyn/borrow justification corrected (borrows *can* cross `dyn` — the forcer is serde's local `Value`); F8 split into F8a/F8b with divergence classes replacing the incoherent expected-seed plan; generic pipeline row added (F13); filter `keep()->bool` → `verdict()` enum (F2b count); `#[from]` → `#[source]`+path helper; F11/F12 verification ownership named | 004 audits | this combined gate |

</details>
