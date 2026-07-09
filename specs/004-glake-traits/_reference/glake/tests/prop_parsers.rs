//! [P] F8 — backend agreement, in two properties (F8 as amended: the 004
//! audit proved EXACT equivalence on arbitrary input is unsatisfiable —
//! the hand scanner is lenient and keeps escapes raw; serde_json is strict
//! and normalizes — so the requirement splits):
//!
//! - **F8a** (`prop_f8a_…`): on WELL-FORMED input — valid JSON objects, no
//!   duplicate top-level keys, escape-free extracted values — the two
//!   backends are EXACTLY equivalent. This is the class of input real
//!   envelope writers produce; it's why `--parser` is a free choice on the
//!   real lake.
//! - **F8b** (`prop_f8b_…`): on the FULL mixed strategy (003's R8 blend:
//!   arbitrary garbage + JSON-ish fragments + valid envelopes, plus
//!   deliberately degenerate envelopes), every line either gets identical
//!   verdicts, or the disagreement falls in one of EXACTLY three
//!   documented classes:
//!   1. **strictness** — serde says `Unparseable` where the scanner,
//!      which never parses, still answers (truncations, trailing junk,
//!      non-object JSON);
//!   2. **escape normalization** — serde unescapes extracted values,
//!      the scanner returns them raw;
//!   3. **duplicate top-level keys** — the scanner finds the FIRST,
//!      serde's `Value` keeps the LAST.
//!
//!   Any divergence outside those classes fails the property. Behavior on
//!   classes 2–3 is unspecified-for-glake (see parser.rs docs and
//!   specs/004-glake-traits/_reference/NOTES.md).

use glake::classify::REQUIRED_KEYS;
use glake::parser::{ClassifiedLine, EventParser, HandParser, SerdeParser};
use proptest::prelude::*;
use serde_json::{Value, json};

// ---------- the well-formed strategy (F8a) ----------

/// Strings that survive `serde_json::to_string` byte-identical: no quotes,
/// no backslashes, no control chars (multibyte is fine — serde_json emits
/// raw UTF-8).
fn safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .:_🦀-]{0,16}"
}

/// Any JSON value whose serialization contains no escapes.
fn safe_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        safe_string().prop_map(Value::String),
        any::<i64>().prop_map(|n| json!(n)),
        Just(json!(null)),
        Just(json!(true)),
        Just(json!({"inner":"x"})),
        Just(json!(["a", "b"])),
    ]
}

/// `ts` values covering every day-rule bucket: real days, short,
/// multibyte, 10-char-but-wrong-shape, and non-string.
fn ts_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        (2020u32..2030, 1u32..13, 1u32..29)
            .prop_map(|(y, m, d)| json!(format!("{y:04}-{m:02}-{d:02}T00:00:00Z"))),
        Just(json!("2026")),
        Just(json!("🦀🦀🦀🦀")),
        Just(json!("2026/07/09T00:00:00Z")),
        Just(json!(123)), // non-string → bad-ts on both backends
    ]
}

/// `event_type` values: safe strings or a non-string (→ "non-string").
fn kind_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        "[a-z🦀]{1,8}\\.[a-z0-9]{1,8}".prop_map(Value::String),
        Just(json!(7)),
    ]
}

/// A well-formed line: a real JSON object built in a `serde_json::Map`
/// (which cannot hold duplicate keys) from any subset of the required
/// keys plus safe extras, then serialized by serde_json itself — valid by
/// construction, escape-free by charset.
fn wellformed_line() -> impl Strategy<Value = String> {
    (
        prop::sample::subsequence(REQUIRED_KEYS.to_vec(), 0..=REQUIRED_KEYS.len()),
        ts_value(),
        kind_value(),
        prop::collection::btree_map("[a-z_]{1,8}", safe_value(), 0..4),
    )
        .prop_map(|(present, ts, kind, extras)| {
            let mut map = serde_json::Map::new();
            for key in present {
                let value = match key {
                    "ts" => ts.clone(),
                    "event_type" => kind.clone(),
                    "schema_version" => json!(1),
                    "payload" => json!({}),
                    _ => json!("s"),
                };
                map.insert(key.to_owned(), value);
            }
            // Extras may overwrite a required key's value — still a single
            // occurrence in the output, so still duplicate-free.
            for (k, v) in extras {
                map.insert(k, v);
            }
            Value::Object(map).to_string()
        })
}

