//! [E] A1/A2/A3 — the routes as examples, driven entirely in-process:
//! `tower::ServiceExt::oneshot` calls the `Router` like an async function,
//! so these tests need no port, no socket, and no running binary. (The
//! real listener + real SIGINT are exercised in tests/shutdown.rs.)

#![allow(clippy::unwrap_used)]

mod common;

use axum::http::StatusCode;
use common::{TempLake, body_json, canon, disk_lines, get_healthz, post_event, spawn_relay};
use tower::ServiceExt;

/// The A1 fixture: a REAL hook-emitted envelope, copied verbatim from the
/// live lake (datalake/raw-local/dt=2026-07-09/events.jsonl, line 1) — the
/// requirements insist the example is an event the hooks actually produced,
/// not a hand-crafted lookalike. Note the extra `spec_id` key: the door
/// checks required keys are PRESENT, it doesn't forbid more.
const REAL_ENVELOPE: &str = r#"{"event_id":"1783556456860138087-1438-29228","ts":"2026-07-09T00:20:56Z","session_id":"unknown","spec_id":null,"actor":"claude-main","event_type":"session.start","schema_version":1,"payload":{"session_id":"98001615-fa64-5110-8061-de223f0734c5","transcript_path":"/root/.claude/projects/-home-user-rust-class/98001615-fa64-5110-8061-de223f0734c5.jsonl","cwd":"/home/user/rust-class","hook_event_name":"SessionStart","source":"resume"}}"#;

/// [E] A1 — a valid body gets 202 and lands as one JSONL line in the
/// `dt=` partition named by the event's OWN ts (2026-07-09), on disk no
/// later than shutdown.
#[tokio::test]
async fn a1_valid_event_202_and_lands_in_its_day() {
    let lake = TempLake::new();
    let (app, writer) = spawn_relay(lake.path());

    let response = app
        .clone()
        .oneshot(post_event(REAL_ENVELOPE))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // "on disk no later than shutdown": drop every router handle → the
    // last tx drops → the writer drains and flushes → its handle yields.
    drop(app);
    assert_eq!(writer.await.unwrap(), 1);

    let lines = disk_lines(lake.path());
    assert_eq!(lines.len(), 1);
    let (day, line) = &lines[0];
    assert_eq!(day, "2026-07-09", "day comes from the event's ts");
    // Compared as parsed JSON values (A4 rev 2.1's equality — the relay
    // re-serializes, so byte-equality is not promised).
    assert_eq!(canon(line), canon(REAL_ENVELOPE));
}

/// A1 corollary (the reason A4 speaks of re-serialization): the same
/// envelope pretty-printed spans many lines as a BODY but must land as
/// exactly ONE compact line in the lake.
#[tokio::test]
async fn a1_multiline_body_lands_as_one_line() {
    let pretty = serde_json::to_string_pretty(
        &serde_json::from_str::<serde_json::Value>(REAL_ENVELOPE).unwrap(),
    )
    .unwrap();
    assert!(pretty.contains('\n'));

    let lake = TempLake::new();
    let (app, writer) = spawn_relay(lake.path());
    let response = app.clone().oneshot(post_event(&pretty)).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    drop(app);
    assert_eq!(writer.await.unwrap(), 1);

    let lines = disk_lines(lake.path());
    assert_eq!(lines.len(), 1, "one event, one line");
    assert_eq!(canon(&lines[0].1), canon(REAL_ENVELOPE));
}

/// [E] A2 — the three 400s, in the fixed first-problem order, each with a
/// one-line JSON error; and NOTHING reaches the lake.
#[tokio::test]
async fn a2_invalid_bodies_400_in_fixed_order_and_write_nothing() {
    let lake = TempLake::new();
    let (app, writer) = spawn_relay(lake.path());

    let missing_actor = REAL_ENVELOPE.replace(r#""actor":"claude-main","#, "");
    let bad_ts = REAL_ENVELOPE.replace("2026-07-09T00:20:56Z", "not-a-timestamp");
    // Both problems at once → the EARLIER clause wins (missing key beats
    // uncomparable ts).
    let missing_and_bad_ts = missing_actor.replace("2026-07-09T00:20:56Z", "nope");

    let cases: &[(&str, &str)] = &[
        // clause 1: not a JSON object (junk, non-object JSON, blank)
        ("not json at all", "not a json object"),
        ("[1,2,3]", "not a json object"),
        ("", "not a json object"),
        // clause 2: first missing key, in REQUIRED_KEYS order
        (&missing_actor, "missing key: actor"),
        (r#"{"not":"an envelope"}"#, "missing key: event_id"),
        (&missing_and_bad_ts, "missing key: actor"),
        // clause 3: all keys present, no comparable day
        (&bad_ts, "no comparable day in ts"),
    ];

    for (body, want) in cases {
        let response = app.clone().oneshot(post_event(body)).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body:?}");
        let error = body_json(response.into_body()).await;
        assert_eq!(
            error,
            serde_json::json!({ "error": want }),
            "for body {body:?}"
        );
    }

    // Rejected bodies appear nowhere: the writer never saw a single line.
    drop(app);
    assert_eq!(writer.await.unwrap(), 0);
    assert!(disk_lines(lake.path()).is_empty());
}

/// [E] A3 — /healthz returns the three counters, and once no requests are
/// in flight (each awaited to completion here), received = accepted +
/// rejected exactly.
#[tokio::test]
async fn a3_healthz_counters_are_quiescently_conserved() {
    let lake = TempLake::new();
    let (app, writer) = spawn_relay(lake.path());

    // 2 accepts + 3 rejects, sequentially — quiescent by construction.
    for body in [
        REAL_ENVELOPE.to_owned(),
        REAL_ENVELOPE.replace("00:20:56", "11:22:33"),
    ] {
        let response = app.clone().oneshot(post_event(&body)).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
    for body in ["junk", r#"{"not":"an envelope"}"#, ""] {
        let response = app.clone().oneshot(post_event(body)).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    let response = app.clone().oneshot(get_healthz()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let health = body_json(response.into_body()).await;
    assert_eq!(
        health,
        serde_json::json!({ "received": 5, "accepted": 2, "rejected": 3 })
    );

    drop(app);
    assert_eq!(writer.await.unwrap(), 2);
}
