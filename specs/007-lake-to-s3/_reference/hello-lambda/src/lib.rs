//! hello-lambda — the door's sink grows up (spec 007 reference, v1;
//! evolved from the 006 reference at exactly one seam).
//!
//! 006's diagram, one box swapped — everything crossed out in 006 STAYS
//! crossed out, and one exile ends:
//!
//! ```text
//!  curl ─► Function URL ─► Lambda service ─► handler() ─► store.put()
//!                          (spawns/kills instances,          │
//!                           ONE request per instance    s3://goldeneye-lake/
//!                           at a time, scales by        raw/dt=<day>/
//!                           adding instances)           evt-<event_id>.json
//!
//!  ~~one stdout line ─► CloudWatch~~   (006 H1, superseded by S6b: the
//!                                       lake is no longer in exile)
//! ```
//!
//! What survives the evolution untouched is the **door** — the same
//! 202/400 verdict, reached the same way, through the same glake calls;
//! the H4 property (tests/prop_door.rs) re-proves it against relay AFTER
//! the change. What's new is where a "yes" lands: an S3 object whose key
//! carries the event's identity (`evt-<event_id>`), which makes retries
//! overwrite themselves instead of duplicating (T5).
//!
//! What it costs is the OTHER half of this spec's lesson (S8): the
//! aws-sdk-s3 + aws-config stack rides along in the binary now, and the
//! bootstrap's size delta against 006's 1,843,256-byte baseline is
//! measured, not guessed (see NOTES.md for the bill).
//!
//! Reading order for the learner: `handler` (the door + the put, S6) →
//! `state` (per-instance counters, unchanged) → `main.rs` (the client-
//! once cold-start discipline, T3 — the file that grew the most, from
//! four lines of wiring to one real decision).

#![forbid(unsafe_code)]

pub mod handler;
pub mod state;
