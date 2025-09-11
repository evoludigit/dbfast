/// Database cloning functionality for `DBFast`
///
/// This module provides fast database cloning using `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE`
/// functionality. It's designed to achieve database cloning in under 100ms for optimal performance.
use crate::database::{DatabaseError, DatabasePool};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
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

/// Configuration for clone operations
#[derive(Debug, Clone)]
pub struct CloneConfig {
    /// Maximum timeout for clone operations
    pub clone_timeout: Duration,
    /// Maximum number of concurrent clone operations
    pub max_concurrent_clones: usize,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            clone_timeout: Duration::from_secs(300), // 5 minutes default
            max_concurrent_clones: 10,
        }
    }
}

/// Manager for database cloning operations
///
/// The `CloneManager` handles creating database clones from templates using `PostgreSQL`'s
/// native `CREATE DATABASE WITH TEMPLATE` command for maximum performance with proper
/// connection management and timeout handling.
///
/// # Performance
/// Target: Database cloning in <200ms for small-to-medium databases
///
/// # Connection Management
/// - Automatic connection pool exhaustion detection
/// - Timeout handling for long-running operations  
/// - Resource cleanup on operation failure
/// - Concurrent operation limits
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
    config: CloneConfig,
    active_operations: Arc<AtomicUsize>,
}

impl CloneManager {
    /// Create a new clone manager with the given database pool using default configuration
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing clone operations
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self::new_with_config(pool, CloneConfig::default())
    }

    /// Create a new clone manager with the given database pool and configuration
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing clone operations
    /// * `config` - Configuration for clone operations (timeouts, limits, etc.)
    #[must_use]
    pub fn new_with_config(pool: DatabasePool, config: CloneConfig) -> Self {
        Self {
            pool,
            config,
            active_operations: Arc::new(AtomicUsize::new(0)),
        }
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
    /// to create a fast, complete copy of an existing template database with proper
    /// connection management and timeout handling.
    ///
    /// # Arguments
    /// * `template_name` - Name of the template database to clone from
    /// * `clone_name` - Name for the new cloned database
    ///
    /// # Performance
    /// Target: <200ms for small-to-medium databases
    ///
    /// # Connection Management
    /// - Checks for connection pool exhaustion before starting
    /// - Enforces timeout limits to prevent hanging operations
    /// - Tracks concurrent operations to prevent overload
    /// - Ensures proper cleanup on success or failure
    ///
    /// # Errors
    /// Returns `CloneError` if:
    /// - Template database doesn't exist
    /// - Clone database name already exists  
    /// - Database connection fails or pool is exhausted
    /// - Operation exceeds configured timeout
    /// - `PostgreSQL` permissions are insufficient
    pub async fn clone_database(&self, template_name: &str, clone_name: &str) -> CloneResult<()> {
        let start = Instant::now();

        // Validate both template and clone names for security
        Self::validate_database_name(template_name)?;
        Self::validate_database_name(clone_name)?;

        // Check if we've exceeded concurrent operation limits
        let active_count = self.active_operations.fetch_add(1, Ordering::SeqCst);
        if active_count >= self.config.max_concurrent_clones {
            self.active_operations.fetch_sub(1, Ordering::SeqCst);
            return Err(CloneError::ConnectionPoolExhausted);
        }

        // Wrap the operation in a timeout
        let clone_future = self.execute_clone_operation(template_name, clone_name);
        let timeout_result = tokio::time::timeout(self.config.clone_timeout, clone_future).await;

        // Decrement active operations count regardless of outcome
        self.active_operations.fetch_sub(1, Ordering::SeqCst);

        match timeout_result {
            Ok(clone_result) => {
                // Log performance metrics for monitoring
                let duration = start.elapsed();
                match &clone_result {
                    Ok(()) => {
                        println!(
                            "Database cloned: {template_name} -> {clone_name} in {}ms",
                            duration.as_millis()
                        );
                    }
                    Err(_) => {
                        println!(
                            "Database clone failed: {template_name} -> {clone_name} after {}ms",
                            duration.as_millis()
                        );
                    }
                }
                clone_result
            }
            Err(_) => {
                // Timeout occurred
                let duration = start.elapsed();
                println!(
                    "Database clone timed out: {template_name} -> {clone_name} after {}ms",
                    duration.as_millis()
                );
                Err(CloneError::CloneTimeout {
                    timeout_ms: self.config.clone_timeout.as_millis() as u64,
                })
            }
        }
    }

    /// Execute the actual clone operation
    /// 
    /// This is separated from the main method to allow for timeout wrapping
    async fn execute_clone_operation(&self, template_name: &str, clone_name: &str) -> CloneResult<()> {
        // Execute the clone command using PostgreSQL's native template functionality
        let clone_sql = format!("CREATE DATABASE {clone_name} WITH TEMPLATE {template_name}");

        // Execute the clone operation with proper error mapping
        self.pool
            .query(&clone_sql, &[])
            .await
            .map_err(|db_err| {
                // Map specific database errors to more helpful clone errors
                let error_msg = db_err.to_string().to_lowercase();
                
                if error_msg.contains("does not exist") && error_msg.contains(&template_name.to_lowercase()) {
                    CloneError::TemplateNotFound {
                        template: template_name.to_string(),
                    }
                } else if error_msg.contains("already exists") && error_msg.contains(&clone_name.to_lowercase()) {
                    CloneError::CloneAlreadyExists {
                        clone: clone_name.to_string(),
                    }
                } else if error_msg.contains("permission") || error_msg.contains("denied") {
                    CloneError::InsufficientPermissions {
                        name: clone_name.to_string(),
                    }
                } else {
                    CloneError::DatabaseError { source: db_err }
                }
            })?;

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
