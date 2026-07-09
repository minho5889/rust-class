# Intent — 005 async-relay

> **Doc 1 of 4. WRITE-ONCE.** Audited post-hoc by `intent-assurance`
> (`_assurance/intent-review.md`). No human gate.

**Created:** 2026-07-09 · **Spec status:** active

## Raw prompt (verbatim)

> "great can you make 2 more parts further?"

Same message as 004's intent — "2 more parts" covers course Parts 3 **and 4**.
This doc records the Part-4 half. Said under the materials-ahead directive
("you develop all the materials ahead. so I can just follow along later").

## Distilled intent

Author **Part 4 of the course** ahead of time: spec 005 from the approved
Phase-1 plan — *"local axum service receiving envelope events (hooks can POST);
async/await, tokio, channels, Send/Sync, graceful shutdown."* The learner
builds **relay**, a small local HTTP service that accepts envelope events over
`POST /events` and appends them to the lake's `dt=` partitions — the same job
the shell hooks do today, but concurrent, validated at the door, and shaped
like every Rust network service they'll ever deploy (006 lifts this exact
handler into Lambda).

## Learning goal

SKILLS **1d/2a** (async/await, `tokio`, tasks vs threads, channels as ownership
transfer, `Send`/`Sync` meaning, graceful shutdown) — the async foundations the
constitution's AWS targets all assume. Explicitly out: anything AWS (006+),
TLS/auth (localhost-only lab tool), batching/S3 (007).

## Assumptions made

- "2 more parts" = Parts 3 and 4 = specs 004 and 005, authored materials-ahead
  like Parts 1–3 (worksheets + validated reference).
- Part 4 assumes Part 3's ending shape: relay **depends on the learner's own
  `glake` lib** for envelope validation — the "public API a stranger can call"
  payoff of 004's T4. A learner who diverged reconciles in sitting K.
- Fast-path pipeline; gates presented for ack at the end (standing pattern).
- Local-only unit: binds `127.0.0.1`, writes `datalake/raw-local/` (or a temp
  lake in tests). No deploy; deployment of this handler IS spec 006.

## Rejected interpretations

- ❌ Jump straight to Lambda (006) — the plan sequences local async first;
  debugging tokio locally is strictly easier than debugging it in a cold start.
- ❌ Replace the shell hooks now — the relay is a parallel path the hooks *can*
  POST to; retiring hooks is not this spec's call.
- ❌ A message queue / background daemon system — smallest teaching service:
  one POST route, one health route, one writer.

## Addendum (append-only)

_(none yet)_
