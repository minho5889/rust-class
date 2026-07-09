// Step 8 — lifetimes-lite: the FIXED version.
// The erroring variant you were asked to provoke is preserved below,
// because the error was the point.
//
// --- What you typed to break it (does NOT compile) ----------------------
//
//     fn longer(a: &str, b: &str) -> &str {
//         //                         ^ error[E0106]: missing lifetime
//         //                                        specifier
//         //   help: this function's return type contains a borrowed value,
//         //         but the signature does not say whether it is borrowed
//         //         from `a` or `b`
//         //   help: consider introducing a named lifetime parameter
//         if a.len() >= b.len() { a } else { b }
//     }
//
// The body is fine — the SIGNATURE is the problem. When checking your
// callers, the compiler consults the signature alone; it never re-reads the
// body at each call site. With one input reference there is only one thing
// the output could borrow from, so the compiler fills the lifetime in
// (elision). With two inputs it is genuinely ambiguous, so YOU must say it.
// ------------------------------------------------------------------------

// teach: this is exactly what you typed in move 3 — and exactly what elision
// writes for you when you type the bare `fn first_word(s: &str) -> &str`.
// One input reference, so the shared name 'a is forced: the output can only
// borrow from the one thing there is to borrow from. Delete every 'a from
// this signature and it still compiles, unchanged.
fn first_word<'a>(s: &'a str) -> &'a str {
    // teach: the return value is a SLICE of s — a pointer + length into
    // s's own bytes. Nothing is copied, nothing is allocated. Lifetimes
    // exist to make exactly this zero-copy move provably safe: cheap reads
    // with no GC keeping the source alive behind your back.
    // teach: .next() yields Option<&str> (a line with no words is real);
    // .unwrap_or("") lands on the empty slice instead of panicking — no
    // unwrap() on a data path, per house rules.
    s.split_whitespace().next().unwrap_or("")
}

// teach: what does 'a promise? — the returned reference borrows from the
// inputs marked 'a, so it cannot outlive them.
//
// teach: two input references, one borrowed output — elision has no rule
// for this (see the block above), so the annotation becomes necessary.
// Declared once after the name, then referenced three times: one shared
// name is HOW the signature says "output borrows from these inputs".
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
    // teach: nothing here changed to fix E0106 — it was never a body
    // problem. And 'a compiles to NOTHING: no check, no field, no
    // nanosecond. It is a fact the borrow checker verifies, then erases.
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    // teach: an owned String on the heap (step 4) — first_word will lend us
    // a view into these exact bytes rather than copying them.
    let line = String::from("2026-07-09T10:00:00Z memlens.alloc 64");

    // teach: `word` borrows from `line`, and the (elided) 'a ties them
    // together: the compiler now refuses any code that kills `line` while
    // `word` is still in use. Insert `drop(line);` on the next line and
    // E0505 shows you the promise being enforced.
    let word = first_word(&line);
    println!("first word: {word}");

    // teach: the no-word path — .next() finds nothing, .unwrap_or("")
    // hands back the empty slice, nobody panics.
    println!("empty in -> {:?}", first_word(""));

    let a = "memlens.alloc";
    let b = "memlens.dealloc";
    // teach: both inputs live to the end of main, so a span 'a covering
    // this call trivially exists and the checker is satisfied. The
    // annotation only bites when one input dies early — which is exactly
    // the bug it exists to catch.
    let winner = longer(a, b);
    println!("longer:     {winner}");
}
