//! [E] R1a/R1b/R2/R3a/R3b/R5/R6 — the tool as the user experiences it.
//! Runs the real binary (CARGO_BIN_EXE_glake, a std-only cargo feature) over
//! the fixtures: 3 valid events, 1 blank line, 1 missing-actor line, across
//! two dt= partitions.

use std::path::Path;
use std::process::{Command, Output};

fn glake(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_glake"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("binary runs")
}

fn fixture(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel)
        .display()
        .to_string()
}

#[test]
fn r1a_r1b_validate_reports_and_exits_nonzero() {
    let out = glake(&["validate", &fixture("lake")]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("missing key \"actor\""),
        "R1a names the key: {stdout}"
    );
    assert!(
        stdout.contains("dt=2026-07-02") && stdout.contains(":2"),
        "R1a file:line: {stdout}"
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "R1b: exit 1 when malformed exist"
    );
}

#[test]
fn r1b_validate_clean_lake_exits_zero() {
    let out = glake(&["validate", &fixture("lake/dt=2026-07-01/events.jsonl")]);
    assert_eq!(out.status.code(), Some(0), "R1b: clean file exits 0");
}

#[test]
fn r2_r3a_r5_stats_counts_both_axes_recursively() {
    let out = glake(&["stats", &fixture("lake")]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // R3a: both dt= partitions found; R5: the blank line counted nowhere
    assert!(
        stdout.contains("2 files · 3 events"),
        "grand total: {stdout}"
    );
    assert!(
        stdout.contains("gate.approved") && stdout.contains("2"),
        "by type: {stdout}"
    );
    assert!(
        stdout.contains("2026-07-01") && stdout.contains("2026-07-02"),
        "by day: {stdout}"
    );
    assert!(
        stdout.contains("1 malformed"),
        "malformed surfaced, not dropped: {stdout}"
    );
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn r3b_single_file_reads_just_that_file() {
    let out = glake(&["stats", &fixture("lake/dt=2026-07-01/events.jsonl")]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("1 files · 2 events"),
        "single file only: {stdout}"
    );
}

#[test]
fn r6_bad_path_stderr_exit2_no_panic() {
    let out = glake(&["stats", "/no/such/path"]);
    assert_eq!(out.status.code(), Some(2), "R6: exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("/no/such/path"),
        "R6: clear stderr: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "R6: no panic");
}

#[test]
fn r6_usage_on_bad_args() {
    for args in [&[][..], &["frobnicate", "x"][..], &["stats"][..]] {
        let out = glake(args);
        assert_eq!(out.status.code(), Some(2), "usage exits 2 for {args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    }
}
