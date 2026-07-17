use std::future::Future;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use ::axum::Router;
use clap::{Parser, ValueEnum};
use tokio::net::TcpListener;

use crate::api::axum as axum_adapter;
use crate::api::boundary::{ErrorReporter, StderrReporter};
use crate::storage::markdown::MarkdownRepository;
use crate::storage::sqlite::SqliteRepository;
use crate::{TaskError, TaskRepository, TaskResult, TaskService};

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
    listener: TcpListener,
    router: Router,
    address: SocketAddr,
}

impl BoundServer {
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.address
    }

    pub async fn serve<F>(self, shutdown: F) -> TaskResult<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        ::axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|error| TaskError::lifecycle("serve", error))
    }
}

pub async fn bind(config: ServerConfig) -> TaskResult<BoundServer> {
    bind_with_reporter(config, Arc::new(StderrReporter)).await
}

pub async fn bind_with_reporter(
    config: ServerConfig,
    reporter: Arc<dyn ErrorReporter>,
) -> TaskResult<BoundServer> {
    if config.server == ServerKind::Actix {
        return Err(TaskError::incomplete(
            "Actix Web server adapter (Milestone 5)",
        ));
    }
    let address = resolve_address(&config.host, config.port)?;
    let backend = config.backend;
    let data = config.data.clone();
    let repository = tokio::task::spawn_blocking(move || open_repository(backend, data))
        .await
        .map_err(|error| TaskError::internal("open task repository", error))??;
    let router = axum_adapter::router_with_reporter(TaskService::new(repository), reporter)?;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| TaskError::lifecycle("listen", error))?;
    let address = listener
        .local_addr()
        .map_err(|error| TaskError::lifecycle("read listener address", error))?;
    Ok(BoundServer {
        listener,
        router,
        address,
    })
}

pub async fn run_with_shutdown<F>(config: ServerConfig, shutdown: F) -> TaskResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    bind(config).await?.serve(shutdown).await
}

pub async fn run(config: ServerConfig) -> TaskResult<()> {
    run_with_shutdown(config, async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("tasks-api: failed to install Ctrl-C handler: {error}");
        }
    })
    .await
}

fn open_repository(backend: BackendKind, data: PathBuf) -> TaskResult<Arc<dyn TaskRepository>> {
    match backend {
        BackendKind::Sqlite => Ok(Arc::new(SqliteRepository::open(data)?)),
        BackendKind::Markdown => Ok(Arc::new(MarkdownRepository::open(data)?)),
    }
}

fn resolve_address(host: &str, port: u16) -> TaskResult<SocketAddr> {
    let ip = match host {
        "localhost" => IpAddr::from([127, 0, 0, 1]),
        value => value.parse::<IpAddr>().map_err(|_| {
            TaskError::lifecycle(
                "validate host",
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "host must be an IP address or localhost",
                ),
            )
        })?,
    };
    Ok(SocketAddr::new(ip, port))
}
