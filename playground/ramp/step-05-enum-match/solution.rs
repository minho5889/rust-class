// Step 5 — enums and exhaustive match: the FIXED version.
// Both E0004s you were asked to trigger are preserved below, because the
// errors were the point.
//
// --- What you typed first (does NOT compile) ── E0004 ──────────────────
//
//     fn describe(l: &LineKind) -> String {
//         match l {
//             LineKind::Blank => String::from("blank line"),
//         }
//     }
//
//     error[E0004]: non-exhaustive patterns: `&LineKind::Broken(_)` and
//       `&LineKind::Event { .. }` not covered
//
// A match must prove it handled every shape of the enum; the compiler
// lists the missing ones by name.
//
// --- The point-6 punchline (does NOT compile) ── E0004 ─────────────────
//
//     Add `Truncated` to the enum, change nothing else:
//
//     error[E0004]: non-exhaustive patterns: `&LineKind::Truncated`
//       not covered
//
// The error points at `describe`, not at the enum: the compiler walked
// the whole program and found every match the new variant breaks. That
// pressure is the feature — and it's exactly what a `_` catch-all arm
// would trade away.
// ------------------------------------------------------------------------

enum LineKind {
    Blank,                  // teach: unit variant — carries no data, just a name
    Broken(String),         // teach: tuple variant — the payload rides inside the value
    Event { kind: String }, // teach: struct-like variant — payload with a named field
    Truncated,              // teach: the point-6 addition that re-broke `describe`
}
// teach: a LineKind is exactly ONE of these shapes at a time (a "sum type").
// In memory: one small tag saying which variant + room for the largest payload,
// flat on the stack — no null, no heap box, no GC. Modeling states this way
// costs nothing at runtime, which is part of why Rust stays lean on Lambda.

fn describe(l: &LineKind) -> String {
    // teach: `l` is a borrow (step 3) — describing only requires looking.
    match l {
        // teach: patterns mirror construction — same shape, a fresh name where the data sat
        LineKind::Blank => String::from("blank line"),
        LineKind::Broken(msg) => format!("broken line: {msg}"), // teach: `msg` binds the payload; through `&LineKind` it's a borrow, not a move
        LineKind::Event { kind } => format!("event of kind '{kind}'"),
        LineKind::Truncated => String::from("truncated line"),
        // teach: no `_` arm, deliberately — a wildcard compiles today and silently
        // rots when the enum grows tomorrow; naming every variant keeps E0004 armed.
    } // teach: no `return`, no trailing semicolon — the match is an expression, and it IS the function body
}

fn main() {
    // teach: construction mirrors declaration — unit / tuple / struct-like
    let blank = LineKind::Blank;
    let broken = LineKind::Broken(String::from("unclosed brace at byte 42"));
    let event = LineKind::Event {
        kind: String::from("session.start"),
    };
    let cut = LineKind::Truncated;

    println!("{}", describe(&blank)); // teach: `&` at the call site — lend, don't give (step 3, unchanged)
    println!("{}", describe(&broken));
    println!("{}", describe(&event));
    println!("{}", describe(&cut));
}
