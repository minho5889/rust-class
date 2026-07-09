# Requirements — 003 rust-bedrock

**Status:** approved
**Approved:** 2026-07-09 by Minho ("proceed to the next spec") · **Assurance:**
intent 90%, requirements 72% → revised (rev 2)

---

## In plain words

This is the first spec where you write real Rust. You build **`glake`** (short for
"goldeneye lake") — a small terminal tool that reads the telemetry files this
project has been quietly collecting all along: the `.jsonl` files under
`datalake/`. `glake` answers two questions about them — **"is this data valid?"**
and **"what's in it?"**

Here's the whole tool, from your side:

```console
$ glake stats datalake/raw-local
3 files · 214 events

by type
  spec.doc_written      86
  research.finding      29
  gate.notified          9
  …

by day
  2026-07-05           214

$ glake validate datalake/raw-local
datalake/raw-local/dt=2026-07-05/events.jsonl:41  missing key "actor"
1 malformed line
$ echo $?
1
```

That's it — count things, or flag broken lines. Simple on purpose.

**Why this is spec 003 and not something flashier:** the tool is an excuse. Its
real job is to teach Rust's core — ownership, borrowing, `&str` vs `String`,
enums, `Result` — by making you *hand-write* the parts a real project would grab
a crate for. So `glake` uses **the standard library only**: no `serde` to parse
the JSON for you, no `clap` for the command-line arguments. You write those
yourself, feel exactly where Rust's borrow checker pushes back, and watch the
whole thing run in the memory lens you built in spec 002. Spec 004 then lets you
refactor it *with* the nice crates — but only after you've earned the intuition
by doing it the hard way once.

## What it does

- **`glake validate <path>`** — reads every line, checks each is a proper event
  record, and prints any that aren't with the file name and line number. Exits
  with an error code if it found problems (so a script or CI can trust it).
- **`glake stats <path>`** — counts events, grouped by their type and by their
  day, with a grand total.
- Give it **a folder** → it walks every `.jsonl` file beneath it (daily partitions and memlens traces alike). Give it **one
  file** → it reads just that. Blank lines are ignored. A path that doesn't exist
  gives a clear error message, never a crash.

## What we're *not* building yet (and where it goes)

