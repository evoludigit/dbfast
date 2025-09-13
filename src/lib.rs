//! # `DBFast` - Lightning-Fast `PostgreSQL` Database Seeding
//!
//! `DBFast` transforms database fixtures from a 60-second bottleneck into a 100ms delight.
//! Built for `PostgreSQL`-first workflows with modern async Rust architecture.
//!
//! ## Core Features
//!
//! - **Template-based fixtures**: Pre-built database images for instant seeding
//! - **Intelligent change detection**: xxHash-based rebuilding only when SQL files change
//! - **Remote deployment**: Production database deployment with safety features
//! - **Environment management**: Multi-environment configuration support
//! - **Backup integration**: Automatic backup before destructive operations
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dbfast::{Config, DatabasePool, FileScanner};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = Config::load("dbfast.toml")?;
//!
//!     // Create database pool
//!     let pool = DatabasePool::new("postgresql://localhost:5432/mydb").await?;
//!
//!     // Scan SQL files
//!     let scanner = FileScanner::new("./sql");
//!     let files = scanner.scan_sql_files()?;
//!
//!     println!("Found {} SQL files", files.len());
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! `DBFast` implements a template-based approach where database fixtures are:
//! 1. **Scanned**: SQL files are discovered and hashed for change detection
//! 2. **Built**: Templates are created from SQL migrations and seed data
//! 3. **Cached**: Pre-built database images enable instant seeding
//! 4. **Deployed**: Remote deployment with backup and safety checks

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
/// Database connection management
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
/// SQL query building utilities
pub mod query;
/// Remote deployment management
pub mod remote;
/// Retry and recovery mechanisms
pub mod retry;
/// File scanning and hash calculation
pub mod scanner;
/// SQL repository management for file discovery and loading
pub mod sql_repository;
/// Template management functionality
pub mod template;

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
