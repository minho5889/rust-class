//! [P] R8 — the scanner, fed ANY string, returns cleanly: no panic, no
//! out-of-bounds, and any returned slice really lies inside the input.
//! Strategy per the design's Properties table: 50% arbitrary garbage
//! (incl. multibyte), 50% JSON-ish fragments with escapes and truncations.

use glake::scan::{get_str, has_key};
use proptest::prelude::*;

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
            // cut at a char boundary to build a broken-but-valid &str
            let mut cut = line.len() / 2;
            while !line.is_char_boundary(cut) {
                cut += 1;
            }
            line.truncate(cut);
        }
        line
    })
}

fn input() -> impl Strategy<Value = String> {
    prop_oneof![any::<String>(), json_ish()]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn r8_never_panics_and_slices_stay_in_bounds(line in input(), key in "[a-z_🦀]{0,8}") {
        // has_key: just must not panic
        let _ = has_key(&line, &key);
        // get_str: any returned slice must physically live inside `line`
        if let Some(s) = get_str(&line, &key) {
            let line_start = line.as_ptr() as usize;
            let s_start = s.as_ptr() as usize;
            prop_assert!(s_start >= line_start);
            prop_assert!(s_start + s.len() <= line_start + line.len());
        }
    }

    /// Round-trip half of R8: a well-formed line we construct ourselves is
    /// always found, with exactly the value we put in (no escapes case).
    #[test]
    fn r8_wellformed_roundtrip(val in "[a-zA-Z0-9 .:-]{0,20}") {
        let line = format!(r#"{{"probe":"{val}","other":1}}"#);
        prop_assert_eq!(get_str(&line, "probe"), Some(val.as_str()));
        prop_assert!(has_key(&line, "other"));
    }
}
