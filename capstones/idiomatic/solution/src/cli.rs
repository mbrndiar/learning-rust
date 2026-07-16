//! Clap surface and terminal execution boundary.

use crate::IndexError;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// File indexer command-line arguments.
#[derive(Debug, Parser)]
#[command(about = "Build and query a deterministic local text index")]
pub struct Cli {
    /// Emit one structured JSON error on stderr.
    #[arg(long, global = true)]
    pub json_errors: bool,
    #[command(subcommand)]
    pub command: Command,
}

/// Observable file indexer commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    Index {
        #[arg(long)]
        index: PathBuf,
        #[arg(long, required = true)]
        root: Vec<String>,
        #[arg(long)]
        workers: Option<usize>,
        #[arg(long, default_value_t = 1_048_576)]
        max_bytes: u64,
        #[arg(long)]
        extension: Vec<String>,
    },
    Search {
        #[arg(long)]
        index: PathBuf,
        #[arg(long, required = true)]
        term: Vec<String>,
        #[arg(long)]
        path_prefix: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: usize,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    Stats {
        #[arg(long)]
        index: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

/// Supported success output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

/// Executes one parsed CLI command and returns its stdout payload.
pub fn execute(cli: Cli) -> Result<String, IndexError> {
    let _ = cli;
    Err(IndexError::incomplete("file indexer CLI execution"))
}
