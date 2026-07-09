# Tasks — 003 rust-bedrock (`glake` v0)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Seven ramp steps (already underway in `playground/ramp/`), then **six sittings**
to build glake — each sitting is one concept-cluster, one working session, one
commit. **You write the code; Claude coaches** (poses the step, reviews, explains
compiler errors — never pastes the answer first). The two property tests are
co-written, per the constitution. At ~10 h/week this is roughly two weeks.

**Done means:** you can run `glake stats datalake/raw-local` on the real lake,
the numbers match `scan.sh`, all tests are green, and you've watched your own
tool's memory in the lens.

---

## 0. The ramp (`playground/ramp/`, no spec, in progress)

- [ ] 0.1 Steps 1–7, one concept each (`fn main`/`let` → ownership → borrowing →
      `&str` vs `String` → `enum`+`match` → `Result`+`?` → read a file).
      One commit per step: `ramp: step N — <concept>`. **Step 1 is posed and
      waiting.** Rustlings/100-Exercises bound as optional warm-ups
      (`research/rust-study-materials.md`).

## 1. glake, one sitting at a time (learner writes, Claude coaches)

### Sitting A — a program that reads a file
- [ ] 1.1 `cargo new` in `crates/glake` (joins the workspace); read one `.jsonl`
      path from `std::env::args`, print how many lines it has. Uses ramp steps
      1–3 and 7. *(commit: `003: sitting A — count lines`)*

### Sitting B — commands and folders
- [ ] 1.2 Add the `validate`/`stats` command argument (`match` on it; unknown →
      usage message, exit 2). Folder paths: walk `dt=*/` recursively collecting
      `*.jsonl` (design's `jsonl_files`); single files still work. (R3a, R3b,
      R6 groundwork.) *(commit: `003: sitting B — args + walk`)*

### Sitting C — the scanner (the heart)
- [ ] 1.3 Write `has_key` / `get_str` — the ~40-line character scanner. Claude
      coaches through quotes/escapes edge cases; you drive. Blank-line skipping
      (R5). *(commit: `003: sitting C — scanner`)*
- [ ] 1.4 **Co-write the R8 property test** (never panics, any input; garbage +
      JSON-ish strategies; seeds committed). Your first proptest — walked
      through together, then it hammers *your* scanner with thousands of cases.
      *(commit: `003: sitting C2 — scanner property (R8)`)*

### Sitting D — validate
- [ ] 1.5 Read the required-key list from `datalake/schema/envelope.v1.json`
      (dogfooding your own scanner); `classify` each line (the `Line` enum);
      `validate` reports `file:line missing key "…"`, exits 1 iff any (R1a,
      R1b). Example tests from small fixtures — you write these.
      *(commit: `003: sitting D — validate`)*

### Sitting E — stats
- [ ] 1.6 `stats`: tally by type and by day (`HashMap::entry`; day = `&ts[0..10]`
      slice) + grand total (R2). Clear stderr + exit codes for bad paths (R6).
      **Co-write the R9 conservation property** (both axes sum to total).
      *(commit: `003: sitting E — stats + R9`)*

### Sitting F — proof and the payoff
- [ ] 1.7 Machine checks (Claude drives, you watch): R4 `cargo tree` std-only
      check; R7a/b lens feature wired exercise-02-style, feature-off symbol
      check; R10 cross-check vs `scan.sh`; fmt/clippy clean.
- [ ] 1.8 **The lens moment (L6):** run `glake stats` under `--features lens`,
      open the trace in `viewer/memlens.html`, and see the scanner's zero-copy
      borrowing (few allocations) vs the `HashMap` tallies (real ones). Notes →
      `evidence.md`. *(commit: `003: sitting F — verified + lens evidence`)*

## 2. Close-out (Claude, machine work)

- [ ] 2.1 `evidence.md`: L1–L6 notes, property outcomes, R10 numbers, mistake-
      ledger summary (`learning.*` events accumulated during coaching).
- [ ] 2.2 property-auditor pre-close run; SKILLS 1a/1b updates (iterators +
      `HashMap` added as line items, statuses moved on demonstrated mastery);
      MEMORY session log; `main` fast-forward + `spec-close/003-rust-bedrock`
      marker branch.

---

## Operations checklist

- [ ] `cargo fmt` + `cargo clippy -- -D warnings` clean (both feature configs)
- [ ] R8 + R9 properties pass (≥256 cases), seeds committed if any fail
- [ ] All example tests pass; R4/R7b/R10 operational checks recorded
- [ ] `evidence.md` written; `MEMORY.md` + `SKILLS.md` updated
- [ ] property-auditor: every [P] REQ has a faithful, passing property

*Template deviation (declared): no AWS deploy/teardown lines — local-only unit
(same as 002).*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial draft (fast path with design.md); sittings sized for coached mode | requirements approved | this gate |

</details>
