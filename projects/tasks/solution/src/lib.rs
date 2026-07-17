//! Task REST API applied project: one domain behind two native async HTTP servers.
//!
//! This crate is the completed reference solution. It manages tasks with three
//! fields (`id`, `title`, `completed`) behind Axum and Actix Web adapters that
//! both drive one framework-neutral core through a strict HTTP boundary, plus a
//! Reqwest [`client`] that speaks only the portable HTTP contract.
//!
//! # Dependency direction
//!
//! Dependencies point inward. [`domain`], [`error`], and [`application`] never
//! import a web framework or Reqwest. [`storage`] adapters implement the
//! [`TaskRepository`] trait, [`api`] adapters translate inbound HTTP at the
//! boundary, and [`server`] plus the two binaries are the composition roots that
//! wire a concrete repository to a chosen server. Axum and Actix Web stay
//! separate adapters so each framework's native routing, state, and lifecycle
//! remain visible rather than hidden behind a home-grown universal router.

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
