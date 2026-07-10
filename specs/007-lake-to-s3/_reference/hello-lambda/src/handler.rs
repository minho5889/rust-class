//! The one function that is the whole service — v1: the sink grows up
//! (S6/S6b).
//!
//! Relay needed a `Router`, an `AppState`, extractors, and a serve loop to
//! connect four handlers to the world. Here the platform hands us each
//! request already parsed into an `http::Request` and takes back an
//! `http::Response` — so the entire HTTP surface is one `match` on
//! `(method, path)`. What axum's router did with a radix tree, we do with
//! three match arms; at this size, the match IS the router.
//!
//! **What changed since 006 (S6b, the supersession made explicit):** the
//! emit seam. 006's `Emit` trait carried accepted lines to `println!` —
//! "the lake in exile", stdout → CloudWatch. That sink RETIRES here
//! (006 H1 is superseded): the same returned line now goes to
//! `store.put("raw/dt=<day>/evt-<event_id>.json", line)` through the
//! [`ObjectStore`] seam — the lake's real home. The door itself — the
//! 202/400 verdict — did NOT move, and the H4 property re-proves it
//! against relay after the evolution.
//!
//! The seam also changed *shape*, and the change is a lesson: 006's
//! `&dyn Emit` was a synchronous, zero-context sink, fine for one
//! `println!`. A store put is ASYNC (an awaited network call), and RPITIT
//! traits like [`ObjectStore`] aren't `dyn`-friendly — so the handler core
//! goes **generic**: `handle_with<S: ObjectStore>` monomorphizes over the
//! real store in production and the fake in tests (the F13 pattern,
//! second appearance — and the same swap relay's tests did with a channel,
//! now done with a bucket).
//!
//! Ingest is idempotent FOR FREE (T5): the key contains the `event_id`,
//! so a retried event overwrites itself — one event, one object, however
//! many times the client retries. Contrast lake-sync, which has to *earn*
//! idempotence with a list+compare; here it falls out of key derivation.

use std::sync::atomic::Ordering;

use glake::classify::REQUIRED_KEYS;
use glake::parser::{ClassifiedLine, EventParser, SerdeParser};
use lake_store::ObjectStore;
use lambda_http::{Body, Error, Request, Response};

use crate::state;

/// The whole HTTP surface, parameterized over the store seam (H5's
/// test-without-AWS discipline, S6: tests drive THIS with hand-built
/// `lambda_http` Requests and a `FakeStore` — no network, no bucket).
///
/// Anything that isn't exactly `POST /events` or `GET /healthz` is 404 —
/// including "right path, wrong method" (unchanged from 006; H4 can't see
/// the difference because its generator only speaks POST /events).
pub async fn handle_with<S: ObjectStore>(req: Request, store: &S) -> Result<Response<Body>, Error> {
    match (req.method().as_str(), req.uri().path()) {
        ("POST", "/events") => post_events(&req, store).await,
        ("GET", "/healthz") => healthz(),
        _ => empty_response(404),
    }
}

/// `POST /events` (S6): count it, run the door, then either PUT the
/// compact line at its derived key and 202, or answer with the first
/// problem as 400.
///
/// Same choreography as 006, with `emit.emit(&line)` replaced by an
/// awaited `store.put(key, line)`: 202 still means "validated and handed
/// to the sink" — the sink is just durable now. And a NEW branch exists
/// that 006 could not have: the sink itself can FAIL. A lost put must not
/// 202 (that would claim durability we don't have), so it answers 500;
/// see the counter note there.
async fn post_events<S: ObjectStore>(req: &Request, store: &S) -> Result<Response<Body>, Error> {
    let instance = state::instance();
    // teach: Ordering::Relaxed for the same reason as relay — occurrence
    // counters need atomicity, not cross-counter ordering. On this platform
    // it's doubly cheap: one request at a time means the atomics are never
    // even contended (see state.rs).
    instance.received.fetch_add(1, Ordering::Relaxed);

    // lambda_http hands the body over as an enum: Function-URL payloads
    // arrive as Text (or Binary when base64-encoded). A body that isn't
    // UTF-8 can't be a JSON object, so it gets the door's first words.
    let body = match req.body() {
        Body::Empty => "",
        Body::Text(text) => text.as_str(),
        Body::Binary(bytes) => std::str::from_utf8(bytes).unwrap_or(""),
        // teach: Body is #[non_exhaustive] — the wildcard arm is the crate
        // author reserving the right to add variants without breaking us.
        // Anything we don't recognize can't be a JSON object either.
        _ => "",
    };

    match door(body) {
        Ok(accepted) => {
            let key = ingest_key(&accepted.day, &accepted.event_id);
            match store.put(key, accepted.line.into_bytes()).await {
                Ok(()) => {
                    instance.accepted.fetch_add(1, Ordering::Relaxed);
                    // 202 Accepted, same deliberate choice as relay:
                    // "validated and handed off" — now to the lake itself.
                    empty_response(202)
                }
                Err(store_error) => {
                    // The event was VALID but did not land: telling the
                    // caller 202 would be a durability lie, so this is the
                    // one 500 in the crate (relay's "writer unavailable"
                    // analog). Diagnostics to stderr via tracing, never
                    // stdout. Counter note: this outcome is neither
                    // `accepted` (it isn't in the lake) nor `rejected`
                    // (the door said yes), so received = accepted +
                    // rejected only holds on store-failure-free runs —
                    // the healthz test pins the healthy-path identity.
                    tracing::error!(error = %store_error, "put failed after accept");
                    json_response(500, &serde_json::json!({ "error": "lake unavailable" }))
                }
            }
        }
        Err(error) => {
            instance.rejected.fetch_add(1, Ordering::Relaxed);
            // Rejections are diagnostics, not events: stderr via tracing,
            // never stdout (nothing is stored for rejected bodies — S6).
            tracing::debug!(error, "rejected at the door");
            json_response(400, &serde_json::json!({ "error": error }))
        }
    }
}

