//! [P] H4 — door equivalence: for ANY generated body, hello-lambda's
//! `(status, response text)` on `POST /events` equals relay's. The two
//! deployment shapes of the same door may never drift.
//!
//! The oracle is not a table — it is **relay itself**: the 005 reference
//! lib is a dev-dependency, its real Router driven in-process through
//! tower's `oneshot`, exactly as its own tests drive it. Each generated
//! body goes through both doors and the verdicts are compared pairwise.
//! If either side ever rewords an error, reorders the clauses, or changes
//! a status, this property is the tripwire.
//!
//! The generator is 005's A4 message strategy, readapted (see
//! specs/005-async-relay/_reference/relay/tests/prop_relay.rs): valid
//! envelopes over a MULTI-DAY domain (sometimes pretty-printed, so bodies
//! span lines) ∪ the three malformed arms (non-objects, one required key
//! removed, uncomparable ts) ∪ occasional exact duplicates. Duplicates
//! matter even without a lake: they check the door is stateless — the same
//! body must get the same verdict the second time (no dedup "helpfulness").
//!
//! What relay does AFTER its door (channel → writer → disk) is not under
//! test — H4 is about the verdict. But the harness must respect relay's
//! own precondition: its handler only 202s while the writer channel's
//! RECEIVER is alive (a dead receiver turns every accept into the
//! "writer unavailable" 500). So the relay side is built the way relay's
//! own tests build it — live channel, receiver held for the whole
//! scenario, capacity comfortably above the batch size so `send().await`
//! never blocks with nobody draining (both bounds are asserted in the
//! harness, not just claimed). The runtime pattern is 005's: proptest
//! drives a sync body, the body builds its own runtime and `block_on`s —
//! generation stays deterministic. A current-thread runtime suffices:
//! equivalence is per-request, nothing needs to race.
//!
//! **Domain bound (H4 as amended): generated bodies stay ≤ 64 KiB** — and
//! in practice far under (envelopes are a few hundred bytes; the harness
//! asserts the bound). Above the harnesses' own limits the two shapes
//! genuinely diverge for reasons that are NOT the door: relay's axum
//! answers oversized bodies with its stock 413 before our handler runs,
//! while Lambda's platform rejects payloads over ~6 MB before the handler
//! ever sees them. That divergence is documented here and lives OUTSIDE
//! the property's domain.

#![allow(clippy::unwrap_used)]

use hello_lambda::handler::{Emit, handle_with};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use tower::ServiceExt;

// ------------------------------------------------------------- generators
// (005's domains, verbatim — the property is only as strong as the domain,
// and this domain already found relay's edges once.)

/// A MULTI-DAY ts domain; weird-but-comparable days on purpose — the door
/// accepts what it can partition, not what a calendar blesses.
const DAYS: [&str; 5] = [
    "2026-07-05",
    "2026-07-08",
    "2026-07-09",
    "1999-01-01",
    "2027-12-31",
];
const KINDS: [&str; 4] = [
    "session.start",
    "gate.approved",
    "bolt.done",
    "learning.fight",
];
/// ts values with no comparable day (glake's F2 rule says bad-ts).
const BAD_TS: [&str; 4] = ["nope", "2026/07/09T01:02:03Z", "today", "20260709"];
/// Bodies that aren't JSON objects at all.
const NON_OBJECTS: [&str; 6] = [
    "not json at all",
    "[1,2,3]",
    "123",
    "\"quoted\"",
    "null",
    "",
];

/// A well-formed envelope as a `Value` (all REQUIRED_KEYS present).
fn envelope(id: &str, ts: &str, kind: &str) -> serde_json::Value {
    serde_json::json!({
        "event_id": id,
        "ts": ts,
        "session_id": "prop-door",
        "actor": "proptest",
        "event_type": kind,
        "schema_version": 1,
        "payload": { "case": id }
    })
}

fn ts() -> impl Strategy<Value = String> {
    (0..DAYS.len(), 0u32..24, 0u32..60, 0u32..60)
        .prop_map(|(d, h, m, s)| format!("{}T{h:02}:{m:02}:{s:02}Z", DAYS[d]))
}

/// A valid body — sometimes pretty-printed, so it SPANS LINES: both doors
/// must re-serialize, and both must still say 202.
fn valid_body() -> impl Strategy<Value = String> {
    ("[a-z0-9]{6}", ts(), 0..KINDS.len(), any::<bool>()).prop_map(|(id, ts, k, pretty)| {
        let value = envelope(&id, &ts, KINDS[k]);
        if pretty {
            serde_json::to_string_pretty(&value).unwrap()
        } else {
            value.to_string()
        }
    })
}

/// The malformed arms, one per door clause: non-objects, an envelope with
/// one required key removed, and an envelope whose ts has no comparable day.
fn invalid_body() -> impl Strategy<Value = String> {
    prop_oneof![
        (0..NON_OBJECTS.len()).prop_map(|i| NON_OBJECTS[i].to_owned()),
        ("[a-z0-9]{6}", ts(), 0..glake::classify::REQUIRED_KEYS.len()).prop_map(|(id, ts, k)| {
            let mut value = envelope(&id, &ts, "prop.malformed");
            value
                .as_object_mut()
                .unwrap()
                .remove(glake::classify::REQUIRED_KEYS[k]);
            value.to_string()
        }),
        ("[a-z0-9]{6}", 0..BAD_TS.len())
            .prop_map(|(id, t)| envelope(&id, BAD_TS[t], "prop.badts").to_string()),
    ]
}

