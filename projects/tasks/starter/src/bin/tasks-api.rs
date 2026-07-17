//! Binary entry point for the `tasks-api` server.
//!
//! A thin composition root: parse [`ServerConfig`] from the CLI and run until a
//! shutdown signal. Any startup or serve failure exits non-zero.

use clap::Parser;

use tasks_starter::server::{ServerConfig, run};

#[tokio::main]
async fn main() {
    if let Err(error) = run(ServerConfig::parse()).await {
        eprintln!("tasks-api: {error}");
        std::process::exit(1);
    }
}
