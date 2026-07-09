//! [E] A6a/A6b — graceful shutdown, tested against the REAL thing: the
//! actual binary (CARGO_BIN_EXE_relay), a real TCP listener, and a real
//! SIGINT delivered with `kill -INT` — exactly what a terminal ^C sends.
//! The tempting shortcut (injecting an abstract shutdown future into an
//! in-process server) would never exercise the `tokio::signal::ctrl_c`
//! wiring, the drop-tx choreography in `main`, or the process exit code —
//! which are the three things A6 is ABOUT.
//!
//! HTTP here is hand-rolled over `TcpStream` on purpose (and it teaches
//! something): a request is four header lines, a blank line, and the body.
//! No client crate needed to speak it.

#![allow(clippy::unwrap_used)]

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use common::{TempLake, canon, disk_lines};

/// A valid envelope body (all REQUIRED_KEYS, comparable day in ts).
fn envelope(id: &str, ts: &str) -> String {
    serde_json::json!({
        "event_id": id,
        "ts": ts,
        "session_id": "shutdown-test",
        "actor": "test",
        "event_type": "test.event",
        "schema_version": 1,
        "payload": {}
    })
    .to_string()
}

/// One hand-rolled HTTP/1.1 POST /events; returns (status, body).
/// `Connection: close` makes the server end the stream after responding,
/// so "read to EOF" is a complete-response detector — no framing needed.
fn http_post_events(port: u16, body: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        stream,
        "POST /events HTTP/1.1\r\n\
         Host: 127.0.0.1:{port}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    // Status line: "HTTP/1.1 202 Accepted" — the second word is the code.
    let status: u16 = response.split_whitespace().nth(1).unwrap().parse().unwrap();
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_owned())
        .unwrap_or_default();
    (status, body)
}

/// Poll `try_wait` with a deadline — a plain `wait()` would hang the whole
/// test run forever if shutdown regressed, which is exactly when we want a
/// loud failure instead.
fn wait_with_timeout(child: &mut Child, limit: Duration) -> ExitStatus {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("relay did not exit within {limit:?} after SIGINT");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// [E] A6a + A6b in one child lifetime: 202'd events survive ctrl-c, late
/// requests are refused, the exit code is 0.
#[test]
fn a6_sigint_drains_flushes_and_exits_zero() {
    let lake = TempLake::new();

    // --port 0: the OS assigns a free port (parallel test runs can't
    // collide), and the relay prints the REAL port for us to parse.
    let mut child = Command::new(env!("CARGO_BIN_EXE_relay"))
        .arg("--port")
        .arg("0")
        .arg("--lake")
        .arg(lake.path())
        .stdout(Stdio::piped())
        // Under --features lens the binary opens a memlens session; keep
        // its trace out of the repo (same convention as glake's cli tests).
        .env("MEMLENS_TRACE", "/dev/null")
        .spawn()
        .unwrap();

    // Startup handshake: the first stdout line announces the bound port.
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    assert!(
        line.starts_with("relay: listening on 127.0.0.1:"),
        "unexpected first line: {line:?}"
    );
    let port: u16 = line.trim().rsplit(':').next().unwrap().parse().unwrap();

    // A couple of valid events across two days, plus one reject.
    let bodies = [
        envelope("shutdown-1", "2026-07-09T01:00:00Z"),
        envelope("shutdown-2", "2026-07-05T02:00:00Z"),
        envelope("shutdown-3", "2026-07-09T03:00:00Z"),
    ];
    for body in &bodies {
        let (status, _) = http_post_events(port, body);
        assert_eq!(status, 202);
    }
    let (status, error) = http_post_events(port, "not an envelope");
    assert_eq!(status, 400);
    assert!(error.contains("not a json object"), "got: {error:?}");

    // The real signal — what ^C delivers. std::process::Command + kill(1),
    // no signal crate needed.
    let kill = Command::new("kill")
        .arg("-INT")
        .arg(child.id().to_string())
        .status()
        .unwrap();
    assert!(kill.success());

    // A6b: exits ON ITS OWN, with code 0 (a drain that needed SIGKILL, or
    // a writer deadlock — the footgun — would trip the timeout instead).
    let status = wait_with_timeout(&mut child, Duration::from_secs(10));
    assert_eq!(status.code(), Some(0), "clean exit after SIGINT");

    // The goodbye line reports the writer's own drained count.
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    assert!(
        rest.contains("relay: drained, 3 events on disk. bye."),
        "stdout after shutdown: {rest:?}"
    );

    // A6a: the listener is gone — a late request can't even connect.
    assert!(
        TcpStream::connect(("127.0.0.1", port)).is_err(),
        "post-SIGINT connect must be refused"
    );

    // A6b: every 202'd event is on disk (as parsed values), each in the
    // dt= folder its own ts names; the reject is nowhere.
    let found = disk_lines(lake.path());
    let mut on_disk: Vec<String> = found.iter().map(|(_, l)| canon(l)).collect();
    let mut sent: Vec<String> = bodies.iter().map(|b| canon(b)).collect();
    on_disk.sort();
    sent.sort();
    assert_eq!(on_disk, sent, "202'd events on disk, nothing else");
    for (day, line) in &found {
        let value: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(
            value["ts"].as_str().unwrap().starts_with(day.as_str()),
            "line in dt={day} claims ts {}",
            value["ts"]
        );
    }
}
