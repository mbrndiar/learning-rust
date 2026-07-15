//! Lesson 3.2: references, mutable borrows, and slices.
//!
//! Borrowing lends access to a value without transferring ownership. You may
//! hold many shared `&T` borrows at once, or one exclusive `&mut T`, but those
//! accesses may not overlap. A slice such as `&str` or `&[T]` borrows a
//! contiguous range of an existing value. Non-lexical lifetimes end a borrow at
//! its last use.

fn byte_length(text: &str) -> usize {
    // A shared `&str` borrow only reads; it never takes ownership of the text.
    text.len()
}

#[allow(clippy::ptr_arg)] // Growing the caller's owned String is the lesson.
fn append_period(text: &mut String) {
    // `&mut String` is an exclusive borrow: no other access to `text` may
    // overlap its use, which is what makes in-place mutation sound.
    if !text.ends_with('.') {
        text.push('.');
    }
}

fn first_word(text: &str) -> &str {
    // The returned slice borrows from `text`, so it stays valid only as long as
    // `text` does. `char_indices` yields byte offsets that fall on character
    // boundaries, so slicing here never splits a multi-byte character.
    for (index, character) in text.char_indices() {
        if character.is_whitespace() {
            return &text[..index];
        }
    }
    text
}

fn middle(values: &[i32]) -> &[i32] {
    // `&[T]` is a borrowed view into a contiguous run of elements; no copying.
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
