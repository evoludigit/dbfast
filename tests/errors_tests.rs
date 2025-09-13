//! Comprehensive tests for the error handling system

use dbfast::errors::{
    ConfigurationError, DatabaseError, DbFastError, DeploymentError, ErrorContext, ErrorSeverity,
    RemoteError,
};
use std::collections::HashMap;

#[test]
fn test_error_context_creation() {
    let context = ErrorContext {
        operation: "test_operation".to_string(),
        component: "test_component".to_string(),
        details: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map
        },
        severity: ErrorSeverity::High,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(context.operation, "test_operation");
    assert_eq!(context.severity, ErrorSeverity::High);
}

#[test]
fn test_configuration_error_variants() {
    let missing_field = ConfigurationError::MissingField {
        field: "database_url".to_string(),
    };

    let invalid_value = ConfigurationError::InvalidValue {
        field: "port".to_string(),
        value: "invalid".to_string(),
    };

    assert!(matches!(
        missing_field,
        ConfigurationError::MissingField { .. }
    ));
    assert!(matches!(
        invalid_value,
        ConfigurationError::InvalidValue { .. }
    ));
}

#[test]
fn test_database_error_variants() {
    let connection_error = DatabaseError::ConnectionFailed {
        details: "Connection refused".to_string(),
    };

    let query_error = DatabaseError::QueryFailed {
        query: "SELECT * FROM non_existent".to_string(),
    };

    assert!(matches!(
        connection_error,
        DatabaseError::ConnectionFailed { .. }
    ));
    assert!(matches!(query_error, DatabaseError::QueryFailed { .. }));
}

#[test]
fn test_error_severity_ordering() {
    assert!(ErrorSeverity::Critical > ErrorSeverity::High);
    assert!(ErrorSeverity::High > ErrorSeverity::Medium);
    assert!(ErrorSeverity::Medium > ErrorSeverity::Low);
}

#[test]
fn test_dbfast_error_creation() {
    let context = ErrorContext {
        operation: "database_connection".to_string(),
        component: "connection_pool".to_string(),
        details: HashMap::new(),
        severity: ErrorSeverity::Critical,
        timestamp: chrono::Utc::now(),
    };

    let db_error = DatabaseError::ConnectionFailed {
        details: "Connection timeout".to_string(),
    };

    let dbfast_error = DbFastError::Database {
        source: db_error,
        context: Box::new(context.clone()),
    };

    match dbfast_error {
        DbFastError::Database { source, context } => {
            assert!(matches!(source, DatabaseError::ConnectionFailed { .. }));
            assert_eq!(context.severity, ErrorSeverity::Critical);
        }
        _ => panic!("Expected Database error variant"),
    }
}

#[test]
fn test_error_display_formatting() {
    let error = DbFastError::FileSystem {
        message: "Permission denied".to_string(),
        context: Box::new(ErrorContext {
            operation: "read_config".to_string(),
            component: "config_loader".to_string(),
            details: HashMap::new(),
            severity: ErrorSeverity::High,
            timestamp: chrono::Utc::now(),
        }),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("File system error"));
    assert!(error_string.contains("Permission denied"));
}

#[test]
fn test_remote_error_variants() {
    let auth_error = RemoteError::AuthenticationFailed {
        name: "remote.example.com".to_string(),
    };

    let connection_error = RemoteError::ConnectionFailed {
        url: "postgres://remote.example.com:5432/db".to_string(),
    };

    assert!(matches!(
        auth_error,
        RemoteError::AuthenticationFailed { .. }
    ));
    assert!(matches!(
        connection_error,
        RemoteError::ConnectionFailed { .. }
    ));
}

#[test]
fn test_deployment_error_variants() {
    let validation_error = DeploymentError::PreValidationFailed {
        reason: "Missing required environment variable".to_string(),
    };

    let backup_error = DeploymentError::BackupFailed {
        reason: "Insufficient storage space".to_string(),
    };

    match validation_error {
        DeploymentError::PreValidationFailed { reason } => {
            assert!(reason.contains("environment variable"));
        }
        _ => panic!("Expected PreValidationFailed variant"),
    }

    match backup_error {
        DeploymentError::BackupFailed { reason } => {
            assert!(reason.contains("storage space"));
        }
        _ => panic!("Expected BackupFailed variant"),
    }
}

