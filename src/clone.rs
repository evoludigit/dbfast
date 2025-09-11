/// Database cloning functionality for `DBFast`
///
/// This module provides fast database cloning using `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE`
/// functionality. It's designed to achieve database cloning in under 100ms for optimal performance.
use crate::database::{DatabaseError, DatabasePool};
use std::time::Instant;
use thiserror::Error;

/// Enhanced error types for database cloning operations
#[derive(Debug, Error)]
pub enum CloneError {
    /// Database name validation failed due to security or compatibility issues
    #[error("Invalid database name: {name}. {reason}")]
    InvalidDatabaseName {
        /// The invalid database name that was provided
        name: String,
        /// Detailed reason why the name is invalid
        reason: String,
    },

    /// Template database does not exist in the `PostgreSQL` instance
    #[error("Template database '{template}' does not exist")]
    TemplateNotFound {
        /// Name of the template database that was not found
        template: String,
    },

    /// Clone database name already exists, preventing creation
    #[error("Clone database '{clone}' already exists")]
    CloneAlreadyExists {
        /// Name of the clone database that already exists
        clone: String,
    },

    /// Clone operation exceeded the configured timeout limit
    #[error("Clone operation timed out after {timeout_ms}ms")]
    CloneTimeout {
        /// Timeout duration in milliseconds that was exceeded
        timeout_ms: u64,
    },

    /// User lacks sufficient `PostgreSQL` permissions for database creation
    #[error("Insufficient permissions to create database '{name}'")]
    InsufficientPermissions {
        /// Name of the database that couldn't be created due to permissions
        name: String,
    },

    /// Database connection pool has no available connections
    #[error("Connection pool exhausted during clone operation")]
    ConnectionPoolExhausted,

    /// Post-clone verification detected data integrity issues
    #[error("Clone verification failed: {reason}")]
    CloneVerificationFailed {
        /// Specific reason the clone verification failed
        reason: String,
    },

    /// Underlying database operation failed
    #[error("Database error: {source}")]
    DatabaseError {
        /// The underlying database error
        #[from]
        source: DatabaseError,
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

    /// Validate database name for security and `PostgreSQL` compatibility
    ///
    /// This function provides comprehensive validation to prevent SQL injection attacks
    /// and ensure compatibility with `PostgreSQL` naming conventions.
    ///
    /// # Arguments
    /// * `name` - Database name to validate
    ///
    /// # Returns
    /// * `Ok(())` if name is valid and safe to use
    /// * `Err(CloneError::InvalidDatabaseName)` if name violates security or compatibility rules
    ///
    /// # Security Features
    /// - Prevents SQL injection through character filtering
    /// - Blocks `PostgreSQL` reserved words
    /// - Enforces secure naming conventions
    ///
    /// # Examples
    /// ```rust,no_run
    /// use dbfast::clone::CloneManager;
    ///
    /// // Valid names
    /// assert!(CloneManager::validate_database_name("user_data_2024").is_ok());
    /// assert!(CloneManager::validate_database_name("app_staging").is_ok());
    ///
    /// // Invalid names (SQL injection attempts)
    /// assert!(CloneManager::validate_database_name("db'; DROP TABLE users; --").is_err());
    /// assert!(CloneManager::validate_database_name("select").is_err()); // reserved word
    /// ```
    #[allow(clippy::too_many_lines)] // Security validation requires comprehensive checks
    pub fn validate_database_name(name: &str) -> CloneResult<()> {
        const MAX_NAME_LENGTH: usize = 63; // PostgreSQL identifier limit
        const MIN_NAME_LENGTH: usize = 2;
        const DANGEROUS_CHARS: &[char] = &[
            '\'', '"', ';', '-', '.', '(', ')', '[', ']', '{', '}', '\\', '/', '*', '+', '=', '<',
            '>', '!', '?', '|', '&', '%', '$', '#', '@',
        ];
        const RESERVED_WORDS: &[&str] = &[
            // SQL standard reserved words
            "select",
            "insert",
            "update",
            "delete",
            "create",
            "drop",
            "alter",
            "table",
            "database",
            "index",
            "view",
            "trigger",
            "function",
            "procedure",
            "schema",
            "user",
            "group",
            "role",
            "grant",
            "revoke",
            "commit",
            "rollback",
            "transaction",
            "begin",
            "end",
            "if",
            "else",
            "while",
            "for",
            "loop",
            "return",
            "declare",
            "set",
            "with",
            "as",
            "from",
            "where",
            "order",
            "group",
            "having",
            "union",
            "join",
            "inner",
            "outer",
            "left",
            "right",
            "full",
            "on",
            "and",
            "or",
            "not",
            "null",
            "true",
            "false",
            "exists",
            "in",
            "like",
            "between",
            "case",
            "when",
            "then",
            "distinct",
            "all",
            "any",
            "some",
            // PostgreSQL specific
            "postgresql",
            "postgres",
            "pg_catalog",
            "information_schema",
            "public",
            "template0",
            "template1",
            "admin",
            "root",
            "superuser",
            "replication",
            "backup",
            "restore",
        ];

        // 1. Check for empty or too short names
        if name.is_empty() {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot be empty".to_string(),
            });
        }

        if name.len() < MIN_NAME_LENGTH {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: format!("Database name must be at least {MIN_NAME_LENGTH} characters long"),
            });
        }

        // 2. Check PostgreSQL length limit
        if name.len() > MAX_NAME_LENGTH {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: format!(
                    "Database name exceeds PostgreSQL limit of {MAX_NAME_LENGTH} characters (got {})",
                    name.len()
                ),
            });
        }

        // 3. SQL Injection prevention - check for dangerous characters
        if let Some(dangerous_char) = name.chars().find(|c| DANGEROUS_CHARS.contains(c)) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: format!(
                    "Database name contains dangerous character '{dangerous_char}' that could enable SQL injection"
                ),
            });
        }

        // 4. Check for whitespace characters
        if name.chars().any(char::is_whitespace) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot contain whitespace characters".to_string(),
            });
        }

        // 5. Enforce lowercase convention (PostgreSQL folds to lowercase anyway)
        if name.chars().any(char::is_uppercase) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name must be lowercase only for consistency".to_string(),
            });
        }

        // 6. Check first character (must be letter or underscore)
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_ascii_lowercase() && first_char != '_' {
                return Err(CloneError::InvalidDatabaseName {
                    name: name.to_string(),
                    reason: "Database name must start with a lowercase letter or underscore"
                        .to_string(),
                });
            }
        }

        // 7. Check all characters are alphanumeric or underscore
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason:
                    "Database name can only contain lowercase letters, numbers, and underscores"
                        .to_string(),
            });
        }

        // 8. Check for PostgreSQL reserved words
        let name_lower = name.to_lowercase();
        if RESERVED_WORDS.contains(&name_lower.as_str()) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: format!(
                    "'{name}' is a reserved word and cannot be used as a database name"
                ),
            });
        }

        // 9. Additional security checks for common attack patterns
        if name.contains("__") {
            // double underscore might be used for injection
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot contain consecutive underscores".to_string(),
            });
        }

        if name.starts_with('_') && name.len() == MIN_NAME_LENGTH {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot be only an underscore and one character".to_string(),
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
        self.pool
            .query(&clone_sql, &[])
            .await
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
        self.pool
            .query(&drop_sql, &[])
            .await
            .map_err(|db_err| CloneError::DatabaseError { source: db_err })?;

        println!("Database dropped: {database_name}");
        Ok(())
    }
}
