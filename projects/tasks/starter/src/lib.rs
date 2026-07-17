//! Compileable boundary scaffold for the Task REST API applied project.
//!
//! Phase 1 defines the public architecture only. Externally invoked project
//! operations return [`TaskError::Incomplete`] until their milestone is built.

pub mod client;
pub mod core;
pub mod server;

pub use core::{
    AsyncTaskService, MAX_TITLE_LENGTH, Task, TaskApplication, TaskError, TaskFilter, TaskPatch,
    TaskRepository, TaskResult, TaskService, normalize_filter, normalize_patch, normalize_title,
    validate_id, validate_patch, validate_title,
};
