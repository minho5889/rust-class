# 004 reference — implementation notes (answer-key build)

Notes from building/validating the v1 reference at `_reference/glake`.
These feed the sitting-J teaching material, the F8 property design, and the
F11 dependency-policy record. (Measured F9 numbers for `evidence.md` come
from the learner's own run — task 1.7; the smoke numbers below just prove
the pipeline works and the lesson is real.)

## F8: where the hand scanner and serde_json legitimately diverge

Exact equivalence on *arbitrary* input is unsatisfiable, so F8 is split
(per the amended requirements): **F8a** proves exact equivalence on
well-formed input; **F8b** proves every divergence on arbitrary input falls
in one of exactly three classes. Both at 512 cases, no failing seeds.

| # | Class | Example input | HandParser | SerdeParser |
|---|---|---|---|---|
| 1 | **Strictness** | `{"a":1` (truncated), `[1,2]`, `not json` | scans anyway → `Malformed { missing: "event_id" }` (first key it can't find) | `Unparseable` |
| 2 | **Escape normalization** | `"event_type":"a\"b"`, or a unicode escape (backslash-`u002d`) for the `-` inside `ts` | raw slice: kind `a\"b`; a ts prefix containing an escape fails the day pattern → `bad-ts` | unescaped: kind `a"b`; the unescaped ts yields a real day |
| 3 | **Duplicate top-level keys** | `{…,"event_type":"k1",…,"event_type":"k2"}` | FIRST occurrence: `k1` | `Value` map keeps LAST: `k2` |

**Status: unspecified-for-glake.** Real envelope writers (the hooks, the
memlens recorder) never emit duplicate keys, escaped `event_type`/`ts`
values, or partial lines, so glake does not define which backend is "right"
there — `parser.rs` documents the behavior as unspecified, and F8b *pins*
it (classes 2–3 arms assert the exact expected divergence, so a silent
behavior change still fails the suite). The hand scanner never returns
`Unparseable`: it has no parse-failure concept.

Why the property can still be airtight: in the F8b mixed strategy the
garbage/JSON-ish arms can't produce all seven required keys (generated key
names are ≤6 chars, never `event_id`), so any line the two backends would
*classify* differently there is necessarily serde-unparseable → class 1.
Class 2/3 inputs are generated deliberately, with the generator carrying
the expected verdict for each backend.

### The planned sitting-J triage discussion

The worksheet has the learner write F8 as a single exact-equivalence
property first (red), then meet a counterexample — most likely a truncated
JSON-ish line (class 1) or the seeded duplicate-key envelope (class 3).
Triage per the pipeline's counterexample protocol:

- **Verdict: spec bug** (the requirement over-promised "both parsers
  classify identically" for inputs where "identical" isn't even
  well-defined) → amend the requirement (F8 → F8a/F8b), re-gate the
  amendment, and *classify* rather than forbid the divergence.
- Not a code bug: neither backend is wrong — they answer different
  questions on degenerate input ("what does a byte-scan see?" vs "what
  does a JSON document mean?").
- Teaching point: this is what "the same semantics" costs when one side is
  a parser and the other is a scanner; equivalence claims need an input
  class attached.
- Log as `test.counterexample` + `test.triage` events in the mistake
  ledger; no failing regression seed is committed (the final properties
  pass — `proptest-regressions/` stays absent by design).

## Semantic decisions made while building (beyond the spec text)

1. **Day rule** (amended F2) lives in one helper pair
   (`classify::is_day`/`day_of_ts`) used by *both* backends and `--since`
   validation. It is a shape check only (`9999-99-99` passes): lexicographic
   order on the shape is all `--since` needs; calendar validity is out of
   scope.
2. **`--since` value validation** happens in `Filter::new` →
   `GlakeError::Usage` → exit 2 (this is what exercises the `Usage`
   variant; clap can't know our date grammar).
3. **`SkipBadTs` counts only since-rule exclusions**: an event that already
   fails `--type` is a plain `Skip` even if its day is bad-ts, so the
   printed note counts exactly the events the F2 rule (and nothing else)
   excluded. The bad-ts note prints only when the count is > 0.
4. **Non-events always pass filters** (`Blank`/`Malformed`/`Unparseable` →
   `Keep`): stats' malformed count is never silently filtered away; the F3
   property asserts `kept.malformed == total.malformed`.
5. **`Unparseable` in `validate`** prints as `<file>:<line>  not a JSON
   object` and counts as a finding (exit 1) — only reachable with
   `--parser serde` (fixture `tests/fixtures/junk.jsonl` pins the split at
   the CLI).
6. **`#[source]` instead of `#[from]`** on `GlakeError::Io` (amended F5):
   `#[from]` can't build a variant that also carries the path, and the path
   context is the point. `GlakeError::io(path, e)` is the `map_err` helper.
7. **Unfiltered stats output is byte-identical to v0** — `(filtered
   from M)` appears only when a filter is active (F4), so 003 checkpoints
   still hold against the v1 binary.
8. **`tally` now consumes `ClassifiedLine`s** (classification moved behind
   the trait); R9's property drives HandParser → tally, same conservation
   assertions, expected days via the shared `day_of_ts`.
9. **F13 shape:** `tally_filtered<P: EventParser + ?Sized>` — unit tests
   call it with `&HandParser`/`&SerdeParser` (monomorphized), the binary
   with `&*Box<dyn EventParser>` (the crate's only dyn seam).
10. **Version bumped to 0.2.0** to mark v1 (the 003 reference stays 0.1.0).

## F11 — dependency policy check (`cargo tree`, recorded 2026-07-09)

Default features (`cargo tree --edges normal --depth 1`):

```
glake v0.2.0 (specs/004-glake-traits/_reference/glake)
├── clap v4.6.1
├── serde_json v1.0.150
└── thiserror v2.0.18
```

With `--features lens`:

```
glake v0.2.0 (specs/004-glake-traits/_reference/glake)
├── clap v4.6.1
├── memlens v0.1.0 (crates/memlens)
├── serde_json v1.0.150
└── thiserror v2.0.18
```

memlens enters the tree **only** with the feature; a full-depth grep of the
default tree finds nothing async (no tokio/hyper/async-* anywhere). Policy
holds: clap + serde_json + thiserror allowed, lens optional, nothing async.

## F9 smoke (debug builds, real lake, 284 events / 5 files — indicative only)

Same command (`stats datalake/raw-local`), identical stdout, per backend:

| Backend | allocs | deallocs | reallocs | trace size |
|---|---|---|---|---|
| `--parser hand` | 737 | 737 | 24 | 336 KB |
| `--parser serde` | 7,148 | 7,148 | 41 | 3.2 MB |

≈ 9.7× the allocations for serde — the owned-`Value`-per-line cost the
hand-first curriculum was built to make visible. The learner re-measures
this for `evidence.md` in task 1.7 (memlens refuses release builds by
design; traces here were redirected via `MEMLENS_TRACE`, not written into
the repo lake).
