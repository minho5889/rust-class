//! The one function that is the whole service (H1–H3).
//!
//! Relay needed a `Router`, an `AppState`, extractors, and a serve loop to
//! connect four handlers to the world. Here the platform hands us each
//! request already parsed into an `http::Request` and takes back an
//! `http::Response` — so the entire HTTP surface is one `match` on
//! `(method, path)`. What axum's router did with a radix tree, we do with
//! three match arms; at this size, the match IS the router.
//!
//! The door is relay's door — not a copy of its *code* but a third caller
//! of the same glake API (`SerdeParser::classify` + the bad-ts sentinel),
//! which is what makes the 202/400 behavior identical by construction. The
//! H4 property (tests/prop_door.rs) then proves the identity instead of
//! trusting it: same generated body in, same (status, body) out, hello and
//! relay compared case by case.
//!
//! Where relay SENT accepted lines to a writer task that owned the lake
//! files, this handler PRINTS them: a Lambda instance's disk is a cache
//! that dies with it, and **stdout is the only durable-by-default sink the
//! platform gives you** (every line lands in CloudWatch Logs). One event,
//! one compact JSON line — the lake entry in exile until 007 gives it an
//! S3 home. tracing goes to STDERR (main.rs) so log noise can never
//! interleave with event lines: the same stdout-is-protocol discipline as
//! relay's "listening on"/"drained" lines.

use std::sync::atomic::Ordering;

use glake::classify::REQUIRED_KEYS;
use glake::parser::{ClassifiedLine, EventParser, SerdeParser};
use lambda_http::{Body, Error, Request, Response};

use crate::state;

/// Where accepted event lines go — the testability seam, mirroring relay's
/// writer seam (there: handlers send to a channel a test can drain; here:
/// the handler emits through a trait a test can swap for a `Vec` sink).
/// Production code has exactly one implementor, [`StdoutEmit`], and the
/// seam costs one dynamic dispatch per *accepted* event — nothing on the
/// reject or healthz paths.
///
/// `Sync` is required so `&dyn Emit` can live across `.await` points in a
/// future the runtime may move between worker threads.
pub trait Emit: Sync {
    /// Emit one accepted event as one line.
    fn emit(&self, line: &str);
}

/// The production sink: one compact JSON line to stdout → CloudWatch (H1).
pub struct StdoutEmit;

impl Emit for StdoutEmit {
    fn emit(&self, line: &str) {
        println!("{line}");
    }
}

/// The entry point `main.rs` hands to `lambda_http::run` — the production
/// wiring of [`handle_with`], emitting to stdout.
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    handle_with(req, &StdoutEmit).await
}

/// The whole HTTP surface, parameterized over the emit seam (H5: tests
/// drive THIS with hand-built `lambda_http` Requests and a Vec sink — no
/// AWS, no network, no stdout capture gymnastics).
///
/// Anything that isn't exactly `POST /events` or `GET /healthz` is 404 —
/// including "right path, wrong method" (H3 says any other method/path;
/// there is no method-not-allowed nuance at this door, and H4 can't see
/// the difference because its generator only speaks POST /events).
pub async fn handle_with(req: Request, emit: &dyn Emit) -> Result<Response<Body>, Error> {
    match (req.method().as_str(), req.uri().path()) {
        ("POST", "/events") => post_events(&req, emit),
        ("GET", "/healthz") => healthz(),
        _ => empty_response(404),
    }
}

/// `POST /events` (H1/H2): count it, run the door, then either emit the
/// compact line and 202, or answer with the first problem as 400.
///
/// Same choreography as relay's `post_events`, with the writer replaced by
/// [`Emit`]: 202 still means "validated and handed to the sink" — the sink
/// just changed from an fsync'd file to CloudWatch's stdout pipe.
fn post_events(req: &Request, emit: &dyn Emit) -> Result<Response<Body>, Error> {
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
        Ok(line) => {
            emit.emit(&line);
            instance.accepted.fetch_add(1, Ordering::Relaxed);
            // 202 Accepted, same deliberate choice as relay: "validated and
            // handed off" — here to CloudWatch's pipe, not yet to a lake.
            empty_response(202)
        }
        Err(error) => {
            instance.rejected.fetch_add(1, Ordering::Relaxed);
            // Rejections are diagnostics, not events: stderr via tracing,
            // never stdout (H2: nothing is emitted for rejected bodies).
            tracing::debug!(error, "rejected at the door");
            json_response(400, &serde_json::json!({ "error": error }))
        }
    }
}

