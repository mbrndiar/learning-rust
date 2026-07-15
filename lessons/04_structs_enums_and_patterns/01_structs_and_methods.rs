//! Lesson 4.1: structs, constructors, methods, and tuple structs.
//!
//! A `struct` groups named fields into one type. Methods live in an `impl`
//! block: `&self` borrows the receiver to read, `&mut self` borrows it to
//! mutate, and an associated function like `new` (no `self`) acts as a
//! constructor. `#[derive]` generates common trait impls such as `Debug`.

// `#[derive(...)]` asks the compiler to generate these trait implementations so
// the type can be printed with `{:?}`, duplicated, and compared for equality.
#[derive(Debug, Clone, PartialEq)]
struct Book {
    title: String,
    author: String,
    pages: u32,
    checked_out: bool,
}

impl Book {
    // An associated function has no `self` receiver; by convention `new` builds a
    // value. `Self` is shorthand for the surrounding type (`Book`).
    fn new(title: &str, author: &str, pages: u32) -> Self {
        Self {
            // `to_owned` copies the borrowed `&str` into an owned `String` field.
            title: title.to_owned(),
            author: author.to_owned(),
            pages,
            checked_out: false,
        }
    }

    // `&self` borrows the receiver to read its fields without consuming it.
    fn description(&self) -> String {
        format!("{} by {} ({} pages)", self.title, self.author, self.pages)
    }

    // `&mut self` is an exclusive borrow, so the method can change state.
    fn check_out(&mut self) -> bool {
        if self.checked_out {
            false
        } else {
            self.checked_out = true;
            true
        }
    }

    fn is_long(&self) -> bool {
        self.pages >= 400
    }
}

// A tuple struct has positional fields accessed by index (`.0`) rather than name.
#[derive(Debug)]
struct Meters(f64);

fn main() {
    let mut book = Book::new("The Rust Journey", "Ferris Crab", 420);
    println!("{}", book.description());
    println!("long book? {}", book.is_long());
    println!("first checkout succeeded? {}", book.check_out());
    println!("second checkout succeeded? {}", book.check_out());

    let revised = Book {
        pages: 450,
        checked_out: false,
        // Struct update syntax fills the remaining fields from another value.
        ..book.clone()
    };
    println!("revised: {revised:?}");

    let distance = Meters(12.5);
    println!("distance={} m", distance.0);
}