/// A mixed batch: ~3:1 valid:invalid, with occasional EXACT duplicates
/// (repeat count 2). No oracle tag travels with the bodies — relay IS the
/// oracle here.
fn batch() -> impl Strategy<Value = Vec<String>> {
    let body = prop_oneof![
        3 => valid_body(),
        1 => invalid_body(),
    ];
    let repeat = prop_oneof![5 => Just(1usize), 1 => Just(2usize)];
    prop::collection::vec((body, repeat), 1..8).prop_map(|pairs| {
        pairs
            .into_iter()
            .flat_map(|(body, n)| std::iter::repeat_n(body, n))
            .collect()
    })
}

// ------------------------------------------------------------ the plumbing

/// The null-ish sink for hello's side: collects emitted lines so the
/// property can also check the emit discipline (one line iff 202).
#[derive(Default)]
struct VecSink(std::sync::Mutex<Vec<String>>);

impl Emit for VecSink {
    fn emit(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_owned());
    }
}

fn lambda_post(body: &str) -> lambda_http::Request {
    lambda_http::http::Request::builder()
        .method("POST")
        .uri("/events")
        .header("content-type", "application/json")
        .body(lambda_http::Body::Text(body.to_owned()))
        .unwrap()
}

fn axum_post(body: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method("POST")
        .uri("/events")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body.to_owned()))
        .unwrap()
}

/// hello-lambda's verdict on one body: (status, response text, lines emitted).
async fn hello_verdict(body: &str) -> Result<(u16, String, usize), TestCaseError> {
    let sink = VecSink::default();
    let response = handle_with(lambda_post(body), &sink)
        .await
        .map_err(|e| TestCaseError::fail(format!("hello handler errored: {e}")))?;
    let status = response.status().as_u16();
    let text = match response.into_body() {
        lambda_http::Body::Empty => String::new(),
        lambda_http::Body::Text(text) => text,
        // Body is #[non_exhaustive]; the handler only answers Empty/Text.
        _ => return Err(TestCaseError::fail("hello answered a non-text body")),
    };
    let emitted = sink.0.lock().unwrap().len();
    Ok((status, text, emitted))
}

/// relay's verdict on the same body, through its real Router.
async fn relay_verdict(app: &axum::Router, body: &str) -> Result<(u16, String), TestCaseError> {
    // Router's service error is Infallible; the unwrap-shaped path is total.
    let response = app
        .clone()
        .oneshot(axum_post(body))
        .await
        .map_err(|e| TestCaseError::fail(format!("relay router errored: {e}")))?;
    let status = response.status().as_u16();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .map_err(|e| TestCaseError::fail(format!("relay body: {e}")))?;
    let text = String::from_utf8(bytes.to_vec())
        .map_err(|e| TestCaseError::fail(format!("relay answered non-utf8: {e}")))?;
    Ok((status, text))
}

// ------------------------------------------------------------ the property

proptest! {
    // 512 cases (design floor is 256; batches make each case several
    // bodies, so a run compares a few thousand verdict pairs).
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// [P] H4 — ∀ bodies: hello-lambda's (status, body text) ≡ relay's,
    /// duplicates included; and hello emits exactly one line iff 202.
    #[test]
    fn prop_h4_door_equivalence(bodies in batch()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build: {e}")))?;

        rt.block_on(async move {
            // relay minus its writer: hold the receiver ALIVE so sends
            // succeed (relay's precondition — a dropped receiver degrades
            // every 202 into a 500 and the property would be comparing
            // harness damage, not doors). `_rx` is a real binding: unlike
            // a bare `_`, it keeps the receiver alive to end of scope.
            // Capacity exceeds the max batch size, so send().await can
            // never block on backpressure with nobody draining.
            let (tx, _rx) = tokio::sync::mpsc::channel::<String>(256);
            prop_assert!(bodies.len() < 256, "batch must fit the channel");
            let app = relay::routes::router(relay::routes::AppState::new(tx));

            for body in &bodies {
                // The documented domain bound (module docs): stay far below
                // both harnesses' own body limits.
                prop_assert!(body.len() <= 64 * 1024, "body over the 64 KiB domain bound");
                let (relay_status, relay_text) = relay_verdict(&app, body).await?;
                let (hello_status, hello_text, emitted) = hello_verdict(body).await?;

                prop_assert_eq!(
                    (hello_status, &hello_text),
                    (relay_status, &relay_text),
                    "doors drifted on body {:?}",
                    body
                );
                // The emit discipline rides along: one stdout line per 202,
                // nothing per 400 (H1/H2 in property form).
                prop_assert_eq!(
                    emitted,
                    usize::from(hello_status == 202),
                    "emit count wrong on body {:?} (status {})",
                    body,
                    hello_status
                );
            }
            Ok(())
        })?;
    }
}
