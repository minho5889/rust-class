// Step 2 — ownership & move: the FIXED version.
// The erroring variant you were asked to type first is preserved below,
// because the error was the point.
//
// --- What you typed first (does NOT compile) ---------------------------
//
//     let city = String::from("Seoul");
//     let destination = city;      // ownership of the heap data MOVES to `destination`
//     println!("{city}");          // error[E0382]: borrow of moved value: `city`
//
// The compiler's note points at `let destination = city;` as the move,
// and its help line suggests: `let destination = city.clone();`
// -----------------------------------------------------------------------

fn main() {
    // ── Fix 1: clone ────────────────────────────────────────────────────
    // COSTS AN ALLOCATION: clone() copies the string's bytes into a brand-new
    // heap buffer, so `city` and `destination` each own their own data.
    let city = String::from("Seoul");
    let destination = city.clone(); // teach: deep copy — new heap allocation; `city` keeps ownership of the original
    println!("{city}"); // teach: legal now — `city` was never moved, only cloned from
    println!("{destination}");

    // ── Fix 2: use the new owner ────────────────────────────────────────
    // FREE — no allocation: a move copies only the (pointer, length, capacity)
    // triple on the stack. The heap bytes never move; only the deed changes hands.
    let region = String::from("us-east-1");
    let primary = region; // teach: `region` is invalid from this line on — `primary` owns the heap data now
    println!("{primary}"); // teach: print the OWNER; touching `region` here would be E0382 again

    // teach: exactly one owner at a time means Rust knows exactly when to free
    // the memory (when the owner goes out of scope) — no garbage collector,
    // no GC pauses. That's a big part of why Rust is cheap on Lambda.
}
