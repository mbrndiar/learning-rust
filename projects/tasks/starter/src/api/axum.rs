use std::sync::Arc;

use axum::Router;

use crate::api::boundary::ErrorReporter;
use crate::{TaskError, TaskResult, TaskService};

pub fn router(_service: TaskService) -> TaskResult<Router> {
    Err(TaskError::incomplete("Axum routes"))
}

pub fn router_with_reporter(
    _service: TaskService,
    _reporter: Arc<dyn ErrorReporter>,
) -> TaskResult<Router> {
    Err(TaskError::incomplete("Axum routes"))
}
