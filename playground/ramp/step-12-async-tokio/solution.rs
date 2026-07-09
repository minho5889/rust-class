// Step 12 — async/await + tokio: the FIXED version.
// (To run me: save yours first — `cp src/main.rs my-solution.rs` — then
// `cp solution.rs src/main.rs && cargo run`.)
// The two errors and the warning you were asked to provoke are preserved
// below, because they were the point.
//
// --- What you typed first (does NOT compile) ── E0308 ──────────────────
//
//     fn main() {
//         let slept: u64 = nap("A", 200);
//     }
//
//     error[E0308]: mismatched types
//       expected `u64`, found future
//
// Calling an async fn runs NOTHING. It builds and returns a paused state
// machine — a future — that merely DESCRIBES the nap. (Older toolchains
// spell it "found opaque type `impl Future<Output = u64>`" — same fact.)
//
// --- Obeying the arrow (still does NOT compile) ── E0728 ───────────────
//
//     let slept: u64 = nap("A", 200).await;   // in plain fn main
//
//     error[E0728]: `await` is only allowed inside `async` functions
//       and blocks
//        |
//        | fn main() {
//        | --------- this is not `async`
//
// `.await` means "yield here, let others run" — but yield to WHOM? Someone
// ordinary must sit in a loop driving futures: an executor. The standard
// library ships none; #[tokio::main] below hires one.
//
// --- The silent one (compiles, does less than it looks) ── warning ─────
//
//     nap("seq-B", 200);        // bare statement, no .await
//
//     warning: unused implementer of `Future` that must be used
//       = note: futures do nothing unless you `.await` or poll them
//
// B never sleeps, never prints. A line that looks like work and performs
// none — the sneakiest bug shape async Rust has. Read warnings like errors.
// ------------------------------------------------------------------------

use std::time::{Duration, Instant};
use tokio::time::sleep;

// teach: `async fn` is sugar — this REALLY returns impl Future<Output = u64>:
// a plain struct holding the fn's paused locals plus a "where was I" marker.
// No OS thread, no private stack, no GC — why one Lambda vCPU can juggle
// thousands of these where a thread-per-request model drowns in stacks.
async fn nap(label: &str, ms: u64) -> u64 {
    println!("{label}: falling asleep for {ms} ms");
    // teach: THE await. tokio's sleep doesn't block the thread — it parks
    // this state machine, tells the runtime "wake me in {ms}", and the
    // thread is free to run other futures meanwhile. (std::thread::sleep
    // here would hold the whole worker thread hostage — the classic
    // blocking-in-async bug.)
    sleep(Duration::from_millis(ms)).await;
    println!("{label}: awake");
    ms
}

// teach: the hired executor — expands to a plain fn main that builds a tokio
// Runtime (rt-multi-thread: a work-stealing pool) and drives this async body.
// teach: main returns Result (step 7's shape, new error type): awaiting a
// JoinHandle yields Result<T, JoinError> because a spawned task can panic —
// `?` gives that verdict somewhere to go. No unwrap(), per house rules.
#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    // --- sequential: .await one, THEN .await the other ---------------------
    let start = Instant::now();
    let a = nap("seq-A", 200).await; // teach: yields — but nothing else is scheduled, so we just wait
    let b = nap("seq-B", 200).await; // teach: only NOW does B's future even get built
    println!(
        "sequential: {}+{} ms of sleep took {} ms of wall clock",
        a,
        b,
        start.elapsed().as_millis()
    ); // ≈ 401 ms

    // --- concurrent: spawn BOTH, then await both ---------------------------
    let start = Instant::now(); // teach: shadowing the old `start` — idiomatic for "new measurement"
    // teach: tokio::spawn hands the future to the runtime and returns a
    // JoinHandle IMMEDIATELY — the nap is already running. Spawn both first:
    // `spawn A, await A, spawn B, await B` would re-serialize to 400 ms.
    let handle_a = tokio::spawn(nap("spawn-A", 200));
    let handle_b = tokio::spawn(nap("spawn-B", 200));
    // teach: a JoinHandle is itself a future; awaiting it = "wait for that
    // task, give me its return value" — the ? peels Result<u64, JoinError>.
    let a = handle_a.await?;
    let b = handle_b.await?;
    println!(
        "spawned:    {}+{} ms of sleep took {} ms of wall clock",
        a,
        b,
        start.elapsed().as_millis()
    ); // ≈ 201 ms — 400 ms of sleep overlapped. That line is the whole lesson.

    // --- select!: race two futures; first finisher wins --------------------
    // teach: select! polls both until ONE completes, then CANCELS the other
    // by dropping it mid-flight — the tortoise's remaining 30 ms never
    // happen. "Next request OR ctrl-c" — relay's graceful shutdown (spec
    // 005, sitting N) is this select in work clothes.
    tokio::select! {
        _ = sleep(Duration::from_millis(50)) => println!("hare wins"),
        _ = sleep(Duration::from_millis(80)) => println!("tortoise wins"),
    }

    Ok(())
}
