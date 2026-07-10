//! [E] H1/H2/H3 (+404 fallthrough) — the handler as examples, driven
//! entirely in-process with hand-built `lambda_http::Request` values: no
//! AWS, no network, no running binary (H5). Where relay's tests spoke to a
//! Router through tower's `oneshot`, these call the handler as the plain
//! async function it is — there is no router left to go through.
//!
//! Accepted-event output is observed through the [`Emit`] seam (a Vec
//! sink), not by capturing the process's real stdout — same idea as relay's
//! writer seam, where tests drained the channel instead of scraping disk
//! through the binary.
//!
//! One process-wide subtlety these tests must respect: the counters live in
//! a process GLOBAL (`OnceLock` — that's the T4 point of the crate), and
//! the test harness runs `#[tokio::test]`s on parallel threads *in one
//! process*. Every test that drives the handler therefore holds the
//! [`SERIAL`] lock, so H3's before/after counter deltas are exact instead
//! of racing with neighboring tests. (tests/prop_door.rs is a separate
//! process — separate instance, no interference.)

#![allow(clippy::unwrap_used)]

use std::sync::Mutex;

use hello_lambda::handler::{Emit, handle_with};
use lambda_http::http::Request;
use lambda_http::{Body, Response};

/// Serializes handler-driving tests within this process (see module docs).
/// A tokio Mutex, not std: the guard is held across `.await` points, which
/// is exactly what tokio's lock is for (clippy::await_holding_lock is the
/// lint that catches the std version of this mistake).
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The Vec sink behind the [`Emit`] seam: what production prints to stdout,
/// tests collect and assert on.
#[derive(Default)]
struct VecSink(Mutex<Vec<String>>);

impl Emit for VecSink {
    fn emit(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_owned());
    }
}

