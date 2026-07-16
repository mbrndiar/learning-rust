//! Task REST API applied project.
//!
//! The framework-neutral task domain and service are complete. Persistence,
//! HTTP, client, and CLI adapters remain explicit milestone placeholders.

pub mod api;
pub mod application;
pub mod cli;
pub mod client;
pub mod domain;
pub mod error;
pub mod server;
pub mod storage;

pub use application::{TaskRepository, TaskService};
pub use domain::{
    MAX_TITLE_LENGTH, Task, TaskFilter, TaskPatch, normalize_filter, normalize_patch,
    normalize_title, validate_id, validate_patch, validate_title,
};
pub use error::{TaskError, TaskResult};
