# Requirements — 004 glake-traits (glake v1)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

glake grows up. Three things change, and each one is a lesson:

```console
$ glake stats datalake/raw-local --type gate.approved --since 2026-07-06
2 files · 9 events (filtered from 221)
...

$ glake stats datalake/raw-local --parser serde
(same numbers as the hand parser — proven by a property test)

$ glake validate /root/forbidden
glake: cannot read /root/forbidden: permission denied (os error 13)
```

1. **Filters** (`--type`, `--since`) — your first *closures and iterator
   adapters* doing real work.
2. **A designed error type** — one `GlakeError` enum (`thiserror`) instead of
   ad-hoc prints; the user still sees one clear line, scripts still get the same
   exit codes.
3. **The crates arrive, on trial.** `clap` replaces your hand-rolled args;
   `serde_json` becomes a *second* parsing backend behind a `trait`, selectable
   with `--parser hand|serde`. A property test proves both backends agree on
   well-formed input — and a second property maps out exactly *where and why*
   they can disagree on garbage (your scanner is lenient and keeps escapes
   raw; serde is strict and unescapes — that's not a bug, it's a boundary, and
   you'll prove its shape). Then you run both under the **lens** and *see*
   what the hand-written version was saving you. That measurement is the
   whole reason 003 made you do it by hand first.

## What we're *not* building yet

- Async, HTTP, tokio → **005**. Anything AWS → **006+**. No new commands.

## What you'll learn building it

- **T1 · Traits** — define one (`EventParser`), implement it twice.
- **T2 · Static vs dynamic dispatch** — generics vs `dyn`, chosen *and measured*.
- **T3 · Error design** — `thiserror`, source chains, the C-GOOD-ERR rubric.
- **T4 · Modules & lib/bin split** — a public API you'd let a stranger call.
- **T5 · Closures + iterator adapters** — filters as composable predicates.
- **T6 · The cost of abstraction, observed** — trait-boundary allocations in the
  lens, hand vs serde.

---

## Precise acceptance criteria

> Tags: **[P]** property · **[E]** example · **[O]** operational.

> **The day rule, defined once** (normative for v1; F2/F8 use it): an event's
> **day** is the first 10 characters of its `ts` **iff** they match the
> `YYYY-MM-DD` digit pattern (4 digits, dash, 2, dash, 2); otherwise the event
> is **bad-ts**. This tightens v0's bucketing (which took any 10-char prefix);
> v1's stats put pattern-violating prefixes in the `bad-ts` bucket.
> Duplicate top-level JSON keys are **unspecified input** — real envelope
> writers never emit them; the two backends may disagree there (see F8b).

**Filters** *(stats only — `validate` given a filter flag exits 2 with usage:
malformed lines have no `event_type` or day to filter on)*
- **[E] F1** — `--type <t>` keeps only events whose `event_type` equals `<t>` exactly.
- **[E] F2a** — `--since <YYYY-MM-DD>` keeps events whose day ≥ the date (string comparison of comparable days).
- **[E] F2b** — under `--since`, bad-ts events are excluded (no comparable day) and one line reports how many were excluded.
- **[P] F3** — filter partition: for any generated event set and any filter, (kept) + (excluded) = (unfiltered total); filters never invent or lose events.
- **[E] F4** — filtered stats show both numbers: `N events (filtered from M)`.

**Errors**
- **[E] F5** — the binary maps every failure to one clear stderr line; exit codes, **normative for v1**: 2 usage/io, 1 validation findings, 0 clean.
- **[O] F6** — all fallible library paths return `Result<_, GlakeError>` (`thiserror`, `#[source]`-chained io errors carrying the offending path), and `GlakeError` passes the C-GOOD-ERR checklist (Display lowercase-no-period, `std::error::Error + Send + Sync`, sources chained).

**The trait + backends**
- **[E] F7** — an `EventParser` trait with two impls: `HandParser` (the 003 scanner) and `SerdeParser` (`serde_json`); `--parser hand|serde` selects the backend at runtime, default `hand`.
- **[P] F8a** — backend equivalence where it must hold: for any generated **well-formed** line (valid JSON object, no duplicate top-level keys, extracted values escape-free), both parsers produce the identical classification — verdict, `event_type`, and day.
- **[P] F8b** — divergence containment everywhere else: for any generated line from the full mixed strategy (garbage, JSON-ish, valid), the parsers either agree or disagree inside exactly three documented classes — (1) serde-unparseable where the lenient scanner still classifies, (2) escape normalization in extracted values, (3) duplicate top-level keys. Any divergence outside those classes is a bug.
- **[O] F9** — the lens comparison: the same stats run under `--features lens` with each parser; allocation counts + trace sizes recorded in `evidence.md` (the measured cost of `serde` and of the owned trait boundary).
- **[E] F13** — static dispatch exists alongside dynamic: the lib's core pipeline is generic over `P: EventParser` and unit tests call it monomorphized with each parser directly; the bin's `--parser` seam is the only `dyn` point.

**CLI & hygiene**
- **[E] F10** — args via `clap` derive; usage/help auto-generated; bad usage still exits 2.
- **[O] F11** — dependency policy: `clap`, `serde`, `serde_json`, `thiserror` now allowed; still nothing async; lens stays optional-dep; verified by a recorded `cargo tree` check.
- **[O] F12** — the crate is split lib + thin bin, and the lib's public API is reviewed against C-COMMON-TRAITS (Debug/Clone/PartialEq where applicable).

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft (with design+tasks); 003 audit lessons partially pre-applied (properties in design, test-first, optional-dep lens; the "no compound EARS" claim proved false — see rev 2) | Part-3 directive | pending combined ack |
| 2026-07-09 | Rev 2 per requirements audit (70%) + design audit (62%): F13 added (static dispatch had zero coverage — MAJOR-1); day/bad-ts rule defined normatively (MAJOR-2, the hole 003 deferred here); F8 split into F8a equivalence + F8b divergence containment (design MAJOR-1: exact equivalence unsatisfiable — lenient/raw-escape scanner vs strict/unescaping serde); duplicate keys declared unspecified input (design MAJOR-2: old triage plan incoherent); F2/F5 de-compounded, exit codes normative-for-v1, `#[from]`→`#[source]`+path (unimplementable as drafted); filters stats-only (validate has nothing to filter); F12 owns the lib/bin split | 004 audits | this combined gate |

</details>
