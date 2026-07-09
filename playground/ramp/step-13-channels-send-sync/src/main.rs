// Step 13 — channels + Send/Sync. This file is YOURS: the worksheet
// (../README.md) rebuilds it move by move. It compiles as-is — `cargo run`.
//
// The plot: three spawned producers cook up String events; every event
// travels through ONE channel; a single collector task owns a Vec the
// producers can't even name. Move 2 starts the wiring.

#[tokio::main]
async fn main() {
    println!("step 13: runtime up — build the pipeline (move 2 onward)");
}
