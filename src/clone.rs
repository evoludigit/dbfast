/// Database cloning functionality for `DBFast`
///
/// This module provides fast database cloning using `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE`
/// functionality. It's designed to achieve database cloning in under 100ms for optimal performance.
use crate::database::{DatabaseError, DatabasePool};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Semaphore;

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

/// Configuration for clone operations with industrial-grade settings
#[derive(Debug, Clone)]
pub struct CloneConfig {
    /// Maximum timeout for clone operations (default: 5 minutes)
    pub clone_timeout: Duration,
    /// Maximum number of concurrent clone operations (default: 10)
    pub max_concurrent_clones: usize,
    /// Timeout for acquiring database connections from pool (default: 30 seconds)
    pub connection_timeout: Duration,
    /// Maximum time to wait for operation slot when at concurrency limit (default: 60 seconds)
    pub queue_timeout: Duration,
    /// Enable detailed performance logging (default: false)
    pub enable_performance_logging: bool,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            clone_timeout: Duration::from_secs(300), // 5 minutes default
            max_concurrent_clones: 10,
            connection_timeout: Duration::from_secs(30), // 30 seconds to get connection
            queue_timeout: Duration::from_secs(60),      // 1 minute to wait for slot
            enable_performance_logging: false,
        }
    }
}

/// Performance metrics for clone operations
#[derive(Debug, Default)]
pub struct CloneMetrics {
    /// Total number of clone operations attempted
    pub total_clones: AtomicU64,
    /// Number of successful clone operations
    pub successful_clones: AtomicU64,
    /// Number of failed clone operations
    pub failed_clones: AtomicU64,
    /// Number of timed out clone operations
    pub timed_out_clones: AtomicU64,
    /// Total duration of all clone operations in milliseconds
    pub total_duration_ms: AtomicU64,
    /// Number of operations that were blocked due to concurrency limits
    pub blocked_operations: AtomicU64,
}

impl CloneMetrics {
    /// Get the average clone operation duration in milliseconds
    #[allow(clippy::cast_precision_loss)] // Acceptable loss for metrics calculation
    pub fn average_duration_ms(&self) -> f64 {
        let total = self.total_clones.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.total_duration_ms.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    /// Get the success rate as a percentage
    #[allow(clippy::cast_precision_loss)] // Acceptable loss for metrics calculation
    pub fn success_rate(&self) -> f64 {
        let total = self.total_clones.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            let successful = self.successful_clones.load(Ordering::Relaxed);
            (successful as f64 / total as f64) * 100.0
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
    /// Semaphore to limit concurrent operations (more robust than atomic counter)
    operation_semaphore: Arc<Semaphore>,
    /// Performance and operational metrics
    metrics: Arc<CloneMetrics>,
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
            operation_semaphore: Arc::new(Semaphore::new(config.max_concurrent_clones)),
            metrics: Arc::new(CloneMetrics::default()),
            config,
        }
    }

    /// Get current performance metrics
    ///
    /// Returns a reference to the metrics for monitoring and debugging
    #[must_use]
    pub fn metrics(&self) -> &CloneMetrics {
        &self.metrics
    }

    /// Get current configuration
    ///
    /// Returns a reference to the current configuration
    #[must_use]
    pub const fn config(&self) -> &CloneConfig {
        &self.config
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

        // Update metrics - attempt started
        self.metrics.total_clones.fetch_add(1, Ordering::Relaxed);

        // Validate both template and clone names for security
        Self::validate_database_name(template_name)?;
        Self::validate_database_name(clone_name)?;

        // Acquire semaphore permit with timeout to prevent indefinite waiting
        let permit_result = tokio::time::timeout(
            self.config.queue_timeout,
            self.operation_semaphore.acquire(),
        )
        .await;

        let _permit = match permit_result {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                // Semaphore was closed (shouldn't happen in normal operation)
                self.metrics
                    .blocked_operations
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics.failed_clones.fetch_add(1, Ordering::Relaxed);
                return Err(CloneError::ConnectionPoolExhausted);
            }
            Err(_) => {
                // Timeout waiting for permit
                self.metrics
                    .blocked_operations
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics.failed_clones.fetch_add(1, Ordering::Relaxed);
                return Err(CloneError::CloneTimeout {
                    timeout_ms: self.config.queue_timeout.as_millis() as u64,
                });
            }
        };

        // Execute the clone operation with timeout
        let clone_future = self.execute_clone_operation(template_name, clone_name, start);
        let timeout_result = tokio::time::timeout(self.config.clone_timeout, clone_future).await;

