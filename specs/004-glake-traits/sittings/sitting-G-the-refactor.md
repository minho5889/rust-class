# Sitting G — the refactor

**Builds:** nothing the user can see — and that's the whole assignment. glake's
insides get reshaped for v1 (lib/bin shape confirmed, `tally`'s signature
generalized, the public API dressed per C-COMMON-TRAITS) while its *behavior*
provably does not move: every test green before, every test green after, and
`stats` output byte-identical across the change.
**Requirements:** task 1.1 (T4 — modules & lib/bin split; F12 groundwork).
**Ramp you'll use:** step 10 (generics vs `dyn` — `impl Trait` is a generic in
disguise) and step 9's contract instincts; step 11 makes a cameo in `main`.

## Where you are

003 is closed: glake v0 validates and counts the real lake, R8/R9 hold under
thousands of cases, and Sitting F proved the std-only tree and the zero-copy
scanner on screen. Part 3 (this spec) grows v0 into v1 — filters, a designed
error type, and the crates we deliberately withheld (`clap`, `serde_json`,
`thiserror`), with the lens putting a number on what they cost. But sittings
H–J are about to perform surgery all over the crate, so today establishes the
discipline every refactor in your career should follow: **a green suite is a
license to change shape.** You may move anything, rename anything, generalize
anything — as long as the tests that were green stay green and the tool's
output doesn't shift by a byte. No red commit this sitting; that's not a
relaxation, it's the *point*. (C and E taught fail-first for new behavior;
G teaches its mirror: for *no* new behavior, nothing may ever fail.)

New this sitting: your work now cites F-requirements from
`specs/004-glake-traits/requirements.md` — read its "In plain words" section
before move 1, plus the design's shape table.

## The build, move by move

All commands from the repo root. Two commits this sitting — the reshape, then
the API dressing — and **both are green commits**.

1. **Photograph the behavior you must preserve.** Before touching anything:

   ```console
   cargo test -p glake            # count the greens — write the number down
   cargo run -p glake -- stats datalake/raw-local > target/g-before.txt
   ```

   (The 003 reference suite is 17 tests; yours may differ — what matters is
   *your* number, unchanged at every commit today.) `target/` is gitignored,
   so the snapshot is scaffolding, not telemetry. This file is the sitting's
   real gate: at the end, `diff` against a fresh run must print nothing.
   One honest caveat: your own session's hooks append to `dt=<today>` as you
   work, so re-take the snapshot right before each comparison — sitting F's
   Hint-2 lesson about collecting numbers back-to-back applies all day.

