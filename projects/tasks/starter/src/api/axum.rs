use axum::Router;

use crate::{TaskError, TaskResult, TaskService};

pub fn router(_service: TaskService) -> TaskResult<Router> {
    Err(TaskError::incomplete("Axum routes"))
}
