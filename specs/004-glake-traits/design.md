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

The big design move: `classify` goes behind a **trait**. That forces the trait
boundary to return an **owned** value (`ClassifiedLine` with `String`s) — the
borrowed `Line<'a>` can't cross a `dyn` boundary that outlives its input. This
is deliberate: v0's zero-copy pipeline vs v1's owned trait boundary is exactly
the allocation difference the lens run (F9) makes visible. Abstraction has a
price; this spec puts a number on it.

## The shape

| Part | What changes | Rust you learn | REQs |
|---|---|---|---|
| **cli.rs** | `clap` derive struct: command, path, `--type`, `--since`, `--parser` | derive macros, how clap owns usage/exit-2 | F1, F2, F10 |
| **parser.rs** | `trait EventParser { fn classify(&self, line: &str, required: &[&str]) -> ClassifiedLine }` + `HandParser` (wraps the v0 scanner, allocates only at the boundary) + `SerdeParser` (parse to `Value`, extract) | traits, `dyn` vs generics, `Box<dyn EventParser>` from `--parser` | F7, F8 |
| **filter.rs** | `Filter { kind: Option<String>, since: Option<String> }` → `fn keep(&self, e: &ClassifiedLine) -> bool`; applied as `.filter(|e| f.keep(e))`; excluded counted, incl. the `bad-ts`-under-`--since` rule | closures, iterator adapters, Option combinators | F1–F4 |
| **error.rs** | `#[derive(thiserror::Error)] enum GlakeError { Io{path, #[source] source}, Usage(String) }`; lib returns `Result<_, GlakeError>`, `main` maps to stderr + exit code | error design, source chains, C-GOOD-ERR | F5, F6 |
| **lib/bin split** | already the reference shape; the learner's own crate refactors here (sitting G) | modules, `pub` discipline, API rubric | F12 |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Trait return type | `Line<'a>` (borrowed) · **owned `ClassifiedLine`** | owned | a `dyn` boundary can't return borrows of serde's internal `Value`; the forced allocation IS the T6 lesson, measured in F9 |
| Dispatch | generics everywhere · **`Box<dyn EventParser>` at the CLI seam, generics inside** | both, deliberately | teaches static vs dynamic side by side; one `dyn` at the outermost seam is idiomatic CLI shape |
| `--since` vs `bad-ts` | error · include · **exclude + count** | exclude + count | no comparable day exists; silent inclusion lies, erroring punishes old data — count keeps R9-style conservation visible |
| Filter composition | trait objects · **plain struct + closure** | struct + closure | smallest thing that teaches closures; predicate traits are overkill at this size |

## Properties (co-written, test-first)

| REQ | Property | Generation strategy |
|---|---|---|
| F3 | ∀ event sets, ∀ filters: kept + excluded = total (per kind and per day too) | reuse 003's line-set generator; filter drawn from {none, type∈generated kinds ∪ absent, since∈generated days ∪ extremes} |
| F8 | ∀ lines: `HandParser.classify ≡ SerdeParser.classify` | 003's R8 mixed strategy (garbage + JSON-ish + valid envelopes); equality on the owned `ClassifiedLine` |

Divergence rule for F8 counterexamples: triage decides whether hand or serde is
"right" per the *requirements'* definition (top-level keys-present) — expected
seed: duplicate keys, where serde keeps the last and the hand scanner finds the
first. That triage is a planned teaching moment, not a surprise.

## How we verify

- F3/F8 proptests (≥256, seeds committed) written before their code.
- [E] fixtures extend 003's; clap usage tested via `CARGO_BIN_EXE`.
- [O] F6/F12 rubric review recorded in evidence; F9 lens comparison with both
  parsers on the real lake, numbers in evidence.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-3 directive | pending combined ack |

</details>
