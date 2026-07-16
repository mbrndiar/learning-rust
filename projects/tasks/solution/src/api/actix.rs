use actix_web::Scope;

use crate::{TaskError, TaskResult, TaskService};

pub fn scope(_service: TaskService) -> TaskResult<Scope> {
    Err(TaskError::incomplete("Actix Web routes"))
}
