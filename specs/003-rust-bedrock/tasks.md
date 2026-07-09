# Tasks — 003 rust-bedrock (`glake` v0)

**Status:** approved
**Approved:** 2026-07-09 by Minho (combined fast-path gate) · **Assurance:** design+tasks 68% → rev 2 (with design.md)

---

## In plain words

Eight ramp steps (underway in `playground/ramp/`), then **six sittings** to
build glake — each one concept-cluster, one working session. **You write the
code; Claude coaches.** Property tests are co-written and come **before** the
code they test. Commit grain: at least one commit per sitting; a sitting that
lands a property test and its implementation makes two. At ~10 h/week this is
roughly two weeks.

**Done means:** `glake stats datalake/raw-local` matches `scan.sh` on the real
lake, all tests green, and you've watched your own tool's memory in the lens.

---

## 0. The ramp (`playground/ramp/`, no spec, in progress)

- [ ] 0.1 Steps 1–8, one concept each (incl. **step 8: lifetimes-lite**). One
      commit per step (`ramp: step N — <concept>`), your code in
      `my-solution.rs`. **All eight worksheets are pre-authored and validated**
      (materials-ahead directive 2026-07-09; solutions compile-swept, critic
      verdict 78%→96% after fixes) — start anytime at
      `playground/ramp/step-01-hello/`.

## 1. glake, one sitting at a time (learner writes, Claude coaches)

> All six sitting guides are pre-authored and validated against the reference
> implementation (`_reference/glake`, 17/17 tests green): open
> `sittings/sitting-A-count-lines.md` when the ramp is done.

### Sitting A — a program that reads a file *(ramp 1–3, 7)*
- [ ] 1.1 `cargo new` in `crates/glake`; read one `.jsonl` path from
      `std::env::args`; print its line count.

### Sitting B — commands, folders, and failing well *(R3a, R3b, R6)*
- [ ] 1.2 `validate`/`stats` argument (`match`; unknown → usage on stderr, exit
      2). Folder → recursive `dt=*/` walk (`jsonl_files`); file → just it.
      **All R6 error paths land here**: missing/unreadable path → clear stderr,
      exit 2, no panic — with example tests.

### Sitting C — the scanner, test-first *(R8; ramp 8 recap)*
- [ ] 1.3 **Co-write the R8 property test first** against stub signatures
      (`get_str<'a>`/`has_key` returning `todo!()`) — red. Your first proptest,
      walked through together; garbage + JSON-ish strategies per the design's
      Properties table. *(commit: `003: sitting C — R8 property, red`)*
- [ ] 1.4 Write the **flat scanner** (no escape handling yet) until R8 passes on
      the simple strategy + your scanner-correctness `#[test]` table (known
      lines → expected answers, incl. top-level-only). Blank-line skip (R5).
      *(commit: `003: sitting C — flat scanner green`)*

### Sitting D — hardening + validate *(R1a, R1b; `Line<'a>`)*
- [ ] 1.5 Harden the scanner: escaped quotes `\"`, backslashes, multibyte —
      driven by turning the R8 strategy up to full and adding correctness rows.
- [ ] 1.6 `REQUIRED_KEYS` constant + **schema drift test** (reads
      `envelope.v1.json`, asserts the constant matches — the rev-3 design).
      `classify` → `Line<'a>` (the stretch lesson, ramp 8 paying off);
      `validate` reports `file:line missing key`, exits 1 iff any; fixture
      tests. *(commits: one per item)*

### Sitting E — stats, test-first *(R2, R9)*
- [ ] 1.7 **Co-write the R9 conservation property first** (generator emits its
      own expected counts; both axes; short/multibyte `ts` included) — red.
- [ ] 1.8 Implement the tallies (`HashMap::entry`; `day = ts.get(0..10)` with
      the `bad-ts` bucket) + output formatting until R9 and the R2 fixtures
      pass. *(commits: red, then green)*

### Sitting F — proof and the payoff *(R4, R7a/b, R10, L6)*
- [ ] 1.9 Machine checks (Claude drives, you watch and ask): R4 `cargo tree`
      std-only check; lens as **optional dep** (`lens = ["dep:memlens",
      "memlens/memlens"]`, allocator under `#[cfg(feature = "lens")]`); R7b
      symbol check; R10 cross-check vs `scan.sh`; fmt/clippy both configs.
- [ ] 1.10 **The lens moment (L6) — you drive, Claude navigates:** run
      `glake stats` under `--features lens`, open the trace in
      `viewer/memlens.html`, find the scanner's near-zero allocations vs the
      `HashMap`'s real ones. Your observations → `evidence.md`.

## 2. Close-out (Claude, machine work)

- [ ] 2.1 `evidence.md`: L1–L6 notes, property outcomes, R10 numbers,
      mistake-ledger summary from the coaching sessions.
- [ ] 2.2 property-auditor pre-close run; SKILLS updates (add iterators +
      `HashMap` + lifetimes-lite as 1b items, statuses per demonstrated
      mastery); MEMORY log; `main` fast-forward + `spec-close/003-rust-bedrock`
      marker branch.

---

## Operations checklist

- [ ] `cargo fmt` + `cargo clippy -- -D warnings` clean (both feature configs)
- [ ] R8 + R9 properties pass (≥256 cases), written before their code, seeds
      committed if any failure occurred
- [ ] Scanner-correctness examples + all fixtures pass; R4/R7b/R10 recorded
- [ ] `evidence.md` written; `MEMORY.md` + `SKILLS.md` updated
- [ ] property-auditor: every [P] REQ has a faithful, passing property

*Template deviation (declared): no AWS deploy/teardown lines — local-only unit
(same as 002).*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial draft (fast path with design.md) | requirements approved | — |
| 2026-07-09 | Rev 2 per combined audit (68%): properties reordered test-FIRST (C and E now start red — MAJOR-4); scanner split flat→hardened across C/D (MODERATE); R6 error paths consolidated into B (MODERATE); ramp step 8 added and cited (MAJOR-5); lens optional-dep + correctness-example items added (MAJOR-1/MODERATE); commit-grain phrasing fixed; 1.10 driver named (MINORs) | design+tasks audit | this combined gate |

</details>
