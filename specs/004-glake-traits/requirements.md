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
   with `--parser hand|serde`. A property test proves both backends agree — and
   then you run both under the **lens** and *see* what the hand-written version
   was saving you. That measurement is the whole reason 003 made you do it by
   hand first.

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

**Filters**
- **[E] F1** — `--type <t>` keeps only events whose `event_type` equals `<t>` exactly (both commands).
- **[E] F2** — `--since <YYYY-MM-DD>` keeps events whose day ≥ the date; `bad-ts` events are excluded when `--since` is used (they have no comparable day) and a one-line note says how many were excluded.
- **[P] F3** — filter partition: for any generated event set and any filter, (kept) + (excluded) = (unfiltered total); filters never invent or lose events.
- **[E] F4** — filtered stats show both numbers: `N events (filtered from M)`.

**Errors**
- **[E] F5** — all fallible library paths return `Result<_, GlakeError>` (`thiserror`, `#[from]` io source chain); the binary maps them to one clear stderr line; exit codes unchanged from v0 (2 usage/io, 1 validation findings).
- **[O] F6** — `GlakeError` passes the C-GOOD-ERR checklist (Display lowercase-no-period, `std::error::Error + Send + Sync`, sources chained).

**The trait + backends**
- **[E] F7** — an `EventParser` trait with two impls: `HandParser` (the 003 scanner) and `SerdeParser` (`serde_json`); `--parser hand|serde` selects at runtime (`dyn EventParser`), default `hand`.
- **[P] F8** — backend equivalence: for any generated line (valid envelopes, malformed, blanks, junk), both parsers classify identically.
- **[O] F9** — the lens comparison: the same stats run under `--features lens` with each parser; allocation counts + trace sizes recorded in `evidence.md` (the measured cost of `serde` and of the owned trait boundary).

**CLI & hygiene**
- **[E] F10** — args via `clap` derive; usage/help auto-generated; bad usage still exits 2.
- **[O] F11** — dependency policy: `clap`, `serde`, `serde_json`, `thiserror` now allowed; still nothing async; lens stays optional-dep.
- **[O] F12** — public API of the lib reviewed against C-COMMON-TRAITS (Debug/Clone/PartialEq where applicable).

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft (with design+tasks); lessons from 003 audits pre-applied: no compound EARS, properties named with strategies in design, test-first in tasks, optional-dep lens | Part-3 directive | pending combined ack |

</details>
