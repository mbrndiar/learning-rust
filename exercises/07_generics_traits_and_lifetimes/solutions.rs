//! Reference solutions for module 7.

fn largest<T: PartialOrd>(values: &[T]) -> Option<&T> {
    values
        .iter()
        .reduce(|left, right| if left >= right { left } else { right })
}

trait Label {
    fn label(&self) -> String;
}

struct User {
    name: String,
}

struct Project {
    name: String,
    active: bool,
}

impl Label for User {
    fn label(&self) -> String {
        format!("user: {}", self.name)
    }
}

impl Label for Project {
    fn label(&self) -> String {
        let state = if self.active { "active" } else { "inactive" };
        format!("project: {} [{state}]", self.name)
    }
}

fn render_labels(items: &[&dyn Label]) -> Vec<String> {
    items.iter().map(|item| item.label()).collect()
}

fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.chars().count() >= right.chars().count() {
        left
    } else {
        right
    }
}

fn main() {
    assert_eq!(largest(&[3, 9, 4]), Some(&9));
    let user = User {
        name: String::from("Ada"),
    };
    let project = Project {
        name: String::from("Compiler"),
        active: true,
    };
    assert_eq!(
        render_labels(&[&user, &project]),
        vec!["user: Ada", "project: Compiler [active]"]
    );
    assert_eq!(longest("Rust", "Ferris"), "Ferris");
    println!("Module 7 solutions passed.");
}
