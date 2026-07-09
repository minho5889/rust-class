//! [E] the tool as the user experiences it — v0's R1–R6 cases adapted,
//! plus v1's F1/F2/F4/F5/F7/F10 cases. Runs the real binary
//! (CARGO_BIN_EXE_glake, a std-only cargo feature) over the fixtures:
//!
//! - `lake/`: 5 valid events (one with an uncomparable ts → bad-ts),
//!   1 blank line, 1 missing-actor line, across three dt= partitions;
//! - `junk.jsonl`: 1 non-JSON line + 1 valid event (the documented
//!   hand-vs-serde strictness split, visible at the CLI).

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

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

// ---- carried over from v0 (003 R1–R6), counts adapted to the v1 lake ----

#[test]
fn r1a_r1b_validate_reports_and_exits_nonzero() {
    let out = glake(&["validate", &fixture("lake")]);
    let stdout = stdout_of(&out);
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
        "R1b/F5: exit 1 when malformed exist"
    );
}

#[test]
fn r1b_validate_clean_lake_exits_zero() {
    let out = glake(&["validate", &fixture("lake/dt=2026-07-01/events.jsonl")]);
    assert_eq!(out.status.code(), Some(0), "R1b/F5: clean file exits 0");
}

#[test]
fn r2_r3a_r5_stats_counts_both_axes_recursively() {
    let out = glake(&["stats", &fixture("lake")]);
    let stdout = stdout_of(&out);
    // R3a: all three dt= partitions found; R5: the blank line counted nowhere
    assert!(
        stdout.contains("3 files · 5 events"),
        "grand total: {stdout}"
    );
    assert!(
        !stdout.contains("filtered from"),
        "F4: no filter → v0's plain header: {stdout}"
    );
    assert!(
        stdout.contains("gate.approved") && stdout.contains("3"),
        "by type: {stdout}"
    );
    assert!(
        stdout.contains("2026-07-01")
            && stdout.contains("2026-07-02")
            && stdout.contains("2026-07-03"),
        "by day: {stdout}"
    );
    assert!(
        stdout.contains("bad-ts"),
        "uncomparable ts lands in a visible bucket: {stdout}"
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
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("1 files · 2 events"),
        "single file only: {stdout}"
    );
}

#[test]
fn r6_f5_bad_path_one_stderr_line_exit2_no_panic() {
    let out = glake(&["stats", "/no/such/path"]);
    assert_eq!(out.status.code(), Some(2), "R6/F5: exit 2");
    let stderr = stderr_of(&out);
    // F5: exactly one clear line, glake-prefixed, with the path context
    // that GlakeError::Io carries.
    assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
    assert!(
        stderr.starts_with("glake: cannot read /no/such/path:"),
        "R6/F6: clear prefixed stderr: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "R6: no panic");
}

#[test]
fn r6_f10_usage_on_bad_args_exits_2() {
    // clap's default error exit code is 2 — same contract as v0 (F10).
    for args in [
        &[][..],
        &["frobnicate", "x"][..],
        &["stats"][..],
        &["stats", "lake", "--parser", "nope"][..],
        &["stats", "lake", "--frobnicate"][..],
    ] {
        let out = glake(args);
        assert_eq!(out.status.code(), Some(2), "usage exits 2 for {args:?}");
        // clap prints a full usage block for structural errors and an
        // `error: invalid value … [possible values: …]` hint for bad
        // values — either way the user is told what's legal.
        let stderr = stderr_of(&out).to_lowercase();
        assert!(
            stderr.contains("usage") || stderr.contains("possible values"),
            "clap explains bad usage for {args:?}: {stderr}"
        );
    }
}

// ---- new in v1 ----

/// [E] F10: help is auto-generated and exits 0.
#[test]
fn f10_help_is_generated_and_exits_zero() {
    let out = glake(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = stdout_of(&out);
    assert!(stdout.contains("Usage"), "help text: {stdout}");
    assert!(
        stdout.contains("validate") && stdout.contains("stats"),
        "both commands listed: {stdout}"
    );
    let stats_help = stdout_of(&glake(&["stats", "--help"]));
    for flag in ["--type", "--since", "--parser"] {
        assert!(stats_help.contains(flag), "{flag} in stats help");
    }
}

/// [E] F1 + F4: --type keeps exact matches only and shows both numbers.
#[test]
fn f1_f4_type_filter_and_both_numbers() {
    let out = glake(&["stats", &fixture("lake"), "--type", "gate.approved"]);
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("3 files · 3 events (filtered from 5)"),
        "F4 both numbers: {stdout}"
    );
    assert!(
        !stdout.contains("spec.doc_written"),
        "other kinds filtered out: {stdout}"
    );
    assert_eq!(out.status.code(), Some(0));

    // exact match: a prefix is NOT a match
    let out = glake(&["stats", &fixture("lake"), "--type", "gate"]);
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("3 files · 0 events (filtered from 5)"),
        "F1 exactness: {stdout}"
    );
}

