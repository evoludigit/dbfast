//! Comprehensive error handling system for DBFast
//!
//! This module provides a unified error handling approach with:
//! - Contextual error information
//! - Error categorization for better handling
//! - Rich diagnostic information
//! - Structured error reporting

use std::fmt;
use thiserror::Error;
use tracing::{error, warn};

/// The main error type for DBFast operations
#[derive(Debug, Clone, Error)]
pub enum DbFastError {
    /// Configuration-related errors
    #[error("Configuration error: {source}")]
    Config {
        source: ConfigurationError,
        context: ErrorContext,
    },

    /// Database operation errors
    #[error("Database error: {source}")]
    Database {
        source: DatabaseError,
        context: ErrorContext,
    },

    /// Remote operation errors
    #[error("Remote operation error: {source}")]
    Remote {
        source: RemoteError,
        context: ErrorContext,
    },

    /// Deployment operation errors
    #[error("Deployment error: {source}")]
    Deployment {
        source: DeploymentError,
        context: ErrorContext,
    },

    /// File system operation errors
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        context: ErrorContext,
    },

    /// Network operation errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        context: ErrorContext,
    },

    /// Validation errors
    #[error("Validation error: {source}")]
    Validation {
        source: ValidationError,
        context: ErrorContext,
    },

    /// Authentication/authorization errors
    #[error("Authentication error: {source}")]
    Auth {
        source: AuthenticationError,
        context: ErrorContext,
    },

    /// System resource errors (memory, disk, etc.)
    #[error("Resource error: {source}")]
    Resource {
        source: ResourceError,
        context: ErrorContext,
    },
}

/// Configuration-specific errors
#[derive(Debug, Clone, Error)]
pub enum ConfigurationError {
    #[error("Configuration file not found: {path}")]
    NotFound { path: String },

    #[error("Invalid configuration format: {details}")]
    InvalidFormat { details: String },

    #[error("Missing required configuration: {field}")]
    MissingField { field: String },

    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Configuration parsing failed: {details}")]
    ParseError { details: String },
}

/// Database-specific errors
#[derive(Debug, Clone, Error)]
pub enum DatabaseError {
    #[error("Connection failed: {details}")]
    ConnectionFailed { details: String },

    #[error("Query execution failed: {query}")]
    QueryFailed { query: String },

    #[error("Transaction failed: {operation}")]
    TransactionFailed { operation: String },

    #[error("Database not found: {name}")]
    DatabaseNotFound { name: String },

    #[error("Permission denied for operation: {operation}")]
    PermissionDenied { operation: String },

    #[error("Connection pool exhausted")]
    PoolExhausted,

    #[error("Timeout during operation: {operation}")]
    Timeout { operation: String },
}

/// Remote operation errors
#[derive(Debug, Clone, Error)]
pub enum RemoteError {
    #[error("Remote not configured: {name}")]
    NotConfigured { name: String },

    #[error("Connection to remote failed: {url}")]
    ConnectionFailed { url: String },

    #[error("Authentication failed for remote: {name}")]
    AuthenticationFailed { name: String },

    #[error("Invalid remote URL: {url}")]
    InvalidUrl { url: String },

    #[error("Remote operation timeout: {operation}")]
    Timeout { operation: String },
}

/// Deployment-specific errors
#[derive(Debug, Clone, Error)]
pub enum DeploymentError {
    #[error("Pre-deployment validation failed: {reason}")]
    PreValidationFailed { reason: String },

    #[error("Backup creation failed: {reason}")]
    BackupFailed { reason: String },

    #[error("Template creation failed: {reason}")]
    TemplateCreationFailed { reason: String },

    #[error("Deployment transfer failed: {reason}")]
    TransferFailed { reason: String },

    #[error("Post-deployment validation failed: {reason}")]
    PostValidationFailed { reason: String },

    #[error("Rollback failed: {reason}")]
    RollbackFailed { reason: String },

    #[error("Environment mismatch: expected {expected}, got {actual}")]
    EnvironmentMismatch { expected: String, actual: String },
}

/// Validation errors
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Invalid format for {field}: {value}")]
    InvalidFormat { field: String, value: String },

    #[error("Value out of range for {field}: {value}")]
    OutOfRange { field: String, value: String },

    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },
}

/// Authentication/authorization errors
#[derive(Debug, Clone, Error)]
pub enum AuthenticationError {
    #[error("Credentials not found: {credential}")]
    CredentialsNotFound { credential: String },

    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("Access denied for operation: {operation}")]
    AccessDenied { operation: String },

    #[error("Token expired")]
    TokenExpired,
}

/// System resource errors
#[derive(Debug, Clone, Error)]
pub enum ResourceError {
    #[error("Insufficient disk space: {required} bytes needed")]
    InsufficientDiskSpace { required: u64 },

    #[error("Memory allocation failed: {size} bytes")]
    MemoryAllocation { size: u64 },

    #[error("File descriptor limit exceeded")]
    FileDescriptorLimit,

    #[error("Connection limit exceeded")]
    ConnectionLimit,
}

/// Error context provides additional information about when and where an error occurred
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed when error occurred
    pub operation: String,

    /// Component where error originated
    pub component: String,

    /// Additional context information
    pub details: std::collections::HashMap<String, String>,

    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Error severity level
    pub severity: ErrorSeverity,
}

