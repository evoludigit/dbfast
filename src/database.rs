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
//!     allow_multi_statement: true,
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
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
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
    connection_info: Option<ConnectionInfo>,
}

/// Connection information for psql fallback
#[derive(Clone)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    pub database: String,
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

        // Parse URL for connection info
        let connection_info = Self::parse_connection_info(database_url)?;

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
        Ok(Self {
            pool,
            connection_info: Some(connection_info),
        })
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

        // Store connection info for psql fallback
        let connection_info = ConnectionInfo {
            host: config.host.clone(),
            port: config.port,
            user: config.user.clone(),
            password: if password.is_empty() {
                None
            } else {
                Some(password.clone())
            },
            database: database_name.to_string(),
        };

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
        Ok(Self {
            pool,
            connection_info: Some(connection_info),
        })
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

    /// Execute multi-statement SQL content using hybrid execution strategy
    ///
    /// This method automatically determines the best execution approach:
    /// - **Single statements**: Uses fast prepared statements via tokio-postgres
    /// - **Multi-statement content**: Uses psql fallback to handle complex SQL
    ///
    /// This hybrid approach solves GitHub issue #39 by supporting concatenated
    /// SQL files with multiple statements while maintaining performance for simple queries.
    ///
    /// # Examples
    ///
    /// Single statement (uses prepared statements):
    /// ```sql
    /// SELECT version();
    /// ```
    ///
    /// Multi-statement content (uses psql fallback):
    /// ```sql
    /// CREATE SCHEMA myschema;
    /// CREATE TABLE myschema.users (id SERIAL PRIMARY KEY);
    /// INSERT INTO myschema.users DEFAULT VALUES;
    /// ```
    ///
    /// # Requirements
    ///
    /// For psql fallback to work, `psql` must be installed and accessible in the system PATH.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError` if:
    /// - Database connection fails
    /// - SQL execution fails (either via prepared statements or psql)
    /// - psql is not available when needed for multi-statement content
    /// - Temporary file creation fails (psql fallback)
    pub async fn execute_sql_content(&self, sql_content: &str) -> Result<(), DatabaseError> {
        // Default to advanced parsing for backward compatibility
        self.execute_sql_content_with_config(sql_content, true)
            .await
    }

    /// Execute multi-statement SQL content with hybrid execution strategy and configurable parsing
    ///
    /// This method implements the hybrid execution strategy described in GitHub issue #39:
    /// 1. **Detection**: Analyzes SQL content to determine if it contains multiple statements
    /// 2. **Single statements**: Uses tokio-postgres prepared statements for optimal performance
    /// 3. **Multi-statement content**: Falls back to psql subprocess execution
    ///
    /// The `allow_multi_statement` parameter controls SQL parsing behavior when using
    /// prepared statements (ignored for psql fallback).
    ///
    /// # Arguments
    ///
    /// * `sql_content` - The SQL content to execute
    /// * `allow_multi_statement` - Whether to use advanced SQL parsing for prepared statements
    ///
    /// # Performance
    ///
    /// - **Prepared statements**: ~1-5ms overhead, optimal for single statements
    /// - **psql fallback**: ~50-100ms overhead, necessary for complex multi-statement SQL
    ///
    /// # Error Handling
    ///
    /// Both execution paths provide detailed error reporting:
    /// - Prepared statement errors include specific SQL and line information
    /// - psql errors include full stderr output from the subprocess
    pub async fn execute_sql_content_with_config(
        &self,
        sql_content: &str,
        allow_multi_statement: bool,
    ) -> Result<(), DatabaseError> {
        // Use hybrid execution strategy based on statement count
        if Self::should_use_psql_fallback(sql_content) {
            info!("Using psql fallback for multi-statement content");
            self.execute_via_psql_fallback(sql_content).await
        } else {
            info!("Using prepared statement execution");
            self.execute_via_prepared_statements(sql_content, allow_multi_statement)
                .await
        }
    }

    /// Execute SQL content via prepared statements (original method)
    async fn execute_via_prepared_statements(
        &self,
        sql_content: &str,
        allow_multi_statement: bool,
    ) -> Result<(), DatabaseError> {
        let mut conn = self.pool.get().await?;

        // Begin transaction
        let transaction = conn.transaction().await.map_err(DatabaseError::Query)?;

        let statements = Self::parse_sql_statements_with_config(sql_content, allow_multi_statement);

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

    /// Execute SQL content via psql fallback for concatenated multi-statement files
    async fn execute_via_psql_fallback(&self, sql_content: &str) -> Result<(), DatabaseError> {
        // Write content to temporary file
        let mut temp_file = NamedTempFile::new().map_err(|e| {
            DatabaseError::Config(format!("Failed to create temporary file: {}", e))
        })?;

        temp_file.write_all(sql_content.as_bytes()).map_err(|e| {
            DatabaseError::Config(format!(
                "Failed to write SQL content to temporary file: {}",
                e
            ))
        })?;

        let temp_path = temp_file.path().to_str().ok_or_else(|| {
            DatabaseError::Config("Failed to get temporary file path".to_string())
        })?;

        // Build psql command arguments
        let mut psql_args = self.build_psql_args()?;
        psql_args.extend_from_slice(&["-f".to_string(), temp_path.to_string()]);

        let connection_info = self.connection_info.as_ref().unwrap();
        debug!(
            "Executing via psql: host={}, port={}, user={}, database={}",
            connection_info.host,
            connection_info.port,
            connection_info.user,
            connection_info.database
        );

        // Execute via psql subprocess
        let output = Command::new("psql")
            .args(&psql_args)
            .output()
            .map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to execute psql command: {}. Ensure psql is installed and accessible.",
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DatabaseError::Config(format!(
                "psql execution failed: {}",
                stderr
            )));
        }

        // Log successful execution
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("psql execution completed successfully: {}", stdout);

        Ok(())
    }

    /// Determine if psql fallback should be used for SQL content
    ///
    /// This method implements the detection logic described in GitHub issue #39.
    /// It uses a simple but effective heuristic: count semicolons in the SQL content.
    ///
    /// # Detection Logic
    ///
    /// - **Single statement** (≤1 semicolon): Use fast prepared statements
    /// - **Multiple statements** (>1 semicolon): Use psql fallback
    ///
    /// This approach handles the most common cases including:
    /// - Simple queries and single DDL statements
    /// - Concatenated SQL files with multiple CREATE, INSERT, GRANT statements
    /// - Complex `PostgreSQL` functions with additional metadata (COMMENT, GRANT)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbfast::database::DatabasePool;
    /// assert!(!DatabasePool::should_use_psql_fallback("SELECT version();")); // false - single stmt
    /// assert!(DatabasePool::should_use_psql_fallback("CREATE TABLE t (id INT); INSERT INTO t DEFAULT VALUES;")); // true - multi stmt
    /// ```
    ///
    /// # Future Improvements
    ///
    /// This heuristic could be enhanced to handle edge cases like:
    /// - Semicolons within string literals or comments
    /// - More sophisticated SQL parsing
    /// - Configuration-based fallback thresholds
    #[must_use]
    pub fn should_use_psql_fallback(sql_content: &str) -> bool {
        // Simple heuristic: count semicolons to detect multi-statement content
        // This is the approach suggested in GitHub issue #39
        let semicolon_count = sql_content.matches(';').count();

        // Use psql fallback for content with multiple statements
        semicolon_count > 1
    }

    /// Check if connection info is available (for testing)
    #[must_use]
    pub const fn has_connection_info(&self) -> bool {
        self.connection_info.is_some()
    }

    /// Get connection info for testing
    #[must_use]
    pub const fn get_connection_info(&self) -> Option<&ConnectionInfo> {
        self.connection_info.as_ref()
    }

    /// Parse connection info from database URL
    fn parse_connection_info(database_url: &str) -> Result<ConnectionInfo, DatabaseError> {
        let url = url::Url::parse(database_url)
            .map_err(|e| DatabaseError::Config(format!("Failed to parse database URL: {}", e)))?;

        let host = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or(5432);
        let user = url.username().to_string();
        let password = url.password().map(std::string::ToString::to_string);
        let database = url.path().trim_start_matches('/').to_string();

        if database.is_empty() {
            return Err(DatabaseError::Config(
                "Database name is required in URL".to_string(),
            ));
        }

        Ok(ConnectionInfo {
            host,
            port,
            user,
            password,
            database,
        })
    }

    /// Build psql command arguments from connection info
    fn build_psql_args(&self) -> Result<Vec<String>, DatabaseError> {
        let connection_info = self.connection_info.as_ref().ok_or_else(|| {
            DatabaseError::Config("No connection info available for psql fallback".to_string())
        })?;

        let args = vec![
            "-h".to_string(),
            connection_info.host.clone(),
            "-p".to_string(),
            connection_info.port.to_string(),
            "-U".to_string(),
            connection_info.user.clone(),
            "-d".to_string(),
            connection_info.database.clone(),
            "-v".to_string(),
            "ON_ERROR_STOP=1".to_string(),
        ];

        // Set password via environment variable if available
        if let Some(password) = &connection_info.password {
            env::set_var("PGPASSWORD", password);
        }

        Ok(args)
    }

    /// Parse SQL content into individual statements with advanced `PostgreSQL` function support
    ///
    /// This parser correctly handles:
    /// - `PostgreSQL` functions with dollar-quoted bodies (including inline functions)
    /// - Multiple statements on the same line after dollar quotes end
    /// - Nested dollar quotes with different tags
    /// - Comments outside dollar quotes (ignored) and inside dollar quotes (preserved)
    #[must_use]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub fn parse_sql_statements_advanced(sql_content: &str) -> Vec<String> {
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

            // Process the line, potentially handling inline dollar quotes and multiple statements
            if in_dollar_quote {
                // We're inside a dollar quote, look for the end
                if let Some(end_pos) = line.find(&dollar_tag) {
                    let end_tag_pos = end_pos + dollar_tag.len();
                    // Add everything up to and including the closing dollar tag
                    current_statement.push('\n');
                    current_statement.push_str(&line[..end_tag_pos]);
                    in_dollar_quote = false;
                    dollar_tag.clear();

                    // Check if there's more content after the dollar quote on the same line
                    let remaining_line = &line[end_tag_pos..];
                    if let Some(semicolon_pos) = remaining_line.find(';') {
                        // Add content up to semicolon and complete the statement
                        current_statement.push_str(&remaining_line[..=semicolon_pos]);
                        let stmt = current_statement.trim().trim_end_matches(';');
                        if !stmt.is_empty() {
                            statements.push(stmt.to_string());
                        }
                        current_statement.clear();

                        // Process any content after the semicolon as start of new statement
                        let after_semicolon = remaining_line[semicolon_pos + 1..].trim();
                        if !after_semicolon.is_empty() {
                            current_statement.push_str(after_semicolon);
                        }
                    } else {
                        // Add remaining content after dollar quote (no semicolon found)
                        current_statement.push_str(remaining_line);
                    }
                } else {
                    // Still inside dollar quote, add the whole line
                    if !current_statement.is_empty() {
                        current_statement.push('\n');
                    }
                    current_statement.push_str(line);
                }
            } else {
                // Not in dollar quote, check for start of dollar quote
                if let Some(start_pos) = find_dollar_quote_start(line) {
                    // Found start of dollar quote
                    let tag = extract_dollar_tag(&line[start_pos..]);
                    if tag.is_empty() {
                        // False positive dollar sign, treat as regular line
                        if !current_statement.is_empty() {
                            current_statement.push('\n');
                        }
                        current_statement.push_str(line);
                    } else {
                        dollar_tag.clone_from(&tag);

                        // Add content before dollar quote if any
                        let before_dollar = &line[..start_pos];
                        if (!current_statement.is_empty() || !before_dollar.trim().is_empty())
                            && !current_statement.is_empty()
                        {
                            current_statement.push('\n');
                        }
                        current_statement.push_str(line);

                        // Check if dollar quote ends on the same line
                        let dollar_start_in_line = start_pos;
                        let tag_end_pos = dollar_start_in_line + tag.len();
                        if let Some(end_pos) = line[tag_end_pos..].find(&tag) {
                            // Dollar quote starts and ends on same line
                            let absolute_end_pos = tag_end_pos + end_pos + tag.len();

                            // Check for semicolon after the closing dollar quote
                            let after_dollar = &line[absolute_end_pos..];
                            if let Some(semicolon_pos) = after_dollar.find(';') {
                                // Complete statement found
                                let stmt = current_statement.trim().trim_end_matches(';');
                                if !stmt.is_empty() {
                                    statements.push(stmt.to_string());
                                }
                                current_statement.clear();

                                // Start new statement with content after semicolon
                                let after_semicolon = after_dollar[semicolon_pos + 1..].trim();
                                if !after_semicolon.is_empty() {
                                    current_statement.push_str(after_semicolon);
                                }
                            }
                            // If no semicolon, dollar quote is complete but statement continues
                        } else {
                            // Dollar quote starts but doesn't end on this line
                            in_dollar_quote = true;
                        }
                    }
                } else {
                    // Regular line, no dollar quotes
                    if !current_statement.is_empty() || !trimmed.is_empty() {
                        if !current_statement.is_empty() {
                            current_statement.push('\n');
                        }
                        current_statement.push_str(line);
                    }

                    // Check for statement termination
                    if trimmed.ends_with(';') {
                        let stmt = current_statement.trim().trim_end_matches(';');
                        if !stmt.is_empty() {
                            statements.push(stmt.to_string());
                        }
                        current_statement.clear();
                    }
                }
            }
        }

        // Add any remaining statement
        let remaining = current_statement.trim();
        if !remaining.is_empty() {
            statements.push(remaining.to_string());
        }

        statements
    }

    /// Simple SQL statement parser (legacy mode) - splits on semicolons only
    ///
    /// This is a basic parser that splits statements on semicolons without
    /// advanced `PostgreSQL` function support. Use only when `allow_multi_statement` is false.
    #[must_use]
    pub fn parse_sql_statements_simple(sql_content: &str) -> Vec<String> {
        sql_content
            .split(';')
            .map(str::trim)
            .filter(|stmt| !stmt.is_empty())
            .map(String::from)
            .collect()
    }

    /// Parse SQL content into individual statements based on configuration
    ///
    /// Uses advanced parsing if `allow_advanced` is true, otherwise uses simple parsing
    #[must_use]
    pub fn parse_sql_statements(sql_content: &str) -> Vec<String> {
        // Default to advanced parsing for backward compatibility
        Self::parse_sql_statements_advanced(sql_content)
    }

    /// Parse SQL content with configurable parsing mode
    #[must_use]
    pub fn parse_sql_statements_with_config(
        sql_content: &str,
        allow_multi_statement: bool,
    ) -> Vec<String> {
        if allow_multi_statement {
            Self::parse_sql_statements_advanced(sql_content)
        } else {
            Self::parse_sql_statements_simple(sql_content)
        }
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
