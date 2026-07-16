use serde::{Deserialize, Serialize};

use crate::{TaskError, TaskResult};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Task {
    id: u64,
    title: String,
    completed: bool,
}

impl Task {
    pub fn from_parts(_id: u64, _title: impl Into<String>, _completed: bool) -> TaskResult<Self> {
        Err(TaskError::incomplete("validated Task construction"))
    }

    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub const fn completed(&self) -> bool {
        self.completed
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TaskFilter {
    pub completed: Option<bool>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaskPatch {
    pub title: Option<String>,
    pub completed: Option<bool>,
}