/// Error severity levels (ordered from lowest to highest severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Low priority errors, mostly informational
    Low,

    /// Medium priority errors that may cause issues
    Medium,

    /// High priority errors that affect functionality
    High,

    /// Critical errors that require immediate attention
    Critical,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            operation: "unknown".to_string(),
            component: "unknown".to_string(),
            details: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            severity: ErrorSeverity::Medium,
        }
    }
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            details: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            severity: ErrorSeverity::Medium,
        }
    }

    /// Set the severity level
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Add additional context detail
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }

    /// Add multiple context details
    pub fn with_details(mut self, details: std::collections::HashMap<String, String>) -> Self {
        self.details.extend(details);
        self
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
        }
    }
}

/// Result type for DBFast operations
pub type DbFastResult<T> = Result<T, DbFastError>;

/// Trait for adding context to errors
pub trait ErrorContextExt<T> {
    /// Add context to an error
    fn with_context(self, operation: &str, component: &str) -> DbFastResult<T>;

    /// Add context with custom severity
    fn with_context_severity(
        self,
        operation: &str,
        component: &str,
        severity: ErrorSeverity,
    ) -> DbFastResult<T>;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<DbFastError>,
{
    fn with_context(self, operation: &str, component: &str) -> DbFastResult<T> {
        self.map_err(|e| {
            let mut error: DbFastError = e.into();

            // Add context to the error
            match &mut error {
                DbFastError::Config { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Database { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Remote { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Deployment { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::FileSystem { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Network { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Validation { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Auth { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
                DbFastError::Resource { context, .. } => {
                    *context = ErrorContext::new(operation, component);
                }
            }

            error
        })
    }

    fn with_context_severity(
        self,
        operation: &str,
        component: &str,
        severity: ErrorSeverity,
    ) -> DbFastResult<T> {
        self.with_context(operation, component)
            .map_err(|mut error| {
                // Update severity in context
                match &mut error {
                    DbFastError::Config { context, .. }
                    | DbFastError::Database { context, .. }
                    | DbFastError::Remote { context, .. }
                    | DbFastError::Deployment { context, .. }
                    | DbFastError::FileSystem { context, .. }
                    | DbFastError::Network { context, .. }
                    | DbFastError::Validation { context, .. }
                    | DbFastError::Auth { context, .. }
                    | DbFastError::Resource { context, .. } => {
                        context.severity = severity;
                    }
                }
                error
            })
    }
}

/// Error reporting utilities
impl DbFastError {
    /// Get the error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            Self::Config { context, .. }
            | Self::Database { context, .. }
            | Self::Remote { context, .. }
            | Self::Deployment { context, .. }
            | Self::FileSystem { context, .. }
            | Self::Network { context, .. }
            | Self::Validation { context, .. }
            | Self::Auth { context, .. }
            | Self::Resource { context, .. } => context,
        }
    }

    /// Log the error with appropriate level based on severity
    pub fn log(&self) {
        match self.context().severity {
            ErrorSeverity::Critical | ErrorSeverity::High => {
                error!(
                    error = %self,
                    operation = %self.context().operation,
                    component = %self.context().component,
                    severity = %self.context().severity,
                    "Error occurred"
                );
            }
            ErrorSeverity::Medium => {
                warn!(
                    error = %self,
                    operation = %self.context().operation,
                    component = %self.context().component,
                    "Warning occurred"
                );
            }
            ErrorSeverity::Low => {
                tracing::info!(
                    error = %self,
                    operation = %self.context().operation,
                    component = %self.context().component,
                    "Info: Minor issue occurred"
                );
            }
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Config { source, .. } => {
                format!("Configuration issue: {source}")
            }
            Self::Database { source, .. } => {
                format!("Database problem: {source}")
            }
            Self::Remote { source, .. } => {
                format!("Remote connection issue: {source}")
            }
            Self::Deployment { source, .. } => {
                format!("Deployment failed: {source}")
            }
            Self::FileSystem { message, .. } => {
                format!("File operation failed: {message}")
            }
            Self::Network { message, .. } => {
                format!("Network issue: {message}")
            }
            Self::Validation { source, .. } => {
                format!("Invalid input: {source}")
            }
            Self::Auth { source, .. } => {
                format!("Access denied: {source}")
            }
            Self::Resource { source, .. } => {
                format!("System resource issue: {source}")
            }
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Config { .. } | Self::Validation { .. } => false, // Usually not recoverable
            Self::Network { .. } | Self::Database { .. } => true,   // Often recoverable with retry
            Self::Remote { .. } => true,                            // May recover with retry
            Self::Deployment { source, .. } => {
                // Some deployment errors are recoverable
                !matches!(source, DeploymentError::RollbackFailed { .. })
            }
            Self::FileSystem { .. } => true, // Often recoverable
            Self::Auth { .. } => false,      // Usually requires user intervention
            Self::Resource { .. } => false,  // System-level issue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::new("test_operation", "test_component")
            .with_severity(ErrorSeverity::High)
            .with_detail("key", "value");

        assert_eq!(ctx.operation, "test_operation");
        assert_eq!(ctx.component, "test_component");
        assert_eq!(ctx.severity, ErrorSeverity::High);
        assert_eq!(ctx.details.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_error_recoverability() {
        let config_error = DbFastError::Config {
            source: ConfigurationError::NotFound {
                path: "test".to_string(),
            },
            context: ErrorContext::default(),
        };
        assert!(!config_error.is_recoverable());

        let network_error = DbFastError::Network {
            message: "Connection timeout".to_string(),
            context: ErrorContext::default(),
        };
        assert!(network_error.is_recoverable());
    }

    #[test]
    fn test_user_message_formatting() {
        let error = DbFastError::Database {
            source: DatabaseError::ConnectionFailed {
                details: "Connection refused".to_string(),
            },
            context: ErrorContext::default(),
        };

        let message = error.user_message();
        assert!(message.contains("Database problem"));
        assert!(message.contains("Connection refused"));
    }
}
