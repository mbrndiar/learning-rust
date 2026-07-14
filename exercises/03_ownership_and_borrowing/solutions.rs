//! Reference solutions for module 3.

fn first_word(text: &str) -> &str {
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    &text[..end]
}

fn append_period(text: &mut String) {
    if !text.ends_with('.') {
        text.push('.');
    }
}

fn total_byte_length(values: &[String]) -> usize {
    values.iter().map(String::len).sum()
}

fn into_uppercase(text: String) -> String {
    text.to_uppercase()
}

fn main() {
    let sentence = String::from("borrow precisely");
    assert_eq!(first_word(&sentence), "borrow");

    let mut text = String::from("Rust");
    append_period(&mut text);
    append_period(&mut text);
    assert_eq!(text, "Rust.");

    let values = vec![String::from("ab"), String::from("rust")];
    assert_eq!(total_byte_length(&values), 6);
    assert_eq!(into_uppercase(String::from("move me")), "MOVE ME");
    println!("Module 3 solutions passed.");
}
