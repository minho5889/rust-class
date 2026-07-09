// Step 4 — &str vs String: the FIXED version.
// The erroring variant you were asked to provoke is preserved below,
// because the error was the point.
//
// --- What you typed to break it (does NOT compile) ----------------------
//
//     let ts;
//     {
//         let line = String::from("2026-07-09T10:00:00Z memlens.alloc");
//         ts = line.get(0..10).unwrap_or("bad-ts");
//     } // <- `line` dies here; its heap buffer is freed
//     println!("{ts}"); // error[E0597]: `line` does not live long enough
//
// `ts` is a pointer into `line`'s heap bytes. After the `}` those bytes
// are gone, so using `ts` would be a dangling pointer. C compiles this and
// crashes (or silently corrupts) at runtime; Rust refuses at compile time.
// ------------------------------------------------------------------------

// teach: `&str` in, `&str` out — the returned slice borrows from `line`, so
// this function allocates NOTHING. Rust infers that lifetime link on its own
// when there's one input reference; you'll write it by hand in step 8.
fn day(line: &str) -> &str {
    // teach: `.get(0..10)` returns Option<&str>, because slicing can fail
    // (line too short, or the cut lands mid-character in UTF-8). `.unwrap_or`
    // turns "maybe" into "value or fallback" — no unwrap(), no panic path.
    line.get(0..10).unwrap_or("bad-ts")
}

fn main() {
    let line = String::from("2026-07-09T10:00:00Z memlens.alloc");

    // teach: a BORROWED window into `line`'s heap bytes — zero allocation.
    // Compare step 2's clone(), which copied the bytes to a new buffer.
    let ts: &str = line.get(0..10).unwrap_or("bad-ts");

    // teach: both alive at once — a borrow is not a move. `line` still owns
    // every byte; `ts` just points at the first ten of them.
    println!("slice: {ts}  (owner intact: {line})");

    // teach: String = 24 bytes on the stack — pointer + length + CAPACITY.
    // It owns a growable heap buffer, so it must track how big it can get.
    println!("String, on the stack: {} bytes", std::mem::size_of_val(&line));
    // teach: &str = 16 bytes — pointer + length, a "fat pointer". No
    // capacity, because a borrower may look but never grow the buffer.
    println!("&str,   on the stack: {} bytes", std::mem::size_of_val(&ts));
    // teach: without the `&`, size_of_val measures what ts POINTS AT: the
    // 10 UTF-8 bytes of "2026-07-09", still sitting in line's buffer.
    println!("bytes it points at:   {} bytes", std::mem::size_of_val(ts));

    // teach: `&line` is a &String but `day` wants &str — deref coercion
    // converts for free, so one function serves Strings, literals, slices.
    println!("day(&line)  -> {}", day(&line));
    // teach: a literal already IS a &str (a window into bytes baked into the
    // binary). Too short for 0..10, so the fallback fires — no panic.
    println!("day(\"oops\") -> {}", day("oops"));
}
