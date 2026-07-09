# Step 3 — Borrowing with `&`

**Concept:** a reference (`&`) lets a function *read* your value without taking ownership — you lend the data, you keep the deed.
**You can already:** create a `String`, watch a plain assignment move it (E0382), and fix the move with `clone()` or by respecting the new owner (steps 1–2).
**After this step you can:** pass a value to a function and still use it afterwards — and explain the difference between a shared loan (`&`) and an exclusive one (`&mut`).

## The exercise

Runs at [play.rust-lang.org](https://play.rust-lang.org) — no local setup needed.

1. Above `main`, write a function named `shout` that takes one parameter of type `&String` and returns a `String`. Its whole job: return an uppercased *copy* of what it borrowed. There's a method on strings for this — the playground has no autocomplete, so open the [std `String` docs](https://doc.rust-lang.org/std/string/struct.String.html) and scan the method list for the one about case. (Note what the `&` in the parameter type promises: *I will only look, I won't keep it.*)
2. In `main`, make a `String` — a codename, a city, your call. Call `shout` on it, but pass the variable **plain**, no `&`, exactly like you'd pass a value in any GC language. Store the result.
3. Run it. It won't compile — **that's the exercise.** You want `error[E0308]`. Read it: the expected type, the found type, and the `help:` line that fixes it with a single character.
4. Apply the fix at the call site, then print the shouted result **and the original variable after it**. In step 2 this second print was a crime (E0382). Now it compiles. Ask yourself why: who owned the data the whole time?
5. Now the writing loan. Write a second function, `exclaim`, taking one parameter of type `&mut String` and returning nothing. In its body, append `"!"` in place — `push_str` is the method. No return; the caller's own string changes.
6. Call it from `main`. Two things must say `mut` before this compiles: the `let` that created your string, and the borrow at the call site. Get it compiling, then print the variable — the `!` is there, and nobody returned anything.
7. Break it on purpose one more time: right *before* the `exclaim` call, take a shared reference to your string (`let peek = &...;`) — and print `peek` on the line *after* the call. Run it. You want `error[E0502]`. Read the three arrows: where the shared borrow starts, where the mutable borrow intrudes, where the shared borrow is used. Then delete the experiment (or move the `println!` above the call) so the file compiles again.

## Errors you should EXPECT (and want)

- **`error[E0308]: mismatched types`** — expected `&String`, found `String`. What it's really saying: a reference is a *different type* from the value it points at. The function's signature declared "I borrow"; you offered to hand over ownership; those are different contracts and the compiler refuses to guess. The fix is one `&` at the call site — you lend instead of give. Nothing is copied, nothing is allocated: a `&String` is just an address on the stack. That's why borrowing is the default way to pass data in Rust — it's free, and it's exactly the no-hidden-copies discipline that keeps Rust binaries lean on Lambda.
- **`error[E0502]: cannot borrow ... as mutable because it is also borrowed as immutable`** — the borrow-checker's one big rule: **any number of readers, XOR exactly one writer.** While `peek` is still alive (it gets used after the call), nobody may mutate the string — a writer could reallocate the heap buffer and leave `peek` pointing at freed memory. Other languages find this bug at 3 a.m. in production; Rust finds it at compile time. Note what makes the error appear: not *taking* the reference, but *using* it after the mutation. Borrows end at their last use.
- You may also brush past **`error[E0596]`** if you forgot `mut` on the `let` — the compiler telling you a variable must opt in to being mutated before anyone can borrow it mutably. The `help:` line fixes it.

## Checkpoint

- Your broken version from move 3 produced E0308, and you can say what type was expected vs. found.
- The fixed program prints three lines: the shouted copy, the original (unchanged, still owned by `main`), and the original again with `!` appended.
- Your move-7 experiment produced E0502, and you can point at the shared borrow's *last use* — the line that kept it alive into the mutation.
- Say this out loud and mean it: *"`&` is a shared read-only loan, `&mut` is an exclusive write loan — many readers or one writer, never both."*

**Save what YOU wrote:** paste your playground code into `my-solution.rs` next to this README, then commit it — `ramp: step 3 — borrowing` (one commit per step; `my-solution.rs` is yours, `solution.rs` is the answer key).

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

The E0308 fix is one character, at the call site, and the compiler's `help:` line writes it for you — read to the bottom. The uppercase method is `to_uppercase()`; it returns a brand-new `String`, which is exactly why `shout` can promise to only borrow. For `exclaim`: it returns nothing at all — no `-> ...` in the signature — because mutating through `&mut` *is* the output.

</details>

<details><summary>Hint 2 — the shape</summary>

```rust
fn shout(s: &String) -> String { /* one line: uppercase s */ }
fn exclaim(s: &mut String) { /* one line: push_str */ }
```

And in `main`, the calls look like `shout(&word)` and `exclaim(&mut word)` — the `&`/`&mut` at the call site must match the parameter type, and `word`'s `let` needs `mut` for the second one to be legal.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
