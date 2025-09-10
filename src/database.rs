/// Database connection and pooling for `DBFast`
use crate::config::DatabaseConfig;
use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use std::env;
use thiserror::Error;
use tokio_postgres::{NoTls, Row};

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
pub struct DatabasePool {
    pool: PostgresPool,
}

impl DatabasePool {
    /// Create a new database connection pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        // Get password from environment variable
        let password = config
            .password_env
            .as_ref()
            .map_or_else(String::new, |password_env| {
                env::var(password_env).unwrap_or_else(|_| String::new())
            });

        // Build connection string
        let connection_string = format!(
            "host={} port={} user={} password={} dbname=postgres",
            config.host, config.port, config.user, password
        );

        // Create connection manager
        let manager = PostgresConnectionManager::new_from_stringlike(connection_string, NoTls)
            .map_err(|e| DatabaseError::Config(e.to_string()))?;

        // Create pool
        let pool = Pool::builder().max_size(10).build(manager).await?;

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

    /// Get a connection for more complex operations (simplified for now)
    pub async fn get(&self) -> Result<DatabaseConnection, DatabaseError> {
        let _conn = self.pool.get().await?;
        Ok(DatabaseConnection {})
    }
}

/// Simplified connection wrapper for testing
pub struct DatabaseConnection {}

impl DatabaseConnection {
    /// Execute a query and return rows (simplified version for testing)
    pub async fn query(
        &self,
        _query: &str,
        _params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<Row>, DatabaseError> {
        // For testing purposes, we'll return an empty result
        // In a real implementation, this would use the actual connection
        // The Row type is complex to construct manually, so we return empty for now
        Ok(vec![])
    }
}
