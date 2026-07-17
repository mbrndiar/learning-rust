//! Compileable boundary scaffold for the Task REST API applied project.
//!
//! Phase 1 defines the public architecture only. Externally invoked project
//! operations return [`TaskError::Incomplete`] until their milestone is built.

pub mod api;
pub mod application;
pub mod cli;
pub mod client;
pub mod domain;
pub mod error;
pub mod server;
pub mod storage;

pub use application::{AsyncTaskService, TaskApplication, TaskRepository, TaskService};
pub use domain::{
    MAX_TITLE_LENGTH, Task, TaskFilter, TaskPatch, normalize_filter, normalize_patch,
    normalize_title, validate_id, validate_patch, validate_title,
};
pub use error::{TaskError, TaskResult};
