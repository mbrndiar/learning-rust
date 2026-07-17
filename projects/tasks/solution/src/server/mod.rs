//! Composition root: wires a backend and a framework into a running server.
//!
//! `bind` opens the selected repository, builds the [`TaskService`], and binds
//! the listener up front so the caller learns the real local address before
//! serving. The two frameworks have different lifecycles: Axum serves directly
//! on the Tokio runtime with graceful shutdown, while Actix runs its own
//! `System` on a dedicated thread and is stopped through a oneshot cancel
//! channel. `ActixWorker` guarantees that thread is signaled and joined even
//! if the serving future is dropped, so the listener and repository are always
//! released. This owns process wiring, not cross-process coordination.

pub mod api;
pub mod error;
pub mod storage;

pub use error::{ServerError, ServerResult};

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

use self::api::actix as actix_adapter;
use self::api::axum as axum_adapter;
use self::api::boundary::{ErrorReporter, StderrReporter};
use self::storage::markdown::MarkdownRepository;
use self::storage::sqlite::SqliteRepository;
use crate::{TaskRepository, TaskResult, TaskService};

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

// Framework-specific state captured at bind time; Actix needs a std listener
// because it runs outside the caller's Tokio runtime.
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

/// A bound-but-not-yet-serving server: the port is reserved and the address is
/// known, so callers (and tests) can connect deterministically.
pub struct BoundServer {
    inner: BoundServerInner,
    address: SocketAddr,
}

impl BoundServer {
    /// The actual bound address, including the OS-assigned port when `0` was
    /// requested.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.address
    }

    /// Serves until `shutdown` resolves, dispatching to the framework chosen at
    /// bind time.
    pub async fn serve<F>(self, shutdown: F) -> ServerResult<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        match self.inner {
            BoundServerInner::Axum { listener, router } => ::axum::serve(listener, router)
                .with_graceful_shutdown(shutdown)
                .await
                .map_err(|error| ServerError::lifecycle("serve Axum", error)),
            BoundServerInner::Actix {
                listener,
                service,
                reporter,
            } => serve_actix(listener, service, reporter, shutdown).await,
        }
    }
}

/// Binds a server with the default stderr reporter.
pub async fn bind(config: ServerConfig) -> ServerResult<BoundServer> {
    bind_with_reporter(config, Arc::new(StderrReporter)).await
}

/// Binds the listener and opens the repository, then captures framework state.
///
/// The listener is bound before serving so the caller can read the resolved
/// address; opening the repository (which touches the filesystem) runs on the
/// blocking pool to keep the async runtime responsive.
pub async fn bind_with_reporter(
    config: ServerConfig,
    reporter: Arc<dyn ErrorReporter>,
) -> ServerResult<BoundServer> {
    let requested = resolve_address(&config.host, config.port)?;
    let listener = TcpListener::bind(requested)
        .await
        .map_err(|error| ServerError::lifecycle("listen", error))?;
    let address = listener
        .local_addr()
        .map_err(|error| ServerError::lifecycle("read listener address", error))?;

    let backend = config.backend;
    let data = config.data;
    let repository = tokio::task::spawn_blocking(move || open_repository(backend, data))
        .await
        .map_err(|error| ServerError::internal("open task repository", error))??;
    let service = TaskService::new(repository);

    let inner = match config.server {
        ServerKind::Axum => BoundServerInner::Axum {
            listener,
            router: axum_adapter::router_with_reporter(service, reporter)?,
        },
        ServerKind::Actix => BoundServerInner::Actix {
            listener: listener
                .into_std()
                .map_err(|error| ServerError::lifecycle("prepare Actix listener", error))?,
            service,
            reporter,
        },
    };
    Ok(BoundServer { inner, address })
}

/// Binds and serves until `shutdown` resolves.
pub async fn run_with_shutdown<F>(config: ServerConfig, shutdown: F) -> ServerResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    bind(config).await?.serve(shutdown).await
}

/// Binds and serves until an OS termination signal arrives.
pub async fn run(config: ServerConfig) -> ServerResult<()> {
    run_with_shutdown(config, shutdown_signal()).await
}

