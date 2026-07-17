//! Interchangeable persistence backends behind the shared
//! [`crate::TaskRepository`] port.
//!
//! [`sqlite`] and [`markdown`] are two independent stores that pass the same
//! repository contract; the composition root picks one at startup. Switching
//! backends does not copy or synchronize data between them.

pub mod markdown;
pub mod sqlite;
