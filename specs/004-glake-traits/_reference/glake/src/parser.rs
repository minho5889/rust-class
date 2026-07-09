//! Sitting J: `classify` goes behind a **trait**, and a rival backend
//! arrives. This module is the T1/T2/T6 lesson in one file:
//!
//! - **T1 (traits):** [`EventParser`] is defined once, implemented twice —
//!   [`HandParser`] (the 003 scanner) and [`SerdeParser`] (`serde_json`).
//! - **T2 (dispatch):** the lib's pipeline (`tally::tally_filtered`) is
//!   *generic* over `P: EventParser + ?Sized` (static dispatch,
//!   monomorphized); the binary's `--parser` flag is the ONE dynamic seam
//!   (`Box<dyn EventParser>`). Same trait, both shapes, side by side.
//! - **T6 (the cost):** a `dyn` boundary can't return borrows of serde's
//!   internal `Value`, so the trait must return an **owned**
//!   [`ClassifiedLine`] (`String` fields). v0's zero-copy `Line<'a>` still
//!   exists (`classify.rs`); `HandParser` converts at the boundary — that
//!   conversion is exactly the allocation the lens run (F9) measures.
//!
//! ### Where the backends are allowed to disagree
//!
//! The hand scanner is *lenient* (it walks bytes and answers "is this key
//! here?", happily, on truncated or trailing-junk lines) and returns string
//! values *raw* (escape sequences unexpanded). `serde_json` is *strict*
//! (whole line must be valid JSON) and *normalizing* (unescapes values,
//! and on **duplicate top-level keys** keeps the LAST while the scanner
//! finds the FIRST). Behavior on such inputs is **unspecified for glake**:
//! real envelope writers never emit them. The F8b property
//! (tests/prop_parsers.rs) pins the disagreement to exactly those three
//! classes; on well-formed input F8a proves the backends identical.
//! See specs/004-glake-traits/_reference/NOTES.md for the full write-up.

use crate::classify::{Line, classify, day_of_ts};

/// The owned verdict on one line — [`Line`]'s mirror that can cross a
/// trait boundary (and outlive the input it was parsed from).
///
/// One variant more than `Line`: [`ClassifiedLine::Unparseable`]. The hand
/// scanner has no concept of "failed to parse" (it never parses — it
/// scans), so `HandParser` never returns it; `SerdeParser` returns it for
/// any line that isn't a JSON *object*. Stats fold it into the malformed
/// count (`tally.rs`); `validate` reports it as its own finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifiedLine {
    /// Whitespace-only: skipped everywhere, counted nowhere.
    Blank,
    /// Not parseable as a JSON object at all (strict backends only).
    Unparseable,
    /// Missing a required key; carries the first missing key's name.
    Malformed { missing: String },
    /// A proper event record: its type, and its day (F2 day rule).
    Event { kind: String, day: String },
}

/// The boundary allocation, in one visible place: borrowed in, owned out.
/// This `impl From` is what `HandParser` pays per line — two `String`s per
/// event — and it is the exact delta the F9 lens comparison measures
/// against v0's zero-copy pipeline.
impl From<Line<'_>> for ClassifiedLine {
    fn from(line: Line<'_>) -> Self {
        match line {
            Line::Blank => ClassifiedLine::Blank,
            Line::Malformed { missing } => ClassifiedLine::Malformed {
                missing: missing.to_owned(),
            },
            Line::Event { kind, day } => ClassifiedLine::Event {
                kind: kind.to_owned(),
                day: day.to_owned(),
            },
        }
    }
}

/// One line in, one owned verdict out — the seam both backends implement.
///
/// `&self` (not `Self`) so the trait is **object-safe**: the binary holds a
/// `Box<dyn EventParser>` chosen at runtime from `--parser`. `required` is
/// the caller's key list (always `&REQUIRED_KEYS` in the tool) — kept as a
/// parameter so tests can probe small key sets.
pub trait EventParser {
    /// Classify one line of the lake against `required` top-level keys.
    fn classify(&self, line: &str, required: &[&str]) -> ClassifiedLine;
}

/// Backend 1: the hand-rolled 003 scanner (`scan.rs` + `classify.rs`),
/// wrapped. All the borrowing logic is untouched — this type only converts
/// the borrowed `Line<'a>` to an owned [`ClassifiedLine`] at the boundary.
///
/// Never returns [`ClassifiedLine::Unparseable`]: the scanner doesn't
/// parse, so nothing can *fail* to parse — junk simply comes back as
/// "missing the first required key".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandParser;

