// Step 9 — traits: the FIXED version.
// The three errors you were asked to provoke are preserved below, because
// the errors were the point.
//
// --- First bad signing (does NOT compile) ── E0046 ─────────────────────
//
//     impl Describe for JsonlFile {}
//
//     error[E0046]: not all trait items implemented, missing: `describe`
//
// An impl block is a signed contract; the compiler audits it item by item
// and names what's absent — step 5's E0004 checklist energy, now for
// contracts instead of enums.
//
// --- Second bad signing (does NOT compile) ── E0186 ────────────────────
//
//     impl Describe for JsonlFile {
//         fn describe() -> String { ... }
//     }
//
//     error[E0186]: method `describe` has a `&self` declaration in the
//       trait, but not in the impl
//
// The receiver is part of the signature. Without `self` this is a
// different kind of item (an associated function, like String::from) and
// cannot answer a `value.describe()` call.
//
// --- The move-7 punchline (does NOT compile) ── E0599 ──────────────────
//
//     Wrap the trait in `mod contract`, delete the `use`, and:
//
//     error[E0599]: no method named `describe` found for struct
//       `JsonlFile` in the current scope
//       = help: items from traits can only be used if the trait is in scope
//       help: trait `Describe` which provides `describe` is implemented
//         but not in scope; perhaps you want to import it
//
// Not "doesn't exist" — not in SCOPE. Method calls only consult traits
// that are in scope; impls and bounds name the trait by path and never
// broke. The help output contains the entire fix.
// ------------------------------------------------------------------------

// teach: the mod stands in for "the trait lives in another file" — glake v1
// will keep EventParser in its own module for real. `pub` because anything
// inside a module is private to it until said otherwise.
mod contract {
    // teach: a trait is a contract — signatures only, each ending in `;`
    // where a function would grow a body. It promises WHAT, never HOW.
    pub trait Describe {
        // teach: `&self` bakes step 3 into the contract: describing only
        // requires looking, so every signer receives itself as a borrow.
        fn describe(&self) -> String;
    }
}

// teach: this line is what E0599's help handed you. Impls and bounds can
// name the trait by path; a METHOD CALL needs the trait in scope — this
// `use` is what puts it there. (It's why serde examples open with a `use`.)
use crate::contract::Describe;

// teach: two unrelated types — a file of the lake and a line of it. No
// common ancestor, no inheritance anywhere; the trait is the only thing
// they will share.
struct JsonlFile {
    path: String,
    events: usize,
}

struct Event {
    kind: String,
    day: String,
}

// teach: X signing the contract. The signature is copied from the trait
// letter for letter — receiver included (E0186 guards that) — and the `;`
// becomes a body. Miss an item and E0046 reads you the checklist.
impl contract::Describe for JsonlFile {
    fn describe(&self) -> String {
        // teach: `self` is the value the method was called on; `self.path`
        // reaches its fields. The String we return is owned — built fresh
        // here, handed to the caller, no borrow to track (contrast step 8).
        format!("{} ({} events)", self.path, self.events)
    }
}

// teach: the second signer — same contract, entirely different body. One
// method name, two bodies; the TYPE of the value picks which body runs.
impl contract::Describe for Event {
    fn describe(&self) -> String {
        format!("{} on {}", self.kind, self.day)
    }
}

// teach: one function for every signer. `<T: Describe>` reads "for any
// type T that implements Describe" — same bracket position as step 8's
// <'a>, because both introduce a name the signature needs. Inside, we know
// NOTHING about T except what the contract promises — which is exactly why
// this works for JsonlFile and Event alike. (What the compiler does with
// this — one copy per type — is step 10's whole subject.)
fn announce<T: contract::Describe>(item: &T) {
    // teach: this call compiles without the top-level `use` — the bound
    // itself names the trait, so `describe` is in scope for T here.
    println!("-> {}", item.describe());
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

    // teach: these two calls are what E0599 broke until the `use` — same
    // method name resolving to two different bodies, chosen by type.
    println!("{}", file.describe());
    println!("{}", event.describe());

    // teach: both types flow through the SAME generic function — lend with
    // `&` (step 3, unchanged), let the bound do the accepting.
    announce(&file);
    announce(&event);
}
