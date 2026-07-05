//! R14b: a release (or bench) build with the `memlens` feature enabled must
//! FAIL to compile — the lens is a teaching instrument, and its overhead must
//! never contaminate benchmarks or deployed binaries.
//!
//! Deviation from tasks.md 1.1.2 (noted per the change protocol): the plan
//! said `trybuild`, but trybuild always compiles its cases in the debug
//! profile and cannot exercise a release build, which is the whole point
//! here. So this test invokes cargo itself and asserts the build dies with
//! the guard's message.

use std::process::Command;

#[test]
fn release_build_with_memlens_feature_fails_to_compile() {
    // A separate target dir avoids fighting the outer `cargo test` build
    // for the workspace target/ lock.
    let target = std::env::temp_dir().join("memlens-guard-test-target");

    let out = Command::new(env!("CARGO"))
        .args([
            "build",
            "-p",
            "memlens",
            "--features",
            "memlens",
            "--release",
        ])
        .env("CARGO_TARGET_DIR", &target)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to spawn cargo");

    assert!(
        !out.status.success(),
        "R14b violated: a release build with the memlens feature compiled successfully"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("teaching instrument"),
        "build failed, but not because of the R14b guard. stderr:\n{stderr}"
    );
}