/// [E] F2 + F4: --since keeps day ≥ date, excludes bad-ts with a note.
#[test]
fn f2_f4_since_filter_excludes_and_notes_bad_ts() {
    let out = glake(&["stats", &fixture("lake"), "--since", "2026-07-02"]);
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("3 files · 2 events (filtered from 5)"),
        "07-02 and 07-03 kept: {stdout}"
    );
    assert!(
        stdout.contains("(1 bad-ts event(s) excluded by --since)"),
        "F2 one-line note: {stdout}"
    );
    assert!(
        !stdout.contains("2026-07-01"),
        "old day filtered out: {stdout}"
    );
    assert_eq!(out.status.code(), Some(0));

    // Without --since the same lake keeps its bad-ts event and no note.
    let stdout = stdout_of(&glake(&["stats", &fixture("lake")]));
    assert!(stdout.contains("bad-ts"), "bucket visible: {stdout}");
    assert!(!stdout.contains("excluded by --since"), "no note: {stdout}");
}

/// [E] F1+F2 combined: both filters compose; the note counts only what
/// the since-rule excluded (type-filtered bad-ts events don't inflate it).
#[test]
fn f1_f2_filters_compose() {
    let out = glake(&[
        "stats",
        &fixture("lake"),
        "--type",
        "gate.approved",
        "--since",
        "2026-07-03",
    ]);
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("3 files · 1 events (filtered from 5)"),
        "one gate.approved on/after 07-03: {stdout}"
    );
    assert!(
        !stdout.contains("excluded by --since"),
        "the bad-ts event is session.start — excluded by --type, so no note: {stdout}"
    );
}

/// [E] F5: a bad --since VALUE is a usage error: one glake-prefixed stderr
/// line, exit 2 (GlakeError::Usage through the F5 funnel).
#[test]
fn f5_bad_since_value_is_usage_error() {
    for bad in ["2026/07/02", "yesterday", "2026-7-2"] {
        let out = glake(&["stats", &fixture("lake"), "--since", bad]);
        assert_eq!(out.status.code(), Some(2), "exit 2 for --since {bad}");
        let stderr = stderr_of(&out);
        assert!(
            stderr.starts_with("glake: --since expects YYYY-MM-DD"),
            "clear message for {bad}: {stderr}"
        );
        assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
    }
}

/// [E] F1 (as amended): filters are stats-only — validate rejects them
/// with a usage error, exit 2.
#[test]
fn f1_validate_rejects_filter_flags() {
    for args in [
        &["validate", "lake", "--type", "x"][..],
        &["validate", "lake", "--since", "2026-07-02"][..],
    ] {
        let out = glake(args);
        assert_eq!(out.status.code(), Some(2), "filters rejected: {args:?}");
        let stderr = stderr_of(&out).to_lowercase();
        assert!(stderr.contains("usage"), "usage shown for {args:?}");
    }
}

/// [E] F7: --parser selects the backend; on the (well-formed-enough) lake
/// both backends print byte-identical stats — filtered and unfiltered.
#[test]
fn f7_serde_backend_matches_hand_on_the_lake() {
    let lake = fixture("lake");
    for extra in [
        &[][..],
        &["--type", "gate.approved"][..],
        &["--since", "2026-07-02"][..],
    ] {
        let mut hand_args = vec!["stats", lake.as_str()];
        hand_args.extend_from_slice(extra);
        let mut serde_args = hand_args.clone();
        serde_args.extend_from_slice(&["--parser", "serde"]);

        let hand = glake(&hand_args);
        let serde = glake(&serde_args);
        assert_eq!(hand.status.code(), serde.status.code(), "{extra:?}");
        assert_eq!(
            stdout_of(&hand),
            stdout_of(&serde),
            "identical stats for {extra:?}"
        );
    }
    // validate agrees on the lake too (its malformed line is valid JSON,
    // just missing a key — both backends see the same thing).
    let hand = glake(&["validate", &fixture("lake")]);
    let serde = glake(&["validate", &fixture("lake"), "--parser", "serde"]);
    assert_eq!(stdout_of(&hand), stdout_of(&serde));
    assert_eq!(serde.status.code(), Some(1));
}

/// [E] F7/F8 divergence class (1), pinned at the CLI: on a non-JSON line
/// the scanner reports a missing key, serde reports unparseable — both
/// count it, both exit 1, and stats still agree on the numbers.
#[test]
fn f7_strictness_split_is_visible_in_validate() {
    let hand = glake(&["validate", &fixture("junk.jsonl")]);
    assert_eq!(hand.status.code(), Some(1));
    assert!(
        stdout_of(&hand).contains("missing key \"event_id\""),
        "hand is lenient: {}",
        stdout_of(&hand)
    );

    let serde = glake(&["validate", &fixture("junk.jsonl"), "--parser", "serde"]);
    assert_eq!(serde.status.code(), Some(1));
    assert!(
        stdout_of(&serde).contains("not a JSON object"),
        "serde is strict: {}",
        stdout_of(&serde)
    );

    // ...but the STATS agree: both fold the junk into malformed.
    let hand = stdout_of(&glake(&["stats", &fixture("junk.jsonl")]));
    let serde = stdout_of(&glake(&[
        "stats",
        &fixture("junk.jsonl"),
        "--parser",
        "serde",
    ]));
    assert_eq!(hand, serde);
    assert!(hand.contains("1 files · 1 events"), "{hand}");
    assert!(hand.contains("1 malformed"), "{hand}");
}
