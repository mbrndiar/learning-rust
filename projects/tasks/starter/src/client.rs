use std::time::Duration;

use reqwest::Client;

use crate::{Task, TaskError, TaskFilter, TaskPatch, TaskResult};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

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

    pub fn normalize_base_url(raw: &str) -> TaskResult<String> {
        let mut url = reqwest::Url::parse(raw).map_err(|_| {
            TaskError::client_configuration(
                "base-url",
                "base URL must be an absolute HTTP or HTTPS URL",
            )
        })?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(TaskError::client_configuration(
                "base-url",
                "base URL must be an absolute HTTP or HTTPS URL",
            ));
        }
        let path = url.path().trim_end_matches('/').to_owned();
        url.set_path(if path.is_empty() { "/" } else { &path });
        Ok(if path.is_empty() {
            url.as_str().trim_end_matches('/').to_owned()
        } else {
            url.to_string()
        })
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

    pub async fn get(&self, _id: i64) -> TaskResult<Task> {
        Err(TaskError::incomplete("Reqwest get"))
    }

    pub async fn update(&self, _id: i64, _patch: TaskPatch) -> TaskResult<Task> {
        Err(TaskError::incomplete("Reqwest update"))
    }

    pub async fn delete(&self, _id: i64) -> TaskResult<()> {
        Err(TaskError::incomplete("Reqwest delete"))
    }
}

pub fn normalize_base_url(raw: &str) -> TaskResult<String> {
    TaskClient::normalize_base_url(raw)
}