/// `GET /healthz` (H3, unchanged from 006): identity + the three
/// counters, per warm instance — see state.rs.
fn healthz() -> Result<Response<Body>, Error> {
    let instance = state::instance();
    json_response(
        200,
        &serde_json::json!({
            "instance": instance.id,
            "started": instance.started,
            "received": instance.received.load(Ordering::Relaxed),
            "accepted": instance.accepted.load(Ordering::Relaxed),
            "rejected": instance.rejected.load(Ordering::Relaxed),
        }),
    )
}

/// What the door hands the sink when it says yes: the compact line plus
/// the two identity facts the S1 key needs. In 006 the sink needed only
/// the line; the key derivation is exactly what "thread what's needed
/// cleanly" meant — the door already HAD both values (glake's
/// classification carries the day; the re-serialize parse carries the
/// event_id), it just used to drop them on the floor.
pub struct Accepted {
    /// One compact JSON line — the lake entry (A4-rev-2.1 rule intact).
    pub line: String,
    /// The event's day bucket per glake's F2 rule (`YYYY-MM-DD`; the
    /// bad-ts sentinel never reaches here — the strict door refuses it).
    pub day: String,
    /// The envelope's event_id, stringified (see [`ingest_key`] for the
    /// non-string honesty note).
    pub event_id: String,
}

/// The S1 ingest key: `raw/dt=<day>/evt-<event_id>.json` — one event, one
/// object, self-idempotent under retry because the id IS the address.
///
/// The event_id lands in the key after a conservative character filter:
/// anything outside `[A-Za-z0-9._-]` becomes `_`. S3 keys would tolerate
/// more, but a `/` inside an event_id would nest the object one level
/// deeper and silently escape the scan pack's pinned glob
/// (`raw/dt=*/*.json*` — S11); `_` keeps every object exactly where the
/// queries look. Hook-minted ids are digits-and-dashes, so in practice
/// the filter is a no-op; the collision it theoretically invites
/// (`a/b` vs `a_b`) is not worth defending against in a lab ingest.
pub fn ingest_key(day: &str, event_id: &str) -> String {
    let safe_id: String = event_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("raw/dt={day}/evt-{safe_id}.json")
}

/// Run one body through the door: `Ok(Accepted)` or the FIRST problem's
/// message, in 005's fixed order (not-a-JSON-object → first missing key →
/// no comparable day). UNCHANGED from 006 in everything H4 can observe —
/// the verdict, the strings, the ordering; the only novelty is that the
/// accepted arm now keeps the day and event_id it always had in hand.
///
/// The error strings are relay's `Rejection` Display texts VERBATIM —
/// they are the contract (H4), and the H4 property is the tripwire that
/// fires if either side ever rewords one.
fn door(body: &str) -> Result<Accepted, String> {
    match SerdeParser.classify(body, &REQUIRED_KEYS) {
        // A whitespace-only body and un-JSON junk get the same words at
        // this door — a *door* turns both away (relay's NotAJsonObject).
        ClassifiedLine::Blank | ClassifiedLine::Unparseable => Err("not a json object".to_owned()),
        ClassifiedLine::Malformed { missing } => Err(format!("missing key: {missing}")),
        // glake never rejects on ts — it buckets; the uncomparable day
        // comes back as the "bad-ts" sentinel and the strict door refuses it.
        ClassifiedLine::Event { day, .. } if day == "bad-ts" => {
            Err("no comparable day in ts".to_owned())
        }
        ClassifiedLine::Event { day, .. } => {
            // Re-serialize the parsed Value so a pretty-printed (multi-line)
            // body still lands as ONE compact line — the same A4-rev-2.1
            // rule, and the same "two parses per accepted event" price,
            // as relay and 006.
            let value: serde_json::Value =
                serde_json::from_str(body).map_err(|_| "not a json object".to_owned())?;
            // event_id is PRESENT (a missing one was refused above), but
            // presence-checking never promised a STRING: a numeric or
            // structured id gets its compact JSON rendering — an honest
            // spelling that still derives a stable, retry-idempotent key.
            let event_id = match value.get("event_id") {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(), // unreachable: key presence checked
            };
            Ok(Accepted {
                line: value.to_string(),
                day,
                event_id,
            })
        }
    }
}

