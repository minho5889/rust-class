# Step 11 — Closures & iterator adapters: the loop, industrialized

**Concept:** a closure is a function you make on the spot — `|line| line.trim().is_empty()` — that can *capture* variables from around it (by borrow when reading suffices, by move on request), and iterator adapters (`filter`, `map`, `collect`, `filter_map`) chain closures into pipelines that compile down to the loop you'd have written by hand.
**You can already:** write `for` loops over lines and even one adapter chain — step 7's move 7 `.filter(|l| ...)` was your first closure, used on faith before it had a name; predict moves (step 2) and borrows (step 3); fold tallies over lines (glake v0, sitting E).
**After this step you can:** read and write `|args| body`, predict whether a closure borrows or moves what it captures — and why the borrow checker cares — and reach for filter/map/count/collect/`filter_map` as your default way of processing lake lines.

## The exercise

Runs in this folder — create `adapters.rs` here, then:
`rustc --edition 2024 adapters.rs -o adapters && ./adapters`

This step is sitting I at toy scale: glake v1's filters (`--type`, `--since`) are closures as predicates and adapters as the pipeline, with a property (F3: kept + excluded = total) proving no event is invented or lost. No file today — the batch is inline. Open `main` with it (paste it; data entry isn't the lesson):

```rust
let mut lines = vec![
    r#"{"event":"memlens.alloc","dt":"2026-07-09","bytes":4096}"#,
    r#"{"event":"memlens.dealloc","dt":"2026-07-09","bytes":4096}"#,
    "   ",
    r#"{"event":"gate.approved","dt":"2026-07-08","spec":"004"}"#,
    r#"{"event"#,
    r#"{"event":"memlens.alloc","dt":"2026-07-08","bytes":128}"#,
];
```

