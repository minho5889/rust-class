//! hello-lambda — the 005 door goes to the cloud (spec 006 reference, v0).
//!
//! Put relay's architecture diagram next to this crate's and the lesson is
//! the crossed-out parts:
//!
//! ```text
//!  ~~listener~~   ~~router loop~~   ~~graceful shutdown~~   ~~single-writer task~~
//!
//!  curl ─► Function URL ─► Lambda service ─► handler() ─► one stdout line ─► CloudWatch
//!                          (spawns/kills instances,        (the lake in exile;
//!                           ONE request per instance         S3 home arrives in 007)
//!                           at a time, scales by
//!                           adding instances)
//! ```
//!
//! In 005 you wrote the whole server: bind, serve, route, drain, flush. On
//! Lambda **the platform owns all of that** — what's left of your program is
//! one `async fn(Request) -> Response`. That's T1. And it's why Rust fits:
//! no JVM to warm, no GC to size. The `provided.al2023` runtime just execs
//! a binary named `bootstrap`; your binary IS the runtime (T2).
//!
//! What survives the move untouched is the **door** — the same 202/400
//! verdict, reached the same way relay reached it: through the glake
//! library. The H4 property (tests/prop_door.rs) drives both doors with one
//! generator and proves they can never drift.
//!
//! What survives *changed in meaning* is the state (T4): relay was one
//! process, many tasks, shared counters; Lambda is many instances, one
//! request each, nothing shared. The counters still compile, still count —
//! per warm instance. See [`state`] for that lesson said honestly.
//!
//! Reading order for the learner: `handler` (the one function that is the
//! whole service) → `state` (per-instance, on purpose) → `main.rs` (the
//! four lines of wiring you still own).

#![forbid(unsafe_code)]

pub mod handler;
pub mod state;
