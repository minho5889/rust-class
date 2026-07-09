//! Sitting E (003) evolved: counting as a fold — `HashMap::entry` +
//! iterator chains. Still pure functions over lines so the conservation
//! properties (R9, F3) can hammer them without touching the filesystem.
//!
//! Two layers now:
//! - [`tally`]: fold owned [`ClassifiedLine`]s into [`Stats`] (v0's tally,
//!   lifted from `&str` lines to classified lines — classification moved
//!   behind the `EventParser` trait).
//! - [`tally_filtered`]: the whole stats pipeline — classify → filter →
//!   tally — **generic over the parser** (F13). With `P = HandParser` or
//!   `P = SerdeParser` the call is monomorphized: static dispatch, zero
//!   indirection, one copy of the code per parser type. The binary instead
//!   passes `P = dyn EventParser` (through `&*Box<dyn EventParser>`):
//!   dynamic dispatch through a vtable, one copy of the code total. Same
//!   function, both shapes — that contrast is the T2 lesson, and the
//!   `?Sized` bound is what allows the `dyn` case.

use crate::filter::{Filter, Verdict};
use crate::parser::{ClassifiedLine, EventParser};
use std::collections::HashMap;

/// Counters for one pass over the lake.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    /// Valid events per `event_type`.
    pub by_kind: HashMap<String, u64>,
    /// Valid events per day (F2 day rule; `bad-ts` is a visible bucket).
    pub by_day: HashMap<String, u64>,
    /// Grand total of VALID events (the conserved quantity of R9/F3).
    pub events: u64,
    /// Lines that aren't events and aren't blank: missing-key lines AND
    /// (for strict backends) unparseable lines. Counted, never silently
    /// dropped (design rule since 003).
    pub malformed: u64,
}

impl Stats {
    /// R9, as code: both grouping axes sum to the grand total.
    pub fn is_conserved(&self) -> bool {
        let kinds: u64 = self.by_kind.values().sum();
        let days: u64 = self.by_day.values().sum();
        kinds == self.events && days == self.events
    }
}

/// Fold classified lines into [`Stats`]. Owned lines in, owned keys out —
/// the `String`s move straight into the maps, no re-allocation.
pub fn tally(lines: impl IntoIterator<Item = ClassifiedLine>) -> Stats {
    let mut stats = Stats::default();
    for line in lines {
        match line {
            ClassifiedLine::Blank => {}
            // Unparseable folds into malformed here (amended design): to
            // stats both mean "a line that is not a countable event".
            ClassifiedLine::Unparseable | ClassifiedLine::Malformed { .. } => {
                stats.malformed += 1;
            }
            ClassifiedLine::Event { kind, day } => {
                stats.events += 1;
                *stats.by_kind.entry(kind).or_insert(0) += 1;
                *stats.by_day.entry(day).or_insert(0) += 1;
            }
        }
    }
    stats
}

/// What one filtered stats pass learned (F4's two numbers + F2's note).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilteredStats {
    /// Tally of the lines the filter kept.
    pub kept: Stats,
    /// Total valid events BEFORE filtering — F4's `(filtered from M)`.
    pub total_events: u64,
    /// Events excluded only because `--since` can't compare their bad-ts
    /// day — F2's one-line note.
    pub excluded_bad_ts: u64,
}

