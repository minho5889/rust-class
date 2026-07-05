//! [E] R8 end-to-end — a REAL emitted trace (from running the demo example
//! with the feature on) must conform to the registered schema
//! `datalake/schema/memlens.v1.json`: envelope fields present, every
//! event_type in the memlens family, and every payload carrying that
//! type's `required` fields. Schema-driven: the required lists are read
//! from the schema file itself, so schema and writer cannot drift apart
//! silently.

use std::process::Command;

#[test]
fn real_trace_conforms_to_registered_schema() {
    let target = std::env::temp_dir().join("memlens-r8-target");
    let trace = std::env::temp_dir().join(format!("memlens-r8-{}.jsonl", std::process::id()));
    let _ = std::fs::remove_file(&trace);

    // Debug build WITH the feature (release+feature is banned by R14b).
    let run = Command::new(env!("CARGO"))
        .args([
            "run",
            "-p",
            "memlens",
            "--example",
            "demo",
            "--features",
            "memlens",
        ])
        .env("CARGO_TARGET_DIR", &target)
        .env("MEMLENS_TRACE", &trace)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("spawn cargo run");
    assert!(
        run.status.success(),
        "traced demo run failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // Load the registered schema's $defs → required-field lists.
    let schema_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../datalake/schema/memlens.v1.json"
    );
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(schema_path).expect("schema readable"))
            .expect("schema is JSON");
    let defs = schema["$defs"].as_object().expect("$defs object");
    let required_for = |ty: &str| -> Vec<String> {
        let mut def = &defs[ty];
        // Follow one level of $ref (dealloc/scope_exit/marker alias defs).
        if let Some(r) = def.get("$ref").and_then(|r| r.as_str()) {
            let name = r.trim_start_matches("#/$defs/");
            def = &defs[name];
        }
        def["required"]
            .as_array()
            .expect("required list")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .collect()
    };

    let content = std::fs::read_to_string(&trace).expect("trace exists");
    let mut count = 0usize;
    let mut saw_meta = false;
    let mut saw_realloc = false;
    for line in content.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line).expect("valid JSON line");
        // Envelope fields (envelope.v1.json required set).
        for key in [
            "event_id",
            "ts",
            "session_id",
            "actor",
            "event_type",
            "schema_version",
            "payload",
        ] {
            assert!(v.get(key).is_some(), "envelope missing {key}: {line}");
        }
        assert_eq!(v["actor"], "memlens");
        let ty = v["event_type"].as_str().expect("event_type string");
        assert!(ty.starts_with("memlens."), "family must be memlens.*: {ty}");
        assert!(defs.contains_key(ty), "event_type not in schema: {ty}");
        for field in required_for(ty) {
            assert!(
                v["payload"].get(&field).is_some(),
                "{ty} payload missing required '{field}': {line}"
            );
        }
        saw_meta |= ty == "memlens.meta";
        saw_realloc |= ty == "memlens.realloc";
        count += 1;
    }
    // Teaching note baked into an assertion: 1000 Vec pushes + 500 HashMap
    // inserts produce only a few dozen heap events — amortized growth means
    // the allocator is touched logarithmically, not per-push. (A first draft
    // of this test expected >100 events; the real trace had 41.)
    assert!(count > 20, "demo should produce a real trace, got {count}");
    assert!(saw_meta, "trace must start with a meta header");
    assert!(saw_realloc, "Vec growth must produce realloc events");
}
