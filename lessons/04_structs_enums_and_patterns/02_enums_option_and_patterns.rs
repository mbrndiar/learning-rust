//! Lesson 4.2: enums, `Option`, exhaustive patterns, and destructuring.

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn summarize(message: &Message) -> String {
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
    values.iter().copied().find(|value| value % 2 == 0)
}

fn print_even(values: &[i32]) {
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

    if let Message::Write(text) = &messages[1] {
        println!("the write payload has {} characters", text.chars().count());
    }

    print_even(&[1, 3, 8, 10]);
    print_even(&[1, 3, 5]);
}