/// The core pipeline (F13): classify every line with `parser`, apply
/// `filter`, tally what survives — all as one lazy iterator chain (no
/// intermediate `Vec` of classified lines).
///
/// Generic over `P: EventParser + ?Sized`:
/// - `tally_filtered(&HandParser, …)` → `P = HandParser`, **static**
///   dispatch (the unit tests below call it this way, F13);
/// - `tally_filtered(&*boxed, …)` where `boxed: Box<dyn EventParser>` →
///   `P = dyn EventParser`, **dynamic** dispatch (the binary's `--parser`
///   seam — the only `dyn` point in the crate).
pub fn tally_filtered<'a, P>(
    parser: &P,
    required: &[&str],
    lines: impl IntoIterator<Item = &'a str>,
    filter: &Filter,
) -> FilteredStats
where
    P: EventParser + ?Sized,
{
    let mut total_events = 0u64;
    let mut excluded_bad_ts = 0u64;
    // T5 on display: the pipeline is map → inspect → filter, with closures
    // borrowing the counters mutably from the enclosing scope.
    let kept_lines = lines
        .into_iter()
        .map(|line| parser.classify(line, required))
        .inspect(|classified| {
            if let ClassifiedLine::Event { .. } = classified {
                total_events += 1;
            }
        })
        .filter(|classified| match filter.verdict(classified) {
            Verdict::Keep => true,
            Verdict::Skip => false,
            Verdict::SkipBadTs => {
                excluded_bad_ts += 1;
                false
            }
        });
    let kept = tally(kept_lines);
    FilteredStats {
        kept,
        total_events,
        excluded_bad_ts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::REQUIRED_KEYS;
    use crate::parser::{HandParser, SerdeParser};

    fn lake() -> Vec<String> {
        // 3 valid events (one bad-ts), 1 blank, 1 missing actor, 1 junk.
        let good = |id: &str, ts: &str, kind: &str| {
            format!(
                r#"{{"event_id":"{id}","ts":"{ts}","session_id":"s","actor":"human","event_type":"{kind}","schema_version":1,"payload":{{}}}}"#
            )
        };
        vec![
            good("1", "2026-07-05T01:00:00Z", "gate.approved"),
            good("2", "2026-07-09T02:00:00Z", "gate.approved"),
            good("3", "nope", "session.start"),
            "   ".to_owned(),
            r#"{"event_id":"4","ts":"2026-07-09T03:00:00Z","session_id":"s","event_type":"x.y","schema_version":1,"payload":{}}"#.to_owned(),
            "not json at all".to_owned(),
        ]
    }

    /// [E] F13: the SAME generic function, called with each concrete parser
    /// directly — static dispatch, no `Box`, no `dyn`.
    #[test]
    fn f13_static_dispatch_with_both_parsers() {
        let lake = lake();
        let filter = Filter::default();
        let hand = tally_filtered(
            &HandParser,
            &REQUIRED_KEYS,
            lake.iter().map(String::as_str),
            &filter,
        );
        let serde = tally_filtered(
            &SerdeParser,
            &REQUIRED_KEYS,
            lake.iter().map(String::as_str),
            &filter,
        );
        // Junk is Malformed to hand, Unparseable to serde — but both fold
        // into the same malformed count, so the STATS agree exactly.
        assert_eq!(hand, serde);
        assert_eq!(hand.kept.events, 3);
        assert_eq!(hand.kept.malformed, 2);
        assert_eq!(hand.total_events, 3);
        assert_eq!(hand.excluded_bad_ts, 0);
        assert!(hand.kept.is_conserved());
    }

    /// ...and the same function through the `dyn` seam the binary uses.
    #[test]
    fn f13_dyn_dispatch_matches_static() {
        let lake = lake();
        let filter = Filter::default();
        let boxed: Box<dyn EventParser> = Box::new(HandParser);
        let via_dyn = tally_filtered(
            &*boxed, // P = dyn EventParser
            &REQUIRED_KEYS,
            lake.iter().map(String::as_str),
            &filter,
        );
        let via_static = tally_filtered(
            &HandParser,
            &REQUIRED_KEYS,
            lake.iter().map(String::as_str),
            &filter,
        );
        assert_eq!(via_dyn, via_static);
    }

    /// [E] F1/F2/F4 at the pipeline level: filters drop events (never the
    /// malformed count) and the bad-ts exclusion is counted.
    #[test]
    fn filtered_pipeline_counts_all_three_numbers() {
        let lake = lake();
        let filter = Filter {
            kind: None,
            since: Some("2026-07-06".into()),
        };
        let out = tally_filtered(
            &HandParser,
            &REQUIRED_KEYS,
            lake.iter().map(String::as_str),
            &filter,
        );
        assert_eq!(out.total_events, 3); // M: all valid events
        assert_eq!(out.kept.events, 1); // N: only the 07-09 event
        assert_eq!(out.excluded_bad_ts, 1); // the "nope"-ts event
        assert_eq!(out.kept.malformed, 2); // malformed never filtered away
        assert!(out.kept.is_conserved());
    }
}
