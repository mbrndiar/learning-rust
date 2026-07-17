//! Composition root (stubbed): wires a backend and a framework into a server.
//!
//! When implemented, `bind` opens the selected repository, builds the service,
//! and binds the listener up front so the caller learns the real local address
//! before serving. The two frameworks have different lifecycles: Axum can serve
//! directly on the Tokio runtime, while Actix needs its own `System`, so its
//! serving future must still stop the server and release resources if dropped.
//! This owns process wiring, not cross-process coordination.

pub mod api;
pub mod storage;

use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, ValueEnum};

use self::api::boundary::ErrorReporter;
use crate::{TaskError, TaskResult};

// Which HTTP framework serves the shared boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ServerKind {
    Axum,
    Actix,
}

// Which persistence backend the service uses.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum BackendKind {
    Sqlite,
    Markdown,
}

// Parsed server options: framework, backend, data path, and bind address.
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

/// A bound-but-not-yet-serving server: the port is reserved and the address is
/// known, so callers (and tests) can connect deterministically.
pub struct BoundServer {
    address: SocketAddr,
}

impl BoundServer {
    /// The actual bound address, including the OS-assigned port when `0` was
    /// requested.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.address
    }

    /// Serves until `shutdown` resolves.
    pub async fn serve<F>(self, _shutdown: F) -> TaskResult<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Err(TaskError::incomplete("server lifecycle"))
    }
}

/// Binds a server with the default error reporter.
pub async fn bind(_config: ServerConfig) -> TaskResult<BoundServer> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}

/// Binds a server with a caller-supplied reporter.
pub async fn bind_with_reporter(
    _config: ServerConfig,
    _reporter: Arc<dyn ErrorReporter>,
) -> TaskResult<BoundServer> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}

/// Binds and serves until `shutdown` resolves.
pub async fn run_with_shutdown<F>(_config: ServerConfig, _shutdown: F) -> TaskResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    Err(TaskError::incomplete("server composition and lifecycle"))
}

/// Binds and serves until an OS termination signal arrives.
pub async fn run(_config: ServerConfig) -> TaskResult<()> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}
