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
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::multiple_crate_versions,
    clippy::module_name_repetitions,
    clippy::unused_async,
    missing_docs,
    clippy::uninlined_format_args,
    clippy::single_match_else,
    clippy::match_bool,
    clippy::needless_pass_by_value
)]

/// Backup management
pub mod backup;
/// Change detection for template rebuilding
pub mod change_detector;
/// CLI interface for `DBFast`
pub mod cli;
/// Database cloning functionality
pub mod clone;
/// CLI commands
pub mod commands;
/// Configuration management for `DBFast`
pub mod config;
pub mod connection;
/// Database connection and pooling
pub mod database;
/// Environment filtering for deployments
pub mod environment;
/// Error handling
pub mod error;
/// Comprehensive error handling system
pub mod errors;
/// Database health monitoring
pub mod health;
/// Performance metrics collection
pub mod metrics;
/// Comprehensive observability and monitoring infrastructure
pub mod observability;
pub mod query;
/// Remote deployment management
pub mod remote;
/// Retry and recovery mechanisms
pub mod retry;
/// File scanning and hash calculation
pub mod scanner;
/// Security hardening and audit logging
pub mod security;
/// SQL repository management for file discovery and loading
pub mod sql_repository;
/// Template management functionality
pub mod template;
/// Configuration validation system
pub mod validators;

pub use config::Config;
pub use connection::Connection;
pub use database::DatabasePool;
pub use query::QueryBuilder;
pub use scanner::FileScanner;

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
