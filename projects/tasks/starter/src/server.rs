use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, ValueEnum};

use crate::api::boundary::ErrorReporter;
use crate::{TaskError, TaskResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ServerKind {
    Axum,
    Actix,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum BackendKind {
    Sqlite,
    Markdown,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "tasks-api", about = "Run a local Task REST API")]
pub struct ServerConfig {
    #[arg(long, value_enum, default_value_t = ServerKind::Axum)]
    pub server: ServerKind,
    #[arg(long, value_enum, default_value_t = BackendKind::Sqlite)]
    pub backend: BackendKind,
    #[arg(long, default_value = "tasks.db")]
    pub data: PathBuf,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, default_value_t = 8000)]
    pub port: u16,
}

pub struct BoundServer {
    address: SocketAddr,
}

impl BoundServer {
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.address
    }

    pub async fn serve<F>(self, _shutdown: F) -> TaskResult<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Err(TaskError::incomplete("server lifecycle"))
    }
}

pub async fn bind(_config: ServerConfig) -> TaskResult<BoundServer> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}

pub async fn bind_with_reporter(
    _config: ServerConfig,
    _reporter: Arc<dyn ErrorReporter>,
) -> TaskResult<BoundServer> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}

pub async fn run_with_shutdown<F>(_config: ServerConfig, _shutdown: F) -> TaskResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    Err(TaskError::incomplete("server composition and lifecycle"))
}

pub async fn run(_config: ServerConfig) -> TaskResult<()> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}
