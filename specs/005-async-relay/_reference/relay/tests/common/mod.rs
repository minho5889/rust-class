//! Shared plumbing for the integration tests: a self-cleaning temp lake,
//! an in-process relay (router + writer, no sockets), request builders,
//! and a disk reader. Each integration test file is its own crate, so this
//! module is compiled into each one that declares `mod common;`.

// Not every test file uses every helper; that's fine for test plumbing.
#![allow(dead_code)]
// Tests may unwrap: a panicking test is a FAILING test, which is the point.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use relay::routes::{self, AppState};
use relay::writer;

/// A unique, self-cleaning lake directory under the OS temp dir.
///
/// Why not the real `datalake/raw-local`: the shell hooks append to the
/// live lake concurrently with everything else (requirements, "what we're
/// not building" — no cross-process locking), so property runs and
/// examples must use a lake nobody else is writing to.
///
/// Uniqueness = pid + a per-process counter: parallel tests in one binary
/// get distinct counters, parallel test binaries get distinct pids.
pub struct TempLake {
    path: PathBuf,
}

static NEXT: AtomicU32 = AtomicU32::new(0);

impl TempLake {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "relay-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        TempLake { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// teach: Drop is how Rust spells "cleanup that cannot be forgotten" — the
// dir disappears when the value goes out of scope, pass or fail. (If the
// test PANICS mid-way, unwinding still runs Drop; only abort would skip it.)
impl Drop for TempLake {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// An in-process relay: bounded channel, real writer task, real router —
/// everything except the TCP listener. Exactly the wiring `main.rs` does,
/// minus the socket, so `tower::ServiceExt::oneshot` can drive it.
///
/// The caller gets the router AND the writer's `JoinHandle`; dropping the
/// router (all clones of it) is the graceful-shutdown trigger, after which
/// `writer.await` yields the number of lines flushed to disk.
pub fn spawn_relay(lake: &Path) -> (Router, JoinHandle<u64>) {
    let (tx, rx) = mpsc::channel::<String>(256);
    let writer = tokio::spawn(writer::run(rx, lake.to_path_buf()));
    let app = routes::router(AppState::new(tx));
    (app, writer)
}

/// Build a `POST /events` request with the given raw body.
pub fn post_event(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/events")
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

/// Build a `GET /healthz` request.
pub fn get_healthz() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::empty())
        .unwrap()
}

/// Collect a response body as parsed JSON.
pub async fn body_json(body: Body) -> serde_json::Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// The canonical form of one JSON text: parse, re-serialize compact.
/// serde_json's default map is a BTreeMap, so keys come out sorted —
/// pretty-printed and compact spellings of the same value canonicalize
/// identically, which is what "compared as parsed JSON values" (A4)
/// needs in executable form.
pub fn canon(json_text: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(json_text).unwrap();
    value.to_string()
}

/// Every line in the lake, as `(day, raw line)` pairs — `day` from the
/// `dt=<day>` folder the line was found in.
pub fn disk_lines(lake: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(lake).unwrap() {
        let dir = entry.unwrap().path();
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let Some(day) = name.strip_prefix("dt=") else {
            continue;
        };
        let content = std::fs::read_to_string(dir.join("events.jsonl")).unwrap();
        for line in content.lines() {
            out.push((day.to_owned(), line.to_owned()));
        }
    }
    out
}
