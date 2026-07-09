//! The strict door (A1/A2): decide, for one raw request body, *accept* or
//! *reject with the first problem* — and nothing else. Pure function, no
//! I/O, no state: that's what makes it unit-testable without a runtime.
//!
//! The policy is deliberately asymmetric to glake (requirements, "the door
//! check, defined once"): **glake is a tolerant reader** (malformed lines
//! get counted, never crash a scan) but **relay is a strict gatekeeper**
//! (what fails the door gets a 400 and never touches the lake). A writer
//! that accepts what it can't partition corrupts the lake.
//!
//! "Strict" means exactly two things — and the *code* for both belongs to
//! the glake library (A8: reuse your own public API, don't reimplement):
//!
//! 1. glake's classification passes: the body is a JSON object with every
//!    `REQUIRED_KEYS` entry present at top level (`SerdeParser`);
//! 2. the event's `ts` has a **comparable day** (004's F2 rule) — glake
//!    surfaces failure as the `"bad-ts"` sentinel day.
//!
//! It is NOT full JSON-Schema validation: no `schema_version` value check,
//! no RFC3339 audit. `9999-99-99` is a fine day; partitions mean what the
//! event claims.

use glake::classify::REQUIRED_KEYS;
use glake::parser::{ClassifiedLine, EventParser, SerdeParser};

/// Why a body was turned away — one variant per clause of the door check,
/// in the requirements' fixed first-problem order (A2). The `Display`
/// string of each variant IS the user-visible `{"error":"…"}` message
/// (lowercase, no trailing period — C-GOOD-ERR, same rubric as glake).
///
/// `thiserror` writes `Display` and `std::error::Error` from the
/// attributes; the *messages* are still a design decision we own.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Rejection {
    /// The body isn't a JSON object at all — unparseable junk, a bare
    /// array/number/string, or nothing but whitespace. (glake distinguishes
    /// `Blank` from `Unparseable` because a *reader* must skip blank lines
    /// silently; a *door* turns both away with the same words.)
    #[error("not a json object")]
    NotAJsonObject,

    /// A JSON object, but a required envelope key is absent. Carries the
    /// FIRST missing key in `REQUIRED_KEYS` order — that ordering comes
    /// from glake's classifier, not from us, so both tools always name the
    /// same key for the same body.
    #[error("missing key: {0}")]
    MissingKey(String),

    /// All keys present, but `ts` has no comparable `YYYY-MM-DD` prefix —
    /// glake's F2 day rule said `bad-ts`. We can't partition it, so we
    /// refuse it (the tolerant reader would have bucketed it instead).
    #[error("no comparable day in ts")]
    NoComparableDay,
}

/// A body the door let through: its partition day and the exact line the
/// writer will append.
///
/// `line` is the body **re-serialized compact** (serde_json's canonical
/// one-line form). A raw body may legally span lines — pretty-printed JSON
/// is still JSON — but the lake is JSONL: one event, one line. Re-encoding
/// the parsed `Value` is what guarantees that (requirements A4 rev 2.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accepted {
    /// The event's own day (`YYYY-MM-DD` from its `ts`) — the `dt=` bucket
    /// it belongs to. The event's claim, not the server's clock: partitions
    /// mean "when it happened", not "when we heard about it".
    pub day: String,
    /// The one compact JSONL line to append.
    pub line: String,
}

