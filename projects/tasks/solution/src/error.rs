use thiserror::Error;

pub type TaskResult<T> = Result<T, TaskError>;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("incomplete project capability: {capability}")]
    Incomplete { capability: &'static str },
    #[error("validation error: {message}")]
    Validation { message: String },
    #[error("task {id} was not found")]
    NotFound { id: u64 },
    #[error("storage error: {message}")]
    Storage { message: String },
    #[error("transport error: {message}")]
    Transport { message: String },
    #[error("unexpected response: {message}")]
    Response { message: String },
}

impl TaskError {
    #[must_use]
    pub const fn incomplete(capability: &'static str) -> Self {
        Self::Incomplete { capability }
    }

    #[must_use]
    pub const fn incomplete_capability(&self) -> Option<&'static str> {
        match self {
            Self::Incomplete { capability } => Some(capability),
            _ => None,
        }
    }
}
