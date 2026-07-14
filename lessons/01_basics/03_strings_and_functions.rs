//! Lesson 1.3: UTF-8 strings, functions, statements, and expressions.

fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

fn rectangle_area(width: u32, height: u32) -> u32 {
    width * height
}

fn classify_length(text: &str) -> &'static str {
    let characters = text.chars().count();
    if characters < 5 { "short" } else { "long" }
}

fn main() {
    let literal: &str = "borrowed text";
    let mut owned = String::from("Rust");
    owned.push_str(" is expressive");
    owned.push('!');

    println!("{literal}");
    println!("{owned}");
    println!("{}", greeting(&owned));
    println!("area={}", rectangle_area(4, 3));

    let unicode = "Dobrý den 🦀";
    println!(
        "{unicode:?}: {} bytes, {} characters, {}",
        unicode.len(),
        unicode.chars().count(),
        classify_length(unicode)
    );

    let doubled = {
        let value = 21;
        value * 2
    };
    println!("block expression result={doubled}");
}
