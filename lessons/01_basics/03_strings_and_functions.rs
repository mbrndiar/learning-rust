//! Lesson 1.3: UTF-8 strings, functions, statements, and expressions.
//!
//! Rust is expression-oriented: a function's final expression, written without a
//! trailing semicolon, is its return value. `&str` borrows text (including
//! literals); `String` owns growable, heap-allocated UTF-8. Because text is
//! UTF-8, byte length and character count differ, so strings are not indexable.

fn greeting(name: &str) -> String {
    // `format!` builds a new owned `String`. It is the tail expression, so it is
    // returned without a `return` keyword or a trailing semicolon.
    format!("Hello, {name}!")
}

fn rectangle_area(width: u32, height: u32) -> u32 {
    width * height // tail expression: the function's return value
}

fn classify_length(text: &str) -> &'static str {
    let characters = text.chars().count(); // count characters, not bytes
    // `if` is an expression; each arm yields the value the function returns.
    if characters < 5 { "short" } else { "long" }
}

fn main() {
    // `&str` borrows existing text; a literal is baked into the compiled binary.
    let literal: &str = "borrowed text";
    // `String` owns heap-allocated, growable UTF-8 text.
    let mut owned = String::from("Rust");
    owned.push_str(" is expressive"); // append a &str
    owned.push('!'); // append a single char

    println!("{literal}");
    println!("{owned}");
    // `&owned` lends the String to a function that only needs to read it.
    println!("{}", greeting(&owned));
    println!("area={}", rectangle_area(4, 3));

    let unicode = "Dobrý den 🦀";
    // `len()` is the UTF-8 byte count; `chars().count()` counts Unicode scalar
    // values. They differ whenever the text contains non-ASCII characters.
    println!(
        "{unicode:?}: {} bytes, {} characters, {}",
        unicode.len(),
        unicode.chars().count(),
        classify_length(unicode)
    );

    // A block is itself an expression: its final expression becomes the block's
    // value, so `doubled` is 42.
    let doubled = {
        let value = 21;
        value * 2
    };
    println!("block expression result={doubled}");
}
