# Step 10 — Generics vs `dyn Trait`: two ways to say "any signer"

**Concept:** Rust can call through a trait two ways — generics (`fn f<T: Describe>`), where the compiler stamps out a specialized copy of the function per type at compile time, and trait objects (`&dyn Describe`, `Box<dyn Describe>`), where one copy of the function looks the method up through a pointer at runtime — and each can do one thing the other cannot.
**You can already:** define a trait, sign it with two types, and write a generic function with a `T: Describe` bound (step 9); tell stack from heap and know which one `String`'s bytes live on (step 4).
**After this step you can:** write the same function both ways, say what monomorphization is and what it costs, build a mixed-type `Vec<Box<dyn Describe>>`, and choose between generics and `dyn` with a rule you can defend out loud.

## The exercise

Runs in this folder — create `dispatch.rs` here, then:
`rustc --edition 2024 dispatch.rs -o dispatch && ./dispatch`

This step is glake v1's central layout decision at toy scale: the spec-004 pipeline stays *generic* (`fn run<P: EventParser>` — requirement F13) and spends exactly one `Box<dyn EventParser>` at the CLI seam where `--parser hand|serde` arrives at runtime. Today you build both halves of that decision and hit the wall that forces the `dyn` half.

1. Rebuild step 9's cast from memory — no `mod`, no `use` this time, everything at top level: the `Describe` trait, `JsonlFile`, `Event`, both impls. If this takes more than five minutes, redo step 9 first; this step assumes it.
2. Way one — generics: `fn announce_generic<T: Describe>(item: &T)` printing `[static]  ` + `item.describe()`. Call it on both values. It works, and you knew it would — but now hear what the compiler actually did: it compiled **two** functions, `announce_generic::<JsonlFile>` and `announce_generic::<Event>`, each with direct, inlinable calls to the right `describe`. That's **monomorphization**: generics vanish at compile time into per-type copies. Runtime cost: zero. Price paid elsewhere: compile time, binary size, and every `T` must be known when the compiler runs.
3. Way two — the trait object: `fn announce_dyn(item: &dyn Describe)` — *same body*, no `<T>` anywhere. Call it on `&file` and `&event`; Rust coerces `&JsonlFile` into `&dyn Describe` at the call site by itself. One compiled function this time. How can one function call two different `describe` bodies? Because a `&dyn Describe` is a **fat pointer**: one pointer to the value, one pointer to a table of that type's trait methods (the *vtable*). The call reads the table at runtime and jumps.
4. Don't take the fat pointer on faith — measure it: `println!("{} vs {}", size_of::<&JsonlFile>(), size_of::<&dyn Describe>());` prints **8 vs 16** on this machine. The extra 8 bytes *are* the vtable pointer.
5. Now the wall. A day's haul from the lake is mixed — files and events together: `let batch = vec![file, event];`. Compile. **`error[E0308]`**: expected `JsonlFile`, found `Event`. A `Vec` is homogeneous, and generics cannot rescue it — `Vec<T>` is a vec of *one* `T` chosen at compile time, not "a vec of anythings." (A trailing `E0282: type annotations needed` may follow; ignore it — when errors cascade, trust the *first* one. General rule, worth keeping.)
6. The honest first fix: name the trait as the element type — `let batch: Vec<dyn Describe> = ...`. Compile. **`error[E0277]`**: the size for values of type `dyn Describe` cannot be known at compilation time. Read every `help:` line — the compiler explains the problem (no `Sized`) *and* names the fix (box the values). Why: a `Vec` lays its elements out side by side in one buffer, so it needs one fixed element size — but "some signer of `Describe`" could be any type of any size.
7. The real fix: put the *values* on the heap and store fixed-size pointers inline — `let batch: Vec<Box<dyn Describe>> = vec![Box::new(file), Box::new(event)];`. (`Box::new` moves a value to the heap and hands back an owning pointer — the heap's version of step 2's move.) Loop `for item in &batch { ... }` printing `[batch]   ` + `item.describe()`. A mixed batch, described in one loop — the thing monomorphization *cannot* express, because there is no single `T` to stamp.

