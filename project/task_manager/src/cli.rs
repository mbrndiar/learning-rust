//! Command-line parsing and presentation boundary.
//!
//! This module is the thin edge that maps CLI arguments to domain calls and
//! turns results into user-facing strings. Parsing (via clap) and formatting
//! live here so the domain and storage layers never depend on the CLI.

use crate::domain::{Task, TaskError, TaskId, TaskManager, TaskStore};
use crate::storage::JsonFileTaskStore;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Top-level command-line arguments.
#[derive(Debug, Parser)]
#[command(about = "A small file-backed task manager")]
pub struct Cli {
    /// Path to the JSON storage file; shared by every subcommand.
    #[arg(long, default_value = "tasks.json", global = true)]
    pub storage: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add a new pending task.
    Add { title: String },
    /// List stored tasks.
    List {
        #[arg(long)]
        pending_only: bool,
    },
    /// Mark a task complete.
    Complete { id: u64 },
    /// Remove a task.
    Remove { id: u64 },
}

/// Renders a single task as a checkbox line, e.g. `[x] #1 Ship release`.
#[must_use]
pub fn format_task(task: &Task) -> String {
    let marker = if task.is_done() { 'x' } else { ' ' };
    format!("[{marker}] #{} {}", task.id(), task.title())
}

/// Runs one parsed [`Command`] against `manager`, returning the message to
/// print on success. Storage and validation errors propagate as [`TaskError`].
pub fn execute<S: TaskStore>(
    manager: &mut TaskManager<S>,
    command: Command,
) -> Result<String, TaskError> {
    match command {
        Command::Add { title } => {
            let task = manager.add(&title)?;
            Ok(format!("Added task #{}: {}", task.id(), task.title()))
        }
        Command::List { pending_only } => {
            let tasks = manager.list(!pending_only);
            if tasks.is_empty() {
                Ok(String::from("No tasks yet."))
            } else {
                Ok(tasks
                    .into_iter()
                    .map(format_task)
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
        Command::Complete { id } => {
            let task = manager.complete(TaskId::new(id)?)?;
            Ok(format!("Completed task #{}: {}", task.id(), task.title()))
        }
        Command::Remove { id } => {
            let task = manager.remove(TaskId::new(id)?)?;
            Ok(format!("Removed task #{}: {}", task.id(), task.title()))
        }
    }
}

/// Opens the file-backed store named by `cli.storage` and executes the chosen
/// command. This is the single entry point the binary drives.
pub fn run(cli: Cli) -> Result<String, TaskError> {
    let store = JsonFileTaskStore::open(cli.storage)?;
    let mut manager = TaskManager::new(store);
    execute(&mut manager, cli.command)
}
