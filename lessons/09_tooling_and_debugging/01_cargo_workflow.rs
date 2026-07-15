//! Lesson 9.1: Cargo targets, profiles, and the narrow-to-wide feedback loop.
//!
//! Cargo drives the whole workflow: it reads `Cargo.toml`, records the resolved
//! dependency graph in `Cargo.lock`, and writes build output to `target/`. The
//! habit to build: run the smallest relevant check first (type-check, format,
//! lint, test) and widen the scope only after it passes, keeping feedback fast.

#[derive(Debug)]
struct Command {
    purpose: &'static str,
    invocation: &'static str,
}

// Returns a fixed-size array (`[Command; 6]`); the length is part of the type.
fn recommended_commands() -> [Command; 6] {
    [
        Command {
            purpose: "type-check every target",
            invocation: "cargo check --workspace --all-targets",
        },
        Command {
            purpose: "apply formatting",
            invocation: "cargo fmt --all",
        },
        Command {
            purpose: "lint and reject warnings",
            invocation: "cargo clippy --workspace --all-targets -- -D warnings",
        },
        Command {
            purpose: "test libraries and binaries",
            invocation: "cargo test --workspace --lib --bins",
        },
        Command {
            purpose: "test documentation examples",
            invocation: "cargo test --doc --workspace",
        },
        Command {
            purpose: "build optimized artifacts",
            invocation: "cargo build --workspace --release",
        },
    ]
}

fn main() {
    println!("Cargo reads package and workspace metadata from Cargo.toml.");
    println!("Cargo.lock records the selected dependency graph.");
    println!("The target/ directory contains generated build artifacts.\n");

    for command in recommended_commands() {
        println!("{:<28} {}", command.purpose, command.invocation);
    }

    println!("\nStart with the smallest relevant target; widen after it passes.");
}
