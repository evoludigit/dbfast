//! `DBFast` - A high-performance database library
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

/// Simple hello world function for testing
#[must_use]
pub fn hello_world() -> String {
    "Hello, World from DBFast!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World from DBFast!");
    }
}
