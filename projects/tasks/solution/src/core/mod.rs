//! Framework-neutral task rules, errors, ports, and application services.
//!
//! Nothing in this layer imports an HTTP framework, Reqwest, SQLite, or file
//! persistence. Server and client adapters depend on these shared types, while
//! dependencies never point back out to either adapter layer.

pub mod application;
pub mod domain;
pub mod error;

pub use application::{AsyncTaskService, TaskApplication, TaskRepository, TaskService};
pub use domain::{
    MAX_TITLE_LENGTH, Task, TaskFilter, TaskPatch, normalize_filter, normalize_patch,
    normalize_title, validate_id, validate_patch, validate_title,
};
pub use error::{TaskError, TaskResult};
