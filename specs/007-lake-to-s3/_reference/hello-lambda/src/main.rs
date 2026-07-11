//! The binary — still almost nothing, but the "almost" is now the T3
//! lesson: everything expensive happens ONCE, here, during init, before
//! the platform sends a single event.
//!
//! ```text
//!   init (once per cold start)                 per request
//!   ──────────────────────────────────         ─────────────────────────
//!   subscriber → state → aws_config    ─►      handle_with(req, store)
//!   → S3Store → OnceLock::park                 (no config, no client
//!   → lambda_http::run                          construction, no locks
//!                                               beyond one OnceLock read)
//! ```
//!
//! Why the client is built in `main` and not in the handler: config
//! loading resolves the credential chain and region (env vars the
//! platform injects) and builds a connection-pooled HTTP client — an
//! async, potentially slow dance. Doing it per-request would tax every
//! event; doing it lazily on first request would hide the cost inside
//! someone's latency. Doing it HERE puts it in the init phase, where it
//! runs once per instance. This is the "client built once, before
//! `lambda_http::run`" discipline (design.md), and it's the same reason
//! relay bound its listener before announcing "listening on".
//!
//! Why the `OnceLock`: precise version — a *non-move* closure borrowing a
//! `main`-local store actually compiles here (main never returns while the
//! runtime runs), but the moment you write `move` (which async blocks
//! usually force) the `Fn`-capture rules bite: the moved store would be
//! given away on the first call. Parking the built store in a `static
//! OnceLock` sidesteps the whole cliff edge and mints a plain
//! `&'static S3Store` — no `Box::leak` trick, no global constructor, no
//! unsafe. (Same tool state.rs already uses for the instance record,
//! doing the same job for a value that must be *constructed
//! asynchronously* first.)

#![forbid(unsafe_code)]

use std::sync::OnceLock;

use lake_store::s3::{S3Client, S3Store};
use lambda_http::{Error, run, service_fn};

use hello_lambda::{handler, state};

/// The one store this instance ever builds — written once in `main`,
/// read by every request the instance serves.
static STORE: OnceLock<S3Store> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Tracing to STDERR — unchanged from 006 even though stdout lost its
    // protocol job (S6b): accepted events now go to S3, and stdout simply
    // carries nothing. Keeping logs on stderr preserves the discipline
    // (and the CloudWatch layout) rather than reclaiming stdout for noise.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Touch the instance state EAGERLY, during init: `started` should mark
    // the cold start itself, not whenever the first request happens to
    // arrive (H10's cold-start marker in the stderr logs).
    let instance = state::instance();
    tracing::info!(id = %instance.id, started = %instance.started, "cold start");

    // The bucket is configuration, not code: the CDK stack sets
    // LAKE_BUCKET on the function (S10). Missing it is a deploy bug, and
    // the honest response is to fail INIT loudly (the platform logs it
    // and retries) — not to limp into serving 500s with a made-up name.
    let bucket = std::env::var("LAKE_BUCKET")
        .map_err(|_| Error::from("LAKE_BUCKET must be set (the CDK stateless stack sets it)"))?;

    // The async construction that motivates this whole file: resolve the
    // default chain (env credentials + AWS_REGION on Lambda), then build
    // the ONE client. `S3Client` is lake-store's re-export — the SDK
    // stays named in exactly one crate (S7).
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let store: &'static S3Store =
        STORE.get_or_init(move || S3Store::new(S3Client::new(&config), bucket));

    // The server you don't write: hand the platform one async fn — now a
    // closure over the parked store, monomorphized to S3Store (the same
    // handle_with the tests call with a FakeStore).
    run(service_fn(move |req| handler::handle_with(req, store))).await
}
