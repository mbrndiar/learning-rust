//! Compileable boundary scaffold for the Task REST API applied project.
//!
//! Phase 1 defines the public architecture only. Unfinished core operations fail
//! visibly through the same public [`TaskError`] categories as the completed
//! solution, transparently wrapped by [`ClientError`] or [`ServerError`] when they
//! cross an adapter boundary.

pub mod client;
pub mod core;
pub mod protocol;
pub mod server;

pub use client::{ClientError, ClientResult};
pub use core::{
    AsyncTaskService, MAX_TITLE_LENGTH, Task, TaskApplication, TaskError, TaskFilter, TaskPatch,
    TaskRepository, TaskResult, TaskService, normalize_filter, normalize_patch, normalize_title,
    validate_id, validate_patch, validate_title,
};
pub use server::{ServerError, ServerResult};
