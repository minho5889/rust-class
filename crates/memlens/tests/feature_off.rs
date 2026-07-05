//! [E] R6 — with the `memlens` feature disabled, the lens leaves the program
//! observably unchanged: (a) running an instrumented binary produces no
//! trace output, and (b) no memlens symbols appear in a release binary.
//! Cargo-invoking test (same pattern as compile_guard).

use std::process::Command;

#[test]
fn feature_off_produces_no_trace_and_no_symbols() {
    let target = std::env::temp_dir().join("memlens-r6-target");

    // Build the demo example WITHOUT the feature, in release (guard must not
    // fire — it only bans release builds WITH the feature).
    let build = Command::new(env!("CARGO"))
        .args(["build", "-p", "memlens", "--example", "demo", "--release"])
        .env("CARGO_TARGET_DIR", &target)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("spawn cargo");
    assert!(
        build.status.success(),
        "feature-off release build must succeed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let bin = target.join("release/examples/demo");

    // (a) Run it in a scratch cwd with a trace path pointed there: no trace
    // file may appear (recording code does not exist).
    let scratch = std::env::temp_dir().join(format!("memlens-r6-run-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("mkdir scratch");
    let trace = scratch.join("should-not-exist.jsonl");
    let run = Command::new(&bin)
        .env("MEMLENS_TRACE", &trace)
        .current_dir(&scratch)
        .output()
        .expect("run demo");
    assert!(run.status.success(), "demo must run cleanly");
    assert!(
        !trace.exists() && !scratch.join("datalake").exists(),
        "feature-off run must write no trace output"
    );

    // (b) Symbol check: `nm` over the release binary must show no memlens
    // symbols (with strip=symbols there may be no symbols at all — also a
    // pass; nm's nonzero exit for 'no symbols' is fine).
    let nm = Command::new("nm").arg(&bin).output().expect("run nm");
    let symbols = String::from_utf8_lossy(&nm.stdout);
    let hits = symbols.lines().filter(|l| l.contains("memlens")).count();
    assert_eq!(hits, 0, "memlens symbols found in feature-off binary");
}
