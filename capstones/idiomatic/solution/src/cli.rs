//! Clap surface, orchestration, and deterministic terminal reports.

use crate::build::CancellationToken;
use crate::query::search;
use crate::storage::JsonFileIndexStore;
use crate::{
    ErrorCode, IndexBuilder, IndexError, IndexSettings, IndexStats, IndexStore, SearchQuery,
    SearchResult, StdFileTree, validate_roots,
};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
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

#[derive(Serialize)]
struct IndexReport {
    index: String,
    documents: usize,
    issues: usize,
    unique_terms: usize,
}

/// Executes one parsed CLI command and returns its stdout payload.
pub fn execute(cli: Cli) -> Result<String, IndexError> {
    match cli.command {
        Command::Index {
            index,
            root,
            workers,
            max_bytes,
            extension,
        } => {
            let workers = validate_workers(workers)?;
            let roots = validate_roots(&root)?;
            let settings = IndexSettings::new(extension, max_bytes)?;
            let built = IndexBuilder::new(StdFileTree, workers, CancellationToken::new())
                .build(&roots, &settings)?;
            JsonFileIndexStore::new(&index).replace(&built)?;
            let unique_terms = built
                .documents
                .iter()
                .flat_map(|document| document.terms.iter().map(|term| term.term.as_str()))
                .collect::<BTreeSet<_>>()
                .len();
            json_string(&IndexReport {
                index: index.to_string_lossy().into_owned(),
                documents: built.documents.len(),
                issues: built.issues.len(),
                unique_terms,
            })
        }
        Command::Search {
            index,
            term,
            path_prefix,
            limit,
            format,
        } => {
            let query = SearchQuery::new(term, path_prefix, limit)?;
            let index = JsonFileIndexStore::new(index).load()?;
            let result = search(&index, query)?;
            match format {
                OutputFormat::Json => json_string(&result),
                OutputFormat::Text => Ok(format_search_text(&result)),
            }
        }
        Command::Stats { index, format } => {
            let index = JsonFileIndexStore::new(index).load()?;
            let stats = IndexStats::from_index(&index)?;
            match format {
                OutputFormat::Json => json_string(&stats),
                OutputFormat::Text => Ok(format_stats_text(&stats)),
            }
        }
    }
}

fn validate_workers(value: Option<usize>) -> Result<NonZeroUsize, IndexError> {
    let workers = value.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map_or(1, NonZeroUsize::get)
            .min(8)
    });
    if !(1..=64).contains(&workers) {
        return Err(IndexError::contract(
            ErrorCode::InvalidArgument,
            "workers must be in 1..=64",
        ));
    }
    NonZeroUsize::new(workers)
        .ok_or_else(|| IndexError::contract(ErrorCode::InvalidArgument, "workers must be positive"))
}

fn json_string(value: &impl Serialize) -> Result<String, IndexError> {
    serde_json::to_string(value)
        .map_err(|source| IndexError::json(ErrorCode::IndexWriteFailed, source))
}

fn format_search_text(result: &SearchResult) -> String {
    let terms = result
        .query
        .terms
        .iter()
        .map(|term| term.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let prefix = result.query.path_prefix.as_deref().unwrap_or("-");
    let mut lines = vec![format!(
        "query terms={terms} path_prefix={prefix} limit={}",
        result.query.limit
    )];
    for found in &result.matches {
        let counts = found
            .term_counts
            .iter()
            .map(|term| format!("{}={}", term.term, term.count))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!(
            "match id={} root={} path={} bytes={} {counts}",
            found.document.id.get(),
            found.document.root,
            found.document.path,
            found.document.bytes
        ));
    }
    lines.join("\n")
}

fn format_stats_text(stats: &IndexStats) -> String {
    format!(
        "schema_version={} roots={} documents={} issues={} unique_terms={} indexed_bytes={}",
        stats.schema_version,
        stats.roots,
        stats.documents,
        stats.issues,
        stats.unique_terms,
        stats.indexed_bytes
    )
}
