//! Complete Rust implementation of the shared versioned key/value capstone.
//!
//! The crate implements one language-neutral contract (see `spec/SPEC.md`): a CLI
//! over a versioned key/value store with optimistic-concurrency expectations.
//! Mental model, layered by responsibility:
//!
//! * [`domain`] owns the wire contract — a restricted JSON value model, canonical
//!   number/string normalization, and the `Command`/result shapes.
//! * [`error`] fixes the observable taxonomy: each [`KvError`] maps to an exact
//!   category, `details` object, and process exit code from the spec.
//! * [`store`] is the persistence seam ([`KvStore`]); [`SqliteStore`] is the one
//!   backend, responsible for schema, migration, and revision assignment.
//! * [`application`] is the storage-independent boundary that dispatches commands
//!   to any `KvStore`.
//! * [`cli`] parses the exact argument grammar and renders success/error envelopes.

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
