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
    pub fn from_parts(id: i64, title: impl Into<String>, completed: bool) -> TaskResult<Self> {
        let title = title.into();
        validate_id(id)?;
        validate_title(&title)?;
        Ok(Self {
            id,
            title,
            completed,
        })
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

pub fn normalize_title(title: &str) -> TaskResult<String> {
    let title = title.trim();
    let count = title.chars().count();
    if !(1..=MAX_TITLE_LENGTH).contains(&count) {
        return Err(TaskError::validation(
            "title",
            "title must contain between 1 and 120 characters",
        ));
    }
    for character in title.chars() {
        if is_physical_line_break(character) {
            return Err(TaskError::validation(
                "title",
                "title must occupy one physical line",
            ));
        }
        if character.is_control() {
            return Err(TaskError::validation(
                "title",
                "title must not contain control characters",
            ));
        }
    }
    Ok(title.to_owned())
}

pub fn validate_title(title: &str) -> TaskResult<()> {
    let normalized = normalize_title(title)?;
    if normalized != title {
        return Err(TaskError::validation(
            "title",
            "title must not have leading or trailing whitespace",
        ));
    }
    Ok(())
}

pub fn validate_id(id: i64) -> TaskResult<()> {
    if id <= 0 {
        return Err(TaskError::validation(
            "id",
            "task ID must be a positive integer",
        ));
    }
    Ok(())
}

pub fn normalize_patch(patch: TaskPatch) -> TaskResult<TaskPatch> {
    if patch.title.is_none() && patch.completed.is_none() {
        return Err(TaskError::validation(
            "update",
            "update must include title or completed",
        ));
    }

    let title = patch.title.as_deref().map(normalize_title).transpose()?;
    Ok(TaskPatch {
        title,
        completed: patch.completed,
    })
}

pub fn validate_patch(patch: &TaskPatch) -> TaskResult<()> {
    let normalized = normalize_patch(patch.clone())?;
    if normalized.title != patch.title {
        return Err(TaskError::validation(
            "title",
            "title must not have leading or trailing whitespace",
        ));
    }
    Ok(())
}

#[must_use]
pub const fn normalize_filter(filter: TaskFilter) -> TaskFilter {
    filter
}

const fn is_physical_line_break(character: char) -> bool {
    matches!(
        character,
        '\n' | '\u{000b}'
            | '\u{000c}'
            | '\r'
            | '\u{001c}'
            | '\u{001d}'
            | '\u{001e}'
            | '\u{0085}'
            | '\u{2028}'
            | '\u{2029}'
    )
}