/// Run one body through the door: `Ok(Accepted)` or the FIRST problem, in
/// the A2 fixed order (not-a-JSON-object → first missing key → no
/// comparable day).
///
/// That order isn't enforced here by careful `if` sequencing — it falls
/// out of `ClassifiedLine`'s shape: glake can only report a missing key on
/// something that *did* parse as an object, and only reports a day on
/// something that *had* all its keys. Make bad states unrepresentable and
/// the error order stops being a bug you can write.
pub fn check(body: &str) -> Result<Accepted, Rejection> {
    // teach: SerdeParser takes the whole body as "one line" — serde_json is
    // happy with embedded newlines (JSON whitespace), and glake's blank
    // check (`trim().is_empty()`) is too. No pre-flattening needed.
    match SerdeParser.classify(body, &REQUIRED_KEYS) {
        // A whitespace-only body and un-JSON junk get the same words at
        // this door (see Rejection::NotAJsonObject).
        ClassifiedLine::Blank | ClassifiedLine::Unparseable => Err(Rejection::NotAJsonObject),
        ClassifiedLine::Malformed { missing } => Err(Rejection::MissingKey(missing)),
        // glake never rejects on ts — it *buckets*: an uncomparable day
        // comes back as the "bad-ts" sentinel. The strict door is exactly
        // this one extra rule on top of the tolerant reader.
        ClassifiedLine::Event { day, .. } if day == "bad-ts" => Err(Rejection::NoComparableDay),
        ClassifiedLine::Event { day, .. } => {
            // teach: the classifier already parsed the body once (inside
            // SerdeParser) but only hands back the verdict — its trait
            // boundary returns owned summaries, not the parsed Value. We
            // parse a second time to get the Value for re-serialization.
            // Two parses per *accepted* event is the price of keeping
            // glake's public API untouched; a lab relay pays it happily.
            let value: serde_json::Value =
                serde_json::from_str(body).map_err(|_| Rejection::NotAJsonObject)?;
            // teach: `to_string` on a `Value` cannot actually fail (every
            // Value serializes), but the signature says Result and A9 says
            // no unwrap — so the impossible branch maps to a rejection
            // instead of a panic. Cost: one line. A panicking handler
            // would cost the whole service (the Cloudflare lesson).
            let line = serde_json::to_string(&value).map_err(|_| Rejection::NotAJsonObject)?;
            Ok(Accepted { day, line })
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"event_id":"1","ts":"2026-07-09T01:02:03Z","session_id":"s","actor":"human","event_type":"gate.approved","schema_version":1,"payload":{}}"#;

    /// The door table, one row per Rejection variant plus the happy path.
    #[test]
    fn door_verdicts_in_fixed_order() {
        // clause 1: not a JSON object (junk, non-object JSON, blank)
        for bad in ["not json", "[1,2]", "123", "\"str\"", "null", "", "  \n "] {
            assert_eq!(check(bad), Err(Rejection::NotAJsonObject), "{bad:?}");
        }
        // clause 2: first missing key, in REQUIRED_KEYS order
        let no_actor = GOOD.replace(r#""actor":"human","#, "");
        assert_eq!(
            check(&no_actor),
            Err(Rejection::MissingKey("actor".to_owned()))
        );
        // order check: a body missing a key AND carrying a bad ts reports
        // the missing key — clause 2 fires before clause 3.
        let both = no_actor.replace("2026-07-09T01:02:03Z", "nope");
        assert_eq!(check(&both), Err(Rejection::MissingKey("actor".to_owned())));
        // clause 3: keys all present, day not comparable
        for bad_ts in ["nope", "2026/07/09T01:02:03Z", "2026"] {
            let body = GOOD.replace("2026-07-09T01:02:03Z", bad_ts);
            assert_eq!(check(&body), Err(Rejection::NoComparableDay), "{bad_ts}");
        }
        // and a non-string ts is just as uncomparable as a malformed one
        let numeric_ts = GOOD.replace(r#""ts":"2026-07-09T01:02:03Z""#, r#""ts":1234"#);
        assert_eq!(check(&numeric_ts), Err(Rejection::NoComparableDay));
        // happy path: day extracted, line unchanged (GOOD is already compact)
        let ok = check(GOOD).unwrap();
        assert_eq!(ok.day, "2026-07-09");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&ok.line).unwrap(),
            serde_json::from_str::<serde_json::Value>(GOOD).unwrap()
        );
    }

    /// A4 rev 2.1's reason to exist: a pretty-printed body spans lines but
    /// must land as ONE compact line.
    #[test]
    fn multiline_body_is_reserialized_to_one_line() {
        let pretty =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(GOOD).unwrap())
                .unwrap();
        assert!(pretty.contains('\n'), "fixture should span lines");
        let ok = check(&pretty).unwrap();
        assert!(!ok.line.contains('\n'), "lake line must be one line");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&ok.line).unwrap(),
            serde_json::from_str::<serde_json::Value>(GOOD).unwrap()
        );
    }

    /// Weird-but-comparable days are ACCEPTED — partitions mean what the
    /// event claims (shape check, not calendar check).
    #[test]
    fn weird_but_comparable_days_pass() {
        for day in ["1999-01-01", "9999-99-99"] {
            let body = GOOD.replace("2026-07-09", day);
            assert_eq!(check(&body).unwrap().day, day);
        }
    }

    /// C-GOOD-ERR style: lowercase, no trailing period (same rubric glake's
    /// error type is tested against).
    #[test]
    fn rejection_messages_are_lowercase_no_period() {
        for r in [
            Rejection::NotAJsonObject,
            Rejection::MissingKey("actor".into()),
            Rejection::NoComparableDay,
        ] {
            let msg = r.to_string();
            assert!(!msg.starts_with(char::is_uppercase), "{msg}");
            assert!(!msg.ends_with('.'), "{msg}");
        }
    }
}
