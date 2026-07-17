//! HTTP adapters plus the framework-neutral boundary they share.
//!
//! [`boundary`] is where all request decoding, validation, status selection,
//! and JSON encoding must live. [`axum`] and [`actix`] are meant to be thin
//! translators that turn a native request into boundary calls and turn the
//! boundary's response back into a native response. Keeping the policy in one
//! place is what lets the two frameworks stay black-box interchangeable.

pub mod actix;
pub mod axum;
pub mod boundary;
