// Step 6 — Result and the `?` operator: the FIXED version.
// The erroring variant you were asked to provoke is preserved below,
// because the error was the point.
//
// --- What you typed to break it (does NOT compile) ----------------------
//
//     fn main() {
//         let day = parse_day("2026-07-09")?;
//         //                               ^ error[E0277]: the `?` operator
//         //     can only be used in a function that returns `Result` or
//         //     `Option` (or another type that implements `FromResidual`)
//         println!("{day}");
//     }
//
// `?` means "if this is Err, RETURN that Err from the enclosing function".
// The enclosing function here is main, which returns `()` — an early-returned
// Err has nowhere to land, so the compiler refuses. Two honest fixes: match
// in main (the caller decides what an error MEANS — done below), or declare
// `fn main() -> Result<(), String>`, which real CLI binaries actually do.
// ------------------------------------------------------------------------

// teach: the signature IS the contract — call me and you get Ok(day-number)
// or Err(a String saying why). Failure lives in the type, not in a doc
// comment or a surprise exception, so the caller cannot ignore it.
fn parse_day(s: &str) -> Result<u32, String> {
    // teach: .get(8..10) is Option<&str> — "absent", no story attached.
    // .ok_or_else attaches the story: it upgrades Option into Result,
    // building the Err value from a closure (which only runs on failure).
    // teach: then `?` unfolds to a match — on Err(e) it does `return Err(e)`
    // from parse_day RIGHT HERE; on Ok(v) it hands you v and moves on.
    let day_str = s
        .get(8..10)
        .ok_or_else(|| format!("too short to hold a day: {s:?}"))?;

    // teach: .parse is generic — the `: u32` annotation tells it what to
    // parse INTO (a turbofish `.parse::<u32>()` would say the same thing).
    // teach: parse fails with ParseIntError: right shape, wrong error
    // dialect. .map_err translates only the Err arm (Ok passes untouched),
    // matching our promised Err(String). Then `?` again — two fallible
    // steps, one flat chain, no match staircase.
    let day: u32 = day_str
        .parse()
        .map_err(|e| format!("day {day_str:?} is not a number: {e}"))?;

    // teach: success needs wrapping too — the return type is Result<u32, _>,
    // not u32. Forget the Ok and E0308 reminds you.
    Ok(day)
}

// The staircase version you wrote FIRST — identical behavior, every match
// spelled out. Compare with the body above: `?` is exactly this pattern,
// automated.
//
//     fn parse_day_verbose(s: &str) -> Result<u32, String> {
//         let day_str = match s.get(8..10) {
//             Some(d) => d,
//             None => return Err(format!("too short to hold a day: {s:?}")),
//         };
//         match day_str.parse::<u32>() {
//             Ok(day) => Ok(day),
//             Err(e) => Err(format!("day {day_str:?} is not a number: {e}")),
//         }
//     }

fn main() {
    // teach: the caller decides what an Err MEANS. Here: print it and
    // keep going. A server might answer 400; a Lambda might retry. The
    // type forces a decision; it doesn't dictate which one.
    match parse_day("2026-07-09") {
        Ok(day) => println!("2026-07-09 -> day {day}"),
        Err(why) => println!("2026-07-09 -> error: {why}"),
    }

    // teach: a bad input that fails in the SLICE step — "oops" is too short,
    // so .get(8..10) comes back None and .ok_or_else builds the message.
    // (The exercise's other bad input, "2026-07-xx", gets PAST the slice and
    // fails in the parse step instead — a different message, which is how
    // you prove which fallible step caught which input.)
    match parse_day("oops") {
        Ok(day) => println!("oops -> day {day}"),
        Err(why) => println!("oops -> error: {why}"),
    }
}
