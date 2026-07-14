//! Lesson 1.1: program entry points and formatted output.

fn main() {
    let language = "Rust";
    let edition = 2024;

    println!("Hello, {language}!");
    println!("This course uses Rust edition {edition}.");
    println!("Named values: language={language}, edition={edition}");

    // `eprintln!` writes diagnostics to standard error instead of standard output.
    eprintln!("Tip: compiler messages also use standard error.");
}