impl VecSink {
    fn lines(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

/// The H1 fixture: a REAL hook-emitted envelope, copied verbatim from the
/// live lake (datalake/raw-local/dt=2026-07-09/events.jsonl, line 1) — the
/// SAME line relay's A1 test uses, so the two specs are provably talking
/// about the same traffic. The extra `spec_id` key is part of the point:
/// the door checks required keys are PRESENT, it doesn't forbid more.
const REAL_ENVELOPE: &str = r#"{"event_id":"1783556456860138087-1438-29228","ts":"2026-07-09T00:20:56Z","session_id":"unknown","spec_id":null,"actor":"claude-main","event_type":"session.start","schema_version":1,"payload":{"session_id":"98001615-fa64-5110-8061-de223f0734c5","transcript_path":"/root/.claude/projects/-home-user-rust-class/98001615-fa64-5110-8061-de223f0734c5.jsonl","cwd":"/home/user/rust-class","hook_event_name":"SessionStart","source":"resume"}}"#;

fn post_events(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/events")
        .header("content-type", "application/json")
        .body(Body::Text(body.to_owned()))
        .unwrap()
}

fn request(method: &str, path: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .body(Body::Empty)
        .unwrap()
}

/// A response body as text ("" for `Body::Empty` — the same equivalence
/// the H4 property uses).
fn body_text(response: &Response<Body>) -> &str {
    match response.body() {
        Body::Empty => "",
        Body::Text(text) => text.as_str(),
        // Body is #[non_exhaustive]; nothing in these tests answers binary.
        _ => panic!("no test expects a non-text body"),
    }
}

/// Parse, re-serialize compact: pretty and compact spellings of the same
/// value canonicalize identically (serde_json's map sorts keys).
fn canon(json_text: &str) -> String {
    serde_json::from_str::<serde_json::Value>(json_text)
        .unwrap()
        .to_string()
}

/// [E] H1 — a valid body gets 202 with an EMPTY body, and exactly one
/// compact JSON line (the re-serialized event) reaches the emit seam.
#[tokio::test]
async fn h1_valid_event_202_empty_body_one_line_emitted() {
    let _guard = SERIAL.lock().await;
    let sink = VecSink::default();

    let response = handle_with(post_events(REAL_ENVELOPE), &sink)
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    assert_eq!(body_text(&response), "", "202 carries an empty body");

    let lines = sink.lines();
    assert_eq!(lines.len(), 1, "exactly one line per accepted event");
    assert!(!lines[0].contains('\n'), "the line is one line");
    // Compared as parsed JSON values — the handler re-serializes, so
    // byte-equality is not promised (same equality as relay's A4 rev 2.1).
    assert_eq!(canon(&lines[0]), canon(REAL_ENVELOPE));
}

/// H1 corollary: the same envelope pretty-printed spans many lines as a
/// BODY but must be emitted as exactly ONE compact line.
#[tokio::test]
async fn h1_multiline_body_emitted_as_one_compact_line() {
    let _guard = SERIAL.lock().await;
    let pretty = serde_json::to_string_pretty(
        &serde_json::from_str::<serde_json::Value>(REAL_ENVELOPE).unwrap(),
    )
    .unwrap();
    assert!(pretty.contains('\n'));

    let sink = VecSink::default();
    let response = handle_with(post_events(&pretty), &sink).await.unwrap();
    assert_eq!(response.status(), 202);

    let lines = sink.lines();
    assert_eq!(lines.len(), 1, "one event, one line");
    assert!(!lines[0].contains('\n'));
    assert_eq!(canon(&lines[0]), canon(REAL_ENVELOPE));
}

/// [E] H2 — the three 400 clauses, in 005's fixed first-problem order,
/// each with the same one-line JSON error relay answers; and NOTHING
/// reaches the emit seam for any of them.
#[tokio::test]
async fn h2_invalid_bodies_400_in_fixed_order_and_emit_nothing() {
    let _guard = SERIAL.lock().await;
    let sink = VecSink::default();

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
        let response = handle_with(post_events(body), &sink).await.unwrap();
        assert_eq!(response.status(), 400, "{body:?}");
        assert_eq!(
            response.headers()["content-type"],
            "application/json",
            "{body:?}"
        );
        let error: serde_json::Value = serde_json::from_str(body_text(&response)).unwrap();
        assert_eq!(error, serde_json::json!({ "error": want }), "{body:?}");
    }

    assert!(
        sink.lines().is_empty(),
        "rejected bodies must emit nothing to stdout"
    );
}

/// [E] H3 — /healthz: 200, the five-key shape, and counters that add up.
/// Asserted as BEFORE/AFTER DELTAS because the counters are per-instance
/// state — which in this test binary means per-PROCESS state that the
/// other tests in this file also bump. That awkwardness is not a test
/// smell; it is the T4 lesson leaking into the test harness: "just keep it
/// in memory" makes your numbers hostage to whoever shares the process.
/// (On the real platform the sharer isn't a test thread — it's every other
/// request this warm instance ever served.)
#[tokio::test]
async fn h3_healthz_shape_and_counter_deltas() {
    let _guard = SERIAL.lock().await;
    let sink = VecSink::default();

    let before = healthz_snapshot(&sink).await;

    // 2 accepts + 3 rejects, awaited sequentially — quiescent by construction.
    for body in [
        REAL_ENVELOPE.to_owned(),
        REAL_ENVELOPE.replace("00:20:56", "11:22:33"),
    ] {
        let response = handle_with(post_events(&body), &sink).await.unwrap();
        assert_eq!(response.status(), 202);
    }
    for body in ["junk", r#"{"not":"an envelope"}"#, ""] {
        let response = handle_with(post_events(body), &sink).await.unwrap();
        assert_eq!(response.status(), 400);
    }

    let after = healthz_snapshot(&sink).await;

    // Shape: exactly the five promised keys, with sane types.
    let object = after.as_object().unwrap();
    assert_eq!(object.len(), 5);
    for key in ["instance", "started", "received", "accepted", "rejected"] {
        assert!(object.contains_key(key), "missing {key}");
    }
    // Off-Lambda (no AWS_LAMBDA_LOG_STREAM_NAME in a test run) the id is
    // the local fallback shape; on the platform it would be the log stream
    // name. Shape details are pinned by state.rs's own unit test.
    let instance = after["instance"].as_str().unwrap();
    assert!(!instance.is_empty(), "instance id must not be empty");
    let started = after["started"].as_str().unwrap();
    assert_eq!(started.len(), 20, "YYYY-MM-DDTHH:MM:SSZ: {started}");
    assert!(started.ends_with('Z') && started.contains('T'));

    // Identity is stable across requests on one instance (= this process).
    assert_eq!(after["instance"], before["instance"]);
    assert_eq!(after["started"], before["started"]);

    // The deltas: exactly what this test did, and received = accepted +
    // rejected (the quiescent invariant, carried over from relay's A3).
    let delta = |key: &str| after[key].as_u64().unwrap() - before[key].as_u64().unwrap();
    assert_eq!(delta("received"), 5);
    assert_eq!(delta("accepted"), 2);
    assert_eq!(delta("rejected"), 3);
    assert_eq!(delta("received"), delta("accepted") + delta("rejected"));
}

async fn healthz_snapshot(sink: &VecSink) -> serde_json::Value {
    let response = handle_with(request("GET", "/healthz"), sink).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "application/json");
    serde_json::from_str(body_text(&response)).unwrap()
}

/// [E] H3, second sentence — ANY other method/path is 404: wrong path,
/// wrong method on a right path (no 405 nuance at this door), root, and a
/// method the door never speaks.
#[tokio::test]
async fn h3_everything_else_is_404() {
    let _guard = SERIAL.lock().await;
    let sink = VecSink::default();

    let before = healthz_snapshot(&sink).await;

    for (method, path) in [
        ("GET", "/"),
        ("GET", "/nope"),
        ("GET", "/events"),    // right path, wrong method
        ("POST", "/healthz"),  // right path, wrong method
        ("DELETE", "/events"), // a method the door never speaks
        ("POST", "/events/"),  // trailing slash is a different path
    ] {
        let response = handle_with(request(method, path), &sink).await.unwrap();
        assert_eq!(response.status(), 404, "{method} {path}");
        assert_eq!(body_text(&response), "", "{method} {path}");
    }

    // 404s happen BEFORE the door: no counter moves, nothing emitted.
    let after = healthz_snapshot(&sink).await;
    assert_eq!(after["received"], before["received"]);
    assert!(sink.lines().is_empty());
}
