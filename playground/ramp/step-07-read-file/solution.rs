// Step 7 — reading a file and iterating lines: the FIXED version.
// The two failures you were asked to walk into are preserved below,
// because the failures were the point.
//
// --- What you typed first (does NOT compile) ── E0308 ──────────────────
//
//     let contents: String = std::fs::read_to_string("sample.jsonl");
//
//     error[E0308]: mismatched types
//       expected `String`, found `Result<String, std::io::Error>`
//
// Touching the filesystem can fail — missing file, bad permissions — so
// the return type says so. You asked for the success value; the library
// handed you the whole verdict. Step 6's lesson, arriving this time from
// the standard library instead of code you wrote. (Chaining `.lines()`
// straight onto the call is the same mistake dressed as E0599.)
//
// --- What the compiler could NOT catch (compiles, prints 4) ────────────
//
//     if !line.is_empty() { count += 1; }
//
// Line 3 of sample.jsonl is three spaces. To `.is_empty()`, whitespace
// is content — types can't see inside a string. `.trim()` first. This is
// the boundary of the compiler's protection; past it, correctness is on
// your tests.
// ------------------------------------------------------------------------

use std::fs;

// teach: main itself can be fallible. Declare `-> Result<(), E>` and every
// `?` inside has somewhere to send an error; on Err, Rust prints it and
// exits nonzero — exactly what a CLI should do, with no try/catch anywhere.
fn main() -> Result<(), std::io::Error> {
    // teach: read_to_string returns Result<String, io::Error>; `?` unwraps
    // the Ok or early-returns the Err from main — step 6's operator, first
    // real use. The String that comes back is ONE heap allocation owning
    // every byte of the file.
    let contents = fs::read_to_string("sample.jsonl")?;

    let mut count = 0; // teach: mutation is opt-in; only the counter gets it

    // teach: .lines() iterates &str slices that BORROW into `contents`' one
    // buffer — zero copies, zero per-line allocations. A GC language would
    // typically materialize a fresh string object for every line.
    for line in contents.lines() {
        // teach: .trim() also just borrows — a narrower &str into the same
        // buffer. The whitespace-only line 3 becomes "" and fails the test;
        // without the trim, it counts and the program prints 4.
        if !line.trim().is_empty() {
            count += 1;
        }
    }

    println!("{count} non-blank lines");

    // teach: move 6's payoff — the same computation as one iterator chain.
    // `filter` + `count` compiles down to the loop above (a "zero-cost
    // abstraction"): you pay for the expressiveness at compile time, not at
    // runtime.
    let chained = contents.lines().filter(|l| !l.trim().is_empty()).count();
    assert_eq!(count, chained); // teach: two phrasings, one answer

    Ok(()) // teach: main's success value — unit `()`, wrapped in Ok
}
