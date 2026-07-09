# Step 7 — Reading a file, line by line

**Concept:** `std::fs::read_to_string` pulls a whole file into one owned `String` — and because the filesystem can fail, it hands you a `Result`, not the `String` — then `.lines()` walks that one buffer as borrowed slices, no copies.
**You can already:** handle a `Result` with `match` or `?` and make a `main` that returns one (step 6), tell an owned `String` from a borrowed `&str` and know which one allocates (step 4), and borrow values instead of taking them (step 3).
**After this step you can:** write a program that touches the outside world — open a file, let the failure live in `main`'s signature, iterate its lines without allocating per line, and spot the class of bug the compiler *cannot* catch for you.

## The exercise

First step off the playground: this one runs **in this folder**, because it reads `sample.jsonl` sitting next to this README (3 JSON events + 1 blank line). You need `rustc` — check with `rustc --version`.

1. Create `count.rs` in this folder. Write a plain `fn main()` and inside it, one binding: call `std::fs::read_to_string("sample.jsonl")` and annotate the binding as `String` — you're claiming the call hands you the text directly. Compile: `rustc --edition 2024 count.rs -o count`. Read the E0308 top to bottom. **This failure is the lesson:** the filesystem is allowed to fail, so the return type says so.
2. Fix it the way step 6's E0277 discussion promised real CLI binaries do: change `main`'s return type so it can carry an `std::io::Error` out, put `?` after the call, drop the `String` annotation, and make `main`'s last expression the success value. Compile again — clean.
3. One new piece of syntax, met on its own before it does real work: a `for` loop — `for item in collection { ... }` — borrows each item of a collection in turn and runs the body once per item. That's the whole idea. Warm up if you like: `for n in [1, 2, 3] { println!("{n}"); }` as a `main` body prints three lines.
4. Now point it at the file: `for line in contents.lines() { ... }` with a `let mut count = 0;` above it. Inside, count the line only if it isn't blank — first honest attempt: `!line.is_empty()`. After the loop, print in exactly this shape: `println!("{count} non-blank lines");` — the checkpoint checks that wording.
5. Run it from this folder: `./count`. It prints **`4 non-blank lines`**. But you can see only 3 events in the file. Look closer at the "blank" line — open `sample.jsonl` or run `cat -A sample.jsonl` — and once you've seen what's actually on line 3, find the `&str` method that shaves whitespace off both ends, and put it in front of your emptiness check. Re-run: `3 non-blank lines`.
6. Prove the error path is real: change the filename in your code to `"sample.jsonl.nope"`, recompile, run. Watch the `?` carry the `io::Error` out of `main` — printed for you, exit code nonzero (`echo $?`). No crash handler you wrote, no try/catch anywhere. Restore the filename.
7. Optional payoff: rewrite the loop as one line — `.lines()`, an iterator adapter that keeps only lines passing your blank test, then `.count()`. Same answer, and it compiles to the same machine code as your loop.

## Errors you should EXPECT (and want)

- **`error[E0308]: mismatched types` — expected `String`, found `Result<String, std::io::Error>`** — the main event, from move 1. What it's really saying: you asked for the success value but the library returned the whole verdict. Every step so far, failure was something *you* modeled (step 5's enum, step 6's `Result`); here the standard library does it to you, because "file not found" and "permission denied" are facts of the outside world. There is no exception to catch — the failure *is* the return type. Your two exits are exactly step 6's: `match` it, or give `main` a `Result` return type and use `?`.
- **`error[E0599]: no method named `lines` found for enum `Result``** — the same wrong belief in a different outfit: chaining `.lines()` straight onto the call without unwrapping the `Result` first. The fix is the same fix.
- **`error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`** — if you added `?` in move 2 but forgot to change `main`'s return type. You met this in step 6 and you know both fixes; use the signature one this time.
- **The count of 4 — no error code at all.** The compiler proved your types line up and said nothing, yet the answer is wrong: line 3 of the file is whitespace, and to `.is_empty()` whitespace is content. Types can't see inside a string. This is the boundary of the compiler's protection — past it, correctness is on your tests, which is exactly why this course writes property tests. Remember this bug; you'll meet its family again.

## Checkpoint

- `./count`, run **from this folder**, prints exactly `3 non-blank lines`.
- With the filename misspelled, the program prints an `io::Error` (something like `No such file or directory`) and `echo $?` shows a nonzero exit — and you can point at which character in your code made that happen (the `?`).
- You can explain, without notes, why the count was 4 before the fix and why no compiler on earth would have flagged it.
- Say this out loud and mean it: *"the file arrives as one owned `String`; every `line` is a borrowed `&str` into that same buffer — reading 4 lines cost 1 allocation, not 5."* (A GC language typically materializes a fresh string object per line. On Lambda, allocations are cold-start weight and memory-bill weight; this habit is where Rust's efficiency story starts being yours.)
- Save what YOU wrote: your working file already lives in this folder — name it `my-solution.rs`, then commit — `ramp: step 7 — reading a file, line by line`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

Two separate snags, two nudges.

The `Result`: `read_to_string`'s signature is `fn read_to_string(...) -> std::io::Result<String>`, which is shorthand for `Result<String, std::io::Error>`. Step 6 gave you the pattern for a fallible `main`: return type `Result<(), YourErrorType>`, `?` on the fallible call, `Ok(())` as the last expression. The error type here is `std::io::Error`.

The count of 4: "blank" and "empty" are different claims. `""` is empty; `"   "` is blank but not empty. There's a `&str` method whose name is four letters and starts with `t` — it returns a *narrower borrowed slice* of the same line, whitespace shaved off both ends. Chain it before `.is_empty()`.

</details>

<details><summary>Hint 2 — the shape</summary>

The opening two lines carry the whole step-6 payload:

```rust
fn main() -> Result<(), std::io::Error> {
    let contents = std::fs::read_to_string("sample.jsonl")?;
```

Then a `mut` counter, a `for` over `contents.lines()`, and inside it an `if` whose condition is `!line.trim().is_empty()`. Print the counter, and remember `main`'s last expression must be `Ok(())` — no semicolon story here, it's just the value.

For move 7, the adapter you want is `.filter(|l| ...)` between `.lines()` and `.count()`.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
