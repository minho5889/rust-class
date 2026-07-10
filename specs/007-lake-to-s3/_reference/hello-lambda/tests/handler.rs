//! [E] S6 (+ 006's H2/H3/404, still binding) — the handler as examples,
//! driven entirely in-process with hand-built `lambda_http::Request`
//! values and a **FakeStore** behind the seam: no AWS, no network, no
//! running binary (006's H5 discipline, inherited).
//!
//! SUPERSESSION NOTE (S6b): 006's H1 tests observed accepted events
//! through the `Emit` seam (a Vec sink standing in for stdout). That sink
//! retired with the evolution — the same scenarios now assert against the
//! fake store's ledger and objects: one put, at the S1 key, holding the
//! one compact line. The 400/healthz/404 tests are 006's, unchanged in
//! meaning (rejects put NOTHING; the door and counters didn't move).
//!
//! One process-wide subtlety these tests must respect, unchanged: the
//! counters live in a process GLOBAL (`OnceLock` — the T4 point), and the
//! test harness runs `#[tokio::test]`s on parallel threads *in one
//! process*. Every test that drives the handler therefore holds the
//! [`SERIAL`] lock, so counter deltas are exact instead of racing.
//! (tests/prop_door.rs is a separate process — no interference.)

#![allow(clippy::unwrap_used)]

use hello_lambda::handler::handle_with;
use lake_store::fake::FakeStore;
use lambda_http::http::Request;
use lambda_http::{Body, Response};

/// Serializes handler-driving tests within this process (see module docs).
/// A tokio Mutex, not std: the guard is held across `.await` points, which
/// is exactly what tokio's lock is for (clippy::await_holding_lock is the
/// lint that catches the std version of this mistake).
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The S6 fixture: a REAL hook-emitted envelope, copied verbatim from the
/// live lake (datalake/raw-local/dt=2026-07-09/events.jsonl, line 1) — the
/// SAME line relay's A1 and 006's H1 tests use, so three specs are
/// provably talking about the same traffic. The extra `spec_id` key is
/// part of the point: the door checks required keys are PRESENT, it
/// doesn't forbid more.
const REAL_ENVELOPE: &str = r#"{"event_id":"1783556456860138087-1438-29228","ts":"2026-07-09T00:20:56Z","session_id":"unknown","spec_id":null,"actor":"claude-main","event_type":"session.start","schema_version":1,"payload":{"session_id":"98001615-fa64-5110-8061-de223f0734c5","transcript_path":"/root/.claude/projects/-home-user-rust-class/98001615-fa64-5110-8061-de223f0734c5.jsonl","cwd":"/home/user/rust-class","hook_event_name":"SessionStart","source":"resume"}}"#;

/// The S1 key that envelope must land at: its day and its event_id,
/// mechanically.
const REAL_KEY: &str = "raw/dt=2026-07-09/evt-1783556456860138087-1438-29228.json";

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

/// [E] S6 — a valid body gets 202 with an EMPTY body, and results in
/// EXACTLY ONE put, at the S1 key, holding the one compact line.
/// (Supersedes 006 H1's "one line to stdout" — S6b.)
#[tokio::test]
async fn s6_valid_event_202_one_put_at_the_s1_key() {
    let _guard = SERIAL.lock().await;
    let store = FakeStore::new();

    let response = handle_with(post_events(REAL_ENVELOPE), &store)
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    assert_eq!(body_text(&response), "", "202 carries an empty body");

    assert_eq!(store.puts(), 1, "exactly one put per accepted event");
    assert_eq!(store.put_log(), vec![REAL_KEY.to_owned()], "at the S1 key");

    let object = store.object(REAL_KEY).unwrap();
    let stored = String::from_utf8(object).unwrap();
    assert!(!stored.contains('\n'), "the object is one compact line");
    // Compared as parsed JSON values — the handler re-serializes, so
    // byte-equality is not promised (same equality as relay's A4 rev 2.1).
    assert_eq!(canon(&stored), canon(REAL_ENVELOPE));
}

