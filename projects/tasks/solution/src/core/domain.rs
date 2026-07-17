//! Task domain values and the validation rules every layer shares.
//!
//! These types are the storage- and transport-independent vocabulary of the
//! project. [`Task`] keeps its fields private so a value can only exist once it
//! has passed validation, and the free `normalize_*`/`validate_*` functions are
//! the single source of truth for the title, ID, patch, and filter rules from
//! `docs/SPEC.md`. Concentrating the rules here keeps every adapter honest: no
//! route or repository is allowed to reimplement them.

use serde::Serialize;

use crate::{TaskError, TaskResult};

/// Maximum number of Unicode characters allowed in a trimmed title.
pub const MAX_TITLE_LENGTH: usize = 120;

/// A validated task: a positive `id`, a normalized `title`, and a completion flag.
///
/// Fields are private and exposed through read-only accessors, so the only way
/// to obtain a `Task` is through construction that enforces the domain rules.
/// The `Serialize` derive produces the `id`, `title`, and `completed` JSON fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Task {
    id: i64,
    title: String,
    completed: bool,
}

impl Task {
    /// Builds a `Task` from already-persisted parts, rejecting an invalid ID or
    /// title. Storage adapters use this when mapping a database row or Markdown
    /// line back into a domain value, so corrupt data cannot enter the core.
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

    /// The positive, repository-allocated identifier.
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// The normalized title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Whether the task is completed.
    #[must_use]
    pub const fn completed(&self) -> bool {
        self.completed
    }
}

/// Optional completion filter for listing tasks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TaskFilter {
    /// `Some(state)` keeps only tasks in that completion state; `None` lists all.
    pub completed: Option<bool>,
}

/// A partial update: each `Some` field is applied and each `None` field is left
/// unchanged. An update with no fields set is rejected by [`normalize_patch`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaskPatch {
    /// New title, if the caller is changing it.
    pub title: Option<String>,
    /// New completion state, if the caller is changing it.
    pub completed: Option<bool>,
}

/// Trims a title and enforces the 1–120 character, single-line, no-control rule,
/// returning the normalized owned string. This is the canonical form stored and
/// echoed everywhere.
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

/// Confirms a title is already in normalized form. Persisted or echoed titles
/// must equal their normalized value, so a stored title with surrounding
/// whitespace is treated as corruption rather than silently trimmed.
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

/// Rejects a non-positive task ID. IDs are positive integers allocated by the
/// repository.
pub fn validate_id(id: i64) -> TaskResult<()> {
    if id <= 0 {
        return Err(TaskError::validation(
            "id",
            "task ID must be a positive integer",
        ));
    }
    Ok(())
}

/// Normalizes a patch: an empty update (no fields) is invalid, and a present
/// title is normalized while `completed` passes through unchanged.
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

/// Confirms a patch is already normalized, rejecting a title that carries
/// surrounding whitespace.
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

/// Filters carry no rules today; this hook keeps the layered API symmetric with
/// the title, ID, and patch normalizers.
#[must_use]
pub const fn normalize_filter(filter: TaskFilter) -> TaskFilter {
    filter
}

// The set of characters that would split a title across physical lines: ASCII
// line breaks plus the Unicode vertical/line/paragraph separators.
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
