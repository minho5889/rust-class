# Step 1 — Hello

**Concept:** A Rust program starts at `fn main`, stores values with `let`, prints them with the `println!` macro, and can call methods (like `.len()`) on those values.
**You can already:** Nothing in Rust yet — this is step one. Your AWS instincts (read the error, trust the tooling) transfer directly.
**After this step you can:** Write and run a complete Rust program that binds values to names, calls a method, and prints formatted output.

## The exercise

Runs at [play.rust-lang.org](https://play.rust-lang.org) — no local setup needed for this step.

1. Open the playground. It gives you an empty `fn main() { }`. Everything you write goes between those braces.
2. Bind your name to a variable using `let`. String literals go in double quotes.
3. Print a greeting with `println!`. Use a `{}` placeholder in the format string and pass your variable as the argument after it. Run it — you should see your greeting.
4. **Break it on purpose:** delete the semicolon at the end of your `let` line. Run. Read the whole error, top to bottom. Put the semicolon back.
5. Add a second variable that holds the length of your name — call the `.len()` method on your name variable.
6. Print the length too, with a second `println!` and another `{}` placeholder. Run it.
7. **Break it twice more:** first, delete the argument after a format string (keep the `{}`). Run, read. Then fix it and misspell `println` (try `printl!`). Run, read, fix.

The three deliberate breaks are not detours — they *are* the lesson. You want to have seen these errors in a ten-line program before they find you in a thousand-line one.

## Errors you should EXPECT (and want)

- **Missing semicolon** → ``error: expected `;`, found `println` ``. Pure syntax errors like this carry no `E`-code. What it's really saying: statements in Rust end with `;`, and the parser was still reading your `let` line when it collided with the next one. Rust points at the exact spot and usually suggests the fix.
- **`{}` with no argument** → `error: 1 positional argument in format string, but no arguments were given`. The format string is a promise: one `{}` means one value is coming. You promised and didn't deliver — and Rust catches it at *compile time*. (C's `printf` would happily print garbage at runtime. This is your first taste of "if it compiles, a whole class of bugs is already gone.")
- **Misspelled macro** → ``error: cannot find macro `printl` in this scope``, usually with a `did you mean` suggestion. Related sibling worth knowing: write `println` *without* the `!` and you get **E0423** — `expected function, found macro`. The `!` is not decoration; it marks a macro call, and `println!` is a macro, not a function.

## Checkpoint

Running the fixed program prints something like:

```
Hello, Minho!
Your name is 5 bytes long.
```

(Your text can differ; the shape matters: a greeting that includes your variable, then a line that includes the length.) Before moving on, all of this must be true:

- You triggered all three errors on purpose and can say in one sentence what each one meant.
- You can explain what the `!` in `println!` signifies.
- You know what `.len()` returned (a number — Rust calls this type `usize`; it counts *bytes*, which equals characters only for plain ASCII).

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

Everything lives inside `fn main() { }`. A binding is `let some_name = ...;` — note the trailing `;`. In `println!`, the format string comes first, then a comma, then the value that fills the `{}`.

</details>

<details><summary>Hint 2 — the shape</summary>

The skeleton, with the actual content left to you:

```rust
fn main() {
    let name = /* your name, in double quotes */;
    println!(/* format string with {} */, /* the variable */);
    // a second `let`, calling .len() on name
    // a second println! for it
}
```

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
