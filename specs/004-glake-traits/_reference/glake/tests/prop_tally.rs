//! [P] R9 (003, carried into v1) — conservation: by-type totals and by-day
//! totals EACH sum to the grand total of valid events; nothing
//! double-counted, nothing dropped. The generator emits its own expected
//! counts (independent truth), and includes blanks, malformed lines,
//! short/multibyte/pattern-violating timestamps.
//!
//! v1 adaptation: `tally` now consumes owned `ClassifiedLine`s (the trait
//! boundary's type), so the pipeline under test is HandParser → tally.
//! Expected days follow the tightened F2 day rule via `day_of_ts` — the
//! same one-source-of-truth helper the classifiers use.

use glake::classify::{REQUIRED_KEYS, day_of_ts};
use glake::parser::{EventParser, HandParser};
use glake::tally::tally;
use proptest::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum GenLine {
    Blank(String),
    Malformed, // a required key removed
    Event {
        kind: String,
        ts: String,
        expected_day: String,
    },
}

fn gen_line() -> impl Strategy<Value = GenLine> {
    let kind = prop_oneof![
        Just("gate.approved".to_string()),
        Just("spec.doc_written".to_string()),
        "[a-z]{1,8}\\.[a-z]{1,8}"
    ];
    let ts = prop_oneof![
        // normal RFC3339-ish day
        (2020u32..2030, 1u32..13, 1u32..29)
            .prop_map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}T00:00:00Z")),
        // short → bad-ts bucket
        Just("2026".to_string()),
        // multibyte in the first 10 bytes → bad-ts bucket
        Just("🦀🦀🦀🦀".to_string()),
        // 10 chars but not YYYY-MM-DD shaped → bad-ts bucket (F2, new in v1)
        Just("2026/07/09T00:00:00Z".to_string()),
    ];
    prop_oneof![
        1 => Just(GenLine::Blank("   ".to_string())),
        1 => Just(GenLine::Malformed),
        4 => (kind, ts).prop_map(|(kind, ts)| {
            // the F2 day rule, from the same helper the classifiers use
            let expected_day = day_of_ts(&ts).unwrap_or("bad-ts").to_string();
            GenLine::Event { kind, ts, expected_day }
        }),
    ]
}

fn render(l: &GenLine) -> String {
    match l {
        GenLine::Blank(s) => s.clone(),
        GenLine::Malformed => {
            // valid except: no "actor"
            r#"{"event_id":"1","ts":"2026-01-01T00:00:00Z","session_id":"s","event_type":"x.y","schema_version":1,"payload":{}}"#.to_string()
        }
        GenLine::Event { kind, ts, .. } => format!(
            r#"{{"event_id":"1","ts":"{ts}","session_id":"s","actor":"human","event_type":"{kind}","schema_version":1,"payload":{{}}}}"#
        ),
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn r9_totals_conserved_on_both_axes(lines in prop::collection::vec(gen_line(), 0..60)) {
        // independent truth, computed by the generator
        let mut want_kind: HashMap<String, u64> = HashMap::new();
        let mut want_day: HashMap<String, u64> = HashMap::new();
        let mut want_events = 0u64;
        let mut want_malformed = 0u64;
        for l in &lines {
            match l {
                GenLine::Blank(_) => {}
                GenLine::Malformed => want_malformed += 1,
                GenLine::Event { kind, expected_day, .. } => {
                    want_events += 1;
                    *want_kind.entry(kind.clone()).or_insert(0) += 1;
                    *want_day.entry(expected_day.clone()).or_insert(0) += 1;
                }
            }
        }

        let rendered: Vec<String> = lines.iter().map(render).collect();
        let stats = tally(
            rendered
                .iter()
                .map(|line| HandParser.classify(line, &REQUIRED_KEYS)),
        );

        prop_assert_eq!(stats.events, want_events);
        prop_assert_eq!(stats.malformed, want_malformed);
        prop_assert_eq!(&stats.by_kind, &want_kind);
        prop_assert_eq!(&stats.by_day, &want_day);
        prop_assert!(stats.is_conserved(), "R9: an axis lost or duplicated events");
    }
}