#[test]
fn test_error_context_details() {
    let mut details = HashMap::new();
    details.insert("user_id".to_string(), "12345".to_string());
    details.insert("operation_id".to_string(), "op_987".to_string());
    details.insert("retry_attempt".to_string(), "3".to_string());

    let context = ErrorContext {
        operation: "user_update".to_string(),
        component: "user_service".to_string(),
        details,
        severity: ErrorSeverity::Medium,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(context.details.len(), 3);
    assert_eq!(context.details.get("user_id"), Some(&"12345".to_string()));
    assert_eq!(context.details.get("retry_attempt"), Some(&"3".to_string()));
}

#[test]
fn test_error_context_builder() {
    let context = ErrorContext::new("test_op", "test_component")
        .with_severity(ErrorSeverity::Critical)
        .with_detail("key1", "value1")
        .with_detail("key2", "value2");

    assert_eq!(context.operation, "test_op");
    assert_eq!(context.component, "test_component");
    assert_eq!(context.severity, ErrorSeverity::Critical);
    assert_eq!(context.details.len(), 2);
    assert_eq!(context.details.get("key1"), Some(&"value1".to_string()));
}

#[cfg(test)]
mod error_integration_tests {
    use super::*;

    #[test]
    fn test_error_chain_propagation() {
        let root_cause = ConfigurationError::MissingField {
            field: "database_url".to_string(),
        };

        let context = ErrorContext::new("app_startup", "config_loader")
            .with_severity(ErrorSeverity::Critical)
            .with_detail("startup_phase", "config_loading");

        let app_error = DbFastError::Config {
            source: root_cause,
            context: Box::new(context),
        };

        // Verify error can be formatted and contains relevant information
        let error_string = format!("{}", app_error);
        assert!(error_string.contains("Configuration error"));

        // Verify we can access the nested error information
        match app_error {
            DbFastError::Config { source, context } => {
                match source {
                    ConfigurationError::MissingField { field } => {
                        assert_eq!(field, "database_url");
                    }
                    _ => panic!("Expected MissingField variant"),
                }
                assert_eq!(context.severity, ErrorSeverity::Critical);
            }
            _ => panic!("Expected Config error variant"),
        }
    }

    #[test]
    fn test_database_error_types() {
        let errors = vec![
            DatabaseError::ConnectionFailed {
                details: "timeout".to_string(),
            },
            DatabaseError::QueryFailed {
                query: "SELECT 1".to_string(),
            },
            DatabaseError::TransactionFailed {
                operation: "commit".to_string(),
            },
            DatabaseError::DatabaseNotFound {
                name: "test_db".to_string(),
            },
            DatabaseError::PermissionDenied {
                operation: "SELECT".to_string(),
            },
            DatabaseError::PoolExhausted,
            DatabaseError::Timeout {
                operation: "query".to_string(),
            },
        ];

        assert_eq!(errors.len(), 7);

        // Test that all variants can be created and formatted
        for error in errors {
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_remote_error_types() {
        let errors = vec![
            RemoteError::NotConfigured {
                name: "prod".to_string(),
            },
            RemoteError::ConnectionFailed {
                url: "postgres://host/db".to_string(),
            },
            RemoteError::AuthenticationFailed {
                name: "staging".to_string(),
            },
            RemoteError::InvalidUrl {
                url: "invalid-url".to_string(),
            },
            RemoteError::Timeout {
                operation: "connect".to_string(),
            },
        ];

        assert_eq!(errors.len(), 5);

        for error in errors {
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_complete_error_hierarchy() {
        // Test that we can create errors at all levels
        let config_error = DbFastError::Config {
            source: ConfigurationError::NotFound {
                path: "/etc/dbfast.toml".to_string(),
            },
            context: Box::new(ErrorContext::new("startup", "config")),
        };

        let db_error = DbFastError::Database {
            source: DatabaseError::PoolExhausted,
            context: Box::new(ErrorContext::new("query", "pool")),
        };

        let remote_error = DbFastError::Remote {
            source: RemoteError::Timeout {
                operation: "deploy".to_string(),
            },
            context: Box::new(ErrorContext::new("deployment", "remote")),
        };

        let deploy_error = DbFastError::Deployment {
            source: DeploymentError::TemplateCreationFailed {
                reason: "missing files".to_string(),
            },
            context: Box::new(ErrorContext::new("deploy", "template")),
        };

        let fs_error = DbFastError::FileSystem {
            message: "File not found".to_string(),
            context: Box::new(ErrorContext::new("read", "filesystem")),
        };

        let network_error = DbFastError::Network {
            message: "Connection refused".to_string(),
            context: Box::new(ErrorContext::new("connect", "network")),
        };

        // All errors should format correctly
        let errors = vec![
            config_error,
            db_error,
            remote_error,
            deploy_error,
            fs_error,
            network_error,
        ];

        for error in errors {
            let formatted = format!("{}", error);
            assert!(!formatted.is_empty());
            println!("Error: {}", formatted);
        }
    }
}
