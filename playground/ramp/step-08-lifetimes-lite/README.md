# Step 8 — Lifetimes-lite: `<'a>` says "this output borrows from that input"

**Concept:** a lifetime annotation `'a` is one sentence written into a function signature — "the reference I return borrows from the input(s) marked `'a`, so it cannot outlive them."
**You can already:** borrow with `&` instead of giving values away (step 3), tell a borrowed `&str` from an owned `String` (step 4), and pull lines out of a real file (step 7) — so you've been *using* references that point into other people's data all along; you just haven't had to name how long they're allowed to live.
**After this step you can:** read and write `<'a>` in a signature, and say precisely when the compiler demands it (elision fails) versus when it silently writes it for you.

## The exercise

Runs in this folder — create `lifetimes.rs` here, then:
`rustc --edition 2024 lifetimes.rs -o lifetimes && ./lifetimes`

1. Write `fn first_word(s: &str) -> &str` — **no `<'a>` anywhere yet.** The body is one chained line: split on whitespace with `.split_whitespace()`, take `.next()`, and land safely on `""` with `.unwrap_or` when there's no word at all. In `main`, call it on a step-7-flavored line — `"2026-07-09T10:00:00Z memlens.alloc 64"` — and print what comes back. Also call it on `""` to prove the no-word path doesn't panic.
2. It compiles. Stop and be suspicious. A borrow went in, a borrow came out, and you never said how they relate — yet the compiler let it through. The answer: with exactly **one** input reference, there's only one thing the output could possibly borrow from, so the compiler fills the annotation in itself. This is **elision**. The lifetime isn't absent; it's written for you.
3. Prove it by writing what the compiler wrote: change the signature to `fn first_word<'a>(s: &'a str) -> &'a str`. Recompile — identical behavior, byte for byte. Read the signature out loud: *"there is some span of time `'a`; `s` is valid for at least that span; the `&str` I return is only guaranteed within it."*
4. Now write a function where the compiler *can't* fill it in: `fn longer(a: &str, b: &str) -> &str` — again **no `<'a>`** — returning whichever input is longer (an `if`/`else` comparing `.len()`; remember from step 5 that a block's last expression is its value — the same "no `return`, no semicolon" move as the `match` that *was* `describe`'s body). Compile. It fails. **Read E0106 top to bottom, including the help lines — this failure is the lesson.**
5. Fix it with one lifetime: declare it in angle brackets after the function name, then mark both inputs and the output with it. Recompile, call it in `main` on two event names of different lengths, print the winner.
6. Directly above `longer`, write **one comment sentence in your own words** answering: *what does `'a` promise?* Not what it "is" — what it *promises*, and to whom.
7. Optional stretch — watch the promise get enforced. Make `line` in `main` an owned `String`, and between `let word = first_word(&line);` and the `println!` that uses `word`, insert `drop(line);`. Read the error (E0505), grin, delete the line.

## Errors you should EXPECT (and want)

- **`error[E0106]: missing lifetime specifier`** — the main event, from move 4. What it's really saying: your return type contains a borrowed value, but the signature doesn't say whether it borrows from `a` or from `b` — and the signature is the *only* thing the compiler consults when checking your callers (it never re-reads the body at each call site, which is why compile-time borrow checking stays fast). With one input reference the answer is forced, so elision fills it in; with two it's genuinely ambiguous, so *you* must say it. The fix costs nothing at runtime — `'a` compiles to no code, no check, no field. It's a name for "however long both inputs are valid," used to tie the output to them. Notice the compiler's help text literally hands you the fix: *consider introducing a named lifetime parameter*.
- **`error[E0505]: cannot move out of `line` because it is borrowed`** — only if you took the move-7 stretch. This is `'a` doing its job: `word` borrows from `line`, so `line` must outlive `word`, and `drop(line)` breaks the promise mid-flight. A GC language would quietly keep the string alive and bill you at collection time; Rust proves the ordering at compile time and bills you nothing — the whole reason zero-copy slicing is safe to do in a Lambda handler.

## Checkpoint

- `first_word` on the sample line prints `2026-07-09T10:00:00Z`, and `first_word("")` gives `""` — no panic, no `unwrap()`.
- You provoked E0106 on `longer`, then fixed it, and it prints the longer of two event names.
- You can explain, without notes, why one-input `first_word` never needed the annotation but two-input `longer` did.
- Your comment above `longer` answers "what does `'a` promise?" in a sentence you'd defend — something equivalent to: *the returned reference borrows from the inputs marked `'a`, so it cannot outlive them.*
- Say this out loud and mean it: *"lifetimes never change what the program does — they name facts the borrow checker verifies, then compile to nothing."*
- Save what YOU wrote: your working file already lives in this folder — name it `my-solution.rs`, then commit — `ramp: step 8 — lifetimes-lite`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

E0106's help line names the entire fix. A lifetime parameter is declared once in angle brackets right after the function name — exactly where a generic type would go — and then referenced after each `&` it applies to. Give both inputs and the output the *same* name: one shared name is precisely how the signature says "the output borrows from these inputs."

Syntax gotcha: it's an apostrophe then a letter — `'a` — and it never closes. If your editor's highlighting goes strange thinking it's an unterminated character literal, you typed it right.

For the body of `longer`: no `return` keyword needed — `if a.len() >= b.len() { a } else { b }` is one expression, and as the last expression of the function it *is* the return value.

</details>

<details><summary>Hint 2 — the shape</summary>

The fixed signature has exactly four `'a`s — one declaration, three uses:

```rust
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str
```

Read as: "there is a span of time `'a`; both `a` and `b` are valid for at least that span; the `&str` I return is guaranteed only within it." The body doesn't change at all — E0106 was always a *signature* problem, never a body problem.

For the promise sentence: aim it in the right direction. `'a` doesn't promise the inputs will live long — it promises the **output won't outlive the inputs**. The constraint flows from what you return back to what you were given.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
