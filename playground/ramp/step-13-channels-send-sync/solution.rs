// Step 13 — channels + Send/Sync: the FIXED version.
// (To run me: save yours first — `cp src/main.rs my-solution.rs` — then
// `cp solution.rs src/main.rs && cargo run`.)
// The errors (and one hang) you were asked to provoke are preserved below,
// because they were the point.
//
// --- Honest first producers (does NOT compile) ── E0382 ────────────────
//
//     for id in 1..=3 {
//         tokio::spawn(async move { ... tx.send(msg).await ... });
//     }
//
//     error[E0382]: use of moved value: `tx`
//       value moved here, in previous iteration of loop
//       help: consider cloning the value before moving it into the closure
//
// One Sender, three tasks each demanding to OWN it — impossible, says the
// checker, correctly. mpsc is multi-producer BY CLONING: every producer
// gets its own Sender clone, and the channel counts them.
//
// --- Using a message after sending it ── E0382 again ───────────────────
//
//     tx.send(msg).await ...;
//     println!("just sent: {msg}");
//
//     error[E0382]: borrow of moved value: `msg`
//
// send() takes the String BY VALUE. After the send, this task simply does
// not have the data — the collector owns it. Nothing shared, nothing tears.
//
// --- Sharing an Rc across tasks (does NOT compile) ── Send, felt ───────
//
//     let station = Rc::new(String::from("..."));     // instead of Arc
//
//     error: future cannot be sent between threads safely
//       help: within `{async block ...}`, the trait `Send` is not
//             implemented for `Rc<String>`
//       note: required by a bound in `tokio::spawn`:
//             F: Future + Send + 'static
//
// (Tokio's rendering carries no code; hand the same Rc to
// std::thread::spawn and it wears its badge — error[E0277]: `Rc<String>`
// cannot be sent between threads safely.) Rc's refcount is plain memory:
// two threads cloning at the same instant can both read 1 and both write
// 2 — an increment lost, a double-free later. So Rc opts out of Send, and
// spawn's bound turns the data race into a compile error. Arc = atomic
// refcount, same API, licensed to cross. Two tokens fix it.
//
// --- The one the compiler CANNOT catch (compiles, hangs forever) ───────
//
//     // drop(tx);
//
// recv() returns None only when EVERY Sender is gone. Keep the original tx
// alive in main and the collector waits forever for a message that never
// comes, while main waits forever on the collector. No error, no panic —
// a liveness bug, past the type system's border. This is relay's named
// footgun (spec 005 design); sitting N asks you to predict it cold.
// ------------------------------------------------------------------------

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    // teach: one channel, two halves. Bounded at 8: with 8 messages sitting
    // unread, send().await makes producers WAIT — backpressure for free.
    // relay uses exactly this, bounded at 256, so a slow disk slows the
    // handlers instead of eating RAM until OOM.
    // teach: rx is `mut` because recv() takes &mut self — the type system's
    // way of saying there is only ever ONE consumer (the "sc" in mpsc).
    let (tx, mut rx) = mpsc::channel::<String>(8);

    // teach: the single writer. This task OWNS rx and OWNS the Vec — the
    // async move swallowed both; no other code can even name them. Order
    // and integrity come from single ownership of the sink. In relay, this
    // Vec is the open lake file.
    let collector = tokio::spawn(async move {
        let mut lines: Vec<String> = Vec::new();
        // teach: Some(line) = one message, ownership included — moved from a
        // producer to here through the channel. None = closed = every Sender
        // dropped. The loop's end condition IS the shutdown signal: no
        // flags, no sleeps.
        while let Some(line) = rx.recv().await {
            lines.push(line);
        }
        lines // teach: the task's return value, carried out via its JoinHandle
    });

    // teach: Arc, not Rc — one value READ by three tasks on arbitrary
    // threads, so the refcount must be atomic; Send is the license and Rc
    // doesn't hold one (header block). Cloning an Arc copies a pointer and
    // bumps a counter — the String itself is never copied.
    let station = Arc::new(String::from("goldeneye ground station"));

    for id in 1..=3 {
        // teach: E0382's fix — each producer owns its OWN Sender clone (and
        // its own Arc handle). Clones made out here, then moved in.
        let tx = tx.clone();
        let station = Arc::clone(&station);
        tokio::spawn(async move {
            for n in 1..=4 {
                // teach: the breather — all three producers wake at the same
                // tempo, so every round is a genuine three-way race and the
                // ORDER shuffles run to run. The COUNT never does.
                sleep(Duration::from_millis(2)).await;
                let msg = format!("producer {id} via {station}: event {n}");
                // teach: send() MOVES msg — after this line the producer no
                // longer has the data (that was the second E0382). Err means
                // "collector gone"; a lab producer just quits. And the
                // .await is the backpressure: a full channel makes us wait.
                if tx.send(msg).await.is_err() {
                    return;
                }
            }
            // teach: this producer's Sender clone drops HERE, at task end —
            // one of the four the channel is counting down.
        });
    }
    // teach: THE line (comment it out to feel the hang). main's original tx
    // is the fourth Sender; drop it and the count can actually reach zero
    // once the producers finish. Channel close is the drain signal.
    drop(tx);

    // teach: step 12's move — await the JoinHandle, ? the JoinError. This
    // implicitly waits for the producers too: the collector can only finish
    // after all senders drop.
    let lines = collector.await?;
    println!("collector owns {} lines:", lines.len());
    for line in &lines {
        println!("  {line}");
    }
    Ok(())
}
