// Step 10 — generics vs dyn Trait: the FIXED version.
// The two errors you were asked to walk into are preserved below, because
// the errors were the point.
//
// --- The wall (does NOT compile) ── E0308 ──────────────────────────────
//
//     let batch = vec![file, event];
//
//     error[E0308]: mismatched types
//       expected `JsonlFile`, found `Event`
//
// A Vec is homogeneous: `vec![...]` must infer ONE element type, and
// generics can't rescue it — Vec<T> is a vec of one T chosen at compile
// time, not "a vec of anythings". (A trailing E0282 "type annotations
// needed" cascades after it; when errors cascade, trust the first one.)
//
// --- The honest first fix (does NOT compile) ── E0277 ──────────────────
//
//     let batch: Vec<dyn Describe> = vec![file, event];
//
//     error[E0277]: the size for values of type `dyn Describe` cannot be
//       known at compilation time
//       = help: the trait `Sized` is not implemented for `dyn Describe`
//       = help: `JsonlFile` implements `Describe` so you could box the
//         found value and coerce it to the trait object `Box<dyn Describe>`
//
// `dyn Describe` is a real type but an UNSIZED one — any signer, any
// size — and Vec lays elements side by side in one buffer, so it demands
// Sized. The help names the fix outright: behind a pointer, everything is
// pointer-sized.
// ------------------------------------------------------------------------

// teach: step 9's cast, at top level this time — the trait...
trait Describe {
    fn describe(&self) -> String;
}

// ...two unrelated signers...
struct JsonlFile {
    path: String,
    events: usize,
}

struct Event {
    kind: String,
    day: String,
}

// ...and both contracts signed.
impl Describe for JsonlFile {
    fn describe(&self) -> String {
        format!("{} ({} events)", self.path, self.events)
    }
}

impl Describe for Event {
    fn describe(&self) -> String {
        format!("{} on {}", self.kind, self.day)
    }
}

// teach: WAY ONE — generics, static dispatch. The compiler MONOMORPHIZES
// this: it stamps out announce_generic::<JsonlFile> and
// announce_generic::<Event> as two separate compiled functions, each with
// a direct, inlinable call to the right describe. Runtime cost: zero.
// Price paid elsewhere: compile time, binary size (a copy per type — and
// binary size is cold-start weight on Lambda), and every T fixed at
// compile time. This is glake v1's `fn run<P: EventParser>` in miniature.
fn announce_generic<T: Describe>(item: &T) {
    println!("[static]  {}", item.describe());
}

// teach: WAY TWO — the trait object, dynamic dispatch. Same body, no <T>,
// and only ONE compiled function. It can still reach two different
// describe bodies because `&dyn Describe` is a FAT pointer: one pointer
// to the value + one pointer to the type's vtable (its table of trait
// methods). Each call reads the table at runtime and jumps — one
// indirection, no inlining. This is glake v1's `--parser` seam in
// miniature: the one place where the concrete type arrives at runtime.
fn announce_dyn(item: &dyn Describe) {
    println!("[dynamic] {}", item.describe());
}

fn main() {
    let file = JsonlFile {
        path: String::from("dt=2026-07-09/events.jsonl"),
        events: 3,
    };
    let event = Event {
        kind: String::from("memlens.alloc"),
        day: String::from("2026-07-09"),
    };

    // teach: two calls, two compiled functions — the turbofished names are
    // real; the compiler minted both.
    announce_generic(&file);
    announce_generic(&event);

    // teach: two calls, ONE compiled function. `&file` coerces to
    // `&dyn Describe` at the call site by itself — no cast to write.
    announce_dyn(&file);
    announce_dyn(&event);

    // teach: the fat pointer, measured instead of taken on faith — 8 vs 16.
    // The extra 8 bytes ARE the vtable pointer.
    println!(
        "&JsonlFile is {} bytes; &dyn Describe is {} bytes",
        size_of::<&JsonlFile>(),
        size_of::<&dyn Describe>()
    );

    // teach: the thing monomorphization CANNOT express — a mixed batch.
    // There is no single T to stamp, so the element type is the trait
    // object, boxed: values on the heap (Box::new moves them there — the
    // heap's version of step 2's move; `file` and `event` are gone as
    // names after this line), fixed-size fat pointers inline in the Vec.
    let batch: Vec<Box<dyn Describe>> = vec![Box::new(file), Box::new(event)];
    for item in &batch {
        // teach: item is a &Box<dyn Describe>; the call auto-derefs
        // through the Box and hops the vtable, exactly like announce_dyn.
        println!("[batch]   {}", item.describe());
    }
}
