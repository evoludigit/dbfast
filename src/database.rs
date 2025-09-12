/// Database connection and pooling for `DBFast`
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
    /// Create a new database connection pool for the default database
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
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
                match env::var(password_env) {
                    Ok(pwd) => pwd,
                    Err(_) => {
                        warn!(
                            "Environment variable {} not found, using empty password",
                            password_env
                        );
                        String::new()
                    }
                }
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

    /// Parse SQL content into individual statements, handling comments and edge cases
    fn parse_sql_statements(sql_content: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_comment = false;

        for line in sql_content.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip single-line comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Handle multi-line comments (basic implementation)
            if trimmed.starts_with("/*") {
                in_comment = true;
                continue;
            }
            if trimmed.ends_with("*/") {
                in_comment = false;
                continue;
            }
            if in_comment {
                continue;
            }

            // Add line to current statement
            current_statement.push(' ');
            current_statement.push_str(line);

            // Check if statement ends with semicolon
            if trimmed.ends_with(';') {
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

    /// Create a database with the given name using template0 for a clean database
    pub async fn create_database(&self, database_name: &str) -> Result<(), DatabaseError> {
        let create_db_sql = format!("CREATE DATABASE {database_name} WITH TEMPLATE template0");
        self.query(&create_db_sql, &[]).await.map_err(|e| {
            DatabaseError::Config(format!("Failed to create database '{database_name}': {e}"))
        })?;
        Ok(())
    }

    /// Drop a database with the given name
    pub async fn drop_database(&self, database_name: &str) -> Result<(), DatabaseError> {
        let drop_db_sql = format!("DROP DATABASE IF EXISTS {database_name}");
        self.query(&drop_db_sql, &[]).await.map_err(|e| {
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
