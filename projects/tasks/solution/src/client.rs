use std::time::Duration;

use reqwest::Client;

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskResult};

#[derive(Clone, Debug)]
pub struct TaskClient {
    http: Client,
    base_url: String,
    timeout: Duration,
}

impl TaskClient {
    pub fn new(_base_url: impl Into<String>, _timeout: Duration) -> TaskResult<Self> {
        Err(TaskError::incomplete("Reqwest client configuration"))
    }

    #[must_use]
    pub fn http(&self) -> &Client {
        &self.http
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn create(&self, _title: &str) -> TaskResult<Task> {
        Err(TaskError::incomplete("Reqwest create"))
    }

    pub async fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
        Err(TaskError::incomplete("Reqwest list"))
    }

    pub async fn get(&self, _id: u64) -> TaskResult<Task> {
        Err(TaskError::incomplete("Reqwest get"))
    }

    pub async fn update(&self, _id: u64, _patch: TaskPatch) -> TaskResult<Task> {
        Err(TaskError::incomplete("Reqwest update"))
    }

    pub async fn delete(&self, _id: u64) -> TaskResult<()> {
        Err(TaskError::incomplete("Reqwest delete"))
    }
}
