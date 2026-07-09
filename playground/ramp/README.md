# The ramp — your first Rust, one concept at a time

Throwaway exercises (no spec — this is `playground/`). Each **step** isolates
one concept, small enough for a single sitting. **The whole course is authored
ahead**: every step below has its own folder with a worksheet (`README.md`) and
a validated answer key (`solution.rs` — don't peek until you've honestly tried).
Your own code goes in `my-solution.rs`, committed one step at a time
(`ramp: step N — <concept>`).

The ramp builds *toward* the real tools — every concept here is one you'll use
when we build them. The steps come in waves, interleaved with the specs'
guided sittings:

- **Steps 1–8**, then `specs/003-rust-bedrock/sittings/` (glake v0, A→F)
- **Steps 9–11**, then `specs/004-glake-traits/sittings/` (glake v1, G→J)
- **Steps 12–13**, then `specs/005-async-relay/sittings/` (relay, K→N)

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
| 9 | [step-09-traits](step-09-traits/) | **traits** | define a contract and implement it twice |
| 10 | [step-10-generics-vs-dyn](step-10-generics-vs-dyn/) | generics vs `dyn Trait` | choose static or dynamic dispatch on purpose |
| 11 | [step-11-closures-adapters](step-11-closures-adapters/) | closures & iterator adapters | filter/map/collect real lake lines |
| 12 | [step-12-async-tokio](step-12-async-tokio/) | async/await + `tokio` | prove two sleeps overlap; explain what `.await` yields |
| 13 | [step-13-channels-send-sync](step-13-channels-send-sync/) | channels + `Send`/`Sync` | move data to a single owner across tasks — relay's whole architecture |

Steps 1–6 run at [play.rust-lang.org](https://play.rust-lang.org) (zero install);
steps 7–11 run in their folders with `rustc`; steps 12–13 are tiny cargo
projects (dependencies arrive — the worksheet explains the switch). When a
wave feels solid, open its sittings folder and start building.

Status: **step 1 ready — start whenever.**
