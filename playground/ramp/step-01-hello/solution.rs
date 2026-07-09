// Step 1 — Hello: solution.
// Read this only after honestly trying. Then close it and rewrite from memory.

// teach: every Rust binary starts here. No class, no boilerplate — just `fn main`.
fn main() {
    // teach: `let` binds a value to a name. No type written — Rust *infers* it.
    // teach: this one is a `&str`: a borrowed view of string data baked into the binary.
    let name = "Minho";

    // teach: the `!` means `println!` is a MACRO, not a function (forget the `!`
    // teach: and you get E0423). Each `{}` in the format string is filled, in
    // teach: order, by the arguments after it — checked at compile time.
    println!("Hello, {}!", name);

    // teach: `.len()` is a method call on the string. It returns a `usize` —
    // teach: the length in BYTES, which equals characters only for plain ASCII.
    let name_len = name.len();

    // teach: since Rust 2021 you can also inline the variable: {name_len}.
    // teach: both forms are idiomatic; positional {} is what step 1 practices.
    println!("Your name is {} bytes long.", name_len);

    // The three errors the README has you trigger on purpose (kept here as a
    // reference — uncomment one at a time to see each again):
    //
    //     let broken = "no semicolon"        // error: expected `;` — syntax, no E-code
    //     println!("{}");                    // error: 1 positional argument in format
    //                                        //        string, but no arguments were given
    //     printl!("typo");                   // error: cannot find macro `printl` in this scope
}
