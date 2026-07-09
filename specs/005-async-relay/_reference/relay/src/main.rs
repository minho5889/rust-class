//! Thin binary: parse args, wire channel → writer → router → server, and
//! get the SHUTDOWN ORDER right. The order is the sitting-N lesson:
//!
//! ```text
//! 1. (tx, rx) = mpsc::channel(256)         bounded — 202 means "enqueued"
//! 2. writer = tokio::spawn(writer::run(rx))  the owner starts first
//! 3. state  = AppState::new(tx)            tx MOVES in; main keeps NO clone
//! 4. axum::serve(...).with_graceful_shutdown(ctrl_c).await
//!       └─ ctrl-c → listener closes (late requests refused, A6a),
//!          in-flight requests finish, serve returns, router+state DROP
//!            └─ last Sender drops → writer's recv() → None
//! 5. writer.await                          drain + flush finish (A6b)
//! 6. print the goodbye, exit 0
//! ```
//!
//! **The footgun, demonstrated by its absence:** if step 3 were
//! `AppState::new(tx.clone())` with `tx` still alive in `main`, step 5
//! would wait forever — `recv()` can't return `None` while ANY sender
//! exists, and `main`'s copy would sit in scope until after the `.await`
//! that's waiting on it. Deadlock by refcount. Predict that hang before
//! running it and you understand channel ownership (sitting N's checkpoint
//! question).

#![forbid(unsafe_code)]

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use clap::Parser;
use tokio::sync::mpsc;

use relay::routes::{self, AppState};
use relay::writer;

// A10: the lens, on a service. Same optional-dep pattern as glake — the
// tracking allocator only exists in the binary under `--features lens`.
#[cfg(feature = "lens")]
#[global_allocator]
static LENS: memlens::MemLens<std::alloc::System> = memlens::MemLens::system();

/// relay v0 — receive goldeneye telemetry events over HTTP, land them in
/// the lake (spec 005). Localhost lab tool: binds 127.0.0.1 only.
#[derive(Debug, Parser)]
#[command(name = "relay", version)]
struct Args {
    /// Lake root directory (events land in <lake>/dt=<day>/events.jsonl)
    #[arg(long, default_value = "datalake/raw-local")]
    lake: PathBuf,

    /// Port to listen on (127.0.0.1 only; 0 = let the OS pick)
    #[arg(long, default_value_t = 7311)]
    port: u16,
}

// teach: `#[tokio::main]` is sugar for building a multi-thread Runtime and
// `block_on`-ing this function — async main is a library feature, not a
// language one. `anyhow::Result` is the binary-surface error policy
// (CLAUDE.md): the library speaks typed errors; the binary's `?` collapses
// whatever reaches the top (bind failure, io) into one printed line and a
// nonzero exit. Bad ARGS never get here — clap prints usage and exits 2
// on its own (A7).
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    #[cfg(feature = "lens")]
    let _session = memlens::session("relay");

    // Minimal tracing to STDERR — stdout is spoken for: the two protocol
    // lines below ("listening on", "drained") are parsed by the A6 test
    // and must not be interleaved with log noise.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Step 1 — the bounded channel (capacity 256, a Key-decisions row).
    // Bounded is what makes 202 honest: when the writer falls behind,
    // `send().await` in the handler WAITS (backpressure) instead of
    // buffering unboundedly toward OOM.
    let (tx, rx) = mpsc::channel::<String>(256);

    // Step 2 — the owner. `tokio::spawn` hands `rx` and the lake path to
    // the writer task (both MOVE — nothing shared) and gives back a
    // JoinHandle we hold until the very end.
    let writer_handle = tokio::spawn(writer::run(rx, args.lake.clone()));

    // Step 3 — tx moves INTO the state, state moves into the router.
    // After these two lines, `main` provably holds no sender: the drain
    // trigger now belongs entirely to the router's lifetime.
    let state = AppState::new(tx);
    let app = routes::router(state);

    // 127.0.0.1, never 0.0.0.0 — a lab tool has no business on the network
    // (A7). Binding port 0 makes the OS assign one, which is why the
    // "listening on" line reports `local_addr()` (the REAL port), not
    // `args.port`: the A6 test — and any script — needs the truth.
    let listener =
        tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, args.port))).await?;
    let port = listener.local_addr()?.port();
    println!("relay: listening on 127.0.0.1:{port}");

    // Step 4 — serve until ctrl-c. `with_graceful_shutdown` takes a future;
    // when it completes, axum stops accepting (A6a: the listener closes, so
    // late connections are refused), lets in-flight requests finish, and
    // only then does this `.await` return. The `app` we moved in is dropped
    // right here — and with it the last `tx`.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Step 5 — the writer is now draining a closed channel; await the
    // handle to hold the door until every 202'd line is flushed (A6b).
    // The JoinHandle yields Result<u64, JoinError>: Err only if the writer
    // PANICKED. anyhow's `?` surfaces that as a nonzero exit — a crashed
    // writer must not masquerade as a clean drain.
    let written = writer_handle.await?;

    println!("relay: drained, {written} events on disk. bye.");
    Ok(())
}

/// Completes when the process receives ctrl-c (SIGINT — which is exactly
/// what `kill -INT <pid>` and a terminal ^C both deliver; the A6 test
/// sends the real signal, not a simulated future).
async fn shutdown_signal() {
    // teach: if installing the signal handler itself fails (it practically
    // can't), the least-wrong move is to shut down NOW rather than run a
    // service that ctrl-c cannot stop. No unwrap (A9) — returning IS the
    // shutdown signal here.
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("cannot listen for ctrl-c ({e}); shutting down");
    }
}