/// A status-only response (202's empty body, 404).
fn empty_response(status: u16) -> Result<Response<Body>, Error> {
    // teach: `Response::builder()` returns Result because a caller can feed
    // it garbage (an invalid header name, status 9999). Ours can't fail
    // with these constant inputs, but unwrap is denied — and `?` converts
    // the http::Error into lambda_http::Error (a Box<dyn Error>) for free.
    Ok(Response::builder().status(status).body(Body::Empty)?)
}

/// A JSON response: compact one-line body, correct content-type.
/// `Value::to_string()` prints the same compact form axum's `Json` writes,
/// which is what keeps hello's 400 bodies byte-identical to relay's (H4).
fn json_response(status: u16, value: &serde_json::Value) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(value.to_string()))?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"event_id":"1","ts":"2026-07-09T01:02:03Z","session_id":"s","actor":"human","event_type":"gate.approved","schema_version":1,"payload":{}}"#;

    /// The door table — one row per clause, in the fixed first-problem
    /// order, mirroring relay's validate.rs table. UNCHANGED verdicts from
    /// 006 (the H4 property checks the doors against each other on
    /// generated inputs; this unit test keeps the door debuggable alone).
    #[test]
    fn door_verdicts_in_fixed_order() {
        // clause 1: not a JSON object (junk, non-object JSON, blank)
        for bad in ["not json", "[1,2]", "123", "\"str\"", "null", "", "  \n "] {
            assert_eq!(
                door(bad).err(),
                Some("not a json object".to_owned()),
                "{bad:?}"
            );
        }
        // clause 2: first missing key, in REQUIRED_KEYS order
        let no_actor = GOOD.replace(r#""actor":"human","#, "");
        assert_eq!(door(&no_actor).err(), Some("missing key: actor".to_owned()));
        // a body with BOTH problems reports the missing key — clause 2
        // fires before clause 3
        let both = no_actor.replace("2026-07-09T01:02:03Z", "nope");
        assert_eq!(door(&both).err(), Some("missing key: actor".to_owned()));
        // clause 3: keys all present, day not comparable
        let bad_ts = GOOD.replace("2026-07-09T01:02:03Z", "nope");
        assert_eq!(
            door(&bad_ts).err(),
            Some("no comparable day in ts".to_owned())
        );
        // happy path: compact line JSON-equal to the body, and the two
        // identity facts threaded through for the key (the v1 novelty).
        let accepted = door(GOOD).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&accepted.line).unwrap(),
            serde_json::from_str::<serde_json::Value>(GOOD).unwrap()
        );
        assert_eq!(accepted.day, "2026-07-09");
        assert_eq!(accepted.event_id, "1");
    }

    /// The A4-rev-2.1 rule survives the second move: a pretty-printed body
    /// spans lines but must come out as ONE compact line.
    #[test]
    fn door_reserializes_multiline_bodies_to_one_line() {
        let pretty =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(GOOD).unwrap())
                .unwrap();
        assert!(pretty.contains('\n'), "fixture should span lines");
        let accepted = door(&pretty).unwrap();
        assert!(
            !accepted.line.contains('\n'),
            "emitted line must be one line"
        );
    }

    /// [E] S6 (key half) — the S1 ingest key, including the sanitizer's
    /// glob-preserving rule and the non-string event_id honesty.
    #[test]
    fn s6_ingest_key_shape_and_sanitization() {
        assert_eq!(
            ingest_key("2026-07-09", "1783556456860138087-1438-29228"),
            "raw/dt=2026-07-09/evt-1783556456860138087-1438-29228.json",
            "the real envelope shape passes through untouched"
        );
        assert_eq!(
            ingest_key("2026-07-09", "a/b c\"d"),
            "raw/dt=2026-07-09/evt-a_b_c_d.json",
            "no '/' may nest the object out of the pinned glob's reach (S11)"
        );
        // A numeric event_id arrives as its JSON rendering via the door.
        let numeric_id = GOOD.replace(r#""event_id":"1""#, r#""event_id":42"#);
        let accepted = door(&numeric_id).unwrap();
        assert_eq!(accepted.event_id, "42");
        assert_eq!(
            ingest_key(&accepted.day, &accepted.event_id),
            "raw/dt=2026-07-09/evt-42.json"
        );
    }
}
