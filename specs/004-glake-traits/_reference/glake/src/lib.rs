//! glake — goldeneye lake CLI (spec 004 reference implementation, v1).
//!
//! Library layout so the property tests can reach the pure core. v0's four
//! modules survive intact; v1 adds one joint and two muscles:
//!
//! ```text
//!  cli (clap) ─► walk ─► parser::EventParser ─► filter ─► tally ─► report
//!                         ├─ HandParser (the 003 scanner, wrapped)
//!                         └─ SerdeParser (serde_json)      error::GlakeError
//! ```
//!
//! Reading order for the learner: `scan` → `classify` (003 material),
//! then `error` → `parser` → `filter` → `tally` → `cli` (the 004 story).

#![forbid(unsafe_code)]

pub mod classify;
pub mod cli;
pub mod error;
pub mod filter;
pub mod parser;
pub mod scan;
pub mod tally;
pub mod walk;
