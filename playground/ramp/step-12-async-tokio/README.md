# Step 12 — Async/await + tokio: `.await` says "yield here, let others run"

**Concept:** calling an `async fn` runs nothing — it builds a paused state machine (a *future*); `.await` says "start it, and while it waits, yield my place so other work can run"; tokio is the runtime that owns the schedule.
**You can already:** carry a `Result` out of `main` with `?` (steps 6–7), move values into closures and chain adapters (step 11), and read a compiler error top to bottom before touching code (every step since 2) — and everything you've run so far did exactly one thing at a time.
**After this step you can:** stand up an async `main` with `#[tokio::main]`, prove with a wall clock that two spawned sleeps overlap while two sequential `.await`s don't, and race two futures with `tokio::select!` — knowing the loser gets cancelled.

## The exercise

Runs **in this folder** — and it's a **cargo project**, the ramp's first. Two reasons, both lessons. Async needs an *engine*: `async`/`.await` are language syntax, but the standard library ships no runtime to drive futures — you pick one as a dependency, and ours is **tokio** (relay's, and Rust-on-Lambda's). And dependencies are why cargo exists: `rustc` compiles one file you wrote; cargo fetches tokio, builds its whole tree, then builds yours against it. So from here the loop is `cargo run` **from this folder** instead of `rustc file.rs` (check with `cargo --version`). Your code lives in **`src/main.rs`** — seeded, compiles right now — and the answer key waits at the step root as `solution.rs` (new mechanic: it *replaces* `src/main.rs` when you run it — see the checkpoint's save-first ritual before you even think about peeking).

1. Read `Cargo.toml` top to bottom — it's commented for exactly this reading. Three things to notice: tokio is pulled with only three **features** (opt-in slices of a crate — pay only for what you pull; when `aws-sdk-*` arrives, this habit is binary-size money); the **edition** now lives in the manifest instead of on your command line; and the empty **`[workspace]`** table keeps this folder standalone (without it, cargo walks up, finds the repo's `crates/` workspace at the root, and refuses to build a member it never heard of). Then `cargo run`: watch cargo compile tokio and a few support crates *once*, then your seeded three-line `main` prints. Two new artifacts appeared: `Cargo.lock` (the exact versions cargo resolved — commit it with your step) and `target/` (build output, already gitignored).
2. Above `main`, write `async fn nap(label: &str, ms: u64) -> u64`: print `"{label}: falling asleep for {ms} ms"`, then `tokio::time::sleep(Duration::from_millis(ms)).await`, print `"{label}: awake"`, return `ms`. (You'll want `use std::time::Duration;` and `use tokio::time::sleep;` up top.) Leave `fn main()` exactly as plain as the seed made it, and inside it write `let slept: u64 = nap("A", 200);`. Run. **`error[E0308]`: expected `u64`, found future.** Read that twice: you called a function and got neither a nap nor a number. The call *built a value that describes the nap* — a future — and handed you that, unstarted.
3. Do the obvious fix in place: put `.await` on the call, `main` still plain. Run. **`error[E0728]`: `await` is only allowed inside `async` functions and blocks** — and the arrow points at `fn main()` itself: *this is not `async`*. Think about what `.await` claims — "yield here, let others run." Yield *to whom*? Nothing is scheduling. Two errors, one sentence: futures are inert values, and something must drive them.
4. Hire the driver: `#[tokio::main]` above, `async fn main()`. (The macro expands to a plain `fn main` that builds a tokio `Runtime` and runs your async body on it — no magic, just code you didn't have to write.) Now measure: `let start = std::time::Instant::now();`, await `nap("seq-A", 200)`, await `nap("seq-B", 200)`, then print `start.elapsed().as_millis()`. Run: A sleeps and wakes, *then* B sleeps — **≈400 ms**. Each `.await` dutifully yielded; you'd just given the runtime nothing else to run.
5. Sabotage on purpose: remove seq-B's `.await` and leave the bare call as a statement. It *compiles* — with a warning worth framing: **unused implementer of `Future` that must be used — futures do nothing unless you `.await` or poll them**. Look at the output: B never sleeps, never prints, elapsed ≈200 ms. In every language you've used, calling a function runs it; here it files a to-do. Put the `.await` back.
6. The payoff. Below the sequential block, start a fresh `Instant`, then `let handle_a = tokio::spawn(nap("spawn-A", 200));` and likewise `handle_b` — **both spawns before any await**. `tokio::spawn` hands the future to the runtime and returns a `JoinHandle` *immediately*; the nap is already running. A `JoinHandle` is itself a future: `handle_a.await` waits for the task and yields its return value — wrapped in a `Result`, because a spawned task can panic. You know this move: give `main` the return type `Result<(), tokio::task::JoinError>`, put `?` on both awaits, end with `Ok(())` (step 7's shape, new error type — and no `unwrap()`, per house rules). Print both results and the elapsed time. Run: both fall asleep at once, and 400 ms of sleep costs **≈200 ms of wall clock**. That printed line is the whole step.
7. One more tool — race two futures:
   ```rust
   tokio::select! {
       _ = sleep(Duration::from_millis(50)) => println!("hare wins"),
       _ = sleep(Duration::from_millis(80)) => println!("tortoise wins"),
   }
   ```
   `select!` drives both until the first finishes, then **cancels** the other — the tortoise is dropped mid-sleep; its remaining 30 ms never happen. Swap the numbers, watch the winner flip, swap back. Real services live on this move: "next request OR ctrl-c, whichever first" — relay's graceful shutdown (spec 005, sitting N) is this select in work clothes.

## Errors you should EXPECT (and want)

- **`error[E0308]: mismatched types` — expected `u64`, found future** — the main event, from move 2. What it's really saying: `async fn nap(..) -> u64` is sugar for `fn nap(..) -> impl Future<Output = u64>` — calling it runs **none** of the body; it constructs a state machine that *would* produce a `u64` if someone drove it. (Older toolchains spell the same fact `found opaque type impl Future<Output = u64>`.) And that machine is cheap by design: a plain struct holding the function's paused locals plus a "where was I" marker — no OS thread, no private stack, no GC keeping it alive. That's the memory story behind async Rust on AWS: one Lambda vCPU can hold thousands of these paused structs where a thread-per-request design drowns in stacks.
- **`error[E0728]: `await` is only allowed inside `async` functions and blocks`** — from move 3, pointing at your non-async `main`. What it's really saying: `.await` compiles into "suspend this state machine and return control to the caller" — so it only makes sense *inside* a state machine. Plain `fn main` isn't one, and nothing above it is scheduling. The chain has to bottom out somewhere: someone ordinary must sit in a loop driving futures — an **executor**. `#[tokio::main]` is exactly that someone.
- **`warning: unused implementer of `Future` that must be used`** + note **`futures do nothing unless you `.await` or poll them`** — from move 5, and the sneakiest shape here: it *compiles*. A statement that looks like work and performs none. In a service this is the fire-and-forgot bug — the response never sent, the cleanup never run — which is why futures are `#[must_use]` and why our clippy gate treats warnings as errors. Read your warnings like errors; this one is.

## Checkpoint

- `cargo run` prints the sequential line at ≈400 ms and the spawned line at ≈200 ms (401/201 on a quiet machine), and you can say where the missing 200 ms went — both tasks were spawned *before* either was awaited, so the sleeps overlapped.
- You provoked E0308, then E0728, then the unused-future warning, in that order, and can explain each in one sentence.
- The hare wins your race, and you know exactly what happened to the tortoise — dropped, i.e. cancelled, not "finished quietly somewhere."
- Say this out loud and mean it: *"calling an async fn builds a paused state machine; `.await` starts it and yields my place; tokio decides who runs next."*
- Save what YOU wrote — **new mechanic, order matters**: `cp src/main.rs my-solution.rs` *first*, then commit — `ramp: step 12 — async/await + tokio`. Only after that, if you want the reference running: `cp solution.rs src/main.rs && cargo run` — it *overwrites* `src/main.rs`, which is why you saved yours first.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

The move-6 choreography is where this step is usually fumbled: `spawn A, await A, spawn B, await B` re-serializes everything back to ≈400 ms, because B isn't even created until A has fully finished. Both spawns first, then both awaits.

What goes inside `tokio::spawn(...)` is the **un-awaited call** — `tokio::spawn(nap("spawn-A", 200))` — you're handing the runtime the to-do item, not the result. (Accidentally `.await` inside the parentheses and spawn will complain that `u64` is not a future — a very honest error.)

Timing: `let start = std::time::Instant::now();` before, `start.elapsed().as_millis()` after. Two separate `start`s, one per block — shadowing with a second `let start` is fine and idiomatic.

</details>

<details><summary>Hint 2 — the shape</summary>

```rust
#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    let start = Instant::now();
    let a = nap("seq-A", 200).await;
    let b = nap("seq-B", 200).await;
    println!("sequential: ... {} ms", start.elapsed().as_millis());

    let start = Instant::now();
    let handle_a = tokio::spawn(nap("spawn-A", 200)); // no .await — already running
    let handle_b = tokio::spawn(nap("spawn-B", 200)); // both handles exist NOW
    let a = handle_a.await?; // ? unwraps Result<u64, JoinError> — step 7's move
    let b = handle_b.await?;
    println!("spawned: ... {} ms", start.elapsed().as_millis());

    // move 7's select! goes here
    Ok(())
}
```

Read the two blocks as one contrast: same `nap`, same 400 ms of sleeping — the only difference is *when the futures start*. Awaiting starts one at a time; spawning starts both, and the awaits merely collect.

</details>

*Stuck after honestly trying? `solution.rs` sits at the step root. Save yours (`cp src/main.rs my-solution.rs`), then `cp solution.rs src/main.rs && cargo run` — and read it line by line before rewriting it yourself from memory.*
