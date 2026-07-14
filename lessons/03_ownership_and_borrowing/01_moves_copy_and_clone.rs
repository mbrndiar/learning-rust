//! Lesson 3.1: moves, `Copy`, `Clone`, and ownership across calls.

fn consume(text: String) -> usize {
    println!("consuming {text:?}");
    text.len()
}

fn add_suffix(mut text: String, suffix: &str) -> String {
    text.push_str(suffix);
    text
}

fn main() {
    let original = String::from("ownership");
    let moved = original;
    println!("new owner: {moved}");

    let independent_copy = moved.clone();
    println!("explicit clone: {independent_copy}");

    let number = 42;
    let copied_number = number;
    println!("Copy values remain usable: {number}, {copied_number}");

    let length = consume(independent_copy);
    println!("consumed text had {length} bytes");

    let expanded = add_suffix(moved, " matters");
    println!("ownership returned to caller: {expanded}");
}
