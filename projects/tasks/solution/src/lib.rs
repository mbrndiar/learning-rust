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

pub use application::{TaskRepository, TaskService};
pub use domain::{Task, TaskFilter, TaskPatch};
pub use error::{TaskError, TaskResult};
