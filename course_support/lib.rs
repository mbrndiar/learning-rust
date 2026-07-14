//! Shared package metadata for the learning examples.
//!
//! The teaching programs live under `lessons/` and `exercises/` as Cargo
//! examples. The capstone is a separate workspace package.

/// Return the minimum Rust version used by this course.
///
/// ```
/// assert_eq!(learning_rust_course::minimum_rust_version(), "1.85");
/// ```
#[must_use]
pub const fn minimum_rust_version() -> &'static str {
    "1.85"
}
