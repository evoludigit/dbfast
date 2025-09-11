/// Database cloning functionality for `DBFast`
///
/// This module provides fast database cloning using `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE`
/// functionality. It's designed to achieve database cloning in under 100ms for optimal performance.
use crate::database::{DatabaseError, DatabasePool};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Enhanced error types for database cloning operations
#[derive(Debug, Error)]
pub enum CloneError {
    #[error("Invalid database name: {name}. {reason}")]
    InvalidDatabaseName { name: String, reason: String },
    
    #[error("Template database '{template}' does not exist")]
    TemplateNotFound { template: String },
    
    #[error("Clone database '{clone}' already exists")]
    CloneAlreadyExists { clone: String },
    
    #[error("Clone operation timed out after {timeout_ms}ms")]
    CloneTimeout { timeout_ms: u64 },
    
    #[error("Insufficient permissions to create database '{name}'")]
    InsufficientPermissions { name: String },
    
    #[error("Connection pool exhausted during clone operation")]
    ConnectionPoolExhausted,
    
    #[error("Clone verification failed: {reason}")]
    CloneVerificationFailed { reason: String },
    
    #[error("Database error: {source}")]
    DatabaseError { 
        #[from] 
        source: DatabaseError 
    },
}

/// Database cloning result type
pub type CloneResult<T> = Result<T, CloneError>;

/// Manager for database cloning operations
///
/// The `CloneManager` handles creating database clones from templates using `PostgreSQL`'s
/// native `CREATE DATABASE WITH TEMPLATE` command for maximum performance.
///
/// # Performance
/// Target: Database cloning in <100ms for small-to-medium databases
///
/// # Example
/// ```rust,no_run
/// use dbfast::{Config, DatabasePool, clone::CloneManager};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::from_file("dbfast.toml")?;
/// let pool = DatabasePool::new(&config.database).await?;
/// let clone_manager = CloneManager::new(pool);
///
/// clone_manager.clone_database("my_template", "my_clone").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct CloneManager {
    pool: DatabasePool,
}

impl CloneManager {
    /// Create a new clone manager with the given database pool
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing clone operations
    #[must_use]
    pub const fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Validate database name for security and PostgreSQL compatibility
    /// 
    /// # Arguments
    /// * `name` - Database name to validate
    /// 
    /// # Returns
    /// * `Ok(())` if name is valid
    /// * `Err(CloneError::InvalidDatabaseName)` if name is invalid
    pub fn validate_database_name(name: &str) -> CloneResult<()> {
        // Basic security checks for SQL injection prevention
        if name.contains('\'') || name.contains(';') || name.contains('-') || name.contains('.') {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name contains invalid characters that could be used for SQL injection".to_string(),
            });
        }

        // Check for empty name
        if name.is_empty() {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot be empty".to_string(),
            });
        }

        // Check for too short names
        if name.len() < 2 {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name must be at least 2 characters long".to_string(),
            });
        }

        // Check for PostgreSQL length limit (63 characters)
        if name.len() > 63 {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: format!("Database name exceeds PostgreSQL limit of 63 characters (got {})", name.len()),
            });
        }

        // Check for spaces and special characters
        if name.contains(' ') || name.contains('@') {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot contain spaces or special characters".to_string(),
            });
        }

        // Check for uppercase letters (PostgreSQL converts to lowercase, but we should be explicit)
        if name.chars().any(|c| c.is_uppercase()) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name should be lowercase only".to_string(),
            });
        }

        // Check for names starting with numbers
        if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot start with a number".to_string(),
            });
        }

        // Check for PostgreSQL reserved words
        let reserved_words = ["select", "table", "drop", "create", "database", "user", "group"];
        if reserved_words.contains(&name.to_lowercase().as_str()) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot be a PostgreSQL reserved word".to_string(),
            });
        }

        Ok(())
    }

    /// Clone a database from a template using `CREATE DATABASE WITH TEMPLATE`
    ///
    /// This method executes `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE` command
    /// to create a fast, complete copy of an existing template database.
    ///
    /// # Arguments
    /// * `template_name` - Name of the template database to clone from
    /// * `clone_name` - Name for the new cloned database
    ///
    /// # Performance
    /// Target: <100ms for small-to-medium databases
    ///
    /// # Errors
    /// Returns `DatabaseError` if:
    /// - Template database doesn't exist
    /// - Clone database name already exists
    /// - Database connection fails
    /// - `PostgreSQL` permissions are insufficient
    pub async fn clone_database(&self, template_name: &str, clone_name: &str) -> CloneResult<()> {
        let start = Instant::now();

        // Validate both template and clone names for security
        Self::validate_database_name(template_name)?;
        Self::validate_database_name(clone_name)?;

        // Execute the clone command using PostgreSQL's native template functionality
        let clone_sql = format!("CREATE DATABASE {clone_name} WITH TEMPLATE {template_name}");

        // Execute the clone operation
        self.pool.query(&clone_sql, &[]).await
            .map_err(|db_err| CloneError::DatabaseError { source: db_err })?;

        // Log performance metrics for monitoring
        let duration = start.elapsed();
        println!(
            "Database cloned: {template_name} -> {clone_name} in {}ms",
            duration.as_millis()
        );

        Ok(())
    }

    /// Drop a cloned database
    ///
    /// Removes a database completely. Use with caution.
    ///
    /// # Arguments
    /// * `database_name` - Name of the database to drop
    ///
    /// # Safety
    /// This operation is irreversible. Ensure the database is no longer needed.
    ///
    /// # Errors
    /// Returns `DatabaseError` if:
    /// - Database connection fails
    /// - `PostgreSQL` permissions are insufficient
    /// - Database is currently in use by other connections
    pub async fn drop_database(&self, database_name: &str) -> CloneResult<()> {
        let drop_sql = format!("DROP DATABASE IF EXISTS {database_name}");
        self.pool.query(&drop_sql, &[]).await
            .map_err(|db_err| CloneError::DatabaseError { source: db_err })?;

        println!("Database dropped: {database_name}");
        Ok(())
    }
}
