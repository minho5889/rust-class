//! [P] F3 — filter partition: for ANY generated event set and ANY filter,
//! (kept) + (excluded) = (unfiltered total) — grand total, per kind AND
//! per day. Filters never invent events, never lose them, and never touch
//! the malformed count.
//!
//! Strategy per the design's Properties table: the R9 line generator
//! (adapted from tests/prop_tally.rs), with the filter drawn FROM the
//! generated data — `--type` from the kinds that actually occur (plus an
//! absent one), `--since` from the days that actually occur (plus extreme
//! dates) — via `prop_flat_map`, so the filter usually bites instead of
//! matching nothing.

use glake::classify::{REQUIRED_KEYS, day_of_ts};
use glake::filter::Filter;
use glake::parser::{ClassifiedLine, EventParser, HandParser};
use glake::tally::{Stats, tally, tally_filtered};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum GenLine {
    Blank,
    Malformed,
    Event { kind: String, ts: String },
}

fn gen_line() -> impl Strategy<Value = GenLine> {
    let kind = prop_oneof![
        Just("gate.approved".to_string()),
        Just("spec.doc_written".to_string()),
        "[a-z]{1,8}\\.[a-z]{1,8}"
    ];
    let ts = prop_oneof![
        (2020u32..2030, 1u32..13, 1u32..29)
            .prop_map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}T00:00:00Z")),
        Just("2026".to_string()),                 // → bad-ts
        Just("🦀🦀🦀🦀".to_string()),             // → bad-ts
        Just("2026/07/09T00:00:00Z".to_string()), // → bad-ts (F2 rule)
    ];
    prop_oneof![
        1 => Just(GenLine::Blank),
        1 => Just(GenLine::Malformed),
        4 => (kind, ts).prop_map(|(kind, ts)| GenLine::Event { kind, ts }),
    ]
}

fn render(l: &GenLine) -> String {
    match l {
        GenLine::Blank => "   ".to_string(),
        GenLine::Malformed => {
            r#"{"event_id":"1","ts":"2026-01-01T00:00:00Z","session_id":"s","event_type":"x.y","schema_version":1,"payload":{}}"#.to_string()
        }
        GenLine::Event { kind, ts } => format!(
            r#"{{"event_id":"1","ts":"{ts}","session_id":"s","actor":"human","event_type":"{kind}","schema_version":1,"payload":{{}}}}"#
        ),
    }
}

/// Lines plus a filter drawn from those lines' own kinds and days.
fn lines_and_filter() -> impl Strategy<Value = (Vec<GenLine>, Filter)> {
    prop::collection::vec(gen_line(), 0..60).prop_flat_map(|lines| {
        // --type candidates: no filter, a kind that can't match, and every
        // kind the set actually contains.
        let mut kinds: Vec<Option<String>> = vec![None, Some("absent.kind".to_string())];
        // --since candidates: no filter, extreme dates, and every real
        // (comparable) day the set actually contains.
        let mut sinces: Vec<Option<String>> = vec![
            None,
            Some("0000-01-01".to_string()),
            Some("9999-12-31".to_string()),
        ];
        for l in &lines {
            if let GenLine::Event { kind, ts } = l {
                kinds.push(Some(kind.clone()));
                if let Some(day) = day_of_ts(ts) {
                    sinces.push(Some(day.to_string()));
                }
            }
        }
        let filter = (prop::sample::select(kinds), prop::sample::select(sinces))
            .prop_map(|(kind, since)| Filter { kind, since });
        (Just(lines), filter)
    })
}

/// kept[k] + excluded[k] must equal total[k] for every key on an axis —
/// and neither side may contain a key the unfiltered axis doesn't have
/// (filters never INVENT events either).
fn assert_axis_partition(
    axis: &str,
    total: &std::collections::HashMap<String, u64>,
    kept: &std::collections::HashMap<String, u64>,
    excluded: &std::collections::HashMap<String, u64>,
) -> Result<(), TestCaseError> {
    for key in kept.keys().chain(excluded.keys()) {
        prop_assert!(
            total.contains_key(key),
            "{axis}: invented key {key:?} not in the unfiltered tally"
        );
    }
    for (key, n) in total {
        let k = kept.get(key).copied().unwrap_or(0);
        let e = excluded.get(key).copied().unwrap_or(0);
        prop_assert_eq!(k + e, *n, "{}[{}]: {} kept + {} excluded", axis, key, k, e);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn prop_f3_filter_partitions_events((lines, filter) in lines_and_filter()) {
        let rendered: Vec<String> = lines.iter().map(render).collect();
        let classified: Vec<ClassifiedLine> = rendered
            .iter()
            .map(|line| HandParser.classify(line, &REQUIRED_KEYS))
            .collect();

        // The three tallies: everything, what keep() keeps, what it drops.
        let total: Stats = tally(classified.iter().cloned());
        let kept: Stats = tally(classified.iter().filter(|c| filter.keep(c)).cloned());
        let excluded: Stats = tally(classified.iter().filter(|c| !filter.keep(c)).cloned());

        // F3 grand total: kept + excluded = unfiltered — no invention, no loss.
        prop_assert_eq!(kept.events + excluded.events, total.events);
        // ...per kind and per day too.
        assert_axis_partition("by_kind", &total.by_kind, &kept.by_kind, &excluded.by_kind)?;
        assert_axis_partition("by_day", &total.by_day, &kept.by_day, &excluded.by_day)?;
        // Each side stays internally conserved (R9 survives filtering).
        prop_assert!(kept.is_conserved() && excluded.is_conserved() && total.is_conserved());
        // Filters act on events only: the malformed count is untouchable.
        prop_assert_eq!(kept.malformed, total.malformed);
        prop_assert_eq!(excluded.malformed, 0);

        // And the real pipeline (tally_filtered) reports the same partition:
        // its kept stats, its M, and its bad-ts note all line up.
        let piped = tally_filtered(
            &HandParser,
            &REQUIRED_KEYS,
            rendered.iter().map(String::as_str),
            &filter,
        );
        prop_assert_eq!(&piped.kept, &kept);
        prop_assert_eq!(piped.total_events, total.events);

        // Independent truth for the bad-ts note: events whose day is
        // bad-ts, that pass --type, while --since is active.
        let want_bad_ts: u64 = if filter.since.is_some() {
            classified
                .iter()
                .filter(|c| matches!(
                    c,
                    ClassifiedLine::Event { kind, day }
                        if day == "bad-ts"
                        && filter.kind.as_deref().is_none_or(|want| want == kind)
                ))
                .count() as u64
        } else {
            0
        };
        prop_assert_eq!(piped.excluded_bad_ts, want_bad_ts);
    }
}
