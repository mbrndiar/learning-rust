//! Actix adapter (stubbed): the same boundary translation as the Axum adapter,
//! expressed in Actix terms.
//!
//! When implemented, this returns an Actix [`Scope`] whose handlers dispatch on
//! the request method, stream and cap the body, call the shared boundary, and
//! rebuild its response, so the Actix server stays interchangeable with Axum.

use actix_web::Scope;

use crate::{TaskError, TaskResult, TaskService};

/// Builds the Actix scope with the default error reporter.
pub fn scope(_service: TaskService) -> TaskResult<Scope> {
    Err(TaskError::incomplete("Actix Web routes"))
}
