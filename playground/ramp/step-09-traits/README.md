# Step 9 — Traits: one contract, many types

**Concept:** a trait is a named contract — a list of method signatures with no bodies — and any type can sign it with an `impl` block; a function can then ask for "anything that signed" instead of one concrete type, and the compiler holds every signer to the letter of the contract.
**You can already:** group data into structs with named fields (glake v0's `Stats`; step 5's struct-like variants), write methods' little sibling — free functions taking `&self`-shaped borrows (step 3), and read a compiler error that *lists what's missing by name* (step 5's E0004).
**After this step you can:** define a trait, implement it for two different types, write one generic function that accepts both — and explain why a trait method call sometimes refuses to compile until a `use` brings the trait into scope.

## The exercise

Runs in this folder — create `traits.rs` here, then:
`rustc --edition 2024 traits.rs -o traits && ./traits`

This step is sitting J at toy scale: in glake v1 (spec 004) you'll define `EventParser` — one trait, two signers (your hand scanner, then `serde_json`) — and a property test will prove both honor the contract identically. Today the contract is smaller: "this thing can describe itself."

1. Above `main`, declare the contract: `trait Describe { fn describe(&self) -> String; }`. Look hard at that line: the signature ends in `;` where a function would grow a body. A trait promises nothing about *how* — it only fixes *what*: any signer must offer `describe`, taking `&self` (step 3's lesson baked into a signature: describing only requires looking) and returning an owned `String`.
2. Two lake-flavored types below it: `struct JsonlFile { path: String, events: usize }` (one file of the lake) and `struct Event { kind: String, day: String }` (one line of it). Same field syntax as step 5's struct-like variants, just standing alone.
3. Now sign the contract badly, on purpose — twice. First attempt: `impl Describe for JsonlFile {}` — an empty impl block. Compile. **`error[E0046]`**: read what comes after `missing:` — the compiler keeps the contract's checklist, exactly like step 5's E0004 kept the enum's.
4. Second attempt: write the method inside the impl but leave out `&self` — `fn describe() -> String { ... }`. Compile. **`error[E0186]`**: the trait said `&self`; the impl must match, letter for letter. Fix it — `fn describe(&self) -> String`, body `format!("{} ({} events)", self.path, self.events)`. Inside a method, `self` is the value the method was called on, and `self.path` reaches its fields.
5. Sign for `Event` too — same trait, different body: something like `format!("{} on {}", self.kind, self.day)`. In `main`, build one value of each type, call `.describe()` on both, print. Two lines. One method name, two bodies — the *type* of the value picks which body runs.
6. The payoff — one function for every signer: `fn announce<T: Describe>(item: &T)` that prints `-> ` followed by `item.describe()`. Read the bound out loud: *"for any type `T` that implements `Describe`."* Call it on both values. Inside `announce` you know nothing about `T` except what the contract promises — which is exactly why it works for both. (What the compiler *does* with that `<T: ...>` is step 10's whole subject.)
7. The scope punchline. Pretend the trait lives in another file: wrap the trait in a module — `mod contract { pub trait Describe { ... } }` (note the `pub`) — and qualify the two impls as `impl contract::Describe for ...` and the bound as `T: contract::Describe`. Compile. The impls are fine, the bound is fine — but every direct `.describe()` call in `main` now fails with **`error[E0599]`**. Read the `help:` lines to the end: the trait "is implemented but not in scope," and the compiler hands you the exact `use` line. Add it. Green again. The rule to keep: *implementing* a trait or *bounding* on it just needs its path — but *calling its methods* needs the trait in scope. Library traits will make you live this rule (it's why serde examples always open with a `use`).

## Errors you should EXPECT (and want)

- **`error[E0046]: not all trait items implemented, missing: `describe`** — from move 3. What it's really saying: an `impl Trait for Type` block is a signed contract, and the compiler audits it item by item, naming what's absent. Same checklist energy as step 5's E0004 — there it was "you didn't handle every variant," here it's "you didn't provide every promised method." Both exist so that growth is safe: add a method to the trait next month and every signer's build breaks with a to-do list, instead of some object silently lacking the method at runtime (the fate of duck-typed languages).
- **`error[E0186]: method `describe` has a `&self` declaration in the trait, but not in the impl`** — from move 4. The receiver is *part of the signature*. `fn describe()` with no `self` isn't a lesser method — it's a different kind of item entirely (an associated function, like `String::from`), and it can't stand in for the method the trait promised. Callers will write `value.describe()`, and that only works if a `self` is there to receive it.
- **`error[E0599]: no method named `describe` found for struct `JsonlFile` in the current scope`** — from move 7, and the wording is precise: not "doesn't exist," but not found *in the current scope*. Method calls only consult traits that are in scope — imagine two traits both defining a `describe`; the set of `use`d traits is how Rust knows which contracts you're even considering. The `help:` output is unusually generous here: it names the trait, says "implemented but not in scope," and prints the `use` line ready to paste. Note what *didn't* break: the impls and the `T: contract::Describe` bound, because they name the trait by path explicitly.

## Checkpoint

- The finished program prints four lines: two from direct `.describe()` calls, two through `announce` — and both types flowed through the *same* generic function.
- You provoked E0046 and E0186 on purpose and can say in one sentence what each protects (a missing item; a mismatched signature).
- After the move-7 mod wrap you hit E0599, fixed it with the `use` the compiler suggested, and can state the rule: impls and bounds need the path, method *calls* need the trait in scope.
- Say this out loud and mean it: *"a trait names a capability, not a type; `impl Describe for X` is X signing the contract, and `<T: Describe>` is a function accepting any signer."*
- Save what YOU wrote: your working file already lives in this folder — name it `my-solution.rs`, then commit — `ramp: step 9 — traits`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

The trait block and the impl block mirror each other exactly — copy the signature line out of the trait, replace the `;` with a body, and you've implemented it. Inside that body, fields are reached through `self.` (the `&self` in the signature is what puts `self` in scope).

For move 6: the angle brackets come right after the function name — `fn announce<T: Describe>(item: &T)` — the same position step 8 put `<'a>`. Generic type parameters and lifetime parameters live in the same spot because they're the same kind of thing: names the signature introduces.

For move 7: after wrapping, exactly three places name the trait — two impls, one bound — and they all keep working with the `contract::` path. Only the bare `.describe()` calls need the `use`. The compiler's help line contains the entire fix; read it before reaching for this hint's sibling.

</details>

<details><summary>Hint 2 — the shape</summary>

The final program, skeleton only:

```rust
mod contract {
    pub trait Describe {
        fn describe(&self) -> String;
    }
}

use crate::contract::Describe;

struct JsonlFile { /* path, events */ }
struct Event { /* kind, day */ }

impl contract::Describe for JsonlFile {
    fn describe(&self) -> String { /* format! over self.path, self.events */ }
}

// ...same for Event...

fn announce<T: contract::Describe>(item: &T) {
    println!("-> {}", item.describe());
}

fn main() {
    // one JsonlFile, one Event
    // two direct .describe() calls, two announce(&...) calls
}
```

Before move 7 it's the same program with the trait at top level, no `mod`, no `use`, and `Describe` bare everywhere.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
