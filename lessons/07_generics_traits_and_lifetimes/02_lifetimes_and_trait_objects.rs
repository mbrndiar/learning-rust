//! Lesson 7.2: lifetime relationships, borrowed structs, and trait objects.
//!
//! A lifetime parameter like `'a` describes how long a borrow is valid and ties
//! outputs to inputs; it never extends how long data actually lives. A struct
//! may hold a borrow if it declares that lifetime. `Box<dyn Trait>` is a trait
//! object: it erases the concrete type and dispatches methods dynamically.

fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    // `'a` connects both possible input sources to the borrowed output. It does
    // not extend either string's real lifetime.
    if left.chars().count() >= right.chars().count() {
        left
    } else {
        right
    }
}

// `Excerpt` borrows text it does not own, so it must name the lifetime `'a`; the
// struct cannot outlive the string it points into.
#[derive(Debug)]
struct Excerpt<'a> {
    text: &'a str,
}

trait Draw {
    fn draw(&self) -> String;
}

struct Button {
    label: String,
}

struct TextField {
    placeholder: String,
}

impl Draw for Button {
    fn draw(&self) -> String {
        format!("[ {} ]", self.label)
    }
}

impl Draw for TextField {
    fn draw(&self) -> String {
        format!("<{}>", self.placeholder)
    }
}

fn render(widgets: &[Box<dyn Draw>]) {
    // Each box may contain another concrete type; `dyn Draw` keeps only the
    // shared behavior needed by this loop and dispatches `draw` at runtime.
    for widget in widgets {
        println!("{}", widget.draw());
    }
}

fn main() {
    let sentence = String::from("Borrowed data can live inside a struct.");
    let excerpt = Excerpt {
        text: &sentence[..13],
    };
    println!("{excerpt:?}: {}", excerpt.text);

    let left = "short";
    let right = String::from("a longer string");
    println!("longest={:?}", longest(left, &right));

    // A heterogeneous collection: `Vec<Box<dyn Draw>>` stores different types
    // behind a common trait object.
    let widgets: Vec<Box<dyn Draw>> = vec![
        Box::new(Button {
            label: String::from("Save"),
        }),
        Box::new(TextField {
            placeholder: String::from("Name"),
        }),
    ];
    render(&widgets);
}