(`r#"..."#` is a *raw string* — quotes inside without backslash gymnastics. Note the old friend on line 3 — three spaces, step 7's whitespace trap — and the truncated line 5, cut off mid-key. The `mut` earns its keep in move 5.)

1. Meet a closure standing still: `let is_blank = |line: &str| line.trim().is_empty();` — parameters between pipes, body after, no `fn`, no name required, return type inferred. Calling it looks exactly like a function call: print `is_blank(lines[0])` and `is_blank(lines[2])` — `false`, then `true`.
2. Now the step-7 one-liner, understood instead of copied: `let n = lines.iter().filter(|l| !l.trim().is_empty()).count();` and print `{n} non-blank lines` — **5** (the truncated line isn't blank; types can't see inside a string, and neither can `trim`). Read the chain as a sentence: *iterate, keep only items the closure approves, count what survives.* One wrinkle worth knowing now: `filter` *lends* each item to the predicate, so inside the closure `l` is a reference to the item (here `&&str` — a reference to your `&str`); method calls like `.trim()` see through the layers on their own.
3. `map` + `collect` — transform, then gather: pull each line's event kind out with a hand-scanner move worthy of glake: splitting `{"event":"memlens.alloc",...}` on `"` makes piece 3 the kind (`{`, `event`, `:`, then `memlens.alloc`), so `l.split('"').nth(3)` is `Some("memlens.alloc")` — an `Option`, because a blank or truncated line has no piece 3. First version keeps every line and labels the holes: `let kinds: Vec<&str> = lines.iter().map(|l| l.split('"').nth(3).unwrap_or("<no kind>")).collect();` then print with `{:?}` — six entries, two of them `<no kind>`. (`collect` needs to be told what to build — the annotation on the binding does that. And no `unwrap()` on a data path, house rule; `unwrap_or` lands safely.)
4. Capture — the closure's actual superpower: `let wanted = String::from("memlens.alloc");` then `let is_wanted = |l: &str| l.contains(wanted.as_str());` — read the body again: `wanted` is not a parameter; the closure reached out and *captured* it. Count the hits: `lines.iter().filter(|l| is_wanted(l)).count()` — **2**. Then print `wanted` itself afterwards: still alive, because the closure only needed to read, so it captured by *shared borrow* — the compiler always infers the lightest capture that works (borrow to read, `&mut` to write, move only if forced or asked). Step 3's lending, automated.
5. Break it — the borrow is real: **E0502**. A capture is not a snapshot. Bind a counting closure but *don't call it yet*: `let count_blank = || lines.iter().filter(|l| l.trim().is_empty()).count();` (zero parameters — the pipes are empty; `lines` is captured). Between the binding and the call, push: `lines.push(r#"{"event":"memlens.dealloc","dt":"2026-07-09","bytes":4096}"#);` then call `count_blank()` and print. Compile. Read all three arrows in the error: where the closure took the borrow, where `push` demanded the exclusive one, where the borrow is used later. Step 3's one-writer-XOR-many-readers, with a closure as the reader — and it's protecting you from something real: `push` may reallocate the Vec's buffer and move every element. Undo the break: move the `push` *above* the closure binding, leaving every earlier print where it is.

   Then — before recompiling — predict which printed numbers change and which don't. (Two things to hold at once: chains that already *ran* before the push keep their numbers, and the pushed line is a dealloc *event*, not a blank.) Recompile and check yourself.
6. Break it again — `move`: **E0382**. Put `move` in front of move 4's closure: `let is_wanted = move |l: &str| ...`. Compile. The later `println!` of `wanted` now fails — *value borrowed here after move*. `move` transfers ownership of every captured variable into the closure; afterwards `wanted` is dead in the outer scope, exactly step 2's E0382 with the closure playing the thief. When is `move` right? When the closure must outlive the scope that made it — returned closures, spawned threads, and the async tasks 005 will throw at you. Here it isn't; delete `move`.
7. Stretch — `filter_map`, the adapter that eats `Option`s: replace move 3's map-with-placeholder by `let kinds: Vec<&str> = lines.iter().filter_map(|l| l.split('"').nth(3)).collect();` — `Some(kind)` survives unwrapped, `None` is dropped, no placeholder, no `unwrap` anywhere. Print it: the clean kinds only. One pass, transform and filter fused — this Option-in/value-out shape is how a glake pipeline digests scanner extractors without a single panic path.

## Errors you should EXPECT (and want)

- **`error[E0502]: cannot borrow `lines` as mutable because it is also borrowed as immutable`** — from move 5. What it's really saying: binding the closure *took* a shared borrow of `lines` and holds it until the closure's last use; `push` needs the exclusive borrow; both can't coexist. The diagnostic is unusually cinematic — three arrows: `first borrow occurs due to use of `lines` in closure`, `mutable borrow occurs here` at the push, `immutable borrow later used here` at the call. Why this is a favor: `push` can reallocate the buffer and move every element, and a language that allows mutating a collection while something holds a view into it hands you iterator invalidation — a crash in C++, a `ConcurrentModificationException` at runtime in Java. Rust makes the closure's borrow visible to the checker and settles it at compile time.
- **`error[E0382]: borrow of moved value: `wanted`** — from move 6. Capture is assignment in disguise: `move` made the closure the *owner*, so the outer `wanted` is spent — the same E0382 that taught you moves in step 2. The `help:` even offers the escape hatch (`consider cloning the value before moving it into the closure`) for the times you genuinely need both. And notice the symmetry with the default: borrow-capture left `wanted` printable in move 4; move-capture killed it. The pipes decide nothing — the `move` keyword and the body's needs decide.
- **`error[E0631]: type mismatch in closure arguments`** — a stumble you'll likely meet in move 4 if you hand the named closure straight to the adapter: `.filter(is_wanted)`. `filter` feeds its predicate a *reference to* the item (`&&str`), but you annotated `|l: &str|`. Two escapes: wrap it — `.filter(|l| is_wanted(l))` — and let auto-deref bridge the gap (the worksheet's shape), or drop your annotation and let inference pick the layered type. The layers of `&` around iterator items stop being scary once you've read this error slowly, once.

## Checkpoint

- The finished program prints: `false` / `true`, `5 non-blank lines`, a six-entry kinds list with two `<no kind>` holes, `2 hits` with `wanted` still printable afterwards, `1 blank` from `count_blank` (7 lines by then, but the pushed dealloc is an event, not a blank), and a five-entry clean kinds list from `filter_map` — and you predicted which numbers the push would and wouldn't shift *before* the run confirmed it.
- You provoked E0502 and E0382 on purpose and can narrate both in ownership terms: *a capture is a borrow; `move` makes it a move.*
- You can say what `filter_map` does with a `Some` and with a `None` without looking it up — and why that killed the `<no kind>` placeholder *and* the `unwrap_or`.
- Say this out loud and mean it: *"a closure is a function plus the environment it captured — borrowed by default, moved on request — and an adapter chain is a loop the compiler compiles back to hand-written speed."* (Step 7 proved that last clause with `assert_eq!`; nothing changed.)
- Save what YOU wrote: your working file already lives in this folder — name it `my-solution.rs`, then commit — `ramp: step 11 — closures and iterator adapters`.

## Hints (open one at a time)

<details><summary>Hint 1 — a nudge</summary>

Closure anatomy: `|parameters| body` — the body is one expression, no braces needed (add `{ }` only for multiple statements). A zero-parameter closure is `|| body` — the pipes stay, empty. You never write what it captures; the compiler reads the body and captures exactly what's named there, as lightly as possible.

For the counts: `lines[0]` is a real event (false from `is_blank`), `lines[2]` is the three-space line (true). Six lines, one blank → 5 non-blank. `wanted` appears in two lines (index 0 and 5 — `memlens.dealloc` does *not* contain `memlens.alloc`; check the letters) → 2 hits.

For move 5: the order of the three statements is the whole game — closure binding, then push, then call is the broken order; push, then binding, then call is the fixed one.

</details>

<details><summary>Hint 2 — the shape</summary>

The three chains, fully assembled:

```rust
let n = lines.iter().filter(|l| !l.trim().is_empty()).count();

let kinds: Vec<&str> = lines
    .iter()
    .map(|l| l.split('"').nth(3).unwrap_or("<no kind>"))
    .collect();

let kinds_clean: Vec<&str> = lines
    .iter()
    .filter_map(|l| l.split('"').nth(3))
    .collect();
```

And the two captures, side by side:

```rust
let is_wanted = |l: &str| l.contains(wanted.as_str());      // borrows `wanted` — still usable after
let is_wanted = move |l: &str| l.contains(wanted.as_str()); // owns `wanted` — outer name is spent (E0382)
```

Used in a chain as `.filter(|l| is_wanted(l))` — the extra closure bridges `&&str` to `&str`.

</details>

*Stuck after honestly trying? solution.rs sits next to this file. Read it line by line, then rewrite it yourself from memory.*
