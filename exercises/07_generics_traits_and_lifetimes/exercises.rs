//! Exercises for module 7: generics, traits, trait objects, and lifetimes.
//!
//! Implement each `todo!()` body, then run the example tests. Do not change any
//! signature, including the trait definition and lifetime parameters.

/// Return a reference to the largest element of `values`, or `None` if empty.
///
/// Borrows the slice and returns a borrow into it; nothing is cloned. `T` only
/// needs to be comparable (`PartialOrd`).
pub fn largest<T: PartialOrd>(_values: &[T]) -> Option<&T> {
    todo!("return a reference to the largest item")
}

/// Types that can render a human-readable label for display.
pub trait Label {
    fn label(&self) -> String;
}

pub struct User {
    pub name: String,
}

pub struct Project {
    pub name: String,
    pub active: bool,
}

impl Label for User {
    /// Format as `"user: {name}"`.
    fn label(&self) -> String {
        todo!("format a user label")
    }
}

impl Label for Project {
    /// Format as `"project: {name} [active]"` or `"... [inactive]"` depending on
    /// `active`.
    fn label(&self) -> String {
        todo!("format a project label including its state")
    }
}

/// Return the label of every item, dispatching through the `Label` trait object.
pub fn render_labels(_items: &[&dyn Label]) -> Vec<String> {
    todo!("call the trait method for every item")
}

/// Return whichever of `left` or `right` has more Unicode characters.
///
/// Both inputs share the lifetime `'a`, so the returned borrow is valid as long
/// as both are.
pub fn longest<'a>(_left: &'a str, _right: &'a str) -> &'a str {
    todo!("return the text with more Unicode characters")
}

fn main() {
    println!("Run `cargo test --example ex-07-traits-lifetimes` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_largest_without_copying() {
        let values = [3, 9, 4];
        assert_eq!(largest(&values), Some(&9));
        assert_eq!(largest::<i32>(&[]), None);
    }

    #[test]
    fn dispatches_through_trait_objects() {
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
    }

    #[test]
    fn returns_a_borrowed_longest_value() {
        let owned = String::from("Ferris 🦀");
        assert_eq!(longest("Rust", &owned), "Ferris 🦀");
    }
}
