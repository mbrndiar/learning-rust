//! Exercises for module 4.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
}

impl Priority {
    pub fn label(self) -> &'static str {
        todo!("return a label for every variant")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    title: String,
    priority: Priority,
    done: bool,
}

impl Task {
    pub fn new(_title: &str, _priority: Priority) -> Option<Self> {
        todo!("trim and reject an empty title")
    }

    pub fn complete(&mut self) -> bool {
        todo!("return true only when the task changes from pending to done")
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

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
