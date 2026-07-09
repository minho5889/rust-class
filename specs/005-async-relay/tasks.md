# Tasks — 005 async-relay (relay v0)

**Status:** awaiting-review
**Approved:** — · **Assurance:** —

---

## In plain words

Two new ramp steps (12–13: async basics, channels), then **four sittings**
(K→N) building `crates/relay` from an empty crate to a graceful, property-
tested service. Same rules: you write, worksheets steer, properties red-first,
one commit per move. All materials pre-authored and validated against the 005
reference.

**Done means:** concurrent POSTs land intact in the lake, ctrl-c loses
nothing, `glake stats` agrees with the relay's own counters, and you've seen
a live service's memory in the lens.

---

## 0. Ramp extension (`playground/ramp/`, no spec)

- [ ] 0.3 Steps 12–13, one concept each: **12 async/await + tokio**
      (`#[tokio::main]`, spawn, sleep, join — what `.await` yields to),
      **13 channels + Send/Sync** (mpsc across tasks; move a `String` through;
      why the compiler objects when you try to share instead). Worksheets +
      validated solutions, `my-solution.rs` + one commit per step.

## 1. relay, four sittings (learner writes, Claude coaches)

### Sitting K — hello, axum *(A3, A7 groundwork; T1)*
- [ ] 1.1 `cargo new crates/relay`; tokio + axum deps; `GET /healthz`
      returning static JSON; clap `--port`/`--lake`; run it, curl it.
      Wire the glake lib as a path dep (compile check only — the 004 payoff
      starts here). *(commits: hello axum, then args+dep)*

### Sitting L — the strict door *(A1, A2, A3; T4)*
- [ ] 1.2 `POST /events`: validate via the glake lib, 202/400 with the
      first-problem error line; **naive direct append** in the handler for
      now (single-request world); `Arc<Counters>` atomics behind `/healthz`;
      `#[tokio::test]` examples for A1/A2/A3 via `oneshot`.
      *(commits: door, then counters+tests)*

### Sitting M — the one owner, test-first *(A4, A5; T2, T3, T6)*
- [ ] 1.3 **A4 conservation property first** (red, co-written — the
      async×proptest pattern): fire generated mixed batches concurrently at
      the sitting-L naive writer; watch what happens (and record honestly if
      the strawman won't tear under the test runtime — design's note).
- [ ] 1.4 The **mpsc + single writer task**: handlers send, writer owns the
      files (per-day handle cache); A4 green; **A5 partition property**
      (glake-agrees check) green. *(commits: red, channel, A5 green)*

### Sitting N — drain, flush, prove, observe *(A6, A8–A10; T5)*
- [ ] 1.5 Graceful shutdown: `with_graceful_shutdown(ctrl_c)`, drop-tx drain,
      `writer.await` last; A6 example test (port-0 server + triggered
      shutdown). *(commits: shutdown, then test)*
- [ ] 1.6 Hygiene sweep (Claude drives): A8 `cargo tree`, A9
      `clippy::unwrap_used`, fmt/clippy both configs.
- [ ] 1.7 **The payoff (you drive):** run relay against a temp lake, POST real
      hook events at it (`curl` loop), `glake stats` the result; then one
      short session under `--features lens` — runtime vs handler vs writer
      allocations in the viewer. Notes → `evidence.md`.

## 2. Close-out (Claude, machine work)

- [ ] 2.1 evidence.md (T1–T6 notes, A10 numbers, property outcomes, any
      triage); property-auditor run; SKILLS 1d/2a updates; MEMORY; main FF +
      `spec-close/005-async-relay` marker.

---

## Operations checklist

- [ ] fmt + clippy clean (both feature configs, `unwrap_used` on); A4/A5 ≥256
      cases, red-first, seeds committed on failure; A1–A3/A6/A7 example tests
      green; A8/A10 recorded in evidence; property-auditor pass.

*Deviation declared: no AWS deploy/teardown — local-only unit (deploys as 006).*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-4 directive | pending combined ack |

</details>
