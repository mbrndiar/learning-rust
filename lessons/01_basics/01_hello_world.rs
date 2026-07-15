//! Lesson 1.1: program entry points and formatted output.
//!
//! Every executable Rust program starts at `fn main`. Text is written with the
//! `println!`/`eprintln!` macros, whose `{}` placeholders can name variables in
//! scope. The key idea: these are macros (note the `!`), expanded and
//! type-checked at compile time, not ordinary function calls.

fn main() {
    // `let` binds a value to a name. Types are inferred here: `&str` for the text
    // literal and `i32` (Rust's default integer type) for the number.
    let language = "Rust";
    let edition = 2024;

    // A `{name}` placeholder captures the in-scope binding `name` and formats it
    // with its `Display` representation.
    println!("Hello, {language}!");
    println!("This course uses Rust edition {edition}.");
    println!("Named values: language={language}, edition={edition}");

    // `eprintln!` writes to standard error instead of standard output, keeping
    // diagnostics separate from real output when the streams are redirected.
    eprintln!("Tip: compiler messages also use standard error.");
}
