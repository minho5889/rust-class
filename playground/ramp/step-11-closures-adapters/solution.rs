// Step 11 — closures & iterator adapters: the FIXED version.
// The two errors you were asked to provoke are preserved below, because
// the errors were the point.
//
// --- Break #1 (does NOT compile) ── E0502 ──────────────────────────────
//
//     let count_blank = || lines.iter().filter(|l| l.trim().is_empty()).count();
//     lines.push(r#"{"event":"memlens.dealloc",...}"#);   // <- push AFTER binding
//     println!("{} blank", count_blank());
//
//     error[E0502]: cannot borrow `lines` as mutable because it is also
//       borrowed as immutable
//        | let count_blank = || lines.iter()...
//        |                   -- ----- first borrow occurs due to use of
//        |                   |        `lines` in closure
//        |                   immutable borrow occurs here
//        | lines.push(...)
//        | ^^^^^^^^^^^^^^^ mutable borrow occurs here
//        | println!("{} blank", count_blank());
//        |                      ----------- immutable borrow later used here
//
// A capture is not a snapshot — binding the closure TOOK a shared borrow
// of `lines` and holds it until the closure's last use; push needs the
// exclusive one. Step 3's one-writer-XOR-many-readers, with a closure as
// the reader. The favor is real: push may reallocate the buffer and move
// every element — iterator invalidation, settled at compile time.
//
// --- Break #2 (does NOT compile) ── E0382 ──────────────────────────────
//
//     let is_wanted = move |l: &str| l.contains(wanted.as_str());
//     ...
//     println!("{hits} hits for {wanted}");
//
//     error[E0382]: borrow of moved value: `wanted`
//       help: consider cloning the value before moving it into the closure
//
// Capture is assignment in disguise: `move` made the closure the OWNER of
// `wanted`, so the outer name is spent — step 2's E0382 with the closure
// playing the thief. `move` earns its keep when the closure must OUTLIVE
// its maker (returned closures, threads, 005's async tasks); here it
// doesn't, so it goes.
// ------------------------------------------------------------------------

fn main() {
    // teach: raw strings — r#"..."# — hold quotes without backslash
    // gymnastics. One whitespace-only line (step 7's old trap) and one
    // line truncated mid-key; `mut` because move 5 pushes.
    let mut lines = vec![
        r#"{"event":"memlens.alloc","dt":"2026-07-09","bytes":4096}"#,
        r#"{"event":"memlens.dealloc","dt":"2026-07-09","bytes":4096}"#,
        "   ",
        r#"{"event":"gate.approved","dt":"2026-07-08","spec":"004"}"#,
        r#"{"event"#,
        r#"{"event":"memlens.alloc","dt":"2026-07-08","bytes":128}"#,
    ];

    // teach: a closure standing still — |params| body. No fn, no name
    // needed, parameter and return types inferred from use. Calling it is
    // indistinguishable from calling a function.
    let is_blank = |line: &str| line.trim().is_empty();
    println!("{}", is_blank(lines[0])); // false — a real event line
    println!("{}", is_blank(lines[2])); // true — the three spaces

    // teach: step 7's one-liner, now readable: iterate, keep only items
    // the closure approves, count the survivors. `filter` LENDS each item
    // to the predicate, so `l` here is a &&str — a reference to your
    // &str — and method calls like .trim() see through the layers alone.
    // The truncated line isn't blank, so: 5.
    let n = lines.iter().filter(|l| !l.trim().is_empty()).count();
    println!("{n} non-blank lines");

    // teach: map transforms every item; collect gathers the results into
    // the container the binding's annotation asks for. The extractor is a
    // hand-scanner move worthy of glake: split on `"` and piece 3 is the
    // kind ({, event, :, THE-KIND) — an Option, since a blank or truncated
    // line has no piece 3. unwrap_or labels the holes; no unwrap() on a
    // data path, house rule. Six entries, two of them "<no kind>".
    let kinds: Vec<&str> = lines
        .iter()
        .map(|l| l.split('"').nth(3).unwrap_or("<no kind>"))
        .collect();
    println!("{kinds:?}");

    // teach: CAPTURE — `wanted` is not a parameter; the closure reached
    // out and grabbed it. Reading suffices, so it captures by shared
    // borrow — the compiler always infers the lightest capture that works
    // (borrow to read, &mut to write, move only if forced or asked).
    // Step 3's lending, automated. Prefix with `move` and the println of
    // `wanted` below dies with E0382 (see header).
    let wanted = String::from("memlens.alloc");
    let is_wanted = |l: &str| l.contains(wanted.as_str());

    // teach: the wrapper closure |l| is_wanted(l) bridges the &&str filter
    // feeds us to the &str our closure was annotated with — hand
    // `is_wanted` over bare and E0631 explains the layer mismatch.
    // memlens.dealloc does NOT contain "memlens.alloc" (check the
    // letters), so: 2.
    let hits = lines.iter().filter(|l| is_wanted(l)).count();
    println!("{hits} hits for {wanted}"); // teach: `wanted` still alive — it was only borrowed

    // teach: break #1, in its FIXED order — push BEFORE the closure binds,
    // so no borrow is alive when the exclusive one is needed. Swap this
    // line below the next one and E0502 (header) walks you through the
    // conflict with three arrows.
    lines.push(r#"{"event":"memlens.dealloc","dt":"2026-07-09","bytes":4096}"#);

    // teach: zero parameters — the pipes stay, empty. `lines` is captured
    // by shared borrow; the borrow lives from this binding to the last
    // call below. Seven lines by now, but the newcomer is an event: 1.
    let count_blank = || lines.iter().filter(|l| l.trim().is_empty()).count();
    println!("{} blank", count_blank());

    // teach: the stretch — filter_map, the adapter that eats Options.
    // Some(kind) survives unwrapped; None is dropped. Transform and filter
    // fused in one pass: no placeholder, no unwrap_or, no panic path —
    // how a glake pipeline digests scanner extractors. Seven lines in,
    // five kinds out (blank and truncated vanish).
    let kinds_clean: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.split('"').nth(3))
        .collect();
    println!("{kinds_clean:?}");

    // teach: step 7's promise, re-proven on today's chain — the adapter
    // pipeline and the hand-written loop are the same computation (and
    // compile to the same machine code; zero-cost abstraction, still).
    let mut by_hand = Vec::new();
    for l in &lines {
        // teach: filter_map's two behaviors, spelled out as a step-5 match
        match l.split('"').nth(3) {
            Some(kind) => by_hand.push(kind), // Some -> kept, unwrapped
            None => {}                        // None -> dropped
        }
    }
    assert_eq!(kinds_clean, by_hand); // teach: two phrasings, one answer
}
