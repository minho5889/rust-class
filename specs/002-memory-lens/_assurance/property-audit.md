# Property audit — 002 memory-lens (pre-close)

**Auditor:** property-auditor (fresh context) · **Date:** 2026-07-05
**Scope:** [P] requirements R1, R2, R3, R10a · crates `memlens` (feature `memlens`)
and `memlens-replay` · design Properties table rev 2 · triage record in
`evidence.md` (no separate `_assurance/triage-log.md`).

## REQ → property map

| REQ | Property test | Strategy fidelity vs design Properties table | Result (this audit's run) |
|---|---|---|---|
| R1 | `crates/memlens/tests/prop_r1_capture.rs::r1_every_op_recorded_exactly_once_in_order` | **Faithful.** Sizes `1..=65536` (= 1..64 KiB), aligns 2^0..2^4 = {1,2,4,8,16}, grow factor 1..=4, valid-by-construction indices (modulo live set), executes against `MemLens<System>`, parses its own trace, checks exact event list equality + strictly increasing `seq`. 256 cases explicit. Deviation (disclosed in-file, acceptable): ops go through a local lens instance via direct `GlobalAlloc` calls rather than an installed `#[global_allocator]`; real installation is covered by integration tests (`passthrough.rs`, scope/schema suites). No narrowing found. | **PASS** — 256 cases, `cargo test -p memlens --features memlens` green (7 tests incl. doc-test) |
| R2 | `crates/memlens-replay/tests/prop_engine.rs::r2_validator_accepts_valid_traces`, `::r2_validator_rejects_corrupted_traces`, `::r2_corollary_live_bytes_never_negative` | **Faithful in intent, one soundness hole.** Generator covers the design domain well: address recycling after free, in-place *and* moving reallocs, nested scopes, markers. Adversarial side implements 4 mutation kinds (seq disorder, duplication, size corruption, alloc deletion) ⊇ design's "shuffled/duplicated". Hole: mutation 1 ("duplicate a heap event") is not guaranteed to corrupt — see Finding 1. Minor gap: design says "same harness traces" also feed R2; the property suite uses synthetic traces only (real captured traces are validated only by [E] golden/schema tests). | **FAIL** — `r2_validator_rejects_corrupted_traces` fails deterministically from the committed regression seed; `accepts` and `corollary` pass (256 cases each) |
| R3 | `crates/memlens-replay/tests/prop_engine.rs::r3_live_bytes_equals_reference_at_every_prefix` | **Faithful.** Independent naive reference interpreter written in the test file, no shared code with the engine; checked at *every* prefix `t ∈ 0..=max+1`, 256 cases. The "Σ alloc − Σ freed" formulation is covered jointly with `r2_corollary` (running-sum form). | **PASS** |
| R10a | `crates/memlens-replay/tests/prop_engine.rs::r10a_replay_equals_reference_and_is_deterministic` | **Faithful.** Engine live-set vs reference map (cardinality + per-entry size/align + `born_seq ≤ t`), determinism across repeated calls **and** cloned event slices, at `t ∈ {0, max/2, max, max+7}`; design's edge cases (empty trace via no-op-only op lists, t=0, t=max, realloc chains) reachable by generation/shrinking. Not every prefix, but matches the design's stated strategy; per-prefix coverage exists on the bytes side via R3. | **PASS** |

Case counts: `ProptestConfig { cases: 256 }` explicit in both suites (constitution ≥256 met).

## Regression-seed policy

- `crates/memlens/tests/prop_r1_capture.proptest-regressions` — committed at
  88e47f9 (the red-by-design commit); seed `Alloc{size:1}`; now passes. Policy
  satisfied for R1.
- `crates/memlens-replay/tests/prop_engine.proptest-regressions` — committed at
  **fc38a2f** ("operations" bolt-4.1 commit). This seed is a **live failing
  counterexample** for `r2_validator_rejects_corrupted_traces` (verified this
  audit: the suite fails on every run, since proptest replays committed seeds
  first). It appears to have been swept into fc38a2f alongside unrelated
  operations artifacts, without a triage entry. The premise "no seeds committed
  because none were generated" is **not** true for the current tree.

## Triage record

- Prior counterexamples: evidence.md ("Property-test outcomes" + "Notable
  findings" 3–4) records the R1 red-phase failures, classifies them **test
  bugs** (unflushed BufWriter reads, offset tracking, early sink init) with
  in-harness resolutions. Content-wise this satisfies the constitution's
  classify-before-fix rule for those cases; location deviates (constitution
  says triage is logged in `_assurance/`) — acceptable as a minor deviation
  since no engine-level counterexample existed *at that time*.
- **The committed R2 seed has no triage entry anywhere**, and evidence.md's R2
  row ("Counterexamples: 0") is now contradicted by the tree. Evidence is
  append-only: it needs an appended correction, not an edit.

## Findings

1. **BLOCKER — R2 adversarial property is red, untriaged, with its failing seed
   committed.** `r2_validator_rejects_corrupted_traces` fails on
   `mutation = 1` (duplicate heap event). Root cause (audited by hand from the
   shrunk input): the generator's size ceiling `clamp(1, 262_144)` plus the
   even-factor "grow in place" rule produces
   `Realloc { old_addr == new_addr, old_size == new_size == 262144 }` — a
   semantic **no-op**. Duplicating a no-op realloc yields a trace that is still
   balanced; `validate()` **correctly accepts it**. Classification (auditor's
   assessment, to be confirmed in triage): **test bug** — the mutation
   strategy's "duplicate ⇒ invalid" assumption is unsound for no-op reallocs;
   no evidence of a validator/code bug, and no spec bug (R2's meaning is
   unaffected). Required before close: log the triage (test bug + resolution),
   fix the mutation strategy so mutation 1 guarantees invalidity (e.g., skip
   no-op reallocs as duplication targets or forbid the generator from emitting
   them), keep the seed, re-run green.
2. **MEDIUM — evidence.md R2 row is stale** ("256 cases, 0 counterexamples, no
   seeds"): append a correction recording this counterexample and its triage.
3. **MINOR — R2 "same harness traces" leg is synthetic-only** in the property
   suite; real captured traces reach `validate()` only via [E] golden/schema
   tests. Either note the delegation in design's verification strategy or feed
   a captured trace through `validate()` in a test.
4. **MINOR — R1 property exercises the lens via direct `GlobalAlloc` calls**,
   not an installed `#[global_allocator]`; disclosed in the test header and
   covered by integration tests — no action required, recorded for honesty.
5. **NOTE — shrinking hit `max_shrink_iters` (1024)** on the R2 failure, so the
   stored counterexample is not fully minimal; harmless, but a larger budget
   would teach better counterexamples.

## Suite runs (this audit, 2026-07-05)

- `cargo test -p memlens --features memlens` → **all green** (prop_r1_capture
  256 cases; scope/passthrough/schema/sink-failure/doc-test).
- `cargo test -p memlens-replay` → **5 passed, 1 FAILED**
  (`r2_validator_rejects_corrupted_traces`, from the committed seed;
  `accepts`/`r3`/`r10a`/`corollary`/`scope_labels` green).

VERDICT: 75% — R1, R3, and R10a have faithful, passing 256-case properties, but R2's adversarial property is failing from a committed, untriaged seed (a no-op-realloc hole in the duplicate-mutation strategy), so the spec cannot close until it is triaged as a test bug, the strategy fixed, and the suite green with the seed kept.
