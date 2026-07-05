# Evidence — 002 memory-lens

> Append-only report card. Construction: 2026-07-05, fully autonomous run
> (learner grant: "you do all the bolts").

## Deployment record

| Date | Target | Region | Artifact (size) | Result |
|---|---|---|---|---|
| 2026-07-05 | local only (R11 — no AWS by requirement) | — | `viewer/memlens.html` (~21 KB, self-contained) + `memlens`/`memlens-replay` crates | working end-to-end: program → trace → viewer |

## Observed metrics ([O] requirements)

| REQ | Metric | Observed | Where measured |
|---|---|---|---|
| R14a | Tracing overhead, fixed workload (exercise 02, debug, 10-run median) | **feature-off 2.40 ms → feature-on 2.88 ms = 1.20×**; per-run trace ≈ 14.5 KB | `lens-off`/`lens-on` binaries, python timer |
| R13 | Scrub cost, 100k-event synthetic trace | **1.77 ms/step** over 66,667 lifetimes (budget: 100 ms; canvas paint excluded — node has no canvas, learner verifies visually) | `viewer/test-fold.mjs` |
| R11 | Viewer self-containment | single HTML file, zero network requests, drag-drop load | inspection + fold test |
| R9/R10b/R12/R15 | Panels/scrub/callouts/honesty footer | implemented; **learner session pending (4.1.2)** | — |

## Property-test outcomes ([P] requirements)

| REQ | Suite | Cases run | Counterexamples | Triage | Seed kept? |
|---|---|---|---|---|---|
| R1 | `prop_r1_capture` (real allocator, own-trace parse) | 256 | 0 against the final engine (red-phase failures were harness bugs: unflushed BufWriter reads, offset tracking) | test bugs — fixed in-harness | no engine seeds to keep |
| R2 | `r2_validator_accepts` + `r2_validator_rejects` (4 adversarial mutation kinds) | 256 + 256 | 0 | — | — |
| R3 | `r3_live_bytes_equals_reference_at_every_prefix` | 256 (× every prefix point) | 0 | — | — |
| R10a | `r10a_replay_equals_reference_and_is_deterministic` | 256 (× 4 prefix points, × clone determinism) | 0 | — | — |
| R2 corollary | `r2_corollary_live_bytes_never_negative` | 256 | 0 | — | — |

Cross-checks: JS fold ≡ Rust engine on all 6 golden snapshots, bit-for-bit.
[E] suite: 15 tests green across both feature configurations (schema round-trip
+ conformance, compile guard, feature-off symbols/trace absence, sink failure
loss marker, scope ordering on 3 exit paths, growth fixtures).

## Notable findings & teaching moments (construction)

1. **Amortized doubling, quantified**: the demo's 1000 `Vec` pushes + 500
   `HashMap` inserts produced a **41-event** trace. A first-draft test expected
   \>100 events and failed — the assertion now teaches the lesson.
2. **HashMap growth ≠ Vec growth** (R5 clarification, requirements changelog):
   hashbrown allocates the bigger table and frees the old one — *no realloc
   events*. Vec reallocs with lineage. Two collection types, two different
   bargains with the allocator. ⚠ awaiting learner ack of the R5 wording.
3. **The tracer-reads-its-own-tail bug**: reading a live trace mid-write hit
   torn half-lines (BufWriter flushes mid-line at buffer boundaries) → public
   `flush()` added; the failure is preserved as a comment in the R1 harness.
4. **Global-lens test binaries initialize the sink before `#[test]` runs**
   (the harness allocates first!) → `__retarget` test hook. A concrete lesson
   in how early a global allocator becomes live.
5. **The exercise trace itself**: 64 events for the entire 6-lesson program —
   moves and borrows contributed exactly 0.

## What you learned

_(4.1.4 pending — the learner explains L1–L5 unaided in their session; record
here, including gaps.)_

## Remaining for the learner (the [O] human half)

- **4.1.2**: run `cargo run --features lens` in `playground/02-collections-lens`,
  open `viewer/memlens.html`, drop in the trace from
  `datalake/raw-local/traces/dt=2026-07-05/`, scrub; record observations here.
- **4.1.4**: explain L1–L5 unaided; record explanations (and gaps) here.
- Ack the R5 wording clarification (requirements changelog, 2026-07-05).