/// F8a's full input space: well-formed lines plus blank lines.
fn f8a_input() -> impl Strategy<Value = String> {
    prop_oneof![
        9 => wellformed_line(),
        1 => "[ \t]{0,4}".prop_map(|s| s),
    ]
}

// ---------- the mixed strategy (F8b) ----------

/// 003's R8 JSON-ish fragment generator, verbatim: plausible-to-broken
/// key/value soup, half the time truncated mid-line.
fn json_ish() -> impl Strategy<Value = String> {
    let key = prop_oneof![Just("actor".to_string()), "[a-z🦀\"\\\\]{0,6}"];
    let val = prop_oneof![
        r#"[a-z0-9 🦀]{0,10}"#.prop_map(|s| format!("\"{s}\"")),
        Just("123".to_string()),
        Just("{\"inner\":\"x\"}".to_string()),
        Just("[\"a\",\"b\"]".to_string()),
        Just("\"esc\\\"aped\"".to_string()),
    ];
    (prop::collection::vec((key, val), 0..4), any::<bool>()).prop_map(|(pairs, truncate)| {
        let body: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
        let mut line = format!("{{{}}}", body.join(","));
        if truncate && line.len() > 2 {
            let mut cut = line.len() / 2;
            while !line.is_char_boundary(cut) {
                cut += 1;
            }
            line.truncate(cut);
        }
        line
    })
}

/// One generated F8b case. The degenerate arms carry the generator's own
/// knowledge of what each backend must answer — divergence isn't merely
/// tolerated there, it's PINNED to the exact documented behavior.
#[derive(Debug, Clone)]
enum Case {
    /// Arbitrary unicode garbage (class 1 divergences allowed).
    Garbage(String),
    /// JSON-ish fragments with escapes/truncation (class 1 allowed).
    JsonIsh(String),
    /// Well-formed (F8a's generator): exact agreement REQUIRED.
    WellFormed(String),
    /// Class 3 by construction: `event_type` appears twice at top level.
    DupKeys {
        line: String,
        first_kind: String,
        last_kind: String,
        day: String,
    },
    /// Class 2 by construction: an escape inside an extracted value.
    Escaped {
        line: String,
        hand_expects: ClassifiedLine,
        serde_expects: ClassifiedLine,
    },
}

fn dup_keys_case() -> impl Strategy<Value = Case> {
    (
        "[a-z]{1,6}\\.[a-z]{1,6}",
        "[a-z]{1,6}\\.[a-z]{1,6}",
        (2020u32..2030, 1u32..13, 1u32..29),
    )
        .prop_map(|(first_kind, last_kind, (y, m, d))| {
            let day = format!("{y:04}-{m:02}-{d:02}");
            let line = format!(
                r#"{{"event_id":"1","ts":"{day}T00:00:00Z","session_id":"s","actor":"human","event_type":"{first_kind}","schema_version":1,"payload":{{}},"event_type":"{last_kind}"}}"#
            );
            Case::DupKeys {
                line,
                first_kind,
                last_kind,
                day,
            }
        })
}

fn escaped_case() -> impl Strategy<Value = Case> {
    fn event(kind: &str, day: &str) -> ClassifiedLine {
        ClassifiedLine::Event {
            kind: kind.to_owned(),
            day: day.to_owned(),
        }
    }
    prop_oneof![
        // an escaped quote inside event_type: hand keeps `a\"b` raw,
        // serde normalizes to `a"b`
        ("[a-z]{1,4}", "[a-z]{1,4}").prop_map(|(a, b)| {
            let line = format!(
                r#"{{"event_id":"1","ts":"2026-07-09T00:00:00Z","session_id":"s","actor":"human","event_type":"{a}\"{b}","schema_version":1,"payload":{{}}}}"#
            );
            Case::Escaped {
                line,
                hand_expects: event(&format!("{a}\\\"{b}"), "2026-07-09"),
                serde_expects: event(&format!("{a}\"{b}"), "2026-07-09"),
            }
        }),
        // a unicode escape for '-' inside ts: the raw JSON text becomes
        // `"ts":"2026` + backslash + `u002d07-09…"`. Hand sees a raw
        // 10-byte prefix (`2026` + backslash + `u002d`) that fails the
        // day pattern → bad-ts; serde unescapes it to `2026-07-09…`,
        // a real day.
        Just(()).prop_map(|()| {
            let bs = '\\'; // one backslash character
            let ts_escaped = format!("2026{bs}u002d07-09T00:00:00Z");
            let line = format!(
                r#"{{"event_id":"1","ts":"{ts_escaped}","session_id":"s","actor":"human","event_type":"gate.approved","schema_version":1,"payload":{{}}}}"#
            );
            Case::Escaped {
                line,
                hand_expects: event("gate.approved", "bad-ts"),
                serde_expects: event("gate.approved", "2026-07-09"),
            }
        }),
    ]
}

