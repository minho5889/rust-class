//! relay — goldeneye telemetry relay (spec 005 reference implementation, v0).
//!
//! One rule shapes the whole service: **the file has one owner.**
//!
//! ```text
//!                     ┌────────────────────── tokio runtime ─────────────────────┐
//!  curl ─► POST /events ─► handler: validate (glake lib) ─► tx.send(line) ──┐    │
//!  curl ─► POST /events ─► handler: validate ─► 400, counted, dropped       │    │
//!  curl ─► GET  /healthz ─► counters (atomics)                              ▼    │
//!                                                          mpsc channel ─► writer task
//!                     └─────────────────────────────────────── owns the files ───┘
//!                                             ctrl-c ─► stop accepting ─► drop tx
//!                                                       ─► writer drains ─► flush ─► exit
//! ```
//!
//! Handlers never touch a file. A valid event becomes a `String` line and is
//! **sent** — ownership and all — through an `mpsc` channel to the one task
//! that holds the open files (`writer`). Because that task writes lines one
//! at a time, concurrent requests can't interleave bytes (A4). No
//! `Mutex<File>`, no locks: order and integrity come from single ownership
//! of the sink. If you can explain why `tx.send(line)` compiles — the
//! `String` is `Send`, so it may change tasks; nothing else crosses — you
//! understand the T2/T3 lesson.
//!
//! Shutdown is the same idea run backwards: dropping the last sender IS the
//! shutdown signal. When `axum::serve` finishes (ctrl-c) the router drops,
//! the `tx` inside it drops, the writer's `recv()` returns `None`, and the
//! writer drains, flushes, and returns. No flags, no sleeps (A6).
//!
//! Reading order for the learner: `validate` (the strict door, pure) →
//! `routes` (handlers + shared state) → `writer` (the owner) → `main.rs`
//! (wiring and the shutdown footgun).

#![forbid(unsafe_code)]

pub mod routes;
pub mod validate;
pub mod writer;
