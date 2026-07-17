//! Task REST API applied project: one domain behind two native async HTTP servers.
//!
//! This crate is the completed reference solution. It manages tasks with three
//! fields (`id`, `title`, `completed`) behind Axum and Actix Web adapters that
//! both drive one framework-neutral core through a strict HTTP boundary, plus a
//! Reqwest [`client`] that speaks only the portable HTTP contract.
//!
//! # Dependency direction
//!
//! Dependencies point inward. [`core`] never imports a web framework or Reqwest.
//! [`server::storage`] adapters implement the [`TaskRepository`] trait,
//! [`server::api`] adapters translate inbound HTTP, and [`server`] owns backend
//! selection plus lifecycle. [`client`] contains the outbound Reqwest adapter and
//! CLI policy. The two binaries remain thin composition roots.

pub mod client;
pub mod core;
pub mod server;

pub use core::{
    AsyncTaskService, MAX_TITLE_LENGTH, Task, TaskApplication, TaskError, TaskFilter, TaskPatch,
    TaskRepository, TaskResult, TaskService, normalize_filter, normalize_patch, normalize_title,
    validate_id, validate_patch, validate_title,
};
