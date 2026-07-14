//! Lesson 4.1: structs, constructors, methods, and tuple structs.

#[derive(Debug, Clone, PartialEq)]
struct Book {
    title: String,
    author: String,
    pages: u32,
    checked_out: bool,
}

impl Book {
    fn new(title: &str, author: &str, pages: u32) -> Self {
        Self {
            title: title.to_owned(),
            author: author.to_owned(),
            pages,
            checked_out: false,
        }
    }

    fn description(&self) -> String {
        format!("{} by {} ({} pages)", self.title, self.author, self.pages)
    }

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
        ..book.clone()
    };
    println!("revised: {revised:?}");

    let distance = Meters(12.5);
    println!("distance={} m", distance.0);
}
