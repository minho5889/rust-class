//! [E] S2 / S5 / S9 — the examples, in two registers:
//!
//! - **Binary tests** (`CARGO_BIN_EXE_lake-sync`) pin the shell contract:
//!   usage and io failures exit 2 with one stderr line (glake's
//!   discipline), and `--dry-run` works with NO AWS anything — the
//!   environment is scrubbed to prove it. What a binary test *cannot* see
//!   is a store ledger — the real store needs credentials this authoring
//!   environment deliberately lacks.
//! - **Library tests** drive the same plan/run code the binary calls,
//!   with the FAKE in the store seat — that's where the ledger assertions
//!   live (dry-run's zero puts, S5's mid-run failure, S9's gauge).
//!
//! Together they cover the CLI's whole surface; the only untested line is
//! the aws-config/S3Store construction in main.rs, which is deploy-day
//! material (S12) by design.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use lake_store::ObjectStore;
use lake_store::fake::FakeStore;
use lake_sync::plan::plan;
use lake_sync::run::{MAX_IN_FLIGHT, run};

// ---------------------------------------------------------------- helpers

/// The lake-sync binary with a SCRUBBED environment: no AWS_* variables,
/// no HOME (so no ~/.aws), no nothing. Every test that passes with this
/// command proves its path needs no AWS.
fn bin() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lake-sync"));
    command.env_clear();
    command
}

/// A small fixture lake exercising the S1 domain: a plain day, the
/// quarantine partition, and a traces arm.
fn fixture_lake() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, content) in [
        ("dt=2026-07-05/events.jsonl", "{\"a\":1}\n{\"b\":2}\n"),
        ("dt=bad-ts/events.jsonl", "quarantined but legal\n"),
        ("traces/dt=2026-07-05/run-1.jsonl", "{\"t\":1}\n"),
    ] {
        let path = dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    dir
}

/// N-file lake for the gauge test (every file distinct, all must upload).
fn many_file_lake(n: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..n {
        let path = dir.path().join(format!("dt=2026-07-05/file-{i:02}.jsonl"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("{{\"file\":{i}}}\n")).unwrap();
    }
    dir
}

// ----------------------------------------------------------- binary tests

/// [E] S5 (usage half) — no args: clap's own message, exit 2.
#[test]
fn s5_no_args_is_usage_exit_2() {
    let output = bin().output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty(), "clap explains itself on stderr");
}

/// [E] S5 — a real run needs a destination: path without --bucket is a
/// usage error (exit 2) that NAMES the missing flag.
#[test]
fn s5_missing_bucket_without_dry_run_is_usage_exit_2() {
    let dir = fixture_lake();
    let output = bin().arg(dir.path()).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--bucket"),
        "stderr names the missing flag"
    );
}

