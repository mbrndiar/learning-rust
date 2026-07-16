//! Complete Rust implementation of the shared versioned key/value capstone.

pub mod application;
pub mod cli;
pub mod domain;
pub mod error;
pub mod store;

pub use application::KvApplication;
pub use domain::{
    Command, CommandResult, DeleteExpectation, DeleteResult, Entry, Key, ListResult, Revision,
    SetExpectation, SetResult, parse_json_value,
};
pub use error::KvError;
pub use store::{KvStore, SqliteStore};

/// Frozen shared specification version implemented by this capstone.
pub const SPEC_VERSION: &str = "1.0.0";
