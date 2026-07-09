# The ramp — your first Rust, one concept at a time

Throwaway exercises (no spec — this is `playground/`). Each **step** isolates
one concept, small enough for a single sitting. You write the code; Claude
reviews and explains. We only move to the next step once the current one clicks.

The ramp builds *toward* `glake` (spec 003) — every concept here is one you'll
use when we build the real tool, so nothing is wasted.

| Step | Concept | You'll be able to… |
|---|---|---|
| 1 | `fn main`, `println!`, `let`, method call | run your first program; store and print a value |
| 2 | ownership & **move** | explain why a value can't be used after it's moved |
| 3 | **borrowing** (`&`) | pass data without giving it away |
| 4 | `&str` vs `String` | tell a borrowed string slice from an owned one |
| 5 | `enum` + `match` | model "one of several kinds" and handle every case |
| 6 | `Result` + `?` | write code that can fail without crashing |
| 7 | read a file, iterate lines | read real data from disk |
| 8 | lifetimes-lite (`<'a>`) | say "this returned slice borrows from that input" — just enough for glake's scanner |

When steps 1–8 feel solid, we start `glake` for real — dead-simple first
(count the lines in one file), then one capability per sitting.

Status: **step 1 in progress.**
