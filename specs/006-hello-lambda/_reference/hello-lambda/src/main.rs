//! The binary — and this is ALL of it. Put relay's main.rs (134 lines:
//! channel, writer task, state, listener, graceful-shutdown choreography,
//! the footgun essay) next to these few: everything relay had to get right
//! by hand is now the platform's problem. What you still own is (1) where
//! logs go, (2) the handler. That's T1 in its purest form.
//!
//! What `lambda_http::run` does under the hood — the loop you no longer
//! write: long-poll the Runtime API for the next event, deserialize the
//! Function-URL payload into an `http::Request`, call your handler, post
//! the response back, repeat. One event at a time per instance; the fleet,
//! not the process, is the concurrency unit (T4). When this instance is
//! due to die, the platform just stops sending it events and reaps it —
//! there is no ctrl-c to catch, no channel to drain, no drop-ordering
//! footgun. (`provided.al2023` boots, execs the binary it finds at
//! `bootstrap`, and that's the whole "custom runtime" story — T2.)

#![forbid(unsafe_code)]

use lambda_http::{Error, run, service_fn};

use hello_lambda::{handler, state};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Tracing to STDERR — stdout is spoken for: it carries exactly one
    // compact JSON line per accepted event and NOTHING else (both streams
    // land in CloudWatch, but only stdout is our event protocol). Same
    // discipline as relay, where stdout carried the two protocol lines the
    // A6 test parsed and logs went to stderr.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Touch the instance state EAGERLY, during init: `started` should mark
    // the cold start itself, not whenever the first request happens to
    // arrive. (Init happens before the first invoke is billed at 128 MB —
    // and this line is also your cold-start marker in the stderr logs for
    // the H10 experiment.)
    let instance = state::instance();
    tracing::info!(id = %instance.id, started = %instance.started, "cold start");

    // The server you don't write: hand the platform one async fn.
    run(service_fn(handler::handler)).await
}
