//! A small, testable task manager used as the course capstone.
//!
//! The crate is split into three layers: [`domain`] holds the validated types
//! and business rules, [`storage`] provides in-memory and atomic JSON backends,
//! and [`cli`] maps command-line input to domain calls and formatted output.

pub mod cli;
pub mod domain;
pub mod storage;
