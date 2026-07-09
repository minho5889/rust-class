# Design — 003 rust-bedrock (`glake` v0)

**Status:** approved
**Approved:** 2026-07-09 by Minho (combined fast-path gate) · **Assurance:** design+tasks 68% → rev 2 (all MAJORs fixed, see
collapsed trail)

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
allowed `serde`. You'll write one small helper that answers two questions about
a JSON line: *"does it have this key?"* and *"what's the text value of this
key?"* It's ~40 lines, it's where the borrowing lesson lives, and its property
test is written **before** it (test-first, like everything here).

## The shape, piece by piece

| Part | What it does | Rust you learn | REQs |
|---|---|---|---|
| **args** | `validate`/`stats` + a path from `std::env::args`; anything else → usage on stderr, exit 2 | `String` vs `&str`, `match`, exit codes | R6 (usage/exit discipline) |
| **walk** | one file → just it; folder → every `dt=*/` `*.jsonl`; unreadable path → clear stderr line, exit 2, no panic | `Result` + `?`, recursion, `PathBuf` | R3a, R3b, R6 |
| **scan** (the heart) | `has_key(line, k)` / `get_str(line, k)` — a character-stepping scanner over one line, quote/escape-aware, top level only | `&str` slices (borrowed returns — zero copies), `Option`, `chars`, scanner-state enum, **explicit `<'a>` on `get_str`** (ramp step 8) | R8, feeds R1a |
| **classify** | each line → `Line<'a>` enum: `Blank`, `Malformed{missing}`, `Event{kind, day}`. `day = ts.get(0..10).unwrap_or("bad-ts")` — **never a slice-index panic**; short/odd `ts` lands in a visible `bad-ts` bucket | enums + exhaustive `match`; `Line<'a>` borrows from the line (lifetimes, met in ramp 8, deepened here) | R5, R1a |
| **count / report** | stats: `HashMap<&str, u32>` tallies by kind and by day + grand total; validate: `file:line missing key "…"`, exit 1 iff any | `HashMap::entry`, iterator chains, formatting | R2, R9, R1a, R1b |

## Key decisions

| Decision | Options | Chosen | Why |
|---|---|---|---|
| Schema key list | read file at runtime · **constant + drift test** | constant + test | the v0 scanner is single-line and string-only; the schema is pretty-printed with an array. A test that reads the schema and asserts the constant matches keeps one source of truth without runtime parsing (requirements rev 3 amendment) |
| Lens dependency | exercise-02 pattern (unconditional dep) · **optional dep** | `memlens = { path, optional = true }`, feature `lens = ["dep:memlens", "memlens/memlens"]`, allocator behind `#[cfg(feature = "lens")]` | exercise-02's pattern would put memlens in the default tree and **fail R4's own check**; optional-dep is the correct shape (audit MAJOR-1) |
| `day` extraction | `&ts[0..10]` · **`ts.get(0..10)`** | `get` + `"bad-ts"` bucket | plain indexing panics on short/multibyte `ts` — legal input under keys-present validation (audit MAJOR-3); the bucket keeps R9's conservation intact |
| Scanner scope | full JSON parser · **top-level, one-line, strings-only** | minimal | matches exactly what glake needs; hardening (escapes) is its own sitting |

## Properties (formal, for the co-written proptests)

| REQ | Property | Generation strategy |
|---|---|---|
| R8 | ∀ input `s: String`, ∀ key: `has_key`/`get_str` return without panic and any returned slice lies within `s` | 50% `any::<String>()` (garbage incl. multibyte), 50% JSON-ish fragments (quoted keys/values, escapes `\"` `\\`, one nesting level, truncations) |
| R9 | ∀ generated line-sets (valid events with random kinds/days incl. short & multibyte `ts`, blanks, malformed): Σ by-kind = Σ by-day = grand total | line-set generator emits tagged expected counts alongside, so the test knows truth independently |

## How we verify (beyond the properties)

- **Scanner correctness examples** (R8 proves *no-panic*, not *right answers*):
  a table of known lines → expected `get_str`/`has_key` results, including
  escaped quotes and a key appearing only inside `payload` (must NOT match —
  top-level only). You write these as ordinary `#[test]`s.
- **Examples:** fixtures for R1a/R1b/R2/R3a/R3b/R5/R6.
- **Operational:** R4 `cargo tree` std-only; R7b no-memlens-symbols; R10
  cross-check vs `scan.sh`; L6 lens run + viewer session.

---

## Detailed interfaces

```rust
// scan.rs — the heart. Explicit lifetime: the returned slice borrows from `line`.
fn get_str<'a>(line: &'a str, key: &str) -> Option<&'a str>;
fn has_key(line: &str, key: &str) -> bool;

// classify.rs
const REQUIRED_KEYS: [&str; 7] = [ /* drift-tested against envelope.v1.json */ ];
enum Line<'a> { Blank, Malformed { missing: &'a str }, Event { kind: &'a str, day: &'a str } }
fn classify<'a>(line: &'a str, required: &[&'a str]) -> Line<'a>;

// walk.rs
fn jsonl_files(path: &Path) -> io::Result<Vec<PathBuf>>;

// stats.rs / validate.rs — folds over (file, line_no, Line<'a>)
```

`Line<'a>` is the design's deliberate stretch lesson: the enum borrows from the
line it classified — proof that classifying a million lines allocates nothing.
It arrives in **Sitting D**, after ramp step 8 (lifetimes-lite) and after you've
already written one explicit `<'a>` on `get_str` in Sitting C.

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial draft (fast path with tasks.md) | requirements approved | — |
| 2026-07-09 | Rev 2 per combined audit (68%): lens dep → optional (MAJOR-1); schema dogfood → constant+drift-test w/ requirements rev-3 amendment (MAJOR-2); `ts.get(0..10)`+bad-ts bucket (MAJOR-3); test-first ordering asserted here & in tasks (MAJOR-4); lifetime claims corrected to C/D + ramp step 8 added (MAJOR-5); Key-decisions & Properties tables added, REQ citations per part, scanner-correctness examples added (MODERATEs) | design+tasks audit | this combined gate |

</details>
