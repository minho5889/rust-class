# Assurance audit — 005 async-relay: design.md + tasks.md (combined)

**Auditor:** spec-auditor (fresh context) · **Date:** 2026-07-09
**Audited:** `design.md` (vs approved-pending `requirements.md`), `tasks.md` (vs `design.md`)
**Docs never edited by auditor.**

---

## Traceability summary

- A1–A10 all reachable: A1–A3 (routes/validate/counters, sitting L), A4–A5
  (writer + Properties table, sitting M), A6–A7 (main, sittings K/N), A8–A10
  ("How we verify" + task 1.6/1.7). Both [P]s have Properties rows with real
  generation strategies. Key-decisions rows all carry genuine alternatives and
  reasons. "Rust you learn" column maps cleanly to T1–T6 and SKILLS 1d.
- No scope creep found: every design element cites REQ IDs; tasks add nothing
  beyond the design except the declared no-deploy deviation (properly declared).

The findings below are therefore about **technical soundness and testing
realism**, not structure.

---

## MAJOR

### M1 — The A4 red-first plan will very likely fizzle, and the design leans on a fallback instead of fixing the demo

Three stacked reasons the sitting-L "naive direct append" strawman won't tear
under the planned test:

1. **O_APPEND atomicity.** A per-request open-in-append + single
   `write_all` of one small line is atomic per `write(2)` on local Linux
   filesystems. No interleaving is possible at the byte level.
2. **`oneshot` + `join_all` is cooperative concurrency on one task.** Futures
   interleave only at `.await` points, in deterministic poll order. A handler
   doing blocking `std::fs` I/O runs its whole write without yielding — zero
   interleaving even without O_APPEND.
3. Even `tokio::fs` writes execute as whole `write()` calls on the blocking
   pool.

The design's honesty note ("if the strawman won't tear… records that honestly")
is coherent with the letter of red-first (the property is written before the
*channel* code it ultimately tests), but it concedes the flagship lesson — and
requirements' motivating claim ("two handlers writing one file concurrently
will tear lines") — may rest on a failure that is never demonstrated. The
Properties row also mildly overclaims by calling `join_all` "concurrent".

**Fix (pick one, state it in design):**
- Make the strawman genuinely tearable: multi-thread `Runtime` in the proptest
  body, `tokio::spawn` per request against a cloned `Router` (not `join_all`
  on one task), and a strawman write path with **two await points per line**
  (e.g., write body, then newline, as separate `tokio::fs` writes) — that
  tears reliably; or
- Reframe the sitting-M lesson up front as planned, not fallback: "the OS
  saved you this time (O_APPEND per-write atomicity — a guarantee with
  size/filesystem caveats); the channel saves you by construction" and have
  the property pin conservation. Either is teachable; an accidental green is
  not.

### M2 — Drop-tx shutdown is achievable but has an unstated deadlock footgun the design must name

The ordering works: `axum::serve(...).with_graceful_shutdown(ctrl_c).await`
resolves only after in-flight connection tasks finish (dropping their
`AppState`/`tx` clones), and the serve future's drop releases the router's
`AppState` and its `tx`; then `writer.await` drains and flushes. **But this
holds only if `main` retains no `tx` clone.** The natural way to write it —
build `tx`, clone into `AppState`, keep the original bound in `main` — makes
`rx.recv()` never return `None` and `writer.await` hang forever. This is the
single most likely failure mode of the whole design and the worksheet's most
likely mystery-hang, yet neither design.md ("drop the senders" is stated
generically) nor task 1.5 names it.

**Fix:** one sentence in the main.rs row and the sitting-N worksheet: *move*
`tx` into `AppState` (or explicitly `drop(tx)` before `writer.await`); make
"why does it hang if you keep a clone?" a deliberate checkpoint question — it
is the T2 lesson in its purest form.

---

## MODERATE

### M3 — The A6 test as designed does not test A6

