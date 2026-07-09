//! Sitting E: counting as a fold — `HashMap::entry` + iterator chains.
//! Kept as a pure function over lines so the R9 conservation property can
//! hammer it without touching the filesystem.

use crate::classify::{Line, classify};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub by_kind: HashMap<String, u64>,
    pub by_day: HashMap<String, u64>,
    /// Grand total of VALID events (R9's conserved quantity).
    pub events: u64,
    /// Malformed lines are counted, never silently dropped (design rule).
    pub malformed: u64,
}

pub fn tally<'a>(lines: impl IntoIterator<Item = &'a str>) -> Stats {
    let mut stats = Stats::default();
    for line in lines {
        match classify(line) {
            Line::Blank => {}
            Line::Malformed { .. } => stats.malformed += 1,
            Line::Event { kind, day } => {
                stats.events += 1;
                *stats.by_kind.entry(kind.to_owned()).or_insert(0) += 1;
                *stats.by_day.entry(day.to_owned()).or_insert(0) += 1;
            }
        }
    }
    stats
}

impl Stats {
    /// R9, as code: both grouping axes sum to the grand total.
    pub fn is_conserved(&self) -> bool {
        let kinds: u64 = self.by_kind.values().sum();
        let days: u64 = self.by_day.values().sum();
        kinds == self.events && days == self.events
    }
}
