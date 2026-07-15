//! Lesson 7.1: generic functions, structs, trait bounds, and defaults.
//!
//! Generics let one definition work for many types. Rust normally compiles a
//! concrete version for each used type (monomorphization), avoiding dynamic
//! dispatch. A trait bound such as `T: PartialOrd` states what a type must
//! support. Traits may also provide default methods that implementers override.

// `Point<T>` is generic: the same struct works for any single field type `T`.
#[derive(Debug)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

trait Summary {
    fn title(&self) -> &str;

    // A default method: implementers get this unless they override it.
    fn summarize(&self) -> String {
        format!("Read more: {}", self.title())
    }
}

struct Article {
    title: String,
    author: String,
}

impl Summary for Article {
    fn title(&self) -> &str {
        &self.title
    }

    // Overrides the default `summarize` with article-specific formatting.
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

// `T: PartialOrd` bounds the generic so values can be compared with `>=`.
fn largest<T: PartialOrd>(values: &[T]) -> Option<&T> {
    values
        .iter()
        .reduce(|left, right| if left >= right { left } else { right })
}

// `&impl Summary` accepts any type that implements the trait (static dispatch).
fn announce(item: &impl Summary) {
    println!("announcement: {}", item.summarize());
}

fn main() {
    let integer_point = Point { x: 3, y: 4 };
    let float_point = Point { x: 1.5, y: 2.5 };
    // Parentheses select the `x()` method; `.y` directly accesses the field.
    println!(
        "points: x={}, y={}; x={}, y={}",
        integer_point.x(),
        integer_point.y,
        float_point.x(),
        float_point.y
    );

    let values = [4, 9, 2, 9, 5];
    println!("largest={:?}", largest(&values));

    let article = Article {
        title: String::from("Traits communicate behavior"),
        author: String::from("Ferris"),
    };
    announce(&article);
}
