//! Sitting D: every line becomes one of three kinds — the "make bad states
//! unrepresentable" lesson. `Line<'a>` BORROWS from its inputs (the design's
//! stretch lesson): classifying a million lines allocates nothing.
//!
//! Interface note (and D's teaching question, answered): `missing` borrows
//! from `required` — the key names live in the caller's slice, not in the
//! line. That's why the signature takes `required: &[&'a str]` and why
//! `Line<'a>` can return them without copying.

use crate::scan::{get_str, has_key};

/// The required-key list, drift-tested against
/// `datalake/schema/envelope.v1.json` (see tests/schema_drift.rs). A test
/// fails if the schema ever changes without this constant following —
/// one source of truth, no runtime parsing (requirements rev 3).
pub const REQUIRED_KEYS: [&str; 7] = [
    "event_id",
    "ts",
    "session_id",
    "actor",
    "event_type",
    "schema_version",
    "payload",
];

#[derive(Debug, PartialEq, Eq)]
pub enum Line<'a> {
    /// Whitespace-only: skipped everywhere, counted nowhere (R5).
    Blank,
    /// Missing a required key — reported by `validate` (R1a).
    Malformed { missing: &'a str },
    /// A proper event record: its type, and its day (first 10 chars of ts).
    Event { kind: &'a str, day: &'a str },
}

/// Classify one line against a required-key list (the design's frozen
/// interface — callers hand in `&REQUIRED_KEYS`).
pub fn classify<'a>(line: &'a str, required: &[&'a str]) -> Line<'a> {
    if line.trim().is_empty() {
        return Line::Blank;
    }
    for &key in required {
        if !has_key(line, key) {
            return Line::Malformed { missing: key };
        }
    }
    let kind = get_str(line, "event_type").unwrap_or("non-string");
    // .get(0..10), never [0..10]: a short or multibyte ts is legal input
    // under keys-present validation and must not panic — it lands in a
    // visible "bad-ts" bucket instead (design decision, audit MAJOR-3).
    let day = get_str(line, "ts")
        .and_then(|ts| ts.get(0..10))
        .unwrap_or("bad-ts");
    Line::Event { kind, day }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"event_id":"1","ts":"2026-07-09T01:02:03Z","session_id":"s","actor":"human","event_type":"gate.approved","schema_version":1,"payload":{}}"#;

    #[test]
    fn good_line_is_an_event() {
        assert_eq!(
            classify(GOOD, &REQUIRED_KEYS),
            Line::Event {
                kind: "gate.approved",
                day: "2026-07-09"
            }
        );
    }

    #[test]
    fn blank_and_missing_key() {
        assert_eq!(classify("   \t ", &REQUIRED_KEYS), Line::Blank);
        let no_actor = GOOD.replace(r#""actor":"human","#, "");
        assert_eq!(
            classify(&no_actor, &REQUIRED_KEYS),
            Line::Malformed { missing: "actor" }
        );
    }

    #[test]
    fn short_or_multibyte_ts_never_panics() {
        let short = GOOD.replace("2026-07-09T01:02:03Z", "2026");
        assert_eq!(
            classify(&short, &REQUIRED_KEYS),
            Line::Event {
                kind: "gate.approved",
                day: "bad-ts"
            }
        );
        let multibyte = GOOD.replace("2026-07-09T01:02:03Z", "🦀🦀🦀🦀");
        assert_eq!(
            classify(&multibyte, &REQUIRED_KEYS),
            Line::Event {
                kind: "gate.approved",
                day: "bad-ts"
            }
        );
    }
}
