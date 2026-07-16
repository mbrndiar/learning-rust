use serde::Serialize;

use crate::{TaskError, TaskResult};

pub const MAX_TITLE_LENGTH: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Task {
    id: i64,
    title: String,
    completed: bool,
}

impl Task {
    pub fn from_parts(_id: i64, _title: impl Into<String>, _completed: bool) -> TaskResult<Self> {
        Err(TaskError::incomplete("validated Task construction"))
    }

    #[must_use]
    pub const fn id(&self) -> i64 {
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

pub fn normalize_title(_title: &str) -> TaskResult<String> {
    Err(TaskError::incomplete("title normalization"))
}

pub fn validate_title(_title: &str) -> TaskResult<()> {
    Err(TaskError::incomplete("title validation"))
}

pub fn validate_id(_id: i64) -> TaskResult<()> {
    Err(TaskError::incomplete("task ID validation"))
}

pub fn normalize_patch(_patch: TaskPatch) -> TaskResult<TaskPatch> {
    Err(TaskError::incomplete("task patch normalization"))
}

pub fn validate_patch(_patch: &TaskPatch) -> TaskResult<()> {
    Err(TaskError::incomplete("task patch validation"))
}

#[must_use]
pub const fn normalize_filter(filter: TaskFilter) -> TaskFilter {
    filter
}
