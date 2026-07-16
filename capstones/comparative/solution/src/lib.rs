//! Public scaffold for the shared versioned key/value capstone.
//!
//! The starter and solution packages intentionally expose the same modules and
//! types. Operations return [`KvError::Incomplete`] until their milestone is
//! implemented.

pub mod application;
pub mod cli;
pub mod domain;
pub mod error;
pub mod store;

pub use application::KvApplication;
pub use domain::{
    Command, CommandResult, DeleteExpectation, DeleteResult, Entry, Key, ListResult, Revision,
    SetExpectation, SetResult,
};
pub use error::KvError;
pub use store::KvStore;

/// Frozen shared specification version implemented by this capstone.
pub const SPEC_VERSION: &str = "1.0.0";
