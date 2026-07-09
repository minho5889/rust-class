# Step 5 — Enums and exhaustive `match`

**Concept:** an enum says "this value is exactly one of these shapes," and `match` won't compile until you've handled every shape — the compiler keeps the checklist so you don't have to.
**You can already:** make owned `String`s, predict when an assignment moves them (E0382), and lend values to functions with `&`/`&mut` instead of giving them away (steps 1–4).
**After this step you can:** model "one of several kinds" data with an enum and write a `match` the compiler proves complete — and use that proof as a refactoring tool when the data model grows.

## The exercise

Runs at [play.rust-lang.org](https://play.rust-lang.org) — no local setup needed. We're modeling lines of a log file: empty, unparseable, or a real event — the same three fates a line in our data lake's JSONL can meet.

1. Above `main`, define an `enum` named `LineKind` with three variants, one of each shape Rust offers: `Blank`, a **unit** variant carrying nothing; `Broken`, a **tuple** variant carrying one `String` (the reason it's broken); and `Event`, a **struct-like** variant with one named field, `kind: String`.
2. In `main`, create one value of each variant and bind each to a variable. Construction mirrors the declaration: `LineKind::Blank` takes no parentheses; the other two carry data. Syntax reminder for the struct-like one: `LineKind::Event { kind: ... }`.
3. Above `main`, start `fn describe(l: &LineKind) -> String` — note the `&`, step 3's lesson: describing only requires *looking*. In its body write a `match l { ... }` with **only one arm**, for the `Blank` variant, producing a `String`. Run it. It won't compile — **that's the exercise.** You want `error[E0004]`. Read what the compiler lists after `not covered`: it names the missing variants, exactly.
4. Add the two missing arms. In a pattern, `Broken(msg)` gives the payload a name the arm body can use; `Event { kind }` does the same for the field. Build each arm's output with `format!(...)`. Notice what's absent: no `return`, no semicolon after the closing brace — the `match` **is** the function body.
5. Call `describe` on each of your three values from `main` (pass with `&` — you're lending) and print the results. Three lines.
6. The punchline. Add a **fourth variant** to the enum — `Truncated`, unit shape — and change nothing else. Run it. `error[E0004]` again, but look where it points: not at the enum, at your `match` in `describe`. The compiler just walked the whole program and found every place your new variant breaks. In a GC language this refactor is a grep and a prayer; here it's a compile error with a line number. That pressure is the feature.
7. Add the `Truncated` arm, and a fourth value in `main` so the new arm actually runs. Four lines print. Done.

## Errors you should EXPECT (and want)

- **`error[E0004]: non-exhaustive patterns`** — deliberately triggered twice. First as `` `&LineKind::Broken(_)` and `&LineKind::Event { .. }` not covered ``, then, in point 6, as `` `&LineKind::Truncated` not covered ``. What it's really saying: `match` is not a switch with optional cases — it's a proof obligation. An enum declares a *closed set* of shapes, and the compiler refuses to compile a `match` until every shape has an arm, naming the missing ones for you. Point 6 is why this matters: when the data model grows, the compiler hands you a complete to-do list of every match that must catch up. That's exhaustiveness as a refactoring tool — and it's why idiomatic Rust avoids the wildcard `_` arm unless it truly means "anything else, forever." A `_` would have compiled silently past `Truncated` and rotted at runtime instead. (Once your program works, try swapping the last three arms for `_ => ...`, watch E0004 vanish, feel what you lost, and put the arms back.)
- **`error[E0170]: pattern binding `Blank` is named the same as one of the variants`** — you'll hit this if you write an arm as `Blank =>` instead of `LineKind::Blank =>`. A bare name in a pattern isn't a comparison — it's a *new variable binding* that matches anything, which would silently swallow every variant. The compiler notices the near-collision with your variant's name and stops you; the `help:` line spells out the qualified path. (You'll also see an `unreachable pattern` warning on the arms below it — same root cause: the bare name matched everything first.)
- You may brush past **`error[E0308]: mismatched types`** if one arm produces `"blank line"` (a `&str`) while another produces `format!(...)` (a `String`). The whole `match` is *one expression* with *one type*, so all arms must agree — `String::from("...")` brings a literal arm in line.

## Checkpoint

- Your point-3 one-arm match produced E0004, and you can read the `not covered` list — the compiler names *variants*, not just line numbers.
- The three-variant program prints three lines, one description per value, each mentioning the data carried inside (the broken reason, the event kind).
- Adding `Truncated` re-broke the build with E0004 pointing at `describe` and naming exactly the new variant; one arm (plus a fourth value in `main`) fixed it — four lines print.
- Say this out loud and mean it: *"An enum is a closed set of shapes; `match` must prove it handled all of them — and a `_` arm trades that proof away."*

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

Patterns mirror construction. Whatever syntax *built* the value, the pattern is the same shape with a fresh name where the data sat: built with `LineKind::Broken(String::from("..."))`, matched with `LineKind::Broken(msg)`. Arms read `pattern => expression,`. And because `l` is a `&LineKind`, the names you bind (`msg`, `kind`) are borrows — free to read inside `format!`, never taken, exactly the contract the `&` in the signature promised.

</details>

<details><summary>Hint 2 — the shape</summary>

```rust
enum LineKind {
    Blank,
    Broken(String),
    Event { kind: String },
}

fn describe(l: &LineKind) -> String {
    match l {
        LineKind::Blank => /* a String */,
        // ...one arm per remaining variant, each producing a String
    }
}
```

In `main`, each call looks like `describe(&blank)` — lend, don't give.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