- **Filtering** (`--type gate.notified`, `--since …`) → the first feature of **004**.
- **Traits, generics, custom error types** → **004** (that spec's whole point).
- **`serde` / `clap` / any crate** → deliberately withheld here; **004** brings them in.
- **Async, network, S3, anything cloud** → **005** (async) and **007** (S3).

## What you'll learn building it

`glake` is small, but every piece of it is a fundamentals lesson. Each shows up
in the code and gets a one-line note in `evidence.md` when we're done:

- **L1 · Ownership & moves** — as a line flows *parse → count*, you'll see where
  a value is handed off (moved) versus just looked at (borrowed).
- **L2 · `&str` vs `String`** — the parser reads *slices* of each line (`&str`, no
  copying) instead of allocating new strings; you'll see the few places an owned
  `String` is genuinely needed, and the many where it isn't.
- **L3 · Enums + `match`** — "what kind of event is this?" and "did parsing succeed?"
  become enums the compiler forces you to handle completely.
- **L4 · `Result` and `?`** — anything that can fail returns a `Result`; the `?`
  operator threads errors up cleanly, with no `unwrap` in the real logic.
- **L5 · Iterators + `HashMap`** — the counting is an iterator chain tallied into a
  `HashMap`, not a pile of manual loops.
- **L6 · Seeing the memory** — run `glake --features lens`, open the trace in the
  spec-002 viewer, and *watch* the `&str`-vs-`String` choices above as real
  allocations. This is the payoff of having built the lens first.

---

## Precise acceptance criteria

> *Skim this unless you're writing the design or the tests.* Each row is one
> testable fact. Tags: **[P]** = property (must hold for *all* inputs, tested by
> generating thousands) · **[E]** = example (specific cases) · **[O]** =
> operational (checked by running the tool or the build).
>
> **"A proper event record"** = a JSON object containing every key the schema
> registry (`datalake/schema/envelope.v1.json`) marks required: `event_id`,
> `ts`, `session_id`, `actor`, `event_type`, `schema_version`, `payload`
> (`spec_id` is optional). v0 checks the *keys are present* — not that `ts` is a
> real date (that can come in 004). The key list lives in glake as a constant
> **verified against the schema file by a test** — if the schema ever changes,
> the test fails and the constant must follow. (Amended from "read at startup":
> runtime parsing of a pretty-printed schema needs an array-capable multi-line
> scanner v0 doesn't have — test-time sync keeps one source of truth with none
> of that complexity.)

**Validate**
- **[E] R1a** — a line missing any required key is reported with its file and line number.
- **[E] R1b** — the command exits non-zero exactly when at least one line was malformed.

**Stats**
- **[E] R2** — prints counts by `event_type`, counts by `dt=` day, and a grand total.
- **[P] R9** — the by-type totals and the by-day totals each add up to the grand total (nothing double-counted or dropped, on either axis).

**Reading input**
- **[E] R3a** — a folder is walked recursively into every `*.jsonl` file beneath it — `dt=` partitions and `traces/` alike (the lake is both; amended rev 4).
- **[E] R3b** — a single-file path reads just that file.
- **[E] R5** — blank / whitespace-only lines are skipped (not counted, not flagged).
- **[E] R6** — a missing or unreadable path prints a clear stderr error and exits non-zero — no panic.

**The hand-written parser (this is the property-tested heart)**
- **[P] R8** — the tiny JSON field-reader, given *any* string whatsoever, returns
  either the field it was asked for or a clean "not found / wrong type" — it never
  panics and never reads past the end of the input.

**Std-only & the lens**
- **[O] R4** — with default features the dependency tree is std-only; a check fails if any crate is added. `memlens` appears only under `--features lens`.
- **[E] R7a** — under `--features lens`, glake installs `memlens` as its allocator and writes a trace.
- **[O] R7b** — with the lens off (the default), no memlens code is in the binary.
- **[O] R10** — on the real lake, `glake stats datalake/raw-local`'s grand total **reconciles** with `datalake/queries/scan.sh`: glake's total = scan.sh's process-event count + the lines of memlens trace files (glake walks the whole lake; scan.sh reads only `dt=*/events.jsonl`). Verified at authoring: 221 = 157 + 64 (amended rev 4 from "matches" — the tools measure different scopes by design).

---

<details><summary>Audit trail & changelog</summary>

Intent advisories (90%) resolved: memlens is opt-in behind `--features lens`
(R4/R7); filtering deferred to 004.

Requirements audit (72%, `_assurance/requirements-review.md`) → rev 2:
- **MAJOR** — "proper record" now uses the schema registry's required set
  including **`actor`** (rev 1 omitted it — it would have passed malformed lines)
  and reads the list from the file instead of hardcoding it.
- **MAJOR** — R4 (std-only) retagged [P]→[O]: it's a build constraint, not a property.
- Compound lines split (R3→R3a/b, R7→R7a/b, R7b→[O]); R9 extended to the by-day
  axis; added the "seeing the memory" learning goal; noted iterators + `HashMap`
  aren't yet tracked SKILLS 1b items (reconcile at close).

SKILLS note: L5 uses iterators + `HashMap` (both std, no traits/generics — no 004
boundary crossing) — add them to SKILLS 1b at close.

Readability rev (2026-07-05): rewrote the prose to lead with a concrete worked
example after the first pass read as an under-explained bullet list. Criteria
unchanged.

Rev 4 (2026-07-09, materials-critic M1/M4, change protocol): R3a walk scope
made explicit (whole lake incl. `traces/`, matching the built walk) and R10
changed from "matches" to the reconciliation identity (221 = 157 + 64 at
authoring) — the two tools measure different scopes by design. Worked example's
by-day output corrected to bare days (`2026-07-05`, no `dt=` prefix), matching
the built output. Flagged for learner ack at next gate contact.

Rev 3 (2026-07-09, design-audit feedback, combined-gate amendment): schema key
list = compile-time constant + drift test instead of runtime file read (design
audit MAJOR-2: the v0 scanner can't parse a pretty-printed array; test-time
sync preserves one-source-of-truth). Learning goals labeled L1–L6 (they were
cited downstream but unlabeled here).

</details>
