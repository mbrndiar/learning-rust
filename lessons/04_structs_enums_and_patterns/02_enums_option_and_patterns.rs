//! Lesson 4.2: enums, `Option`, exhaustive patterns, and destructuring.
//!
//! An `enum` is a type that is exactly one of several variants, each able to
//! carry its own data. `match` destructures those variants and must be
//! exhaustive. `Option<T>` is the standard enum for "a value or nothing" and is
//! how Rust models absence without null. `if let` / `let else` match one case.

#[derive(Debug)]
enum Message {
    Quit,                    // a unit variant carries no data
    Move { x: i32, y: i32 }, // a struct-like variant with named fields
    Write(String),           // a tuple variant carrying one value
    ChangeColor(u8, u8, u8), // a tuple variant carrying three values
}

fn summarize(message: &Message) -> String {
    // More specific patterns come first: `Move { x: 0, y: 0 }` is tried before
    // the general `Move { x, y }`. A guard (`if text.is_empty()`) refines an arm.
    match message {
        Message::Quit => "quit".to_owned(),
        Message::Move { x: 0, y: 0 } => "already at origin".to_owned(),
        Message::Move { x, y } => format!("move to ({x}, {y})"),
        Message::Write(text) if text.is_empty() => "empty message".to_owned(),
        Message::Write(text) => format!("write {text:?}"),
        Message::ChangeColor(red, green, blue) => {
            format!("rgb({red}, {green}, {blue})")
        }
    }
}

fn first_even(values: &[i32]) -> Option<i32> {
    // `find` returns `Some(value)` for the first match, or `None` if none match.
    values.iter().copied().find(|value| value % 2 == 0)
}

fn print_even(values: &[i32]) {
    // `let ... else` binds when the pattern matches; otherwise the `else` block
    // must diverge (here by returning), so `value` is available afterwards.
    let Some(value) = first_even(values) else {
        println!("no even value");
        return;
    };
    println!("first even value={value}");
}

fn main() {
    let messages = [
        Message::Move { x: 3, y: 4 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(20, 40, 60),
        Message::Quit,
    ];

    for message in &messages {
        println!("{message:?} -> {}", summarize(message));
    }

    // `if let` matches a single variant and quietly ignores the rest.
    if let Message::Write(text) = &messages[1] {
        println!("the write payload has {} characters", text.chars().count());
    }

    print_even(&[1, 3, 8, 10]);
    print_even(&[1, 3, 5]);
}
