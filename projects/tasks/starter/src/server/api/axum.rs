//! Axum adapter (stubbed): translates Axum requests into
//! [`crate::server::api::boundary::HttpBoundary`] calls.
//!
//! When implemented, this layer makes no policy decisions: handlers pull the
//! raw query, content type, path ID, and bounded body from Axum extractors,
//! hand them to the boundary, and turn the returned response back into an Axum
//! response, registering an explicit method handler plus a fallback per route.

use std::sync::Arc;

use axum::Router;

use super::boundary::ErrorReporter;
use crate::{ServerResult, TaskError, TaskService};

/// Builds the router with the default error reporter.
pub fn router(_service: TaskService) -> ServerResult<Router> {
    Err(TaskError::incomplete("Axum routes").into())
}

/// Builds the router with a caller-supplied reporter (useful in tests).
pub fn router_with_reporter(
    _service: TaskService,
    _reporter: Arc<dyn ErrorReporter>,
) -> ServerResult<Router> {
    Err(TaskError::incomplete("Axum routes").into())
}
