/// Simple database cloning functionality using `PostgreSQL`'s CREATE DATABASE WITH TEMPLATE
use crate::database::DatabasePool;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during database cloning
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

    #[error("Connection pool exhausted")]
    ConnectionPoolExhausted,

    #[error("Database error: {details}")]
    DatabaseError { details: String },

    #[error("Clone verification failed: {reason}")]
    CloneVerificationFailed { reason: String },
}

/// Configuration for clone operations
#[derive(Debug, Clone)]
pub struct CloneConfig {
    pub max_concurrent_clones: usize,
    pub clone_timeout: Duration,
    pub enable_verification: bool,
    pub verify_data_integrity: bool,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            max_concurrent_clones: 5,
            clone_timeout: Duration::from_secs(30),
            enable_verification: false,
            verify_data_integrity: false,
        }
    }
}

/// Simple database clone manager
#[derive(Clone)]
pub struct CloneManager {
    pool: DatabasePool,
    config: CloneConfig,
}

impl CloneManager {
    /// Create a new clone manager with default configuration
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            config: CloneConfig::default(),
        }
    }

    /// Create a new clone manager with custom configuration
    #[must_use]
    pub const fn new_with_config(pool: DatabasePool, config: CloneConfig) -> Self {
        Self { pool, config }
    }

    /// Clone a database using `PostgreSQL`'s template functionality
    pub async fn clone_database(
        &self,
        template_name: &str,
        clone_name: &str,
    ) -> Result<(), CloneError> {
        // Validate database names
        Self::validate_database_name(template_name)?;
        Self::validate_database_name(clone_name)?;

        let start = Instant::now();

        // Execute the clone operation
        let query = format!(
            "CREATE DATABASE {} WITH TEMPLATE {}",
            Self::escape_identifier(clone_name),
            Self::escape_identifier(template_name)
        );

        let result = self.pool.execute_non_transactional(&query, &[]).await;

        // Check for timeout
        if start.elapsed() > self.config.clone_timeout {
            return Err(CloneError::CloneTimeout {
                timeout_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
            });
        }

        // Handle database errors
        result.map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                CloneError::CloneAlreadyExists {
                    clone: clone_name.to_string(),
                }
            } else if error_msg.contains("does not exist") {
                CloneError::TemplateNotFound {
                    template: template_name.to_string(),
                }
            } else if error_msg.contains("permission") {
                CloneError::InsufficientPermissions {
                    name: clone_name.to_string(),
                }
            } else {
                CloneError::DatabaseError { details: error_msg }
            }
        })?;

        Ok(())
    }

    /// Drop/delete a database
    pub async fn drop_database(&self, database_name: &str) -> Result<(), CloneError> {
        Self::validate_database_name(database_name)?;

        let query = format!(
            "DROP DATABASE IF EXISTS {}",
            Self::escape_identifier(database_name)
        );

        self.pool
            .execute_non_transactional(&query, &[])
            .await
            .map_err(|e| CloneError::DatabaseError {
                details: e.to_string(),
            })?;

        Ok(())
    }

    /// Validate a database name for security and `PostgreSQL` compatibility
    pub fn validate_database_name(name: &str) -> Result<(), CloneError> {
        if name.is_empty() {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot be empty".to_string(),
            });
        }

        if name.len() > 63 {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name cannot exceed 63 characters".to_string(),
            });
        }

        // Check for SQL injection attempts
        let dangerous_chars = [';', '\'', '"', '\\', '\0', '\n', '\r'];
        if name.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name contains invalid characters".to_string(),
            });
        }

        // Must start with letter or underscore
        if !name.chars().next().unwrap().is_ascii_alphabetic() && !name.starts_with('_') {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name must start with a letter or underscore".to_string(),
            });
        }

        // Only allow alphanumeric and underscores
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(CloneError::InvalidDatabaseName {
                name: name.to_string(),
                reason: "Database name can only contain letters, numbers, and underscores"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Escape a database identifier for safe SQL usage
    fn escape_identifier(name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_database_name() {
        // Valid names
        assert!(CloneManager::validate_database_name("test_db").is_ok());
        assert!(CloneManager::validate_database_name("_private").is_ok());
        assert!(CloneManager::validate_database_name("db123").is_ok());

        // Invalid names
        assert!(CloneManager::validate_database_name("").is_err());
        assert!(CloneManager::validate_database_name("test'; DROP DATABASE").is_err());
        assert!(CloneManager::validate_database_name("test\"db").is_err());
        assert!(CloneManager::validate_database_name("123db").is_err());
        assert!(CloneManager::validate_database_name("test-db").is_err());
    }

    #[test]
    fn test_clone_config_default() {
        let config = CloneConfig::default();
        assert_eq!(config.max_concurrent_clones, 5);
        assert_eq!(config.clone_timeout, Duration::from_secs(30));
        assert!(!config.enable_verification);
    }
}
