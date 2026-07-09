//! glake — goldeneye lake CLI (spec 003 reference implementation).
//! Library layout so the property tests can reach the pure core.

#![forbid(unsafe_code)]

pub mod classify;
pub mod scan;
pub mod tally;
pub mod walk;