/// [E] S5 — unreadable path: ONE stderr line (`lake-sync: …` with the
/// path in it), exit 2 — glake's error discipline, verbatim.
#[test]
fn s5_unreadable_path_one_stderr_line_exit_2() {
    let output = bin().args(["--dry-run", "/no/such/lake"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
    assert!(stderr.starts_with("lake-sync: "), "{stderr}");
    assert!(stderr.contains("/no/such/lake"), "path context: {stderr}");
    assert!(output.stdout.is_empty(), "errors never go to stdout");
}

/// [E] S2 — --dry-run prints the plan (S1 keys, verbatim `traces/` and
/// `dt=bad-ts`) and performs zero puts. Zero puts is STRUCTURAL for the
/// binary (the dry-run path constructs no store — the scrubbed env is
/// the proof: with any AWS dependency this would fail, not skip), and
/// double-checked below at lib level with the fake's ledger.
#[test]
fn s2_dry_run_plans_zero_puts_no_aws() {
    let dir = fixture_lake();
    let before = std::fs::read(dir.path().join("dt=2026-07-05/events.jsonl")).unwrap();

    // --bucket present AND ignored: deploy-day muscle memory can keep the
    // flag; the plan is the same offline answer either way (S2).
    let output = bin()
        .args(["--bucket", "goldeneye-lake", "--dry-run"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("upload raw/dt=2026-07-05/events.jsonl"),
        "{stdout}"
    );
    assert!(
        stdout.contains("upload raw/dt=bad-ts/events.jsonl"),
        "{stdout}"
    );
    assert!(
        stdout.contains("upload raw/traces/dt=2026-07-05/run-1.jsonl"),
        "{stdout}"
    );
    assert!(stdout.ends_with("plan: upload 3, skip 0\n"), "{stdout}");

    // Stateless and side-effect free: same answer twice, lake untouched.
    let again = bin().args(["--dry-run"]).arg(dir.path()).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&again.stdout), stdout);
    assert_eq!(
        std::fs::read(dir.path().join("dt=2026-07-05/events.jsonl")).unwrap(),
        before,
        "dry-run must not touch the lake"
    );
}

/// The --prefix flag normalizes: `raw` and `raw/` produce the same keys.
#[test]
fn s1_prefix_is_normalized() {
    let dir = fixture_lake();
    let output = bin()
        .args(["--dry-run", "--prefix", "raw"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("upload raw/dt=2026-07-05/events.jsonl"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("rawdt="),
        "missing slash would corrupt every key"
    );
}

// ---------------------------------------------------------- library tests

/// [E] S2, ledger half — the dry-run FLOW (list + plan, never run)
/// against a fake that already holds objects: the plan sees skips, and
/// the ledger stays at zero puts.
#[tokio::test]
async fn s2_dry_run_flow_shows_zero_puts_on_the_ledger() {
    let dir = fixture_lake();
    let store = Arc::new(FakeStore::new());
    // Pre-populate one matching object so the plan has a real skip.
    store
        .put(
            "raw/dt=bad-ts/events.jsonl".to_owned(),
            b"quarantined but legal\n".to_vec(),
        )
        .await
        .unwrap();
    let puts_before = store.puts();

    let remote = store.list("raw/").await.unwrap();
    let the_plan = plan(dir.path(), "raw/", &remote).unwrap();

    assert_eq!(the_plan.upload.len(), 2);
    assert_eq!(the_plan.skip, ["raw/dt=bad-ts/events.jsonl"]);
    assert_eq!(
        store.puts(),
        puts_before,
        "planning performs zero puts (S2)"
    );
}

/// [E] S5 — a put that fails mid-run: the object is NAMED on stderr, the
/// exit code is non-zero, and already-uploaded objects are reported —
/// partial progress stated, not hidden.
#[tokio::test]
async fn s5_fail_on_mid_run_names_object_reports_partial_progress() {
    let dir = fixture_lake();
    let store = Arc::new(FakeStore::new());
    let doomed = "raw/dt=bad-ts/events.jsonl";
    store.fail_on(doomed);

    let the_plan = plan(dir.path(), "raw/", &[]).unwrap();
    let outcome = run(Arc::clone(&store), the_plan).await.unwrap();

    // The failure, named and non-zero.
    assert_eq!(outcome.failed.len(), 1);
    assert_eq!(outcome.failed[0].0, doomed);
    assert_ne!(outcome.exit_code(), 0, "failed puts exit non-zero");
    let stderr = outcome.stderr_report();
    assert!(stderr.contains(doomed), "object named on stderr: {stderr}");
    assert!(
        stderr.lines().all(|l| l.starts_with("lake-sync: ")),
        "{stderr}"
    );

    // Partial progress, stated: both survivors uploaded AND named.
    assert_eq!(outcome.uploaded.len(), 2);
    let stdout = outcome.stdout_report();
    assert!(
        stdout.contains("uploaded raw/dt=2026-07-05/events.jsonl"),
        "{stdout}"
    );
    assert!(
        stdout.contains("uploaded raw/traces/dt=2026-07-05/run-1.jsonl"),
        "{stdout}"
    );
    assert!(
        stdout.contains("uploaded 2, skipped 0, FAILED 1"),
        "{stdout}"
    );

    // And the store agrees: survivors present, doomed absent.
    assert!(store.object(doomed).is_none());
    assert_eq!(store.objects().len(), 2);
}

/// [E] S9 — bounded concurrency: over a many-file lake, the fake's gauge
/// (incremented/decremented INSIDE put) never exceeds MAX_IN_FLIGHT (4).
/// The lower bound guards against a vacuous pass: the fake's put dwells
/// across yield points, so if uploads genuinely overlap the high-water
/// mark must clear 1 — a serial (bugged) runner would be caught here.
#[tokio::test]
async fn s9_gauge_proves_the_concurrency_bound() {
    let dir = many_file_lake(24);
    let store = Arc::new(FakeStore::new());

    let the_plan = plan(dir.path(), "raw/", &[]).unwrap();
    assert_eq!(the_plan.upload.len(), 24);
    let outcome = run(Arc::clone(&store), the_plan).await.unwrap();

    assert_eq!(outcome.uploaded.len(), 24);
    let high_water = store.high_water();
    assert!(
        high_water <= MAX_IN_FLIGHT as u64,
        "S9 bound violated: {high_water} > {MAX_IN_FLIGHT}"
    );
    assert!(
        high_water >= 2,
        "gauge never saw overlap (high water {high_water}) — measurement vacuous"
    );
}
