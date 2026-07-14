//! Lesson 7.2: lifetime relationships, borrowed structs, and trait objects.

fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.chars().count() >= right.chars().count() {
        left
    } else {
        right
    }
}

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
