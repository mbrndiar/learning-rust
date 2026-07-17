//! Public scaffold for the shared versioned key/value capstone.
//!
//! The starter and solution packages intentionally expose the same modules and
//! types so both build against the same conformance suite. Mental model, layered by
//! responsibility:
//!
//! * [`domain`] owns the wire contract — the restricted JSON value model and the
//!   `Command`/result shapes.
//! * [`error`] is where the observable failure taxonomy (category, details, exit
//!   code) will be defined to match `spec/SPEC.md`.
//! * [`store`] is the persistence seam ([`KvStore`]); a backing implementation is
//!   yours to add.
//! * [`application`] dispatches a validated command to any `KvStore`.
//! * [`cli`] parses arguments and renders the response.
//!
//! Operations return [`KvError::Incomplete`] until their milestone is implemented.

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
pub use store::KvStore;

/// Frozen shared specification version implemented by this capstone.
pub const SPEC_VERSION: &str = "1.0.0";