/// S6 corollary: the same envelope pretty-printed spans many lines as a
/// BODY but must land as ONE compact line — in the SAME object as its
/// compact twin would (idempotence by key: the id is the address).
#[tokio::test]
async fn s6_multiline_body_stored_as_one_compact_line_same_key() {
    let _guard = SERIAL.lock().await;
    let pretty = serde_json::to_string_pretty(
        &serde_json::from_str::<serde_json::Value>(REAL_ENVELOPE).unwrap(),
    )
    .unwrap();
    assert!(pretty.contains('\n'));

    let store = FakeStore::new();
    let response = handle_with(post_events(&pretty), &store).await.unwrap();
    assert_eq!(response.status(), 202);

    assert_eq!(store.puts(), 1, "one event, one put");
    let stored = String::from_utf8(store.object(REAL_KEY).unwrap()).unwrap();
    assert!(!stored.contains('\n'));
    assert_eq!(canon(&stored), canon(REAL_ENVELOPE));
}

/// S6, the retry story (T5): the SAME event twice is two puts to ONE key
/// — the second replaces the first, the lake holds one object. (What 006
/// could not promise with stdout: a retried event printed twice.)
#[tokio::test]
async fn s6_retried_event_overwrites_itself() {
    let _guard = SERIAL.lock().await;
    let store = FakeStore::new();

    for _ in 0..2 {
        let response = handle_with(post_events(REAL_ENVELOPE), &store)
            .await
            .unwrap();
        assert_eq!(response.status(), 202);
    }
    assert_eq!(store.puts(), 2, "the handler doesn't dedup — the KEY does");
    assert_eq!(store.objects().len(), 1, "one object per event identity");
}

/// [E] H2 (unchanged from 006) — the three 400 clauses, in 005's fixed
/// first-problem order, each with the same one-line JSON error relay
/// answers; and ZERO puts for all of them (S6's reject half).
#[tokio::test]
async fn h2_invalid_bodies_400_in_fixed_order_and_put_nothing() {
    let _guard = SERIAL.lock().await;
    let store = FakeStore::new();

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
        let response = handle_with(post_events(body), &store).await.unwrap();
        assert_eq!(response.status(), 400, "{body:?}");
        assert_eq!(
            response.headers()["content-type"],
            "application/json",
            "{body:?}"
        );
        let error: serde_json::Value = serde_json::from_str(body_text(&response)).unwrap();
        assert_eq!(error, serde_json::json!({ "error": want }), "{body:?}");
    }

    assert_eq!(store.puts(), 0, "rejected bodies must put nothing (S6)");
    assert!(store.objects().is_empty());
}

/// S6's failure edge (new in v1): the door says yes but the STORE says no
/// — the caller gets 500 (never a durability-lying 202), nothing is
/// counted as accepted, and the failure is the store's named error.
#[tokio::test]
async fn s6_put_failure_answers_500_not_a_lying_202() {
    let _guard = SERIAL.lock().await;
    let store = FakeStore::new();
    store.fail_on(REAL_KEY);

    let before = healthz_snapshot(&store).await;
    let response = handle_with(post_events(REAL_ENVELOPE), &store)
        .await
        .unwrap();
    assert_eq!(response.status(), 500);
    let error: serde_json::Value = serde_json::from_str(body_text(&response)).unwrap();
    assert_eq!(error, serde_json::json!({ "error": "lake unavailable" }));
    let after = healthz_snapshot(&store).await;

    assert!(store.object(REAL_KEY).is_none(), "nothing landed");
    let delta = |key: &str| after[key].as_u64().unwrap() - before[key].as_u64().unwrap();
    assert_eq!(delta("received"), 1);
    assert_eq!(delta("accepted"), 0, "a lost event is not 'accepted'");
    assert_eq!(
        delta("rejected"),
        0,
        "…but the door didn't reject it either"
    );
}

