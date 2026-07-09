//! The heart of glake (Sitting C/D): a tiny scanner that answers two
//! questions about ONE line of JSON — "does it have this top-level key?"
//! and "what's the string value of this key?" — by stepping through the
//! characters. No serde; returned values are **borrowed slices** of the
//! input (`&'a str`): zero copies, which the lens makes visible in L6.
//!
//! Scope (by design): one line, top level only, string values only.
//! Never panics on any input (property R8).

/// What sits after a top-level key's colon.
enum Value<'a> {
    Str(&'a str),
    Other,
}

/// Find `key` at the top level of `line`; classify its value.
/// The single walk both `has_key` and `get_str` share.
fn find_value<'a>(line: &'a str, key: &str) -> Option<Value<'a>> {
    let bytes = line.as_bytes();
    let mut depth: i32 = 0; // { } and [ ] nesting, outside strings
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'{' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b']' => {
                depth -= 1;
                i += 1;
            }
            b'"' => {
                // A string token starts. Scan to its closing quote,
                // honouring \" escapes.
                let start = i + 1;
                // `?` on an Option: unterminated string → give up cleanly.
                let end = string_end(bytes, start)?;
                // Only depth 1 (directly inside the outermost object) and
                // only if the next non-space char is ':' is this a KEY.
                let mut j = end + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let is_key = depth == 1 && j < bytes.len() && bytes[j] == b':';
                if is_key && &line[start..end] == key {
                    // Found our key: classify the value after the ':'.
                    let mut v = j + 1;
                    while v < bytes.len() && bytes[v].is_ascii_whitespace() {
                        v += 1;
                    }
                    if v < bytes.len() && bytes[v] == b'"' {
                        let vstart = v + 1;
                        let vend = string_end(bytes, vstart)?;
                        return Some(Value::Str(&line[vstart..vend]));
                    }
                    return Some(Value::Other);
                }
                // Not our key (or a value string): skip past it either way.
                i = end + 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Index of the closing quote of a string whose content starts at `start`
/// (i.e. `start` is just past the opening quote). Escape-aware.
fn string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    let mut escaped = false;
    while i < bytes.len() {
        match (escaped, bytes[i]) {
            (true, _) => escaped = false,
            (false, b'\\') => escaped = true,
            (false, b'"') => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Does `line` have `key` at the top level (any value type)?
pub fn has_key(line: &str, key: &str) -> bool {
    find_value(line, key).is_some()
}

/// The string value of a top-level `key`, borrowed from `line`.
/// `None` if the key is absent or its value isn't a string.
/// (Escape sequences inside the value are returned raw — unescaping would
/// require allocating, and glake never needs it.)
pub fn get_str<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    match find_value(line, key) {
        Some(Value::Str(s)) => Some(s),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scanner correctness table (design: "R8 proves no-panic, not right
    /// answers" — these prove the answers).
    #[test]
    fn known_lines_give_known_answers() {
        let line = r#"{"event_type":"gate.approved","n":3,"payload":{"actor":"inner"}}"#;
        assert_eq!(get_str(line, "event_type"), Some("gate.approved"));
        assert!(has_key(line, "event_type"));
        assert!(has_key(line, "n")); // present, non-string
        assert_eq!(get_str(line, "n"), None); // ...but not a string
        assert!(has_key(line, "payload"));
        assert_eq!(get_str(line, "payload"), None);
        // top-level only: "actor" lives inside payload, must NOT match
        assert!(!has_key(line, "actor"));
        assert_eq!(get_str(line, "actor"), None);
        // absent key
        assert!(!has_key(line, "missing"));
    }

    #[test]
    fn escaped_quotes_are_handled() {
        let line = r#"{"msg":"he said \"hi\"","tag":"x"}"#;
        assert_eq!(get_str(line, "msg"), Some(r#"he said \"hi\""#));
        assert_eq!(get_str(line, "tag"), Some("x"));
    }

    #[test]
    fn garbage_is_survivable() {
        for junk in [
            "",
            "{",
            "\"",
            "{\"a\"",
            "not json at all",
            "{\"a\":}",
            "🦀🦀🦀",
        ] {
            let _ = has_key(junk, "a");
            let _ = get_str(junk, "a");
        }
    }

    #[test]
    fn arrays_do_not_leak_keys() {
        // strings inside a top-level array value are values, not keys
        let line = r#"{"list":["actor","ts"],"ok":"yes"}"#;
        assert!(!has_key(line, "actor"));
        assert_eq!(get_str(line, "ok"), Some("yes"));
    }
}
