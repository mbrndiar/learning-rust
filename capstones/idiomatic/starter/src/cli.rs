//! Milestones 2 and 5: Clap surface and deterministic terminal reports.
//!
//! This is the outermost application layer: it declares the argument grammar with
//! Clap, wires the concrete production capabilities (`StdFileTree`,
//! `CancellationToken`, `JsonFileIndexStore`) into the generic engine, and renders
//! results. It should perform no I/O policy of its own beyond formatting; every
//! error is an [`IndexError`] whose `exit_code` the binary turns into a process
//! status. Both success formats must be deterministic so output can be asserted
//! byte-for-byte. The grammar is provided; implement `execute` to the contract.

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
    /// Build and atomically publish a complete replacement index.
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
    /// Search for documents containing every exact normalized term.
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
    /// Report deterministic summary statistics.
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
pub fn execute(_cli: Cli) -> Result<String, IndexError> {
    todo!("milestone 5: validate, execute, and format each command")
}
