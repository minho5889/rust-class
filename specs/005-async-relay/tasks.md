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
      (`#[tokio::main]`, spawn, sleep, join, and a `select!` race of two
      sleeps — what `.await` yields to), **13 channels + Send/Sync** (mpsc
      across tasks; move a `String` through; why the compiler objects when you
      try to share instead). Worksheets + validated solutions,
      `my-solution.rs` + one commit per step.

## 1. relay, four sittings (learner writes, Claude coaches)

### Sitting K — hello, axum *(A3 groundwork, A7; T1)*
- [ ] 1.1 `cargo new crates/relay`; tokio + axum deps; `GET /healthz`
      returning static JSON; clap `--port`/`--lake` + the **A7 example test**
      (`CARGO_BIN_EXE`); run it, curl it. Wire the glake lib as a path dep
      (compile check only — the 004 payoff starts here).
      *(commits: hello axum, then args+dep+test)*

### Sitting L — the strict door *(A1, A2, A3; T4)*
> Two ideas share this sitting (the door check and the counters) — the guide
> paces them as two halves with a break point; split across two sessions
> freely if the first half runs long.
- [ ] 1.2 `POST /events`: validate via the glake lib (verdict +
      first-missing-key + comparable day), 202/400 in the requirements' fixed
      error order; **naive direct append** in the handler for now
      (single-request world); `Arc<Counters>` atomics behind `/healthz`;
      `#[tokio::test]` examples for A1/A2/A3 via `oneshot`.
      *(commits: door, then counters+tests)*

### Sitting M — the one owner, test-first *(A4, A5; T2, T3, T6)*
- [ ] 1.3 **A4 conservation property first** (red, co-written — the
      async×proptest pattern: sync body owning a **multi-thread** runtime,
      one `tokio::spawn` per request): fire generated mixed batches at the
      sitting-L naive writer. Two honest outcomes, both taught: it tears (a
      real red — validation found a two-write append tearing ~170 of 200
      lines), or `O_APPEND` saves the single-buffer shape and we name the
      savior and why it's no guarantee (design's note).
- [ ] 1.4 The **mpsc (bounded, 256) + single writer task**: handlers
      `send().await`, writer owns the files (per-day handle cache,
      `tokio::fs`, flush per line). The first harness run *without* the
      drain choreography goes genuinely red (shutdown-mid-queue loses
      enqueued events) — then `drop(app)` + `writer.await` makes A4 green
      **by construction**; **A5 partition property** (glake-agrees check)
      green. *(commits: red, channel, A5 green)*

### Sitting N — drain, flush, prove, observe *(A6a/b, A8–A10; T5)*
- [ ] 1.5 Checkpoint question first (predict before running): *if `main`
      keeps a `tx` clone, what happens at `writer.await`?* Then graceful
      shutdown: `with_graceful_shutdown(ctrl_c)`, move-tx-into-state drain,
      `writer.await` last; **A6a/A6b example test via a `CARGO_BIN_EXE` child
      + real SIGINT** (refused late request, exit 0, 202'd events on disk).
      *(commits: shutdown, then test)*
- [ ] 1.6 Hygiene sweep (Claude drives): A8 `cargo tree`, A9
      `clippy::unwrap_used` denied in Cargo.toml, fmt/clippy both configs.
- [ ] 1.7 **The payoff (you drive):** run relay against a temp lake, POST real
      hook events at it (`curl` loop), `glake stats` the result; then one
      short session under `--features lens` — runtime vs handler vs writer
      allocations in the viewer. Notes → `evidence.md`.

## 2. Close-out (Claude, machine work)

- [ ] 2.1 evidence.md (T1–T6 notes, A10 numbers, property outcomes, any
      triage); property-auditor run; SKILLS 1d updates (async/channels/
      Send-Sync mastery evidence); MEMORY; main FF +
      `spec-close/005-async-relay` marker.

---

## Operations checklist

- [ ] fmt + clippy clean (both feature configs, `unwrap_used` denied); A4/A5
      ≥256 cases, red-first (honest-red protocol per design), seeds committed
      on genuine failure; A1–A3/A6a/A6b/A7 example tests green (A6 via real
      child + SIGINT); A8/A10 recorded in evidence; property-auditor pass.

*Deviation declared: no AWS deploy/teardown — local-only unit (deploys as 006).*

---

<details><summary>Audit trail & changelog</summary>

| Date | Change | Trigger | Re-gated? |
|---|---|---|---|
| 2026-07-09 | Initial fast-path draft | Part-4 directive | pending combined ack |
| 2026-07-09 | Rev 2.1 (materials reconciliation): shutdown-mid-queue red relocated from 1.3 to 1.4 where it's actually reproducible (the strawman has no queue; the channel-without-drain does); 1.3 records the empirical tearing result (~170/200 lines, two-write shape) | sitting-guide authoring | this combined gate |
| 2026-07-09 | Rev 2 per design+tasks audit (72%): sitting M rewritten for real parallelism + honest-red protocol; sitting N gets the tx-retention checkpoint question and real-SIGINT A6 test; A7 test named in K; `select!` added to step 12; sitting L pacing note; SKILLS ref fixed to 1d | 005 audits | this combined gate |

</details>
