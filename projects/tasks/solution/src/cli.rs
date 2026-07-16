use std::time::Duration;

use clap::{Parser, Subcommand};

use crate::{TaskError, TaskResult};

#[derive(Debug, Parser)]
#[command(name = "tasks", about = "Call a local Task REST API")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8000")]
    pub base_url: String,
    #[arg(long, default_value_t = 5)]
    pub timeout: u64,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Add {
        title: String,
    },
    List {
        #[arg(long)]
        completed: Option<bool>,
    },
    Show {
        id: i64,
    },
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        completed: Option<bool>,
    },
    Complete {
        id: i64,
    },
    Remove {
        id: i64,
    },
}

impl Cli {
    #[must_use]
    pub const fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }
}

pub async fn run(_cli: Cli) -> TaskResult<()> {
    Err(TaskError::incomplete("tasks command execution"))
}
