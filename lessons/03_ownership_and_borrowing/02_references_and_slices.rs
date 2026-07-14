//! Lesson 3.2: references, mutable borrows, and slices.

fn byte_length(text: &str) -> usize {
    text.len()
}

fn append_period(text: &mut String) {
    if !text.ends_with('.') {
        text.push('.');
    }
}

fn first_word(text: &str) -> &str {
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    &text[..end]
}

fn middle(values: &[i32]) -> &[i32] {
    if values.len() <= 2 {
        values
    } else {
        &values[1..values.len() - 1]
    }
}

fn main() {
    let mut sentence = String::from("Borrow only what you need");

    let first = first_word(&sentence);
    println!("first word={first:?}, bytes={}", byte_length(first));
    // `first` is not used after this point, so its shared borrow can end here.

    append_period(&mut sentence);
    println!("{sentence}");

    let numbers = [10, 20, 30, 40, 50];
    println!("middle slice: {:?}", middle(&numbers));

    let shared_one = &numbers;
    let shared_two = &numbers[1..];
    println!("multiple shared borrows: {shared_one:?} and {shared_two:?}");
}
