# Design — 003 rust-bedrock (`glake` v0)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

`glake` is one small program in `crates/glake` with four little parts, connected
in a straight line:

```
   your command                 files                    lines                     answer
  ┌────────────┐   ┌──────────────────────┐   ┌────────────────────┐   ┌──────────────────┐
  │ read args  │ → │ find the .jsonl files │ → │ look at each line  │ → │ count OR complain │
  │ (validate/ │   │ (one file, or walk    │   │ blank? broken?     │   │ stats: tally      │
  │  stats +   │   │  dt=…/ folders)       │   │ a real event?      │   │ validate: report  │
  │  a path)   │   │                       │   │                    │   │                   │
  └────────────┘   └──────────────────────┘   └────────────────────┘   └──────────────────┘
```

**You write every part** (coached, one sitting at a time). The only genuinely
tricky piece is the third box — deciding what each line is — because we're not
allowed `serde`. Instead you'll write one small helper function that can answer
two questions about a JSON line: *"does it have this key?"* and *"what's the text
value of this key?"* That helper is the heart of the tool, it's ~40 lines, and
it's where the `&str`-borrowing lesson lives. We property-test it together.

## The shape, piece by piece (and the Rust each one teaches)

| Part | What it does | Rust you learn there |
|---|---|---|
| **args** | reads the command (`validate`/`stats`) and the path from `std::env::args` | `String` vs `&str`, `match` on strings, exiting with a code |
| **walk** | one file → just it; a folder → every `dt=*/` `*.jsonl` inside | `Result` + `?`, recursion, `PathBuf` ownership |
| **scan** (the heart) | `has_key(line, "actor")` and `get_str(line, "event_type")` — a tiny scanner that steps through the characters of one line, respecting quotes/escapes, top level only | `&str` slices (return borrowed pieces of the input — zero copies), `Option`, iterators over `chars`, an enum for scanner state |
| **classify** | each line becomes an enum: `Blank`, `Malformed(missing key)`, or `Event { type, day }` | enums + exhaustive `match` — the "make bad states unrepresentable" habit |
| **count / report** | stats tallies `HashMap<String, u32>` by type and by day (day = first 10 chars of `ts` — a slice, not a parse); validate prints `file:line missing key "actor"` | `HashMap::entry`, iterator chains, formatting output |

Two design choices worth knowing:
- **The required-key list is read from `datalake/schema/envelope.v1.json` at
  startup** (using your own `get_str`-style scanning on the schema file!) — so the
  tool and the schema can never disagree, and the requirement "one source of
  truth" is honored by construction.
- **No third-party crates** (requirement R4). The lens hookup is the one
  exception, behind `--features lens`, copied from exercise-02's two-line pattern.

## What can go wrong, and what the user sees

| Failure | Behavior |
|---|---|
| path doesn't exist / unreadable | one clear line on stderr, exit code 2 — never a panic |
| malformed event line | `validate`: reported with file + line number, final exit code 1; `stats`: counted as "malformed" in the total, not silently dropped |
| blank line | skipped everywhere, counted nowhere |

## How we verify it

- **Properties (co-written, the teaching moments):** R8 — feed the scanner
  thousands of generated strings (random garbage + JSON-ish fragments); it must
  never panic or read out of bounds. R9 — generate mixed valid/blank/broken
  lines; by-type totals and by-day totals must each equal the grand total.
- **Examples:** small fixture files for validate/stats/recursion/blank-skip/bad
  path (R1a–R6), written as ordinary `#[test]`s — you write these, they're good
  practice.
- **Operational:** `cargo tree` proves std-only (R4); feature-off binary has no
  memlens symbols (R7b); `glake stats datalake/raw-local` cross-checked against
  `scan.sh` (R10); a lens run + viewer look for the memory lesson (L6).

---

## Detailed interfaces (for the tasks step and the tests)

```rust
// scan.rs — the heart. Both return borrowed slices; nothing is copied.
fn get_str<'a>(line: &'a str, key: &str) -> Option<&'a str>;  // top-level string value
fn has_key(line: &str, key: &str) -> bool;                    // top-level key of any type

// classify.rs
enum Line<'a> { Blank, Malformed { missing: &'a str }, Event { kind: &'a str, day: &'a str } }
fn classify<'a>(line: &'a str, required: &'a [String]) -> Line<'a>;

// walk.rs
fn jsonl_files(path: &Path) -> Result<Vec<PathBuf>, io::Error>;

// stats.rs / validate.rs fold over (file, line_no, Line) — HashMap tallies / reports
```

The lifetime `'a` on `Line<'a>` is the design's one deliberate stretch-goal
lesson: the enum *borrows* from the line it classified — proof that classifying
a million lines allocates nothing. We'll meet it in the last sitting, with the
lens open to show the payoff.

Scanner strategy for the R8 property (generation): `any::<String>()` mixed 50/50
with a JSON-fragment generator (quoted keys/values with escapes, nesting one
level deep) — co-written with proptest, seeds committed.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial draft (fast path with tasks.md) | requirements approved | this gate |

</details>
