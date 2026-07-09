//! Sitting D (003): every line becomes one of three kinds — the "make bad
//! states unrepresentable" lesson. `Line<'a>` BORROWS from its inputs (the
//! design's stretch lesson): classifying a million lines allocates nothing.
//!
//! v1 (004) keeps this borrowed classifier exactly as the fast zero-copy
//! core — `HandParser` wraps it and pays the owning cost only at the trait
//! boundary (see `parser.rs`). One thing IS tightened for F2: the **day
//! rule**. v0 bucketed ANY 10-char `ts` prefix as a day; v1 accepts the
//! prefix only if it actually looks like `YYYY-MM-DD` (see [`is_day`]), so
//! that `--since` string comparison is always chronological. Everything
//! else lands in the visible `bad-ts` bucket.
//!
//! Interface note (and D's teaching question, answered): `missing` borrows
//! from `required` — the key names live in the caller's slice, not in the
//! line. That's why the signature takes `required: &[&'a str]` and why
//! `Line<'a>` can return them without copying.

use crate::scan::{get_str, has_key};

/// The required-key list, drift-tested against
/// `datalake/schema/envelope.v1.json` (see tests/schema_drift.rs). A test
/// fails if the schema ever changes without this constant following —
/// one source of truth, no runtime parsing (003 requirements rev 3).
pub const REQUIRED_KEYS: [&str; 7] = [
    "event_id",
    "ts",
    "session_id",
    "actor",
    "event_type",
    "schema_version",
    "payload",
];

/// The borrowed verdict on one line. `Copy` because it's only pointers and
/// a tag — cheap by construction (C-COMMON-TRAITS, F12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line<'a> {
    /// Whitespace-only: skipped everywhere, counted nowhere (003 R5).
    Blank,
    /// Missing a required key — reported by `validate` (003 R1a).
    Malformed { missing: &'a str },
    /// A proper event record: its type, and its day (per the F2 day rule).
    Event { kind: &'a str, day: &'a str },
}

/// Is `s` exactly a `YYYY-MM-DD`-shaped day? Four digits, dash, two digits,
/// dash, two digits — a *shape* check, not a calendar check: `9999-99-99`
/// passes, because lexicographic order on the shape is all `--since` needs.
///
/// This is F2's day rule and it has ONE home: [`classify`], the
/// `SerdeParser` (`parser.rs`) and `--since` validation (`filter.rs`) all
/// call it, so the three can never drift apart.
pub fn is_day(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
}

/// The day bucket of a raw `ts` value: its first 10 bytes, IFF they match
/// the [`is_day`] pattern; `None` means the caller's `bad-ts` bucket.
///
/// `.get(0..10)`, never `[0..10]`: a short or multibyte ts is legal input
/// under keys-present validation and must not panic (003 audit MAJOR-3).
/// The `.filter(is_day)` is v1's F2 tightening — `2026/07/09…` used to
/// slip through as a phantom "day"; now it's visibly bad-ts.
pub fn day_of_ts(ts: &str) -> Option<&str> {
    ts.get(0..10).filter(|prefix| is_day(prefix))
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
    let day = get_str(line, "ts").and_then(day_of_ts).unwrap_or("bad-ts");
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

    /// F2 (new in v1): a 10-char prefix that isn't YYYY-MM-DD shaped is
    /// bad-ts now, not a phantom "day".
    #[test]
    fn f2_pattern_violating_prefix_is_bad_ts() {
        let slashes = GOOD.replace("2026-07-09T01:02:03Z", "2026/07/09T01:02:03Z");
        assert_eq!(
            classify(&slashes, &REQUIRED_KEYS),
            Line::Event {
                kind: "gate.approved",
                day: "bad-ts"
            }
        );
    }

    #[test]
    fn f2_day_rule_table() {
        assert!(is_day("2026-07-09"));
        assert!(is_day("9999-99-99")); // shape check, not calendar check
        assert!(!is_day("2026-7-9")); // too short
        assert!(!is_day("2026/07/09")); // wrong separators
        assert!(!is_day("2026-07-09T")); // too long
        assert!(!is_day("🦀🦀-🦀🦀-🦀")); // multibyte
        assert_eq!(day_of_ts("2026-07-09T01:02:03Z"), Some("2026-07-09"));
        assert_eq!(day_of_ts("2026"), None);
        assert_eq!(day_of_ts("2026/07/09T01:02:03Z"), None);
    }
}
