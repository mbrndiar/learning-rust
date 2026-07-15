//! Lesson 7.1: generic functions, structs, trait bounds, and defaults.

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

    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

fn largest<T: PartialOrd>(values: &[T]) -> Option<&T> {
    values
        .iter()
        .reduce(|left, right| if left >= right { left } else { right })
}

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
