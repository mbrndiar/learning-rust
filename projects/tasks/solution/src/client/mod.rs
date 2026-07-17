//! Outbound HTTP client and the user-facing command-line application.
//!
//! [`http`] implements the portable REST contract with Reqwest. [`cli`] owns
//! argument parsing, terminal output, and exit-code policy on top of that client.

pub mod cli;
pub mod error;
pub mod http;

pub use error::{ClientError, ClientResult};
