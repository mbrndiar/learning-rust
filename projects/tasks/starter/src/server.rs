use std::path::PathBuf;

use clap::{Parser, ValueEnum};

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

#[derive(Debug, Parser)]
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

pub async fn run(_config: ServerConfig) -> TaskResult<()> {
    Err(TaskError::incomplete("server composition and lifecycle"))
}
