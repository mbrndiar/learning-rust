//! Reqwest client (stubbed): the portable HTTP boundary the CLI speaks through.
//!
//! When implemented, [`TaskClient`] must talk only the documented wire contract
//! so it works against either server and either backend, validate input with the
//! shared domain rules, send exactly one request per call (never retrying), and
//! decode responses strictly: check status first, then content type, then a
//! strict-JSON body with an exact set of fields. Transport failures must be
//! classified into timeout vs. other connection errors. The `normalize_base_url`
//! helper below is provided ready to use.

use std::time::Duration;

use reqwest::Client;

use crate::{ClientError, ClientResult, Task, TaskError, TaskFilter, TaskPatch};

/// Default request and connect timeout when none is configured.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A client bound to one base URL, reused across calls.
#[derive(Clone, Debug)]
pub struct TaskClient {
    http: Client,
    base_url: String,
    timeout: Duration,
}

impl TaskClient {
    /// Builds a client for `base_url` with the given timeout.
    pub fn new(_base_url: impl Into<String>, _timeout: Duration) -> ClientResult<Self> {
        Err(TaskError::incomplete("Reqwest client configuration").into())
    }

    /// Normalizes a base URL to a canonical form, or reports why it is unusable.
    ///
    /// Accepts only absolute http/https URLs with no credentials, query, or
    /// fragment, and trims a trailing slash so equivalent inputs agree.
    pub fn normalize_base_url(raw: &str) -> ClientResult<String> {
        let mut url = reqwest::Url::parse(raw).map_err(|_| {
            ClientError::configuration("base-url", "base URL must be an absolute HTTP or HTTPS URL")
        })?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(ClientError::configuration(
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

    /// Borrows the underlying Reqwest client.
    #[must_use]
    pub fn http(&self) -> &Client {
        &self.http
    }

    /// The normalized base URL this client targets.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The configured request/connect timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn create(&self, _title: &str) -> ClientResult<Task> {
        Err(TaskError::incomplete("Reqwest create").into())
    }

    pub async fn list(&self, _filter: TaskFilter) -> ClientResult<Vec<Task>> {
        Err(TaskError::incomplete("Reqwest list").into())
    }

    pub async fn get(&self, _id: i64) -> ClientResult<Task> {
        Err(TaskError::incomplete("Reqwest get").into())
    }

    pub async fn update(&self, _id: i64, _patch: TaskPatch) -> ClientResult<Task> {
        Err(TaskError::incomplete("Reqwest update").into())
    }

    pub async fn delete(&self, _id: i64) -> ClientResult<()> {
        Err(TaskError::incomplete("Reqwest delete").into())
    }
}

/// Free-function alias so callers can normalize a URL without a client.
pub fn normalize_base_url(raw: &str) -> ClientResult<String> {
    TaskClient::normalize_base_url(raw)
}
