# Requirements — 003 rust-bedrock

**Status:** awaiting-review
**Approved:** — · **Assurance:** intent 90%, requirements 72% → revised (rev 2)

---

## In plain words

We're building **`glake`** — a small command-line tool that reads goldeneye's
own data lake (the `.jsonl` files under `datalake/raw-local/`) and tells you two
things: *is it well-formed?* and *what's in it?*

It's written with **the Rust standard library only** — no `serde`, no `clap`, no
crates. That's on purpose: hand-writing the parsing and counting is how you build
the ownership and borrowing instincts this whole class is about. It's also a tool
you'll actually reuse, so it's not throwaway practice.

## What it does

- **`glake validate <path>`** — checks every line is a proper envelope event and
  points at any that aren't (by file + line number). Exits with an error code if
  it found problems, so scripts can rely on it.
- **`glake stats <path>`** — counts events, broken down by type and by day, with
  a grand total.
- Point it at **a folder** and it walks every `dt=…/*.jsonl` inside; point it at
  **one file** and it reads just that. Blank lines are ignored. A bad path gives
  a clear error, never a crash.

## What we're *not* building yet (on purpose)

- No filtering by type/day — that's the first feature of spec 004.
- No traits, generics, or custom error types — also 004.
- No async, no network, no S3 — specs 005 and 007.
- No `serde`/`clap` — the whole point is to do it by hand first.

## What you'll learn building it

Each of these shows up in the code and gets a short note in `evidence.md` at the
end (no live session needed — this is internal tooling, same as 002):

- **Ownership & moves** — where a value moves versus gets borrowed as data flows
  parse → count.
- **`&str` vs `String`** — the parser *borrows* slices of each line instead of
  copying them; you'll see where an owned `String` is genuinely needed and where
  it isn't.
- **Enums + `match`** — event kinds and parse outcomes as enums, matched
  exhaustively.
- **`Option` / `Result` / `?`** — fallible steps return `Result`; no `unwrap` in
  the tool's real logic.
- **Iterators + `HashMap`** — counting and grouping as iterator chains, tallied
  with `HashMap`.
- **Seeing the memory** — run glake under the lens (`--features lens`) and note
  what the trace reveals about all of the above. This is why the lens exists.

---

## Precise acceptance criteria

> *Skim this unless you're writing the design or the tests.* Each row is one
> testable fact. Tags: **[P]** = property test (holds for all inputs) · **[E]** =
> example test (specific cases) · **[O]** = checked by running it / the build.
>
> "Well-formed envelope" means: a JSON object with every key the schema registry
> (`datalake/schema/envelope.v1.json`) marks required — `event_id`, `ts`,
> `session_id`, `actor`, `event_type`, `schema_version`, `payload` (`spec_id` is
> optional). v0 checks *keys are present*, not that `ts` is a real date. glake
> reads that key list from the schema file, so the two can't drift apart.

**Validate**
- **[E] R1a** — a line missing any required key is reported with its file and line number.
- **[E] R1b** — the command exits non-zero exactly when at least one line was malformed.

**Stats**
- **[E] R2** — prints counts grouped by `event_type`, counts grouped by `dt=` day, and a grand total.
- **[P] R9** — the per-type totals and the per-day totals each add up to the grand total (nothing double-counted or dropped, on either axis).

**Reading input**
- **[E] R3a** — a folder path is walked recursively through `dt=…/` into every `*.jsonl`.
- **[E] R3b** — a single-file path reads just that file.
- **[E] R5** — blank / whitespace-only lines are skipped (not counted, not flagged).
- **[E] R6** — a missing or unreadable path prints a clear stderr error and exits non-zero, no panic.

**The parser (properties)**
- **[P] R8** — the tiny JSON field-reader, given *any* string, returns either the field or a clean "not found / wrong type" — it never panics and never reads past the end.

**Std-only & the lens**
- **[O] R4** — with default features, the dependency tree is std-only; a check fails if any crate is added. `memlens` shows up only under `--features lens`.
- **[E] R7a** — under `--features lens`, glake installs `memlens` as its allocator and writes a trace.
- **[O] R7b** — with the lens off (default), no memlens code is in the binary.
- **[O] R10** — `glake stats datalake/raw-local` on the real lake matches the counts from `datalake/queries/scan.sh`.

---

<details><summary>Audit trail & changelog</summary>

Intent advisories (90%) resolved: memlens is opt-in behind `--features lens`
(R4/R7); filter deferred to 004.

Requirements audit (72%, `_assurance/requirements-review.md`) → rev 2:
- **MAJOR** — "well-formed" now uses the schema registry's required set including
  **`actor`** (rev 1 omitted it — would have passed malformed lines) and reads
  the list from the file instead of hardcoding it.
- **MAJOR** — R4 (std-only) retagged [P]→[O]: it's a build constraint, not a property.
- Compound lines split (R3→R3a/b, R7→R7a/b, R7b→[O]); R9 extended to the by-day
  axis; added the "seeing the memory" learning goal; noted that iterators and
  `HashMap` aren't yet tracked SKILLS 1b items (reconcile at close).

SKILLS note: L5 uses iterators + `HashMap` (both std, no traits/generics — no
004 boundary crossing) — add them to SKILLS 1b at close, or track here.

</details>
