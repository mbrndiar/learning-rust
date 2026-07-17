//! Outbound HTTP client and the user-facing command-line application.
//!
//! [`http`] is the Reqwest milestone and [`cli`] owns argument parsing, terminal
//! output, and exit-code policy. Both remain visibly separate from the server.

pub mod cli;
pub mod error;
pub mod http;

pub use error::{ClientError, ClientResult};