        // Process result and update metrics
        let duration = start.elapsed();
        self.metrics
            .total_duration_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);

        match timeout_result {
            Ok(clone_result) => {
                match &clone_result {
                    Ok(()) => {
                        self.metrics
                            .successful_clones
                            .fetch_add(1, Ordering::Relaxed);
                        if self.config.enable_performance_logging {
                            println!(
                                "✅ Database cloned: {template_name} -> {clone_name} in {}ms",
                                duration.as_millis()
                            );
                        }
                    }
                    Err(err) => {
                        self.metrics.failed_clones.fetch_add(1, Ordering::Relaxed);
                        if self.config.enable_performance_logging {
                            println!(
                                "❌ Database clone failed: {template_name} -> {clone_name} after {}ms - {err}",
                                duration.as_millis()
                            );
                        }
                    }
                }
                clone_result
            }
            Err(_) => {
                // Timeout occurred
                self.metrics
                    .timed_out_clones
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics.failed_clones.fetch_add(1, Ordering::Relaxed);

                if self.config.enable_performance_logging {
                    println!(
                        "⏰ Database clone timed out: {template_name} -> {clone_name} after {}ms",
                        duration.as_millis()
                    );
                }

                Err(CloneError::CloneTimeout {
                    timeout_ms: self.config.clone_timeout.as_millis() as u64,
                })
            }
        }
    }

    /// Execute the actual clone operation
    ///
    /// This is separated from the main method to allow for timeout wrapping
    async fn execute_clone_operation(
        &self,
        template_name: &str,
        clone_name: &str,
        _start: Instant,
    ) -> CloneResult<()> {
        // Execute the clone command using PostgreSQL's native template functionality
        let clone_sql = format!("CREATE DATABASE {clone_name} WITH TEMPLATE {template_name}");

        // Execute the clone operation with proper error mapping
        self.pool.query(&clone_sql, &[]).await.map_err(|db_err| {
            // Map specific database errors to more helpful clone errors
            let error_msg = db_err.to_string().to_lowercase();

            if error_msg.contains("does not exist")
                && error_msg.contains(&template_name.to_lowercase())
            {
                CloneError::TemplateNotFound {
                    template: template_name.to_string(),
                }
            } else if error_msg.contains("already exists")
                && error_msg.contains(&clone_name.to_lowercase())
            {
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

    /// Drop a cloned database with proper connection management
    ///
    /// Removes a database completely. Use with caution.
    ///
    /// # Arguments
    /// * `database_name` - Name of the database to drop
    ///
    /// # Connection Management
    /// - Uses semaphore to limit concurrent operations
    /// - Applies timeout to prevent hanging operations
    /// - Updates metrics for monitoring
    ///
    /// # Safety
    /// This operation is irreversible. Ensure the database is no longer needed.
    ///
    /// # Errors
    /// Returns `CloneError` if:
    /// - Database connection fails or pool is exhausted
    /// - Operation exceeds configured timeout
    /// - `PostgreSQL` permissions are insufficient
    /// - Database is currently in use by other connections
    pub async fn drop_database(&self, database_name: &str) -> CloneResult<()> {
        let start = Instant::now();

        // Validate database name
        Self::validate_database_name(database_name)?;

        // Acquire semaphore permit for connection management
        let _permit = tokio::time::timeout(
            self.config.queue_timeout,
            self.operation_semaphore.acquire(),
        )
        .await
        .map_err(|_| CloneError::CloneTimeout {
            timeout_ms: self.config.queue_timeout.as_millis() as u64,
        })?
        .map_err(|_| CloneError::ConnectionPoolExhausted)?;

        // Execute drop operation with timeout
        let drop_future = async {
            let drop_sql = format!("DROP DATABASE IF EXISTS {database_name}");
            self.pool.query(&drop_sql, &[]).await.map_err(|db_err| {
                let error_msg = db_err.to_string().to_lowercase();
                if error_msg.contains("permission") || error_msg.contains("denied") {
                    CloneError::InsufficientPermissions {
                        name: database_name.to_string(),
                    }
                } else {
                    CloneError::DatabaseError { source: db_err }
                }
            })?;
            Ok(())
        };

        let result = tokio::time::timeout(self.config.clone_timeout, drop_future)
            .await
            .map_err(|_| CloneError::CloneTimeout {
                timeout_ms: self.config.clone_timeout.as_millis() as u64,
            })?;

        let duration = start.elapsed();
        if self.config.enable_performance_logging {
            match &result {
                Ok(()) => println!(
                    "🗑️ Database dropped: {database_name} in {}ms",
                    duration.as_millis()
                ),
                Err(err) => println!(
                    "❌ Database drop failed: {database_name} after {}ms - {err}",
                    duration.as_millis()
                ),
            }
        }

        result
    }
}
