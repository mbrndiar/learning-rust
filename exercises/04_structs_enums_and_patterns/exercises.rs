//! Exercises for module 4: structs, enums, methods, and patterns.
//!
//! Implement each `todo!()` body, then run the example tests. Do not change any
//! signature or the provided field layout.

/// A task's priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
}

impl Priority {
    /// Return the lowercase label for this priority (`"low"`, `"normal"`,
    /// `"high"`). Every variant must be covered.
    pub fn label(self) -> &'static str {
        todo!("return a label for every variant")
    }
}

/// A to-do item with a non-empty title, a priority, and a completion flag.
#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    title: String,
    priority: Priority,
    done: bool,
}

impl Task {
    /// Build a task, trimming surrounding whitespace from `title`.
    ///
    /// Returns `None` when the title is empty after trimming. New tasks start
    /// out not done.
    pub fn new(_title: &str, _priority: Priority) -> Option<Self> {
        todo!("trim and reject an empty title")
    }

    /// Mark the task done, returning `true` only on the transition from pending
    /// to done (and `false` if it was already done).
    pub fn complete(&mut self) -> bool {
        todo!("return true only when the task changes from pending to done")
    }

    /// Borrow the task's title.
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// Describe an optional task as `"[{priority}] {title}"`, or `"no task"` for
/// `None`.
pub fn describe_task(_task: Option<&Task>) -> String {
    todo!("describe Some task or return no task")
}

fn main() {
    println!("Run `cargo test --example ex-04-domain-types` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_trims_tasks() {
        let task = Task::new("  Learn enums  ", Priority::High).expect("valid task");
        assert_eq!(task.title(), "Learn enums");
        assert!(Task::new("   ", Priority::Low).is_none());
    }

    #[test]
    fn completion_reports_state_change() {
        let mut task = Task::new("Test", Priority::Normal).expect("valid task");
        assert!(task.complete());
        assert!(!task.complete());
    }

    #[test]
    fn covers_every_priority_and_option_state() {
        assert_eq!(Priority::Low.label(), "low");
        assert_eq!(Priority::Normal.label(), "normal");
        assert_eq!(Priority::High.label(), "high");
        assert_eq!(describe_task(None), "no task");
        let task = Task::new("Ship", Priority::High).expect("valid task");
        assert_eq!(describe_task(Some(&task)), "[high] Ship");
    }
}
