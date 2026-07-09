# Step 2 — Ownership & move

**Concept:** every value in Rust has exactly one owner, and a plain assignment *moves* ownership instead of copying the value.
**You can already:** write `fn main`, store a value with `let`, print it with `println!`, and call a method on it (step 1).
**After this step you can:** explain why a variable "dies" after you assign it to another one — and name the two ways to keep using the data.

## The exercise

Runs at [play.rust-lang.org](https://play.rust-lang.org) — no local setup needed.

1. In `main`, create a `String` holding a city name. Use `String::from(...)` — new here: it turns a borrowed string literal like `"Seoul"` into an owned `String`.
2. On the next line, assign that variable to a *second* variable with a plain `let`. No `&`, no method calls — just the variable name on the right-hand side.
3. Now try to print the **first** variable with `println!`.
4. Run it. It will not compile. **That's the exercise.** Read the whole error message slowly: the `error[E0382]` line, the note that says *where* the value moved, and the `help:` line at the bottom. The compiler is teaching; let it finish.
5. **Fix it, way 1:** change the assignment so the second variable gets a *copy* of the data instead of taking ownership. There's a method for this, and the compiler's `help:` line literally names it. Print both variables to prove they both live.
6. **Fix it, way 2:** below that, make a fresh `String` (a region name — you know a few), move it into a second variable exactly like before, and this time print **only the second variable**. Don't touch the first after the move — that's the fix: respect who owns it now.
7. Add a comment above each fix answering: **which of these two fixes costs a heap allocation, and why?** (Think about what `String` actually holds: a pointer to bytes on the heap. What has to happen for two variables to *each* own their own bytes?)

## Errors you should EXPECT (and want)

- **`error[E0382]: borrow of moved value`** — this error *is* the lesson. What it's really saying: the `String`'s heap data has exactly one owner. When you wrote `let b = a;`, ownership of that heap data transferred to `b`, and `a` became invalid — not empty, not null, just *gone from the compiler's point of view*. Rust does this so that exactly one variable is ever responsible for freeing the memory: no double-free, no garbage collector needed to referee. (This is the memory-management story that makes Rust cheap on Lambda — no GC pauses, no runtime babysitting, drop happens at a known point.)
- Read the sub-parts of E0382: the `note: ... value moved here` arrow points at the assignment, and `help:` suggests the exact method for fix 1. Compiler errors in Rust are documentation with your variable names in it.

## Checkpoint

- Your broken version produced E0382 and you can point at the exact line where the move happened.
- Fix 1 compiles and prints **both** variables.
- Fix 2 compiles and prints the second variable.
- Your comment correctly says which fix allocates. Say this sentence out loud and mean it: *"assignment moves ownership; the old variable is invalid after the move."*

**Save what YOU wrote:** paste your playground code into `my-solution.rs` next to this README, then commit it — `ramp: step 2 — ownership & move` (one commit per step; `my-solution.rs` is yours, `solution.rs` is the answer key).

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

Re-read the compiler output from your broken version, bottom to top. The `help:` line at the end names the exact method you need for fix 1 — the compiler solved it for you, you just have to read that far. For fix 2: after `let b = a;`, ask "who owns the data *now*?" and only print that one.

</details>

<details><summary>Hint 2 — the shape</summary>

Fix 1 is a one-character... okay, one-*method* change to the assignment line:

```rust
let b = a.clone();
```

Fix 2 needs no new syntax at all — same move as before, but the `println!` names the second variable, not the first. For the comment in move 7: `clone()` copies the heap bytes into a brand-new allocation; a move copies only the pointer/length/capacity on the stack and hands over the deed. One is a memcpy of your data; the other is free.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
