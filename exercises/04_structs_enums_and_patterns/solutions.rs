//! Reference solutions for module 4.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Priority {
    Low,
    Normal,
    High,
}

impl Priority {
    fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Task {
    title: String,
    priority: Priority,
    done: bool,
}

impl Task {
    fn new(title: &str, priority: Priority) -> Option<Self> {
        let title = title.trim();
        if title.is_empty() {
            return None;
        }
        Some(Self {
            title: title.to_owned(),
            priority,
            done: false,
        })
    }

    fn complete(&mut self) -> bool {
        if self.done {
            false
        } else {
            self.done = true;
            true
        }
    }
}

fn describe_task(task: Option<&Task>) -> String {
    match task {
        Some(task) => format!("[{}] {}", task.priority.label(), task.title),
        None => String::from("no task"),
    }
}

fn main() {
    assert_eq!(Priority::Low.label(), "low");
    assert_eq!(Priority::Normal.label(), "normal");
    let mut task = Task::new("  Ship  ", Priority::High).expect("valid task");
    assert_eq!(describe_task(Some(&task)), "[high] Ship");
    assert!(task.complete());
    assert!(!task.complete());
    assert_eq!(describe_task(None), "no task");
    println!("Module 4 solutions passed.");
}
