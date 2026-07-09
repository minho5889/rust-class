//! The HTTP surface (A1–A3): two routes, one shared state.
//!
//! The T4 lesson lives in [`AppState`]'s two fields, which solve two
//! *different* sharing problems two different ways:
//!
//! - **counters** — many tasks bump three numbers. Numbers compose under
//!   concurrent increment, so `Arc<Counters>` of `AtomicU64` is enough:
//!   share freely, no lock, no channel.
//! - **the file** — many tasks want to append *multi-byte lines*. Bytes do
//!   NOT compose under concurrent write (interleaving tears lines), and no
//!   atomic fixes that. So the file isn't in `AppState` at all: handlers
//!   hold only `tx`, the sending half of a channel, and *ownership of each
//!   line* moves to the one writer task.
//!
//! Same struct, both answers, side by side — that contrast is the sitting-L
//! teaching point.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use tokio::sync::mpsc;

use crate::validate;

/// The three `/healthz` numbers (A3). Independent atomics, NOT one struct
/// behind a `Mutex`: each counter is updated with a single hardware
/// instruction and never blocks a handler.
///
/// The honest cost of that choice: the triple is not a consistent
/// *snapshot*. A reader can observe `received` already bumped while the
/// same request's `accepted` bump hasn't landed yet — so the invariant
/// `received = accepted + rejected` is promised only **once no requests
/// are in flight** (quiescent consistency, A3 as amended). That's the T4
/// lesson stated, not hidden.
#[derive(Debug, Default)]
pub struct Counters {
    /// Every `POST /events` that reached the handler.
    pub received: AtomicU64,
    /// Bodies that passed the door AND were enqueued to the writer (202).
    pub accepted: AtomicU64,
    /// Bodies turned away (400) — plus the can't-happen enqueue failure,
    /// counted here so the quiescent invariant survives even that path.
    pub rejected: AtomicU64,
}

/// Everything a handler can reach, cloned once per request.
///
/// Why the derive compiles — and what it costs — is the T3 lesson:
/// `Clone` here copies one channel-sender handle and bumps one `Arc`
/// refcount; nothing deep. And axum demands `AppState: Send + Sync +
/// 'static` because handlers run on any worker thread; the compiler checks
/// field-by-field that crossing threads is safe (`Sender` and
/// `Arc<Counters>` both are). If you tried to put the open `File` in here
/// instead, it would *also* compile — `File` is `Send` — and then tear
/// lines at runtime. `Send` means "may move/be shared across threads",
/// not "concurrent use is correct". Types rule out the crashes; the
/// single-writer *design* rules out the tearing.
#[derive(Clone)]
pub struct AppState {
    /// The sending half of the bounded channel to the writer task.
    ///
    /// This is the shutdown linchpin (the design's named footgun): `main`
    /// moves `tx` in here and keeps **no clone of its own**, so when the
    /// server future finishes and the router drops, the LAST sender drops,
    /// the writer's `recv()` yields `None`, and the drain begins. A `tx`
    /// clone forgotten in `main` = a writer that waits forever.
    tx: mpsc::Sender<String>,
    /// Shared counters — see [`Counters`] for why atomics and not a Mutex.
    counters: Arc<Counters>,
}

impl AppState {
    /// Build the state, taking `tx` **by value** — the signature itself
    /// enforces half the footgun rule: the caller must give its sender up
    /// (it can still defeat that with `tx.clone()`; sitting N's checkpoint
    /// question is about predicting what happens if it does).
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        AppState {
            tx,
            counters: Arc::new(Counters::default()),
        }
    }

    /// A second handle to the counters (for tests and `main` to read after
    /// the state itself has been consumed by the router).
    pub fn counters(&self) -> Arc<Counters> {
        Arc::clone(&self.counters)
    }
}

/// The whole HTTP surface: `POST /events` (the door) and `GET /healthz`
/// (the counters). `with_state` is what turns `Router<AppState>` into the
/// plain `Router` that `axum::serve` (and `tower::ServiceExt::oneshot` in
/// tests) accepts.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/events", post(post_events))
        .route("/healthz", get(healthz))
        .with_state(state)
}

/// `POST /events` (A1/A2): count it, run the door, then either enqueue the
/// compact line (202) or answer with the first problem (400).
///
/// Note what this handler *doesn't* do: touch a file. Its slowest step is
/// `send().await` — and even that only waits when the channel is full,
/// which is the bounded channel doing its job (backpressure: a 202 must
/// honestly mean "enqueued", so under overload we make the client wait
/// instead of buffering unboundedly toward OOM).
async fn post_events(State(state): State<AppState>, body: String) -> Response {
    // teach: Ordering::Relaxed is enough for plain occurrence counters —
    // we need each increment to happen exactly once (atomicity), not to
    // order other memory operations around it (no reader derives anything
    // from cross-counter ordering; A3 only promises the quiescent sum).
    state.counters.received.fetch_add(1, Ordering::Relaxed);

    let accepted = match validate::check(&body) {
        Ok(accepted) => accepted,
        Err(rejection) => {
            state.counters.rejected.fetch_add(1, Ordering::Relaxed);
            // A2: one-line JSON body naming the first problem. axum's Json
            // serializes compact — exactly the {"error":"…"} shape.
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": rejection.to_string() })),
            )
                .into_response();
        }
    };

    // teach: `send(String)` MOVES the line into the channel — this await is
    // the ownership transfer the whole design pivots on. After it, the
    // handler provably can't touch those bytes again; only the writer can.
    // (`accepted.day` is dropped here on purpose: the channel speaks plain
    // Strings, and the writer re-derives the day — see writer.rs.)
    match state.tx.send(accepted.line).await {
        Ok(()) => {
            state.counters.accepted.fetch_add(1, Ordering::Relaxed);
            // 202 Accepted, chosen over 200/201 deliberately: "validated
            // and enqueued; on disk no later than shutdown" (A1) — not
            // "durably stored right now".
            StatusCode::ACCEPTED.into_response()
        }
        // send() fails only if the receiver is gone, i.e. the writer task
        // died. Unreachable by construction (the writer outlives the
        // router — main awaits it AFTER serve returns), but A9 says no
        // unwrap: the impossible branch costs three lines and keeps the
        // quiescent invariant by counting itself as a rejection.
        Err(_) => {
            state.counters.rejected.fetch_add(1, Ordering::Relaxed);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "writer unavailable" })),
            )
                .into_response()
        }
    }
}

/// `GET /healthz` (A3): the three counters as JSON. Three independent
/// relaxed loads — see [`Counters`] for why the triple is only promised to
/// sum up once the service is quiescent.
async fn healthz(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "received": state.counters.received.load(Ordering::Relaxed),
        "accepted": state.counters.accepted.load(Ordering::Relaxed),
        "rejected": state.counters.rejected.load(Ordering::Relaxed),
    }))
}