2. **Audit the shape against the design (T4/F12).** The design's shape table
   ends with: *"lib/bin split — already the reference shape; the learner's own
   crate refactors here (sitting G)."* If you followed A–F faithfully you
   already have `src/lib.rs` declaring `pub mod classify; pub mod scan;
   pub mod tally; pub mod walk;` with a thin `main.rs` — Sitting B built it
   that way so C's property tests could reach the pure core. Audit, don't
   assume; the checklist:

   - `lib.rs` declares exactly the four modules, all `pub`, plus
     `#![forbid(unsafe_code)]` at the top;
   - `main.rs` contains *only* arg parsing, the two command wrappers, output
     formatting, and exit-code mapping — no scanning, no classifying, no
     counting logic;
   - nothing in the lib prints or exits (B's rule: the library returns
     `Result`s; complaining is main's job);
   - every helper that only the module itself needs is private (`pub` is a
     promise — F12 will hold you to every one you make).

   Anything that drifted — a helper that wandered into `main.rs`, a `pub` that
   should be private, a module declared in the wrong file — moves home now,
   one relocation at a time, `cargo test -p glake` after each. If the audit
   comes up clean, say so out loud and take the free minute; the next two
   moves are the real work either way.

3. **Cash in Sitting E's promissory note: generalize `tally`.** E's worksheet
   told you: *"the reference generalizes this signature with `impl
   IntoIterator`; that's a generic in disguise and 004's territory — the slice
   is the honest v0."* You now own that territory — ramp 10 taught you what
   the compiler does with a type parameter. Change:

   ```rust
   pub fn tally(lines: &[&str]) -> Stats
   ```

   to the design's v1 shape:

   ```rust
   pub fn tally<'a>(lines: impl IntoIterator<Item = &'a str>) -> Stats
   ```

   Before you fix the fallout, answer aloud: `impl IntoIterator` in argument
   position is sugar for which ramp-10 way of dispatching — generic or `dyn`?
   (One of them; know why. Every caller gets its own monomorphized copy, same
   as `announce_generic::<JsonlFile>`.) Then let the compiler walk you to
   every call site — expect the double-reference fight (see below). The prize
   is in `main.rs`: your `stats` currently collects every line into an
   intermediate `Vec<&str>` just to satisfy the slice. Feed the iterator
   straight through instead:

   ```rust
   let stats = tally(contents.iter().flat_map(|c| c.lines()));
   ```

   Read that chain as ramp 11 taught you (`flat_map` is `map` whose closure
   yields iterators, flattened) — your first adapter chain in *production*
   glake, and it deletes an allocation: the `Vec` of line references never
   exists now. Small, but it's the through-line: v1 will keep asking "what
   does this shape cost?", and today the answer moved in the right direction.
   Suite green, snapshot still matching? **Commit point (green):**

   ```console
   git add crates/glake && git commit -m "004: sitting G — shape audited, tally generalized"
   ```

4. **Dress the public API — the C-COMMON-TRAITS pass (F12 groundwork).** The
   constitution's API rubric reviews every public type against the rust-lang
   guidelines; C-COMMON-TRAITS says: eagerly derive the common traits every
   user will expect (`Debug`, `Clone`, `PartialEq`/`Eq`, `Default`,
   `Copy` *where it's honest*). Walk your two public types:

   - **`Line<'a>`** (yours likely derives `Debug, PartialEq, Eq`): add
     `Clone, Copy`. Justify before typing: a `Line` is at most two `&str`s
     and a discriminant — pointers and a tag, cheap by construction. `Copy`
     on it is honest, and Sitting I will quietly collect the interest (a
     `Copy` verdict never picks a move-fight with a `match`).
   - **`Stats`**: add `Clone`. Then *try* adding `Copy` too, on purpose, and
     read the refusal (fights section). `Stats` owns two `HashMap`s — heap
     ownership can be cloned (a conscious, visible cost) but never silently
     copied. That distinction — `Copy` for borrow-and-tag types, `Clone` for
     heap owners — is the entire lesson, and every type H–J add will land on
     one side of it.

   While you're dressing: any public item still missing a `///` doc comment
   gets one now (the rubric treats docs as part of the API — a stranger
   should be able to call your lib from the docs alone), and bump
   `crates/glake/Cargo.toml` to `version = "0.2.0"` — v1 begins here; the
   003 reference stays 0.1.0 (the reference's NOTES call this out).

5. **Prove nothing moved, then close.** The full ritual plus the sitting's
   own gate:

   ```console
   cargo fmt
   cargo clippy -p glake --all-targets -- -D warnings
   cargo test -p glake                                  # same count as move 1
   cargo run -p glake -- stats datalake/raw-local > target/g-after.txt
   diff target/g-before.txt target/g-after.txt          # silence = success
   ```

   (If the diff shows only *new* events in `dt=<today>`, that's the lake
   growing under you — retake both snapshots back-to-back and compare again.
   The validation run of this worksheet did exactly that and the fresh pair
   was byte-identical.) **Commit point (green):**

   ```console
   git add crates/glake && git commit -m "004: sitting G — C-COMMON-TRAITS groundwork, suite green"
   ```

## Compiler fights to expect

Fewer than usual and all shallow — refactor sittings trade drama for
discipline — but each still goes to the mistake ledger (`learning.*`).

- **The double-reference fight** — the moment `tally` takes
  `impl IntoIterator<Item = &'a str>`, any old call site handing it `&lines`
  (a `&Vec<&str>`) fails: iterating a *reference to* a vec yields `&&str`,
  not `&str`, and the compiler reports the mismatch in `IntoIterator::Item`
  terms. Ramp 11's E0631 cousin. Two fixes, both idiomatic: `.iter().copied()`
  on the vec (cheap — copying a `&str` copies a pointer pair, ramp 4), or
  restructure the call to produce `&str` items directly (what `main`'s
  `flat_map` chain does). Your property test in `tests/prop_tally.rs` likely
  needs the same touch — `rendered.iter().map(String::as_str)` is the
  reference's spelling.
- **`error[E0204]: the trait `Copy` cannot be implemented for this type`** —
  move 4's deliberate experiment of `Copy` on `Stats`. The error points at
  the `HashMap` fields: `Copy` means "duplicating this is a bitwise copy,
  free and implicit", and a heap-owning map can never promise that — you'd
  get two owners of one buffer (the double-free ramp 2 exists to prevent).
  `Clone` is the honest spelling: duplication allowed, but visible and paid
  for. Delete the `Copy`, keep the lesson.
- **`missing lifetime specifier` on the generalized `tally`** — if you write
  `impl IntoIterator<Item = &str>` without introducing `<'a>`. Same E0106
  family as ramp 8; the items borrow from *somewhere*, and the signature must
  name it.
- **Clippy: `needless_lifetimes` or `map_flatten`-style nags** — clippy holds
  refactors to a higher bar than first drafts; take its suggestions, they're
  free API polish under `-D warnings`.

## Checkpoint

From the repo root — the sitting counts as done only when all of these hold:

```
cargo fmt --check                                  # no diff
cargo clippy -p glake --all-targets -- -D warnings # clean
cargo test -p glake                                # green, SAME count as move 1
cargo run -p glake -- stats datalake/raw-local > target/g-after.txt
diff target/g-before.txt target/g-after.txt        # empty (snapshots taken back-to-back)
```

- `git log --oneline -2` shows both sitting-G commits, and *neither* is a red
  commit — say why that's correct for this sitting and would be wrong for C.
- `grep version crates/glake/Cargo.toml` shows `0.2.0`.
- You can answer aloud: what did the compiler do to `tally` at each call site
  after the `impl IntoIterator` change (ramp 10 vocabulary)? Why is `Copy`
  honest on `Line<'a>` but a lie on `Stats`? And what, exactly, licensed you
  to change all this code without writing a single new test?

## Hints (one at a time)

<details><summary>Hint 1 — the double-reference error won't die</summary>

Chase the `Item` type, not the error text. Ask of each call site: "if I
`for x in <this>`, what is `x`?" — `for x in &vec_of_refs` gives `&&str`;
`for x in vec_of_refs.iter().copied()` gives `&str`;
`for x in contents.iter().flat_map(|c| c.lines())` gives `&str` (each `line()`
item borrows from the `String` in `contents`). Make every caller answer
`&str` and the bound is satisfied. If a lifetime error follows, check that
the `String`s being borrowed (your `contents` vec) outlive the `tally` call —
they do in the shape main already has.

</details>

<details><summary>Hint 2 — what "thin bin" means, concretely</summary>

Test it mechanically: could an integration test in `tests/` reproduce every
*behavior* of glake without spawning the binary — walking, classifying,
tallying? If some logic only exists inside `main.rs` functions, it can't, and
that logic belongs in a lib module. Output *formatting* and exit-code mapping
are the two legitimate residents of the bin (the reference keeps `validate`,
`stats`, and `sorted` there for exactly that reason — they turn library
values into text and codes, nothing more).

</details>

<details><summary>Hint 3 — the derive lines, spelled out</summary>

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line<'a> { /* unchanged */ }

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats { /* unchanged */ }
```

Order inside `derive(...)` is cosmetic; the *set* is the review. If clippy or
the compiler suggests `Copy` implies `Clone` must also be present — it's
right: `Copy: Clone` in the trait hierarchy, so the pair travels together.

</details>

## If truly stuck

Read, don't copy — take the shape, close the file, write yours:

- `specs/004-glake-traits/_reference/glake/src/lib.rs` — the v1 module list
  and doc header (note: it already lists H–J's modules; yours grows one
  sitting at a time).
- `specs/004-glake-traits/_reference/glake/src/tally.rs` — but **only** the
  signature style; the body has already absorbed Sittings I and J (filters,
  `ClassifiedLine`) and yours must not.
- `specs/004-glake-traits/_reference/glake/src/classify.rs` — the derive line
  on `Line` and its `Copy` justification comment.