/// `GET /healthz` (H3): identity + the three counters. **Per warm
/// instance** — see state.rs for what that means and why we keep it.
/// `instance` and `started` exist precisely so deploy day can watch two
/// concurrent curls land on two different instances (T4).
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

/// Run one body through the door: `Ok(compact line)` or the FIRST problem's
/// message, in 005's fixed order (not-a-JSON-object → first missing key →
/// no comparable day).
///
/// This is relay's `validate::check` reached through the same glake calls —
/// the ordering falls out of `ClassifiedLine`'s shape exactly as it did
/// there (glake can only report a missing key on something that parsed as
/// an object, and only reports a day on something that had all its keys).
/// The error strings are relay's `Rejection` Display texts VERBATIM — they
/// are the contract (H2/H4), and the H4 property is the tripwire that fires
/// if either side ever rewords one.
fn door(body: &str) -> Result<String, String> {
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
        ClassifiedLine::Event { .. } => {
            // Re-serialize the parsed Value so a pretty-printed (multi-line)
            // body still lands as ONE compact line — the same A4-rev-2.1
            // rule, and the same "two parses per accepted event" price,
            // as relay. `Value::to_string()` (Display) is infallible, so
            // unlike relay's `serde_json::to_string` there is no impossible
            // error branch to map away.
            let value: serde_json::Value =
                serde_json::from_str(body).map_err(|_| "not a json object".to_owned())?;
            Ok(value.to_string())
        }
    }
}

/// A status-only response (202's empty body, 404).
fn empty_response(status: u16) -> Result<Response<Body>, Error> {
    // teach: `Response::builder()` returns Result because a caller can feed
    // it garbage (an invalid header name, status 9999). Ours can't fail
    // with these constant inputs, but H7 bans unwrap — and `?` converts the
    // http::Error into lambda_http::Error (a Box<dyn Error>) for free.
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
    /// order, mirroring relay's validate.rs table (the H4 property then
    /// checks the two doors against each other on generated inputs; this
    /// unit test keeps the door debuggable on its own).
    #[test]
    fn door_verdicts_in_fixed_order() {
        // clause 1: not a JSON object (junk, non-object JSON, blank)
        for bad in ["not json", "[1,2]", "123", "\"str\"", "null", "", "  \n "] {
            assert_eq!(door(bad), Err("not a json object".to_owned()), "{bad:?}");
        }
        // clause 2: first missing key, in REQUIRED_KEYS order
        let no_actor = GOOD.replace(r#""actor":"human","#, "");
        assert_eq!(door(&no_actor), Err("missing key: actor".to_owned()));
        // a body with BOTH problems reports the missing key — clause 2
        // fires before clause 3
        let both = no_actor.replace("2026-07-09T01:02:03Z", "nope");
        assert_eq!(door(&both), Err("missing key: actor".to_owned()));
        // clause 3: keys all present, day not comparable
        let bad_ts = GOOD.replace("2026-07-09T01:02:03Z", "nope");
        assert_eq!(door(&bad_ts), Err("no comparable day in ts".to_owned()));
        // happy path: compact line, JSON-equal to the body
        let line = door(GOOD).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&line).unwrap(),
            serde_json::from_str::<serde_json::Value>(GOOD).unwrap()
        );
    }

    /// The A4-rev-2.1 rule survives the move: a pretty-printed body spans
    /// lines but must come out as ONE compact line.
    #[test]
    fn door_reserializes_multiline_bodies_to_one_line() {
        let pretty =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(GOOD).unwrap())
                .unwrap();
        assert!(pretty.contains('\n'), "fixture should span lines");
        let line = door(&pretty).unwrap();
        assert!(!line.contains('\n'), "emitted line must be one line");
    }
}