**When to use which — the honest paragraph.** Default to generics: in library and pipeline code the concrete types are known at compile time, calls are direct and inlinable, and the abstraction costs nothing at runtime — that's why glake v1's core is `fn run<P: EventParser>` and why its unit tests can call it monomorphized with each parser directly (F13). Reach for `dyn` in exactly two situations: the concrete type isn't known until runtime (a user typed `--parser serde`), or one collection must hold mixed signers (move 7). The costs, stated straight in both directions: `dyn` pays a vtable hop per call, usually a heap `Box` per value, and blocks inlining — on a CLI dispatching one flag, that's noise; generics pay a compiled copy per type — real binary weight if a bound fans out across many types, and binary size is cold-start weight on Lambda. So choose by *shape* — who knows the type, and when — not by micro-benchmarks: generic on the inside, one `dyn` at the outermost seam where the runtime choice arrives. That is glake v1's layout, verbatim.

## Errors you should EXPECT (and want)

- **`error[E0308]: mismatched types` — expected `JsonlFile`, found `Event`** — from move 5. What it's really saying: `vec![file, event]` forces the compiler to infer *one* element type, and your two elements disagree. This is monomorphization's blind spot made visible: `Vec<T>`, like `announce_generic::<T>`, gets stamped for a single concrete `T` — there is no compile-time answer to "a vec of different things." The runtime answer is the trait object, and moves 6–7 walk you to it.
- **`error[E0277]: the size for values of type `dyn Describe` cannot be known at compilation time`** — from move 6, and the most instructive dead end in this step. `dyn Describe` is a real type, but an *unsized* one — any signer, any size — and `Vec` stores elements inline in one buffer, so it demands `Sized`. The `help:` text does something rare: it names `Box<dyn Describe>` outright ("you could box the found value and coerce it to the trait object"). Indirection is the universal answer to unsizedness — behind a pointer, everything is pointer-sized.

## Checkpoint

- The finished program prints: two `[static]` lines, two `[dynamic]` lines, the `8 vs 16` measurement, and two `[batch]` lines from the mixed `Vec<Box<dyn Describe>>`.
- You can say what monomorphization means in one sentence and name the two functions the compiler minted from `announce_generic`.
- You provoked E0308 with the mixed vec and E0277 with `Vec<dyn Describe>`, and can explain why the working type is `Vec<Box<dyn Describe>>` — fixed-size fat pointers inline, values on the heap.
- Say this out loud and mean it: *"generics: many copies, no indirection, types fixed at compile time; `dyn`: one copy, a vtable hop, types free at runtime — glake goes generic inside and spends one `dyn` at the CLI seam."*
- Save what YOU wrote: your working file already lives in this folder — name it `my-solution.rs`, then commit — `ramp: step 10 — generics vs dyn`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

The two functions differ only in their signatures — the body line is identical in both:

- generic: angle brackets after the name, bound inside them, parameter `&T`;
- dyn: no angle brackets at all, parameter `&dyn Describe`.

`size_of` is called with a turbofish: `size_of::<&dyn Describe>()` — the type goes in the angle brackets because there's no value to infer it from.

For move 7: every element of the vec literal gets its own `Box::new(...)`, and the binding needs the full annotation `Vec<Box<dyn Describe>>` — leave the annotation off and inference will pick `Vec<Box<JsonlFile>>` from the first element and re-break on the second.

</details>

<details><summary>Hint 2 — the shape</summary>

The two ways, side by side:

```rust
fn announce_generic<T: Describe>(item: &T) {
    println!("[static]  {}", item.describe());
}

fn announce_dyn(item: &dyn Describe) {
    println!("[dynamic] {}", item.describe());
}
```

And the mixed batch:

```rust
let batch: Vec<Box<dyn Describe>> = vec![Box::new(file), Box::new(event)];
for item in &batch {
    println!("[batch]   {}", item.describe());
}
```

Note `batch` takes ownership — `file` and `event` move into their boxes, so do the size measurement (move 4) *before* this line or E0382 will remind you of step 2.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
