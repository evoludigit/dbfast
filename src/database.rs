//! # Database Connection and Pooling Module
//!
//! Provides high-performance database connection pooling for PostgreSQL using bb8.
//! This module handles connection management, query execution, and database operations
//! with automatic reconnection and connection pooling for optimal performance.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use dbfast::{DatabasePool, config::DatabaseConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = DatabaseConfig {
//!     host: "localhost".to_string(),
//!     port: 5432,
//!     user: "postgres".to_string(),
//!     password_env: Some("DB_PASSWORD".to_string()),
//!     template_name: "my_template".to_string(),
//! };
//!
//! let pool = DatabasePool::from_config(&config).await?;
//! let rows = pool.query("SELECT version()", &[]).await?;
//! # Ok(())
//! # }
//! ```

use crate::config::DatabaseConfig;
use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use std::env;
use thiserror::Error;
use tokio_postgres::{NoTls, Row};
use tracing::{debug, error, info, warn};

/// Database-related errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Connection pool creation error
    #[error("Pool creation error: {0}")]
    Pool(#[from] RunError<tokio_postgres::Error>),

    /// Database query error
    #[error("Database query error: {0}")]
    Query(#[from] tokio_postgres::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;

/// Database connection pool wrapper
#[derive(Clone)]
pub struct DatabasePool {
    pool: PostgresPool,
}

impl DatabasePool {
    /// Create a new database connection pool from a database URL
    ///
    /// Creates a connection pool using a `PostgreSQL` URL string.
    /// This is the most convenient method for simple setups.
    ///
    /// # Arguments
    /// * `database_url` - `PostgreSQL` connection URL (e.g., "postgresql://user:pass@host:port/database")
    ///
    /// # Example
    /// ```rust,no_run
    /// use dbfast::DatabasePool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = DatabasePool::new("postgresql://user:pass@localhost:5432/mydb").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        info!("Creating database connection pool from URL");

        // Create connection manager from URL
        let manager =
            PostgresConnectionManager::new_from_stringlike(database_url, NoTls).map_err(|e| {
                error!("Failed to create connection manager from URL: {}", e);
                DatabaseError::Config(e.to_string())
            })?;

        // Create pool
        debug!("Building connection pool with max_size=10");
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .await
            .map_err(|e| {
                error!("Failed to build connection pool: {}", e);
                e
            })?;

        info!("Successfully created connection pool from URL");
        Ok(Self { pool })
    }

    /// Create a new database connection pool from configuration for the default database
    pub async fn from_config(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("Creating database connection pool for default database");
        debug!(
            "Database config: host={}:{}, user={}",
            config.host, config.port, config.user
        );
        Self::new_for_database(config, "postgres").await
    }

    /// Create a new database connection pool for a specific database
    pub async fn new_for_database(
        config: &DatabaseConfig,
        database_name: &str,
    ) -> Result<Self, DatabaseError> {
        info!("Creating connection pool for database: {}", database_name);

        // Get password from environment variable
        let password = config
            .password_env
            .as_ref()
            .map_or_else(String::new, |password_env| {
                debug!(
                    "Reading password from environment variable: {}",
                    password_env
                );
                env::var(password_env).unwrap_or_else(|_| {
                    warn!(
                        "Environment variable {} not found, using empty password",
                        password_env
                    );
                    String::new()
                })
            });

        // Build connection string (hide password in logs)
        let connection_string = format!(
            "host={} port={} user={} password={} dbname={}",
            config.host, config.port, config.user, password, database_name
        );

        debug!(
            "Creating connection pool: host={}:{}, user={}, database={}",
            config.host, config.port, config.user, database_name
        );

        // Create connection manager
        let manager = PostgresConnectionManager::new_from_stringlike(connection_string, NoTls)
            .map_err(|e| {
                error!("Failed to create connection manager: {}", e);
                DatabaseError::Config(e.to_string())
            })?;

        // Create pool
        debug!("Building connection pool with max_size=10");
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .await
            .map_err(|e| {
                error!("Failed to build connection pool: {}", e);
                e
            })?;

        info!(
            "Successfully created connection pool for database: {}",
            database_name
        );
        Ok(Self { pool })
    }

    /// Get a connection from the pool and execute a query
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<Row>, DatabaseError> {
        let conn = self.pool.get().await?;
        let rows = conn.query(query, params).await?;
        Ok(rows)
    }

    /// Execute a query that cannot run in a transaction (like CREATE/DROP DATABASE)
    pub async fn execute_non_transactional(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get().await?;
        conn.execute(query, params).await?;
        Ok(())
    }

    /// Execute multi-statement SQL content (for SQL files) in a single transaction
    pub async fn execute_sql_content(&self, sql_content: &str) -> Result<(), DatabaseError> {
        let mut conn = self.pool.get().await?;

        // Begin transaction
        let transaction = conn.transaction().await.map_err(DatabaseError::Query)?;

        let statements = Self::parse_sql_statements(sql_content);

        // Execute all statements within the transaction
        for statement in statements {
            if !statement.trim().is_empty() {
                transaction
                    .execute(&statement, &[])
                    .await
                    .map_err(DatabaseError::Query)?;
            }
        }

        // Commit the transaction
        transaction.commit().await.map_err(DatabaseError::Query)?;

        Ok(())
    }

    /// Parse SQL content into individual statements, handling comments, dollar-quoted strings, and edge cases
    #[must_use]
    pub fn parse_sql_statements(sql_content: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_multiline_comment = false;
        let mut in_dollar_quote = false;
        let mut dollar_tag = String::new();

        for line in sql_content.lines() {
            let trimmed = line.trim();

            // Skip empty lines when not in a dollar-quoted string
            if trimmed.is_empty() && !in_dollar_quote {
                continue;
            }

            // Skip single-line comments when not in a dollar-quoted string
            if trimmed.starts_with("--") && !in_dollar_quote {
                continue;
            }

            // Handle multi-line comments (basic implementation) when not in a dollar-quoted string
            if !in_dollar_quote {
                if trimmed.starts_with("/*") {
                    in_multiline_comment = true;
                    continue;
                }
                if trimmed.ends_with("*/") {
                    in_multiline_comment = false;
                    continue;
                }
                if in_multiline_comment {
                    continue;
                }
            }

            // Add line to current statement (preserve original spacing for dollar-quoted blocks)
            if !current_statement.is_empty() || !trimmed.is_empty() {
                if !current_statement.is_empty() {
                    current_statement.push('\n');
                }
                current_statement.push_str(line);
            }

            // Handle dollar-quoted strings
            if in_dollar_quote {
                // Look for end of dollar-quoted string (exact match)
                if line.contains(&dollar_tag) {
                    in_dollar_quote = false;
                    dollar_tag.clear();
                }
            } else {
                // Look for start of dollar-quoted string
                if let Some(start_pos) = find_dollar_quote_start(line) {
                    in_dollar_quote = true;
                    dollar_tag = extract_dollar_tag(&line[start_pos..]);
                }
            }

            // Check if statement ends with semicolon (only when not in dollar-quoted string)
            if !in_dollar_quote && trimmed.ends_with(';') {
                // Remove the semicolon and add to statements
                current_statement = current_statement.trim_end_matches(';').trim().to_string();
                if !current_statement.is_empty() {
                    statements.push(current_statement.clone());
                }
                current_statement.clear();
            }
        }

        // Add any remaining statement
        let remaining = current_statement.trim();
        if !remaining.is_empty() {
            statements.push(remaining.to_string());
        }

        statements
    }
}

/// Find the position of a dollar-quoted string start in a line
/// Returns the position where the dollar-quoted string begins (position of first $)
fn find_dollar_quote_start(line: &str) -> Option<usize> {
    line.find('$').and_then(|start| {
        // Look for the closing $ after the tag
        line[start + 1..].find('$').map(|_| start)
    })
}

/// Extract the dollar tag from a dollar-quoted string (e.g., "$$" or "$BODY$")
/// Returns the complete tag including the surrounding $ characters
fn extract_dollar_tag(text: &str) -> String {
    if let Some(start) = text.find('$') {
        if let Some(end) = text[start + 1..].find('$') {
            // Return the complete tag: $ + tag_content + $
            return text[start..=start + 1 + end].to_string();
        }
    }
    String::new()
}

impl DatabasePool {
    /// Create a database with the given name using template0 for a clean database
    pub async fn create_database(&self, database_name: &str) -> Result<(), DatabaseError> {
        let create_db_sql = format!("CREATE DATABASE {database_name} WITH TEMPLATE template0");
        self.execute_non_transactional(&create_db_sql, &[])
            .await
            .map_err(|e| {
                DatabaseError::Config(format!("Failed to create database '{database_name}': {e}"))
            })?;
        Ok(())
    }

    /// Drop a database with the given name
    pub async fn drop_database(&self, database_name: &str) -> Result<(), DatabaseError> {
        let drop_db_sql = format!("DROP DATABASE IF EXISTS {database_name}");
        self.execute_non_transactional(&drop_db_sql, &[])
            .await
            .map_err(|e| {
                DatabaseError::Config(format!("Failed to drop database '{database_name}': {e}"))
            })?;
        Ok(())
    }

    /// Check if a database exists
    pub async fn database_exists(&self, database_name: &str) -> Result<bool, DatabaseError> {
        let check_sql = "SELECT 1 FROM pg_database WHERE datname = $1";
        let rows = self
            .query(check_sql, &[&database_name])
            .await
            .map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to check if database '{database_name}' exists: {e}"
                ))
            })?;
        Ok(!rows.is_empty())
    }
}
