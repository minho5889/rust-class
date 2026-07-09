//! [E] A7 — the CLI as the user experiences it, via the real binary
//! (CARGO_BIN_EXE_relay). clap owns this whole surface: --help exits 0
//! with the flags AND their defaults visible; bad args exit 2 with usage
//! on stderr (same exit-code contract as glake, pinned there too).
//!
//! The bind-127.0.0.1-only half of A7 is observable in tests/shutdown.rs
//! (the child announces `listening on 127.0.0.1:<port>` and answers there).

#![allow(clippy::unwrap_used)]

use std::process::{Command, Output};

fn relay(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_relay"))
        .args(args)
        // lens builds open a memlens session even for --help; keep the
        // trace out of the repo (glake cli-test convention).
        .env("MEMLENS_TRACE", "/dev/null")
        .output()
        .unwrap()
}

/// --help exits 0 and shows both flags with both defaults — a user can
/// discover the contract without reading the spec.
#[test]
fn a7_help_shows_flags_and_defaults() {
    let out = relay(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("--lake"), "help names --lake: {help}");
    assert!(help.contains("--port"), "help names --port: {help}");
    assert!(
        help.contains("datalake/raw-local"),
        "default lake visible: {help}"
    );
    assert!(help.contains("7311"), "default port visible: {help}");
    assert!(help.contains("127.0.0.1"), "localhost-only stated: {help}");
}

/// Bad args exit 2 (clap's usage-error code — our convention since 004)
/// and say so on stderr, not via a panic.
#[test]
fn a7_bad_args_exit_2() {
    for args in [
        &["--nope"][..],
        &["--port", "not-a-number"][..],
        &["--port"][..],
        &["unexpected-positional"][..],
    ] {
        let out = relay(args);
        assert_eq!(out.status.code(), Some(2), "args {args:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!stderr.is_empty(), "clap explains itself for {args:?}");
    }
}
