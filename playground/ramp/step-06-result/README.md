# Step 6 — `Result` and the `?` operator

**Concept:** a function that can fail says so in its return type — `Result<u32, String>` promises either `Ok(number)` or `Err(explanation)` — and `?` is one keystroke for "if that failed, stop here and pass the error up."
**You can already:** slice a string with `.get` and handle the `Option` it hands back (step 4), match on both arms of an enum and refuse to ignore the empty case (step 5), and write functions that borrow instead of take (steps 3–4).
**After this step you can:** design a function whose failures live in its signature, translate other people's error shapes into yours (`.ok_or_else`, `.map_err`), and collapse a staircase of `match`es into a flat chain of `?`s.

## The exercise

Runs at [play.rust-lang.org](https://play.rust-lang.org) — no local setup needed.

1. Type the signature and nothing else: `fn parse_day(s: &str) -> Result<u32, String>`. Before writing the body, say out loud what this promises a caller. Inputs are step-4-style log lines — `2026-07-09T10:00:00Z memlens.alloc` — and characters 8..10 are the day of the month.
2. First fallible move: slice out `s.get(8..10)`. You know from step 4 that this is an `Option<&str>` — but this time **no `.unwrap_or` fallback**. A missing day isn't something to paper over; it's an error the caller deserves to hear about. `match` the Option: in the `None` arm, `return Err(...)` with a `format!` message that names the bad input; in the `Some` arm, bind the two characters.
3. Second fallible move: call `.parse()` on that slice to turn `"09"` into a number. Two snags, both educational: `.parse()` must be *told* what type to produce (annotate the binding, or use the turbofish `::<u32>`), and it fails with a `ParseIntError` — an error type that is not your `String`. Chain `.map_err(|e| format!(...))` onto it to translate the error arm into your dialect. Make the function return the result.
4. In `main`, call `parse_day` on one good input and two bad ones — `"2026-07-09"`, `"2026-07-xx"`, `"oops"` — and `match` each result: print the day on `Ok`, print the message on `Err`. The two bad inputs should produce *different* messages, proving which fallible step caught each one.
5. Break it on purpose. In `main`, try to skip the ceremony: `let day = parse_day("2026-07-09")?;` — run it and read E0277 top to bottom. **This failure is the lesson.** Then delete the line.
6. The payoff refactor: rewrite `parse_day`'s body using `?` instead of `match`. For the parse line you already have a `Result` — just put `?` after the `.map_err(...)`. For the slice line you have an `Option`, which `?` can't early-return a `String` from — find the adapter that converts `Option` into `Result` by attaching an error value (its name starts with `ok_or`). Body should land at three lines: two ending in `?`, then the success wrapped and returned. Re-run: output identical to move 4.

## Errors you should EXPECT (and want)

- **`error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option``** — the main event, from move 5. What it's really saying: `?` is sugar for a `match` whose `Err` arm does `return Err(e)` *from the enclosing function*. Your enclosing function is `main`, which returns `()` — an early-returned `Err` has nowhere to land. Errors are values, and values need a type to live in. Two honest fixes: `match` in `main` (the caller decides what an error *means* — that's your move 4), or declare `fn main() -> Result<(), String>`, which real CLI binaries do — an `Err` then prints and exits nonzero. Note what's *absent* here: no exception tables, no stack unwinding, no hidden try/catch machinery — an `Err` is an ordinary value coming back in a register, which is part of why Rust's error path costs essentially nothing at runtime (Lambda included).
- **`error[E0308]: mismatched types`** — you'll likely meet this twice en route. If you forget `.map_err`: "expected `String`, found `ParseIntError`" — two error dialects, and your signature promised `String`; `.map_err` is the translator. If your last expression is the bare number instead of it wrapped: "expected `Result<u32, String>`, found `u32`" — success needs wrapping too, because the return type is `Result`, not `u32`.
- **`error[E0282]: type annotations needed`** — `.parse()` with nothing saying what to parse *into*. Parse is generic over its output; the annotation or turbofish is you picking.

## Checkpoint

- `"2026-07-09"` prints day `9`. `"oops"` and `"2026-07-xx"` print **different** error messages, each naming the offending input — you can point at which of the two fallible steps produced each.
- You provoked E0277 and can say, without notes, what `?` expands to: *match; on `Err`, return it from the enclosing function; on `Ok`, unwrap and continue.*
- The refactored `parse_day` body is three lines with two `?`s and behaves identically to the `match` staircase.
- Say this out loud and mean it: *"errors are values in the return type; `?` is an early return, not an exception."*
- Save what YOU wrote: paste your playground code into `my-solution.rs` next to this README, then commit — `ramp: step 6 — Result and the ? operator`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

The two failures are different animals. `.get` gives `Option<&str>` — "absent", with no story attached. `.parse` gives `Result<u32, ParseIntError>` — a story, but in the wrong language. Your promise is `Err(String)`, so each needs its own conversion: for the `Option`, search the docs for `.ok_or_else` (it *attaches* an error value, upgrading `Option` to `Result`); for the parse, you already have `.map_err`.

If the compiler says E0282 at the parse: nothing in sight tells `.parse()` its target type. `let day: u32 = ...` fixes it, and so does `.parse::<u32>()`.

For E0277 in move 5: read the error's "help" line — the compiler literally names both fixes.

</details>

<details><summary>Hint 2 — the shape</summary>

The refactored body is exactly three statements. The first is one chained line — slice, attach an error, then `?`:

```rust
let day_str = s.get(8..10).ok_or_else(|| format!("..."))?;
```

The second is the parse wearing the same shape — `.parse()`, then `.map_err(|e| ...)`, then `?`, bound with a `: u32` annotation. The third wraps the number in the success variant and returns it (last expression, no semicolon — step 5, where the `match` *was* the function body).

Each `?` reads as: "if this is `Err`, return it from `parse_day` right now; otherwise hand me the value and keep going."

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