// Runs Actix on its own thread with a private `System`, because Actix cannot
// share the caller's Tokio runtime. `shutdown` and a cancel channel both stop
// the server; the result is sent back over a oneshot and the thread is joined.
async fn serve_actix<F>(
    listener: std::net::TcpListener,
    service: TaskService,
    reporter: Arc<dyn ErrorReporter>,
    shutdown: F,
) -> ServerResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let thread = thread::Builder::new()
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
                .map_err(|error| ServerError::lifecycle("listen with Actix", error))?
                .run();
                let handle = server.handle();
                actix_web::rt::spawn(async move {
                    tokio::select! {
                        () = shutdown => {}
                        _ = cancelled => {}
                    }
                    handle.stop(true).await;
                });
                server
                    .await
                    .map_err(|error| ServerError::lifecycle("serve Actix", error))
            });
            let _ = sender.send(result);
        })
        .map_err(|error| ServerError::lifecycle("start Actix system", error))?;
    let mut worker = ActixWorker::new(cancel, thread);

    let result = receiver
        .await
        .map_err(|error| ServerError::lifecycle("receive Actix server result", error));
    worker.join()?;
    result?
}

// Owns the Actix thread and its cancel sender so that dropping the serving
// future still signals shutdown and joins the thread. `join` takes the sender
// (dropping it triggers cancellation) and waits; `Drop` is the fallback path.
struct ActixWorker {
    cancel: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl ActixWorker {
    const fn new(cancel: tokio::sync::oneshot::Sender<()>, thread: thread::JoinHandle<()>) -> Self {
        Self {
            cancel: Some(cancel),
            thread: Some(thread),
        }
    }

    fn join(&mut self) -> ServerResult<()> {
        self.cancel.take();
        let Some(thread) = self.thread.take() else {
            return Ok(());
        };
        thread.join().map_err(|_| {
            ServerError::lifecycle(
                "join Actix system",
                io::Error::other("Actix system thread panicked"),
            )
        })
    }
}

impl Drop for ActixWorker {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

// Waits for SIGTERM or Ctrl-C on Unix, or Ctrl-C elsewhere. Signal-handler
// installation failures are logged rather than aborting the server.
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

// Selects and opens the backend; runs on the blocking pool since both open
// paths touch the filesystem.
fn open_repository(backend: BackendKind, data: PathBuf) -> TaskResult<Arc<dyn TaskRepository>> {
    match backend {
        BackendKind::Sqlite => Ok(Arc::new(SqliteRepository::open(data)?)),
        BackendKind::Markdown => Ok(Arc::new(MarkdownRepository::open(data)?)),
    }
}

// Accepts only a literal IP or `localhost`; hostnames are not resolved, keeping
// the bind target unambiguous.
fn resolve_address(host: &str, port: u16) -> ServerResult<SocketAddr> {
    let ip = match host {
        "localhost" => IpAddr::from([127, 0, 0, 1]),
        value => value.parse::<IpAddr>().map_err(|_| {
            ServerError::lifecycle(
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

#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;
    use crate::{Task, TaskFilter, TaskPatch};

    struct DropRepository {
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropRepository {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl TaskRepository for DropRepository {
        fn create(&self, _title: &str) -> TaskResult<Task> {
            unreachable!("cancellation test does not call the repository")
        }

        fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
            unreachable!("cancellation test does not call the repository")
        }

        fn get(&self, _id: i64) -> TaskResult<Task> {
            unreachable!("cancellation test does not call the repository")
        }

        fn update(&self, _id: i64, _patch: TaskPatch) -> TaskResult<Task> {
            unreachable!("cancellation test does not call the repository")
        }

        fn delete(&self, _id: i64) -> TaskResult<()> {
            unreachable!("cancellation test does not call the repository")
        }
    }

    // Aborting the serving future (not a graceful shutdown) must still stop the
    // Actix thread: the listener closes and the repository drops exactly once.
    #[tokio::test(flavor = "multi_thread")]
    async fn aborting_actix_serve_closes_listener_and_drops_repository_once() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        listener
            .set_nonblocking(true)
            .expect("prepare test listener");
        let address = listener.local_addr().expect("test listener address");
        let drops = Arc::new(AtomicUsize::new(0));
        let repository: Arc<dyn TaskRepository> = Arc::new(DropRepository {
            drops: drops.clone(),
        });
        let service = TaskService::new(repository);
        let server = BoundServer {
            inner: BoundServerInner::Actix {
                listener,
                service,
                reporter: Arc::new(StderrReporter),
            },
            address,
        };
        let serve = tokio::spawn(server.serve(std::future::pending()));
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("build readiness client");
        let health_url = format!("http://{address}/health");

        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if http
                    .get(&health_url)
                    .send()
                    .await
                    .is_ok_and(|response| response.status().is_success())
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("Actix listener did not start");

        serve.abort();
        assert!(
            serve
                .await
                .expect_err("serve task must be cancelled")
                .is_cancelled()
        );
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(
            TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err(),
            "Actix listener remained open after cancellation"
        );
    }
}
