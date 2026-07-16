//! Command-line surface for the comparative capstone scaffold.

use crate::KvError;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Shared key/value command-line arguments.
#[derive(Debug, Parser)]
#[command(about = "Versioned SQLite-backed configuration store")]
pub struct Cli {
    /// Literal SQLite database path.
    #[arg(long)]
    pub db: PathBuf,
    #[command(subcommand)]
    pub command: CliCommand,
}

/// Four commands frozen by the shared specification.
#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Set {
        key: String,
        #[arg(long)]
        value_json: String,
        #[arg(long)]
        expect: Option<String>,
    },
    Get {
        key: String,
    },
    Delete {
        key: String,
        #[arg(long)]
        expect: Option<String>,
    },
    List,
}

/// Milestone 2 TODO: run a parsed command and return the exact stdout payload.
pub fn run(cli: Cli) -> Result<String, KvError> {
    let _ = cli;
    Err(KvError::incomplete("comparative CLI execution"))
}
