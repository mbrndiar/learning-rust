//! Lesson 3.1: moves, `Copy`, `Clone`, and ownership across calls.
//!
//! Every value has a single owner. Assigning or passing a non-`Copy` value such
//! as `String` *moves* ownership, so the source can no longer be used. Small
//! `Copy` types like integers are duplicated instead. `clone()` opts into an
//! explicit deep copy, and returning a value hands ownership back to the caller.

fn consume(text: String) -> usize {
    // Taking `String` by value moves ownership in; `text` is dropped when this
    // function returns, freeing its heap allocation.
    println!("consuming {text:?}");
    text.len()
}

fn add_suffix(mut text: String, suffix: &str) -> String {
    // `mut` on the parameter lets us grow the moved-in String and then move it
    // back out to the caller as the return value.
    text.push_str(suffix);
    text
}

fn main() {
    let original = String::from("ownership");
    // `String` is not `Copy`: this transfers ownership instead of duplicating
    // the heap allocation. `original` cannot be used after this line.
    let moved = original;
    println!("new owner: {moved}");

    // Clone only because this example needs two independently owned strings.
    let independent_copy = moved.clone();
    println!("explicit clone: {independent_copy}");

    // Integers implement `Copy`, so assignment leaves both bindings usable.
    let number = 42;
    let copied_number = number;
    println!("Copy values remain usable: {number}, {copied_number}");

    let length = consume(independent_copy);
    println!("consumed text had {length} bytes");

    let expanded = add_suffix(moved, " matters");
    println!("ownership returned to caller: {expanded}");
}
