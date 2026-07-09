# The ramp — your first Rust, one concept at a time

Throwaway exercises (no spec — this is `playground/`). Each **step** isolates
one concept, small enough for a single sitting. **The whole course is authored
ahead**: every step below has its own folder with a worksheet (`README.md`) and
a validated answer key (`solution.rs` — don't peek until you've honestly tried).
Your own code goes in `my-solution.rs`, committed one step at a time
(`ramp: step N — <concept>`).

The ramp builds *toward* `glake` (spec 003) — every concept here is one you'll
use when we build the real tool. After step 8, continue in
`specs/003-rust-bedrock/sittings/` (six guided sittings, A→F).

| Step | Folder | Concept | You'll be able to… |
|---|---|---|---|
| 1 | [step-01-hello](step-01-hello/) | `fn main`, `println!`, `let` | run your first program; store and print a value |
| 2 | [step-02-ownership-move](step-02-ownership-move/) | ownership & **move** | explain why a value can't be used after it's moved |
| 3 | [step-03-borrowing](step-03-borrowing/) | **borrowing** (`&`, `&mut`) | pass data without giving it away |
| 4 | [step-04-str-vs-string](step-04-str-vs-string/) | `&str` vs `String` | tell a borrowed slice from an owned string |
| 5 | [step-05-enum-match](step-05-enum-match/) | `enum` + `match` | model "one of several kinds" and handle every case |
| 6 | [step-06-result](step-06-result/) | `Result` + `?` | write code that can fail without crashing |
| 7 | [step-07-read-file](step-07-read-file/) | files, lines, `for` | read real data from disk |
| 8 | [step-08-lifetimes-lite](step-08-lifetimes-lite/) | lifetimes-lite (`<'a>`) | say "this slice borrows from that input" — enough for glake's scanner |

Steps 1–6 run at [play.rust-lang.org](https://play.rust-lang.org) (zero install);
steps 7–8 run in their folders with `rustc`/`cargo`. When 1–8 feel solid, open
[sitting A](../../specs/003-rust-bedrock/sittings/) and start building glake.

Status: **step 1 ready — start whenever.**