A6 is "[E] on **ctrl-c (SIGINT)** … process **exits 0**". The plan ("port-0
server + trigger the shutdown future") never exercises SIGINT wiring,
`tokio::signal::ctrl_c`, or the exit code — it tests only the drain path with
the signal abstracted away. Realistic and cheap alternative the design already
has machinery for (A7 uses `CARGO_BIN_EXE`): spawn the relay binary as a child
with a temp lake, POST events, send a real SIGINT to the child, assert exit
status 0 and disk contents. Keep the in-process drain test as a supplement if
wanted.

### M4 — Bounded vs unbounded channel is an undecided decision that belongs in Key decisions

`mpsc::Sender<Line>` leaves open: bounded (handlers `send().await` →
backpressure, adds a handler yield point that matters to M1's interleaving) vs
unbounded (no backpressure, unbounded memory if the writer stalls), capacity,
and full-channel behavior (await vs `try_send` → 5xx). This is a genuine
alternatives-and-reason row — backpressure is core to T2 — and its absence
means the reference and the learner can silently diverge.

### M5 — Writer I/O model and flush policy are undesigned

Blocking `std::fs` inside the async writer task vs `tokio::fs` vs
`spawn_blocking`; write-through per line vs `BufWriter` flushed at close. This
affects (a) whether the 1.7 live `glake stats` payoff sees data before
shutdown, (b) A4's "on disk after shutdown" timing, (c) the T1 lesson about
what may and may not block a runtime thread. One row in the shape table or Key
decisions settles it. Related, worth one sentence each: the per-day handle
cache is unbounded (fine per-proptest-case with a fresh temp lake and fine for
a lab tool — say so), and 202 deliberately means "accepted into the channel",
not "on disk" (correct use of 202, consistent with A4/A6's after-shutdown
phrasing, but A1's "returns 202 and appends" reads as write-before-respond —
disambiguate).

### M6 — A10's `--features lens` wiring appears nowhere in the design shape

Requirement A10 and task 1.7 build relay with `--features lens`, but no design
element covers the optional-dep/allocator wiring in the relay crate (the
requirements changelog even says "lens optional-dep" was pre-applied). Add it
to the shape table (pattern presumably mirrors 003/004's crates) so sitting N
isn't improvising crate features.

---

## MINOR

- **m1 — A5 generator day domain unspecified.** If generated `ts` values all
  land on one day, the partition property is near-vacuous. State the domain
  (e.g., days drawn from a fixed 3–5 day set, so multi-day partitioning is
  always exercised).
- **m2 — `Line` type named but never defined** (routes.rs row) while the plain
  words say `String`. Unify.
- **m3 — A9 has no design element**; it lives only in "How we verify" / task
  1.6. Acceptable for an [O], but the crate-level `#![deny(clippy::unwrap_used)]`
  (or lint-table) placement should be named somewhere concrete.
- **m4 — Tasks 2.1 "SKILLS 1d/2a updates":** 2a is AWS Lambda, explicitly out
  of scope here (echoes the same slip in intent). Likely means 1d (+ possibly
  2e, the data lake). Also SKILLS 1d lists `select!`, which the design never
  exercises (`with_graceful_shutdown` hides it) — a one-line ramp-12 or
  sitting-N aside would close that curriculum item honestly.
- **m5 — No task line writes the A7 example test** (design promises a clap
  `CARGO_BIN_EXE` run); only the operations checklist implies it. Pin it to
  1.1 or 1.6.
- **m6 — Sitting L density:** door + glake path-dep payoff + counters + three
  example tests is two concepts (validation door; shared-state counters). The
  two-commit split suggests the worksheet already phases it — make the
  worksheet's internal split explicit to stay inside one-concept-per-step
  spirit.
- **m7 — A2's "first problem" error line** depends on the glake lib publicly
  exposing the malformed *reason* and the extracted *day*, not just a bucket.
  003's output (`missing key "actor"`) suggests it does, but since relay is the
  first external caller, the design should name the exact glake API surface
  validate.rs consumes — that's the T4 payoff made concrete, and it catches
  drift in sitting K's compile check.

---

## Tasks-specific checks (vs design)

- Every design element has an owning task; test-first ordering for A4 holds
  (1.3 red before 1.4 channel). A5's "written before the code it tests" is
  ambiguous inside 1.4 (property and per-day cache land in the same item) —
  split the A5-red moment out or note the ordering in the worksheet.
- Coached-mode rules otherwise respected: co-written hard bit (async×proptest)
  flagged as such in both docs; one commit per move; ramp steps 12–13 are one
  concept each; hygiene sweep correctly assigned to Claude; payoff (1.7)
  correctly assigned to the learner.
- Deviation (no AWS deploy) declared and consistent with intent/requirements.

---

**VERDICT: 72% — Structurally clean and fully traceable, but two majors (a likely-deadlocking unstated tx-retention footgun in the drop-tx shutdown, and an A4 tearing demo that almost certainly won't go red as planned) plus an under-testing A6 plan need design-level answers before the human's time is well spent.**
