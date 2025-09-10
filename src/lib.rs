//! DBFast - A high-performance database library
//!
//! This library provides fast database operations with modern async/await support.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms
)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod connection;
pub mod query;

pub use connection::Connection;
pub use query::QueryBuilder;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