/// [E] H3 (unchanged from 006) — /healthz: 200, the five-key shape, and
/// counters that add up, asserted as before/after deltas (the counters
/// are per-instance = per-process state shared with the other tests).
#[tokio::test]
async fn h3_healthz_shape_and_counter_deltas() {
    let _guard = SERIAL.lock().await;
    let store = FakeStore::new();

    let before = healthz_snapshot(&store).await;

    // 2 accepts + 3 rejects, awaited sequentially — quiescent by construction.
    for body in [
        REAL_ENVELOPE.to_owned(),
        REAL_ENVELOPE.replace("00:20:56", "11:22:33"),
    ] {
        let response = handle_with(post_events(&body), &store).await.unwrap();
        assert_eq!(response.status(), 202);
    }
    for body in ["junk", r#"{"not":"an envelope"}"#, ""] {
        let response = handle_with(post_events(body), &store).await.unwrap();
        assert_eq!(response.status(), 400);
    }

    let after = healthz_snapshot(&store).await;

    // Shape: exactly the five promised keys, with sane types.
    let object = after.as_object().unwrap();
    assert_eq!(object.len(), 5);
    for key in ["instance", "started", "received", "accepted", "rejected"] {
        assert!(object.contains_key(key), "missing {key}");
    }
    let instance = after["instance"].as_str().unwrap();
    assert!(!instance.is_empty(), "instance id must not be empty");
    let started = after["started"].as_str().unwrap();
    assert_eq!(started.len(), 20, "YYYY-MM-DDTHH:MM:SSZ: {started}");
    assert!(started.ends_with('Z') && started.contains('T'));

    // Identity is stable across requests on one instance (= this process).
    assert_eq!(after["instance"], before["instance"]);
    assert_eq!(after["started"], before["started"]);

    // The deltas: exactly what this test did, and received = accepted +
    // rejected — the quiescent, failure-free-run invariant (see the put-
    // failure test above for the one branch that suspends it).
    let delta = |key: &str| after[key].as_u64().unwrap() - before[key].as_u64().unwrap();
    assert_eq!(delta("received"), 5);
    assert_eq!(delta("accepted"), 2);
    assert_eq!(delta("rejected"), 3);
    assert_eq!(delta("received"), delta("accepted") + delta("rejected"));

    // v1 addendum: two accepts = two puts, but the second body differs
    // only in clock-TIME — same event_id, same DAY, hence the same S1 key
    // — so it overwrites and the store holds ONE object. The key is the
    // identity, and identity here ignores the time of day on purpose.
    assert_eq!(store.puts(), 2);
    assert_eq!(
        store.objects().len(),
        1,
        "same day + same event_id = same key"
    );
}

/// [E] H3, second sentence (unchanged) — ANY other method/path is 404,
/// counters don't move, and nothing is put.
#[tokio::test]
async fn h3_everything_else_is_404() {
    let _guard = SERIAL.lock().await;
    let store = FakeStore::new();

    let before = healthz_snapshot(&store).await;

    for (method, path) in [
        ("GET", "/"),
        ("GET", "/nope"),
        ("GET", "/events"),    // right path, wrong method
        ("POST", "/healthz"),  // right path, wrong method
        ("DELETE", "/events"), // a method the door never speaks
        ("POST", "/events/"),  // trailing slash is a different path
    ] {
        let response = handle_with(request(method, path), &store).await.unwrap();
        assert_eq!(response.status(), 404, "{method} {path}");
        assert_eq!(body_text(&response), "", "{method} {path}");
    }

    // 404s happen BEFORE the door: no counter moves, nothing stored.
    let after = healthz_snapshot(&store).await;
    assert_eq!(after["received"], before["received"]);
    assert_eq!(store.puts(), 0);
}

async fn healthz_snapshot(store: &FakeStore) -> serde_json::Value {
    let response = handle_with(request("GET", "/healthz"), store)
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "application/json");
    serde_json::from_str(body_text(&response)).unwrap()
}
