//! 2.1.4 — golden fixtures pinning the viewer's JS fold to this engine.
//!
//! Self-checking: on first run the fixtures are generated and committed; on
//! every later run regeneration must match the committed bytes exactly. If
//! the engine's semantics ever change, this test fails loudly and the
//! goldens (and the viewer) must be consciously regenerated together.

use memlens_replay::{parse, replay, validate};
use std::fmt::Write as _;

/// A small but representative trace: nested scopes, realloc lineage chain
/// (in-place AND moving), address reuse after free, markers, interleaving.
fn fixture_jsonl() -> String {
    let lines = [
        (r#""event_type":"memlens.meta","schema_version":1,"payload":{"program":"golden","pid":1,"started_at":"2026-07-05T00:00:00Z","version":"0.1.0"}"#).to_string(),
        line("scope_enter", r#"{"label":"outer","seq":1}"#),
        line("alloc", r#"{"addr":"0x1000","size":100,"align":8,"seq":2}"#),
        line("scope_enter", r#"{"label":"inner","seq":3}"#),
        line("alloc", r#"{"addr":"0x2000","size":32,"align":16,"seq":4}"#),
        line("marker", r#"{"label":"move:v","kind":"move","seq":5}"#),
        line("realloc", r#"{"old_addr":"0x1000","new_addr":"0x1000","old_size":100,"new_size":200,"align":8,"seq":6}"#),
        line("realloc", r#"{"old_addr":"0x1000","new_addr":"0x3000","old_size":200,"new_size":400,"align":8,"seq":7}"#),
        line("dealloc", r#"{"addr":"0x2000","size":32,"align":16,"seq":8}"#),
        line("scope_exit", r#"{"label":"inner","seq":9}"#),
        line("alloc", r#"{"addr":"0x2000","size":64,"align":8,"seq":10}"#),
        line("scope_exit", r#"{"label":"outer","seq":11}"#),
        line("dealloc", r#"{"addr":"0x3000","size":400,"align":8,"seq":12}"#),
    ];
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        if i == 0 {
            writeln!(out, r#"{{"event_id":"g-{i}","ts":"2026-07-05T00:00:00Z","session_id":"golden","spec_id":null,"actor":"memlens",{l}}}"#).expect("write");
        } else {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

fn line(ty: &str, payload: &str) -> String {
    format!(
        r#"{{"event_id":"g-{ty}","ts":"2026-07-05T00:00:00Z","session_id":"golden","spec_id":null,"actor":"memlens","event_type":"memlens.{ty}","schema_version":1,"payload":{payload}}}"#
    )
}

/// Snapshot points chosen to cover: mid-scope, after in-place realloc,
/// after moving realloc, after address reuse, and the final state.
const SNAP_TS: [u64; 6] = [2, 4, 6, 7, 10, 12];

fn snapshots(events_src: &str) -> String {
    let trace = parse(events_src).expect("fixture parses");
    validate(&trace.events).expect("fixture is balanced");
    let mut snaps = serde_json::Map::new();
    for t in SNAP_TS {
        let live = replay(&trace.events, t);
        let mut m = serde_json::Map::new();
        for (addr, a) in &live {
            m.insert(
                format!("0x{addr:x}"),
                serde_json::json!({
                    "size": a.size,
                    "align": a.align,
                    "born_seq": a.born_seq,
                    "scopes": a.scopes,
                    "lineage_root": format!("0x{:x}", a.lineage_root),
                }),
            );
        }
        snaps.insert(
            t.to_string(),
            serde_json::json!({
                "live": m,
                "live_bytes": memlens_replay::live_bytes(&trace.events, t),
            }),
        );
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(snaps)).expect("serialize")
}

#[test]
fn goldens_are_current() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    std::fs::create_dir_all(&dir).expect("fixtures dir");
    let jsonl_path = dir.join("basic.jsonl");
    let snap_path = dir.join("basic.expected.json");

    let jsonl = fixture_jsonl();
    let snaps = snapshots(&jsonl);

    for (path, content) in [(&jsonl_path, &jsonl), (&snap_path, &snaps)] {
        if path.exists() {
            let committed = std::fs::read_to_string(path).expect("readable");
            assert_eq!(
                &committed,
                content,
                "{} drifted from the engine — regenerate goldens AND update the viewer",
                path.display()
            );
        } else {
            std::fs::write(path, content).expect("write fixture");
        }
    }

    // Sanity on the final snapshot: only the reused 0x2000 (64B) survives;
    // the lineage chain 0x1000→0x1000→0x3000 is fully freed by seq 12.
    let trace = parse(&jsonl).expect("parses");
    let final_live = replay(&trace.events, 12);
    assert_eq!(final_live.len(), 1);
    assert_eq!(final_live[&0x2000].size, 64);
    assert_eq!(final_live[&0x2000].scopes, vec!["outer".to_string()]);
}