impl EventParser for HandParser {
    fn classify(&self, line: &str, required: &[&str]) -> ClassifiedLine {
        classify(line, required).into()
    }
}

/// Backend 2: `serde_json`, parsing the whole line to a [`serde_json::Value`]
/// and extracting per the SAME semantics as the hand scanner: top-level
/// keys present, `event_type`/`ts` read only when they're strings, day per
/// the F2 rule.
///
/// Differences from [`HandParser`] on *degenerate* input (strictness,
/// escape normalization, duplicate keys) are documented on the module and
/// pinned by the F8b property; behavior there is unspecified for glake.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SerdeParser;

impl EventParser for SerdeParser {
    fn classify(&self, line: &str, required: &[&str]) -> ClassifiedLine {
        // Same blank rule as the scanner, checked first: a whitespace line
        // is Blank, not Unparseable, on BOTH backends.
        if line.trim().is_empty() {
            return ClassifiedLine::Blank;
        }
        // Strictness lives here: not JSON, or JSON but not an object
        // (`123`, `[..]`, `"str"`, `null`) → Unparseable.
        let Ok(serde_json::Value::Object(map)) = serde_json::from_str(line) else {
            return ClassifiedLine::Unparseable;
        };
        // Keys checked in the caller's order, exactly like the scanner, so
        // both backends report the same FIRST missing key.
        for &key in required {
            if !map.contains_key(key) {
                return ClassifiedLine::Malformed {
                    missing: key.to_owned(),
                };
            }
        }
        // `as_str` mirrors the scanner's "string values only" rule:
        // a non-string event_type/ts lands in the same sentinel buckets.
        let kind = map
            .get("event_type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("non-string");
        let day = map
            .get("ts")
            .and_then(serde_json::Value::as_str)
            .and_then(day_of_ts)
            .unwrap_or("bad-ts");
        ClassifiedLine::Event {
            kind: kind.to_owned(),
            day: day.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::REQUIRED_KEYS;

    const GOOD: &str = r#"{"event_id":"1","ts":"2026-07-09T01:02:03Z","session_id":"s","actor":"human","event_type":"gate.approved","schema_version":1,"payload":{}}"#;

    fn event(kind: &str, day: &str) -> ClassifiedLine {
        ClassifiedLine::Event {
            kind: kind.to_owned(),
            day: day.to_owned(),
        }
    }

    /// [E] F7: both backends, same verdicts on the canonical cases.
    #[test]
    fn f7_backends_agree_on_the_basics() {
        let no_actor = GOOD.replace(r#""actor":"human","#, "");
        for parser in [&HandParser as &dyn EventParser, &SerdeParser] {
            assert_eq!(
                parser.classify(GOOD, &REQUIRED_KEYS),
                event("gate.approved", "2026-07-09")
            );
            assert_eq!(
                parser.classify("   ", &REQUIRED_KEYS),
                ClassifiedLine::Blank
            );
            assert_eq!(
                parser.classify(&no_actor, &REQUIRED_KEYS),
                ClassifiedLine::Malformed {
                    missing: "actor".to_owned()
                }
            );
        }
    }

    /// The documented strictness split: junk is Unparseable to serde,
    /// "missing the first key" to the scanner. HandParser never returns
    /// Unparseable.
    #[test]
    fn strictness_divergence_is_as_documented() {
        for junk in ["not json at all", "[1,2]", "123", "\"str\"", "null", "{"] {
            assert_eq!(
                SerdeParser.classify(junk, &REQUIRED_KEYS),
                ClassifiedLine::Unparseable,
                "serde on {junk:?}"
            );
            assert_eq!(
                HandParser.classify(junk, &REQUIRED_KEYS),
                ClassifiedLine::Malformed {
                    missing: "event_id".to_owned()
                },
                "hand on {junk:?}"
            );
        }
    }

    /// The boundary conversion preserves meaning field-for-field.
    #[test]
    fn owned_mirror_matches_borrowed_verdict() {
        assert_eq!(ClassifiedLine::from(Line::Blank), ClassifiedLine::Blank);
        assert_eq!(
            ClassifiedLine::from(Line::Event {
                kind: "a.b",
                day: "2026-07-09"
            }),
            event("a.b", "2026-07-09")
        );
        assert_eq!(
            ClassifiedLine::from(Line::Malformed { missing: "ts" }),
            ClassifiedLine::Malformed {
                missing: "ts".to_owned()
            }
        );
    }
}
