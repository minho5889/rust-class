# Step 4 — `&str` vs `String`

**Concept:** a `String` owns text on the heap; a `&str` is a borrowed window into text someone else owns — so slicing costs nothing, but the window can never outlive its owner.
**You can already:** create a `String`, move it, clone it (step 2), and lend it out with `&` without giving up ownership (step 3).
**After this step you can:** cut a zero-allocation slice out of a `String`, say exactly what each type costs in bytes, and explain why the compiler kills a slice whose owner died.

## The exercise

Runs at [play.rust-lang.org](https://play.rust-lang.org) — no local setup needed.

1. In `main`, start with this line — it's a memlens-style log line, timestamp first:
   ```rust
   let line = String::from("2026-07-09T10:00:00Z memlens.alloc");
   ```
2. Slice out the first 10 characters by calling `.get(0..10)` on `line`, and bind the result to a new variable. One catch: `.get` doesn't hand you a `&str`, it hands you a *maybe* (`Option<&str>`) — slicing can fail. Chain `.unwrap_or("bad-ts")` onto it to get a plain `&str` with a fallback. **No `.clone()`, no `.to_string()`** — the entire point is that this allocates nothing.
3. Print the slice **and** the original `String` in the same `println!`. Both work. Sit with that for a second: in step 2, using the old variable after handing data to a new one was E0382. This isn't a move — it's a borrow (step 3), and your slice is just a window into bytes that `line` still owns.
4. Now prove the two types are physically different. Add two prints using `std::mem::size_of_val(&line)` and `std::mem::size_of_val(&ts)` (whatever you named your slice — note the `&` in both). **Write your guesses in a comment before running.** Bonus: try `size_of_val(ts)` with no `&` and explain what *that* number is.
5. Break it on purpose. Declare a variable with a bare `let ts;` *before* an inner block, then inside `{ ... }` create the `String` and assign the slice to that outer variable, close the block, and print `ts` *after* the `}`. Run it. **This failure is the lesson** — read every line of the error, especially the two arrows: where the borrow happens and where the owner dies.
6. Delete (or comment out) the broken block and extract your slicing into a function: `fn day(line: &str) -> &str`, whose body uses `.get(0..10)` and `.unwrap_or("bad-ts")`. Note the parameter type — `&str`, not `&String` — this function accepts a slice of *any* string, from anywhere.
7. Back in `main`, call `day(&line)` and print the result. Then call `day("oops")` and print that too. Two things to notice: `&line` is a `&String` but the function wanted `&str` and Rust converted it silently (deref coercion — one free upgrade, details later), and a string literal like `"oops"` already *is* a `&str`. No panic on the short input — your fallback did its job.

## Errors you should EXPECT (and want)

- **`error[E0597]: `line` does not live long enough`** — the main event, from move 5. What it's really saying: your slice is a pointer into `line`'s heap buffer. At the closing `}`, `line` is dropped and that buffer is freed — so using the slice afterward would be reading freed memory, a dangling pointer. C would compile this and let it explode (or worse, silently corrupt) at runtime; a GC language "solves" it by paying a garbage collector to keep the buffer alive. Rust does neither: it proves at compile time that no borrow outlives its owner, and refuses. This is exactly why zero-copy parsing is safe in Rust — glake will slice timestamps out of thousands of log lines with *zero* allocations per line, and the compiler guarantees none of those slices dangle.
- **`error[E0308]: mismatched types`** — you'll meet this if you annotate the slice as `: &str` before adding `.unwrap_or`, or forget the fallback in `day`. It says: "expected `&str`, found `Option<&str>`". Translation: slicing can fail (line too short, or your cut lands in the middle of a multi-byte character), and `.get` makes that failure a *value you must handle* instead of a crash. `.unwrap_or` is you handling it.

## Checkpoint

- Move 4 printed **24** and **16** (bytes, on the 64-bit playground) — and you can say what each holds: `String` = pointer + length + *capacity* (it owns a growable buffer); `&str` = pointer + length (a "fat pointer"; borrowers may look but never grow).
- You provoked E0597 and can point at the exact `}` where the owner died.
- `day(&line)` prints `2026-07-09`; `day("oops")` prints `bad-ts`; nothing panicked and nothing allocated.
- Say this out loud and mean it: *"a `String` owns heap bytes; a `&str` borrows a window into someone else's bytes, so it can never outlive the owner."*

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

For move 2: `.get(0..10)` gives back an `Option<&str>` — a box that's either "here's your slice" or "no". `.unwrap_or(fallback)` opens the box and hands you the fallback if it's empty. Chain it right onto the `.get` call.

For E0597 in move 5: ask two questions — *who owns the bytes my slice points at?* and *on which line does that owner die?* The compiler's error already drew arrows at both answers; read it bottom to top.

For the sizes: `size_of_val(&ts)` with the `&` measures the variable `ts` itself; without the `&` it measures the thing `ts` points at.

</details>

<details><summary>Hint 2 — the shape</summary>

The slice in `main` is one chained line — get, then rescue:

```rust
let ts: &str = line.get(0..10).unwrap_or("bad-ts");
```

And `day` is the same line wearing a function signature:

```rust
fn day(line: &str) -> &str {
```

…with that one expression as the body (no `return`, no semicolon — the last expression *is* the return value, remember step 1). The returned `&str` borrows from the parameter; Rust figures out that link on its own here. In step 8 you'll learn to write it explicitly.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
