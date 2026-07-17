use std::future::Future;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use ::axum::Router;
use actix_web::http::KeepAlive;
use actix_web::{App, HttpServer};
use clap::{Parser, ValueEnum};
use tokio::net::TcpListener;

use crate::api::actix as actix_adapter;
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

enum BoundServerInner {
    Axum {
        listener: TcpListener,
        router: Router,
    },
    Actix {
        listener: std::net::TcpListener,
        service: TaskService,
        reporter: Arc<dyn ErrorReporter>,
    },
}

pub struct BoundServer {
    inner: BoundServerInner,
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
        match self.inner {
            BoundServerInner::Axum { listener, router } => ::axum::serve(listener, router)
                .with_graceful_shutdown(shutdown)
                .await
                .map_err(|error| TaskError::lifecycle("serve Axum", error)),
            BoundServerInner::Actix {
                listener,
                service,
                reporter,
            } => serve_actix(listener, service, reporter, shutdown).await,
        }
    }
}

pub async fn bind(config: ServerConfig) -> TaskResult<BoundServer> {
    bind_with_reporter(config, Arc::new(StderrReporter)).await
}

pub async fn bind_with_reporter(
    config: ServerConfig,
    reporter: Arc<dyn ErrorReporter>,
) -> TaskResult<BoundServer> {
    let requested = resolve_address(&config.host, config.port)?;
    let listener = TcpListener::bind(requested)
        .await
        .map_err(|error| TaskError::lifecycle("listen", error))?;
    let address = listener
        .local_addr()
        .map_err(|error| TaskError::lifecycle("read listener address", error))?;

    let backend = config.backend;
    let data = config.data;
    let repository = tokio::task::spawn_blocking(move || open_repository(backend, data))
        .await
        .map_err(|error| TaskError::internal("open task repository", error))??;
    let service = TaskService::new(repository);

    let inner = match config.server {
        ServerKind::Axum => BoundServerInner::Axum {
            listener,
            router: axum_adapter::router_with_reporter(service, reporter)?,
        },
        ServerKind::Actix => BoundServerInner::Actix {
            listener: listener
                .into_std()
                .map_err(|error| TaskError::lifecycle("prepare Actix listener", error))?,
            service,
            reporter,
        },
    };
    Ok(BoundServer { inner, address })
}

pub async fn run_with_shutdown<F>(config: ServerConfig, shutdown: F) -> TaskResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    bind(config).await?.serve(shutdown).await
}

pub async fn run(config: ServerConfig) -> TaskResult<()> {
    run_with_shutdown(config, shutdown_signal()).await
}

async fn serve_actix<F>(
    listener: std::net::TcpListener,
    service: TaskService,
    reporter: Arc<dyn ErrorReporter>,
    shutdown: F,
) -> TaskResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let worker = thread::Builder::new()
        .name("tasks-actix-system".to_owned())
        .spawn(move || {
            let result = actix_web::rt::System::new().block_on(async move {
                let service_factory = service.clone();
                let reporter_factory = reporter.clone();
                let server = HttpServer::new(move || {
                    App::new().service(actix_adapter::scope_with_reporter(
                        service_factory.clone(),
                        reporter_factory.clone(),
                    ))
                })
                .disable_signals()
                .keep_alive(KeepAlive::Disabled)
                .shutdown_timeout(1)
                .listen(listener)
                .map_err(|error| TaskError::lifecycle("listen with Actix", error))?
                .run();
                let handle = server.handle();
                actix_web::rt::spawn(async move {
                    shutdown.await;
                    handle.stop(true).await;
                });
                server
                    .await
                    .map_err(|error| TaskError::lifecycle("serve Actix", error))
            });
            let _ = sender.send(result);
        })
        .map_err(|error| TaskError::lifecycle("start Actix system", error))?;

    let result = receiver
        .await
        .map_err(|error| TaskError::lifecycle("receive Actix server result", error));
    worker.join().map_err(|_| {
        TaskError::lifecycle(
            "join Actix system",
            io::Error::other("Actix system thread panicked"),
        )
    })?;
    result?
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    result = tokio::signal::ctrl_c() => {
                        if let Err(error) = result {
                            eprintln!("tasks-api: failed to install Ctrl-C handler: {error}");
                        }
                    }
                    _ = terminate.recv() => {}
                }
            }
            Err(error) => {
                eprintln!("tasks-api: failed to install SIGTERM handler: {error}");
                if let Err(error) = tokio::signal::ctrl_c().await {
                    eprintln!("tasks-api: failed to install Ctrl-C handler: {error}");
                }
            }
        }
    }
    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("tasks-api: failed to install Ctrl-C handler: {error}");
    }
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
