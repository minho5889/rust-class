# Step 13 — Channels + `Send`/`Sync`: don't share the file, send the data

**Concept:** an `mpsc` channel *moves ownership* of each message from many producer tasks to the one consumer that owns the sink — nothing is ever shared, so nothing can tear — and `Send` is the compile-time license a type needs to cross between threads at all.
**You can already:** explain a move and read E0382 in your sleep (step 2), say why an owned `String` is one heap allocation with exactly one owner (step 4), and spawn tasks and await their `JoinHandle`s (step 12).
**After this step you can:** wire three producers to one consumer through a bounded `tokio::sync::mpsc`, predict the exact moment the consumer's loop ends, and read a "cannot be sent between threads" error and fix it with `Arc` — which is spec 005's relay architecture, at toy scale.

## The exercise

Runs in this folder, cargo project like step 12 — same mechanics (`cargo run` here; your file is `src/main.rs`; `solution.rs` waits at the root, save-first ritual applies). The plot: three spawned producers each cook up `String` events; every event travels through **one channel**; a single collector task owns a `Vec<String>` the producers can't even name. At the end, `main` prints what the collector gathered.

1. Read `Cargo.toml` and spot the one new tokio feature: `sync`, the crate slice holding the channels. Then `cargo run` the seeded main once — from here, every move ends with another `cargo run`.
2. The channel and its one consumer. In `main`: `let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(8);` — one channel, two halves: `tx` (`Sender`) puts in, `rx` (`Receiver`) takes out, `8` is the capacity, and its being *finite* matters at the end of this step. Now spawn the **collector** and keep its handle: `let collector = tokio::spawn(async move { ... });` — inside, build `let mut lines: Vec<String> = Vec::new();`, loop `while let Some(line) = rx.recv().await { lines.push(line); }`, and make `lines` the block's last expression: the task's return value, carried out through its `JoinHandle` (step 12). The `async move` swallows `rx` and the `Vec` whole — no other code in the program can so much as name them. Note what `recv().await` yields: `Option<String>`. `Some` is a message — owned, yours. `None` means "closed". Hold that thought until move 4.
3. Producers, honest first attempt: `for id in 1..=3 { ... }`, each iteration a `tokio::spawn(async move { ... })` whose body loops `for n in 1..=4`, builds `let msg = format!("producer {id}: event {n}");`, and ships it with `if tx.send(msg).await.is_err() { return; }` (`Err` means "receiver gone" — a lab producer just quits). Run. **`error[E0382]: use of moved value: `tx`` — value moved here, in previous iteration of loop.** Of course: one `Sender`, three tasks each demanding to *own* it. The `help:` line hands you the idiom: clone. `mpsc` is **m**ulti-**p**roducer *by cloning senders* — make `let tx = tx.clone();` the loop's first line, so each task moves in its own clone. While you're in there, add a breather above the send — `tokio::time::sleep(Duration::from_millis(2)).await;` — so all three producers wake at the same tempo and every round becomes a genuine three-way race.
4. Close the loop. After the producer loop: `drop(tx);`, then `let lines = collector.await?;` (step 12's move — `main` returns `Result<(), tokio::task::JoinError>`, `Ok(())` at the end), then print `lines.len()` and every line. Run **twice**. Twelve lines both times, and the order shuffles between runs (ties between producers go to whoever the scheduler woke first) — welcome to concurrency: order is up for grabs, the count never is. Now the experiment that explains the `drop`: comment it out and run. Nothing prints; the program sits forever; ctrl-c it. `recv()` returns `None` only when **every** `Sender` is gone — the three clones died with their producers, but the original was still alive in `main`... while `main` sat in `collector.await`. Deadlock: no error, no panic, no help. Put the `drop` back, and remember the shape — this exact footgun is named in relay's design (spec 005), and sitting N will ask you to predict it cold.
5. Feel the ownership transfer: right after the `send` line, add `println!("just sent: {msg}");`. **`error[E0382]: borrow of moved value: `msg``.** `send` takes the `String` *by value*: after that line the producer simply doesn't have the data — the channel does, then the collector does. Read the full claim: there is no lock anywhere in this program, and yet torn or shared data isn't merely avoided — it's *inexpressible*. Delete the println.
6. The `Send` probe. Give the producers something shared to read: above the loop, `let station = std::rc::Rc::new(String::from("goldeneye ground station"));`; inside it, `let station = Rc::clone(&station);`; and work `{station}` into the format string. Run, and read the whole verdict aloud: **error: future cannot be sent between threads safely**, with the `help:` naming the culprit — **the trait `Send` is not implemented for `Rc<String>`** — and the `note:` naming the law: `tokio::spawn` requires `F: Future + Send + 'static`. (Tokio's rendering of this carries no error code; feed the same `Rc` to `std::thread::spawn` in a two-line scratch `main` and the identical failure wears its badge: `error[E0277]: `Rc<String>` cannot be sent between threads safely`.) *Why* `Rc` lacks the license: its reference count is ordinary memory. Two threads cloning at the same instant can both read 1 and both write 2 — an increment lost, a future double-free. So `Rc` opts out of `Send`, and spawn's bound turns a data race into a compile error. The fix is two tokens: **`Arc`** — *atomically* reference counted, same API, licensed to cross. `std::rc::Rc` → `std::sync::Arc`, `Rc::clone` → `Arc::clone`; green.
7. Name what you built, out loud, because it is the next spec: **the single-writer pattern**. Producers = relay's HTTP handlers (many, concurrent). The `String` = one formatted JSONL line. The collector = relay's writer task, sole owner of the open lake files. The bounded channel = relay's bounded(256): when the writer falls behind, `send().await` makes handlers *wait* — backpressure, so overload slows callers instead of eating RAM until OOM. And `drop(tx)` → `recv()` returns `None` = relay's entire graceful-shutdown signal. When you build spec 005, you are rebuilding this file with a real file where the `Vec` is.

## Errors you should EXPECT (and want)

- **`error[E0382]: use of moved value: `tx`` — "value moved here, in previous iteration of loop"** — from move 3. What it's really saying: step 2's law, now with three owners bidding for one value — the first iteration's `async move` took `tx`, so the second has nothing to take. The `help:` line ("consider cloning the value before moving it into the closure") isn't a workaround, it's the design: a `Sender` clone is a cheap handle onto the same channel, every producer owns its own, and — the part that pays off in move 4 — **the channel counts them**.
- **`error[E0382]: borrow of moved value: `msg``** — from move 5, and it's E0382 as a *feature*. `send(msg)` moves the `String` into the channel; the producer keeps nothing to mutate, race, or tear. A GC language would happily let both sides hold a reference to that message — and the mutate-after-send heisenbug that follows is a support-ticket classic. Here the bug is unwritable.
- **`error: future cannot be sent between threads safely`** (help: **the trait `Send` is not implemented for `Rc<String>`**; note: **required by a bound in `tokio::spawn`** — `F: Future + Send + 'static`) — from move 6. What it's really saying: the multi-thread runtime may run your task on any worker and even *migrate it between awaits*, so everything the async block captures must be safe to hand across threads — `Send`. `Send` is an **auto trait**: you never implement it; the compiler derives it from what your type contains, which is exactly how the error can name the one guilty capture. Tokio's async-flavored rendering drops the code, but it's a trait-bound failure — **E0277** — and `std::thread::spawn` shows it coded, same culprit, same sentence. Its sibling in one breath: `Sync` means "`&T` is `Send`" — many threads may *look* at once. `Arc<String>` flies because `String` is `Sync` (shared *reads* are safe); an `Arc<Cell<u64>>` would still be refused — `Arc` shares, it doesn't bless.
- **The hang — no diagnostic at all.** Move 4's comment-out experiment, and step 7's count-of-4 lesson wearing async clothes: the compiler proves memory safety, not *liveness*. Keeping one forgotten `Sender` alive is type-correct and deadlocks forever. Past the type system's border, correctness is on your design — which is why relay's design document names this exact footgun before any code exists.

## Checkpoint

- `cargo run` prints `collector owns 12 lines:` on every run; the order shuffles between runs, and you can explain why both facts hold.
- With `drop(tx)` commented out you predicted the hang *before* running it, and can narrate the mechanism: three senders died with their tasks, one survived in `main`, `recv()` waits for a count of zero that never comes.
- You read the not-`Send` verdict aloud and can give the one-sentence reason `Rc` lacks the license (non-atomic refcount, racing clones) and why `Arc` has it.
- Say this out loud and mean it: *"many producers, one owner of the sink; data crosses by move; `Send` is the license to cross — and that's relay: handlers send, one writer owns the file."*
- Save what YOU wrote: `cp src/main.rs my-solution.rs` *first*, commit — `ramp: step 13 — channels + Send/Sync` — and only then, for the reference: `cp solution.rs src/main.rs && cargo run`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

Build order that keeps you sane: channel first, collector spawned second (it just waits — the channel buffers), producers third, and *last of all* the `drop(tx)` + `collector.await?`. If your program hangs before you've done move 4's experiment, you've done the experiment early: some `Sender` is still alive — almost always the original `tx` in `main`.

The clone goes *outside* the `async move` block but *inside* the `for` body: each iteration makes one clone, and the block then moves that clone. (Cloning inside the block is too late — the block would have to capture `tx` itself to do it.)

Returning the `Vec`: no `return`, no semicolon — `lines` as the block's last expression, the same "the block's value" move as step 5's match and step 8's if/else. It comes back out through `collector.await?` as the task's output.

</details>

<details><summary>Hint 2 — the shape</summary>

```rust
let (tx, mut rx) = mpsc::channel::<String>(8);

let collector = tokio::spawn(async move {
    let mut lines: Vec<String> = Vec::new();
    while let Some(line) = rx.recv().await {
        lines.push(line);
    }
    lines // the task's return value
});

let station = Arc::new(String::from("goldeneye ground station"));

for id in 1..=3 {
    let tx = tx.clone();                  // each producer owns its own Sender
    let station = Arc::clone(&station);   // ...and its own Arc handle
    tokio::spawn(async move {
        for n in 1..=4 {
            sleep(Duration::from_millis(2)).await;
            let msg = format!("producer {id} via {station}: event {n}");
            if tx.send(msg).await.is_err() {
                return;
            }
        }
    });
}
drop(tx); // THE line — the fourth and final Sender

let lines = collector.await?;
```

Count the senders: one original plus three clones, four. The three clones die when their tasks end; the `drop` kills the original; at zero, `recv()` yields `None` and the collector's loop — and then the whole program — winds down. No flags, no sleeps: channel close *is* the shutdown signal.

</details>

*Stuck after honestly trying? `solution.rs` sits at the step root. Save yours (`cp src/main.rs my-solution.rs`), then `cp solution.rs src/main.rs && cargo run` — and read it line by line before rewriting it yourself from memory.*