fn f8b_input() -> impl Strategy<Value = Case> {
    prop_oneof![
        2 => any::<String>().prop_map(Case::Garbage),
        3 => json_ish().prop_map(Case::JsonIsh),
        3 => wellformed_line().prop_map(Case::WellFormed),
        1 => dup_keys_case(),
        1 => escaped_case(),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    /// [P] F8a — exact backend equivalence on well-formed input.
    #[test]
    fn prop_f8a_backends_identical_on_wellformed(line in f8a_input()) {
        let hand = HandParser.classify(&line, &REQUIRED_KEYS);
        let serde = SerdeParser.classify(&line, &REQUIRED_KEYS);
        prop_assert_ne!(
            &serde,
            &ClassifiedLine::Unparseable,
            "well-formed lines parse: {}", line
        );
        prop_assert_eq!(hand, serde, "backends must agree on: {}", line);
    }

    /// [P] F8b — on arbitrary input, agreement or a documented divergence
    /// class; nothing else.
    #[test]
    fn prop_f8b_divergences_fall_in_documented_classes(case in f8b_input()) {
        match case {
            // Free-range arms: agree, or class (1) — serde refuses what
            // the lenient scanner still classified.
            Case::Garbage(line) | Case::JsonIsh(line) => {
                let hand = HandParser.classify(&line, &REQUIRED_KEYS);
                let serde = SerdeParser.classify(&line, &REQUIRED_KEYS);
                prop_assert_ne!(
                    &hand,
                    &ClassifiedLine::Unparseable,
                    "the scanner never returns Unparseable: {}", line
                );
                if hand != serde {
                    prop_assert_eq!(
                        &serde,
                        &ClassifiedLine::Unparseable,
                        "undocumented divergence on {}: hand={:?} serde={:?}",
                        line, &hand, &serde
                    );
                }
            }
            // Well-formed: exact agreement, no divergence class applies.
            Case::WellFormed(line) => {
                prop_assert_eq!(
                    HandParser.classify(&line, &REQUIRED_KEYS),
                    SerdeParser.classify(&line, &REQUIRED_KEYS),
                    "well-formed input admits no divergence: {}", line
                );
            }
            // Class (3), pinned exactly: hand reads the FIRST duplicate,
            // serde keeps the LAST.
            Case::DupKeys { line, first_kind, last_kind, day } => {
                prop_assert_eq!(
                    HandParser.classify(&line, &REQUIRED_KEYS),
                    ClassifiedLine::Event { kind: first_kind, day: day.clone() },
                    "hand takes the first duplicate: {}", line
                );
                prop_assert_eq!(
                    SerdeParser.classify(&line, &REQUIRED_KEYS),
                    ClassifiedLine::Event { kind: last_kind, day },
                    "serde keeps the last duplicate: {}", line
                );
            }
            // Class (2), pinned exactly: raw vs unescaped extraction.
            Case::Escaped { line, hand_expects, serde_expects } => {
                prop_assert_eq!(
                    HandParser.classify(&line, &REQUIRED_KEYS),
                    hand_expects,
                    "hand keeps escapes raw: {}", line
                );
                prop_assert_eq!(
                    SerdeParser.classify(&line, &REQUIRED_KEYS),
                    serde_expects,
                    "serde normalizes escapes: {}", line
                );
            }
        }
    }
}
