// Step 3 — borrowing: the FIXED version.
// Both erroring variants you were asked to type first are preserved below,
// because the errors were the point.
//
// --- What you typed first (does NOT compile) ── E0308 ──────────────────
//
//     let word = String::from("goldeneye");
//     let loud = shout(word);   // error[E0308]: mismatched types
//                               //   expected `&String`, found `String`
//
// The signature says "I borrow"; passing `word` plain offers ownership.
// Different contracts, different types. help: suggests `shout(&word)`.
//
// --- The point-7 experiment (does NOT compile) ── E0502 ────────────────
//
//     let peek = &word;         // shared borrow starts...
//     exclaim(&mut word);       // error[E0502]: cannot borrow `word` as
//                               //   mutable because it is also borrowed
//                               //   as immutable
//     println!("{peek}");       // ...because it's still USED down here
//
// A writer could reallocate the heap buffer while `peek` points into it.
// Many readers XOR one writer — the borrow-checker's whole job.
// ------------------------------------------------------------------------

fn shout(s: &String) -> String {
    // teach: `&String` means shout only LOOKS at the caller's string — no move,
    // no copy of the heap bytes; the reference is just an address on the stack.
    s.to_uppercase() // teach: builds and returns a brand-new String; the borrowed one is untouched
} // teach: the loan ends here — ownership never left main
// teach: (idiomatic Rust would take `&str` instead of `&String` — that upgrade
// comes in a later step; the borrowing story is identical.)

fn exclaim(s: &mut String) {
    // teach: `&mut` is an EXCLUSIVE loan — while it lives, no other reference
    // to this string may exist, so writing through it is provably safe.
    s.push_str("!"); // teach: mutates the caller's string in place — that's why nothing is returned
}

fn main() {
    let mut word = String::from("goldeneye"); // teach: `mut` opts in — without it, &mut below is E0596

    let loud = shout(&word); // teach: the `&` here is the whole E0308 fix — lend, don't give
    println!("{loud}");
    println!("{word}"); // teach: in step 2 this print was E0382; now it's fine — main owned `word` the whole time

    exclaim(&mut word); // teach: exclusive write loan for exactly this call; it ends at the `)`
    println!("{word}"); // teach: prints "goldeneye!" — mutated in place, nothing returned, nothing reallocated by us
}
