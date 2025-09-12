//! Comprehensive configuration validation system
//!
//! This module provides validation for all DBFast configurations with:
//! - Schema validation
//! - Value range validation
//! - Dependency validation
//! - Security validation

use crate::config::{Config, DatabaseConfig, Environment};
use crate::errors::{DbFastResult, ValidationError};
use crate::remote::RemoteConfig;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};
use url::Url;

/// Configuration validator that performs comprehensive validation
pub struct ConfigValidator;

/// Validation result with details about issues found
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed overall
    pub is_valid: bool,

    /// Warning messages (validation passes but with concerns)
    pub warnings: Vec<ValidationWarning>,

    /// Error messages (validation fails)
    pub errors: Vec<ValidationError>,

    /// Security concerns identified
    pub security_issues: Vec<SecurityIssue>,

    /// Performance recommendations
    pub performance_recommendations: Vec<PerformanceRecommendation>,
}

/// Warning types during validation
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub severity: WarningSeverity,
}

/// Security issues identified during validation
#[derive(Debug, Clone)]
pub struct SecurityIssue {
    pub field: String,
    pub issue: String,
    pub recommendation: String,
    pub severity: SecuritySeverity,
}

/// Performance recommendations
#[derive(Debug, Clone)]
pub struct PerformanceRecommendation {
    pub component: String,
    pub recommendation: String,
    pub impact: PerformanceImpact,
}

/// Warning severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    High,
    Medium,
    Low,
}

/// Security severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Performance impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceImpact {
    High,
    Medium,
    Low,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a complete configuration
    pub fn validate(&self, config: &Config) -> DbFastResult<ValidationResult> {
        info!("Starting comprehensive configuration validation");

        let mut result = ValidationResult {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            security_issues: Vec::new(),
            performance_recommendations: Vec::new(),
        };

        // Validate database configuration
        self.validate_database_config(&config.database, &mut result);

        // Validate environments
        self.validate_environments(&config.environments, &mut result);

        // Validate remotes
        self.validate_remotes(&config.remotes, &config.environments, &mut result);

        // Cross-validation between components
        self.validate_cross_dependencies(config, &mut result);

        // Security validation
        self.validate_security(config, &mut result);

        // Performance validation
        self.validate_performance(config, &mut result);

        // Update overall validity
        result.is_valid = result.errors.is_empty();

        if result.is_valid {
            info!(
                "Configuration validation passed with {} warnings",
                result.warnings.len()
            );
        } else {
            warn!(
                "Configuration validation failed with {} errors",
                result.errors.len()
            );
        }

        Ok(result)
    }

    /// Validate database configuration
    fn validate_database_config(&self, db_config: &DatabaseConfig, result: &mut ValidationResult) {
        debug!("Validating database configuration");

        // Validate host
        if db_config.host.is_empty() {
            result.errors.push(ValidationError::RequiredFieldMissing {
                field: "database.host".to_string(),
            });
        } else if db_config.host == "localhost" || db_config.host == "127.0.0.1" {
            result.warnings.push(ValidationWarning {
                field: "database.host".to_string(),
                message: "Using localhost may not work in containerized environments".to_string(),
                severity: WarningSeverity::Medium,
            });
        }

        // Validate port
        if !(1..=65535).contains(&db_config.port) {
            result.errors.push(ValidationError::OutOfRange {
                field: "database.port".to_string(),
                value: db_config.port.to_string(),
            });
        }

        if db_config.port != 5432 {
            result.warnings.push(ValidationWarning {
                field: "database.port".to_string(),
                message: "Using non-standard PostgreSQL port".to_string(),
                severity: WarningSeverity::Low,
            });
        }

        // Validate user
        if db_config.user.is_empty() {
            result.errors.push(ValidationError::RequiredFieldMissing {
                field: "database.user".to_string(),
            });
        }

        // Check for security issues
        if db_config.user == "postgres" {
            result.security_issues.push(SecurityIssue {
                field: "database.user".to_string(),
                issue: "Using default PostgreSQL superuser account".to_string(),
                recommendation: "Create a dedicated user with minimal required permissions"
                    .to_string(),
                severity: SecuritySeverity::Medium,
            });
        }

        // Validate password configuration
        if db_config.password_env.is_none() {
            result.security_issues.push(SecurityIssue {
                field: "database.password_env".to_string(),
                issue: "No password environment variable configured".to_string(),
                recommendation: "Set password_env to reference an environment variable".to_string(),
                severity: SecuritySeverity::High,
            });
        }

        // Validate template name
        if db_config.template_name.is_empty() {
            result.errors.push(ValidationError::RequiredFieldMissing {
                field: "database.template_name".to_string(),
            });
        } else if !Self::is_valid_database_name(&db_config.template_name) {
            result.errors.push(ValidationError::InvalidFormat {
                field: "database.template_name".to_string(),
                value: db_config.template_name.clone(),
            });
        }
    }

    /// Validate environments configuration
    fn validate_environments(
        &self,
        environments: &HashMap<String, Environment>,
        result: &mut ValidationResult,
    ) {
        debug!("Validating environments configuration");

        if environments.is_empty() {
            result.errors.push(ValidationError::RequiredFieldMissing {
                field: "environments".to_string(),
            });
            return;
        }

        // Check for standard environments
        let _standard_envs = ["local", "development", "staging", "production"];
        let mut found_production = false;

        for (env_name, env_config) in environments {
            if env_name == "production" {
                found_production = true;
            }

            // Validate environment name
            if !Self::is_valid_environment_name(env_name) {
                result.errors.push(ValidationError::InvalidFormat {
                    field: format!("environments.{env_name}"),
                    value: env_name.clone(),
                });
            }

            // Validate directory paths
            if env_config.include_directories.is_empty() {
                result.warnings.push(ValidationWarning {
                    field: format!("environments.{env_name}.include_directories"),
                    message: "No include directories specified".to_string(),
                    severity: WarningSeverity::Medium,
                });
            }

            // Check for common directory patterns
            self.validate_directory_patterns(env_name, env_config, result);
        }

        if !found_production {
            result.warnings.push(ValidationWarning {
                field: "environments".to_string(),
                message: "No production environment configured".to_string(),
                severity: WarningSeverity::High,
            });
        }

        // Performance recommendation
        if environments.len() > 5 {
            result
                .performance_recommendations
                .push(PerformanceRecommendation {
                    component: "environments".to_string(),
                    recommendation: "Consider consolidating environments to reduce complexity"
                        .to_string(),
                    impact: PerformanceImpact::Low,
                });
        }
    }

    /// Validate remote configurations
    fn validate_remotes(
        &self,
        remotes: &HashMap<String, RemoteConfig>,
        environments: &HashMap<String, Environment>,
        result: &mut ValidationResult,
    ) {
        debug!("Validating remotes configuration");

        for (remote_name, remote_config) in remotes {
            // Validate remote name
            if !Self::is_valid_remote_name(remote_name) {
                result.errors.push(ValidationError::InvalidFormat {
                    field: format!("remotes.{remote_name}"),
                    value: remote_name.clone(),
                });
            }

            // Validate URL
            if let Err(e) = Url::parse(&remote_config.url) {
                result.errors.push(ValidationError::InvalidFormat {
                    field: format!("remotes.{remote_name}.url"),
                    value: format!("Invalid URL: {e}"),
                });
            } else {
                self.validate_remote_url(remote_name, &remote_config.url, result);
            }

            // Validate environment reference
            if !environments.contains_key(&remote_config.environment) {
                result.errors.push(ValidationError::ConstraintViolation {
                    constraint: format!(
                        "Remote '{remote_name}' references non-existent environment '{}'",
                        remote_config.environment
                    ),
                });
            }

            // Security validations
            self.validate_remote_security(remote_name, remote_config, result);

            // Performance recommendations
            self.validate_remote_performance(remote_name, remote_config, result);
        }
    }

    /// Validate cross-component dependencies
    fn validate_cross_dependencies(&self, config: &Config, result: &mut ValidationResult) {
        debug!("Validating cross-component dependencies");

        // Check for production remotes with appropriate environments
        for (remote_name, remote_config) in &config.remotes {
            if remote_name.contains("prod") || remote_config.environment == "production" {
                if remote_config.allow_destructive {
                    result.security_issues.push(SecurityIssue {
                        field: format!("remotes.{remote_name}.allow_destructive"),
                        issue: "Production remote allows destructive operations".to_string(),
                        recommendation: "Disable destructive operations for production remotes"
                            .to_string(),
                        severity: SecuritySeverity::High,
                    });
                }

                if !remote_config.backup_before_deploy {
                    result.errors.push(ValidationError::ConstraintViolation {
                        constraint: format!("Production remote '{remote_name}' must have backup_before_deploy enabled"),
                    });
                }
            }
        }
    }

    /// Validate security aspects
    fn validate_security(&self, config: &Config, result: &mut ValidationResult) {
        debug!("Performing security validation");

        // Check for hardcoded credentials
        for (remote_name, remote_config) in &config.remotes {
            if remote_config.url.contains("password=") {
                result.security_issues.push(SecurityIssue {
                    field: format!("remotes.{remote_name}.url"),
                    issue: "URL contains embedded password".to_string(),
                    recommendation: "Use password_env instead of embedding passwords in URLs"
                        .to_string(),
                    severity: SecuritySeverity::Critical,
                });
            }
        }

        // Check password environment variables
        let mut env_vars = HashSet::new();
        if let Some(password_env) = &config.database.password_env {
            env_vars.insert(password_env);
        }

        for remote_config in config.remotes.values() {
            if let Some(password_env) = &remote_config.password_env {
                if env_vars.contains(&password_env) {
                    result.security_issues.push(SecurityIssue {
                        field: "password_env".to_string(),
                        issue: "Multiple components using the same password environment variable"
                            .to_string(),
                        recommendation:
                            "Use unique password environment variables for each component"
                                .to_string(),
                        severity: SecuritySeverity::Medium,
                    });
                }
                env_vars.insert(password_env);
            }
        }
    }

    /// Validate performance aspects
    fn validate_performance(&self, _config: &Config, result: &mut ValidationResult) {
        debug!("Performing performance validation");

        // Add performance recommendations based on configuration patterns
        result
            .performance_recommendations
            .push(PerformanceRecommendation {
                component: "general".to_string(),
                recommendation: "Consider implementing connection pooling optimization".to_string(),
                impact: PerformanceImpact::Medium,
            });
    }

    /// Validate directory patterns in environments
    fn validate_directory_patterns(
        &self,
        env_name: &str,
        env_config: &Environment,
        result: &mut ValidationResult,
    ) {
        // Check for suspicious patterns
        for dir in &env_config.include_directories {
            if dir.contains("..") {
                result.security_issues.push(SecurityIssue {
                    field: format!("environments.{env_name}.include_directories"),
                    issue: format!("Directory path '{dir}' contains parent directory references"),
                    recommendation: "Use absolute paths or relative paths without '..' references"
                        .to_string(),
                    severity: SecuritySeverity::Medium,
                });
            }

            if dir.starts_with('/') && env_name == "production" {
                result.warnings.push(ValidationWarning {
                    field: format!("environments.{env_name}.include_directories"),
                    message: format!("Production environment uses absolute path: {dir}"),
                    severity: WarningSeverity::Medium,
                });
            }
        }

        // Check for common patterns
        let has_schema = env_config
            .include_directories
            .iter()
            .any(|d| d.contains("schema"));
        let has_migration = env_config
            .include_directories
            .iter()
            .any(|d| d.contains("migration"));

        if !has_schema && env_name == "production" {
            result.warnings.push(ValidationWarning {
                field: format!("environments.{env_name}"),
                message: "Production environment might be missing schema directory".to_string(),
                severity: WarningSeverity::High,
            });
        }

        if !has_migration && env_name == "production" {
            result.warnings.push(ValidationWarning {
                field: format!("environments.{env_name}"),
                message: "Production environment might be missing migration directory".to_string(),
                severity: WarningSeverity::Medium,
            });
        }
    }

    /// Validate remote URL for security and format
    fn validate_remote_url(&self, remote_name: &str, url: &str, result: &mut ValidationResult) {
        let parsed_url = Url::parse(url).unwrap(); // Already validated above

        // Check for secure connection
        if parsed_url.scheme() == "postgres" && parsed_url.host_str() != Some("localhost") {
            result.security_issues.push(SecurityIssue {
                field: format!("remotes.{remote_name}.url"),
                issue: "Remote connection not using SSL/TLS".to_string(),
                recommendation: "Use 'postgresql://' scheme for SSL/TLS connections".to_string(),
                severity: SecuritySeverity::Medium,
            });
        }

        // Check for default ports on non-localhost
        if let Some(port) = parsed_url.port() {
            if port == 5432 && parsed_url.host_str() != Some("localhost") {
                result.warnings.push(ValidationWarning {
                    field: format!("remotes.{remote_name}.url"),
                    message: "Using default PostgreSQL port for remote connection".to_string(),
                    severity: WarningSeverity::Low,
                });
            }
        }
    }

    /// Validate remote security configuration
    fn validate_remote_security(
        &self,
        remote_name: &str,
        remote_config: &RemoteConfig,
        result: &mut ValidationResult,
    ) {
        if remote_config.password_env.is_none() {
            result.security_issues.push(SecurityIssue {
                field: format!("remotes.{remote_name}.password_env"),
                issue: "No password environment variable configured for remote".to_string(),
                recommendation: "Configure password_env for secure credential management"
                    .to_string(),
                severity: SecuritySeverity::High,
            });
        }

        if !remote_config.require_confirmation && remote_config.environment == "production" {
            result.security_issues.push(SecurityIssue {
                field: format!("remotes.{remote_name}.require_confirmation"),
                issue: "Production remote doesn't require manual confirmation".to_string(),
                recommendation: "Enable require_confirmation for production deployments"
                    .to_string(),
                severity: SecuritySeverity::High,
            });
        }
    }

    /// Validate remote performance configuration
    fn validate_remote_performance(
        &self,
        remote_name: &str,
        remote_config: &RemoteConfig,
        result: &mut ValidationResult,
    ) {
        if !remote_config.backup_before_deploy {
            result
                .performance_recommendations
                .push(PerformanceRecommendation {
                    component: format!("remotes.{remote_name}"),
                    recommendation: "Consider enabling backups for faster recovery".to_string(),
                    impact: PerformanceImpact::Low,
                });
        }
    }

    /// Validate database name format
    fn is_valid_database_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 63 {
            return false;
        }

        // Must start with letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }

        // Only alphanumeric, underscore, and dollar sign allowed
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    }

    /// Validate environment name format
    fn is_valid_environment_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 50 {
            return false;
        }

        // Only alphanumeric, underscore, and hyphen allowed
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    /// Validate remote name format
    fn is_valid_remote_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 50 {
            return false;
        }

        // Only alphanumeric, underscore, and hyphen allowed
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    /// Check if validation has any issues (errors or high-severity warnings)
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty()
            || self
                .warnings
                .iter()
                .any(|w| w.severity == WarningSeverity::High)
            || self.security_issues.iter().any(|s| {
                matches!(
                    s.severity,
                    SecuritySeverity::Critical | SecuritySeverity::High
                )
            })
    }

    /// Get a summary of validation issues
    pub fn summary(&self) -> String {
        format!(
            "Validation: {} errors, {} warnings, {} security issues, {} performance recommendations",
            self.errors.len(),
            self.warnings.len(),
            self.security_issues.len(),
            self.performance_recommendations.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, RepositoryConfig};

    fn create_test_config() -> Config {
        let mut environments = HashMap::new();
        environments.insert(
            "local".to_string(),
            Environment {
                include_directories: vec!["schema".to_string(), "seeds".to_string()],
                exclude_directories: vec![],
            },
        );

        Config {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "testuser".to_string(),
                password_env: Some("TEST_PASSWORD".to_string()),
                template_name: "test_template".to_string(),
            },
            repository: RepositoryConfig {
                path: "./db".to_string(),
                repo_type: "structured".to_string(),
            },
            environments,
            remotes: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_config_passes_validation() {
        let config = create_test_config();
        let validator = ConfigValidator::new();
        let result = validator.validate(&config).unwrap();

        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_database_name_fails_validation() {
        let mut config = create_test_config();
        config.database.template_name = "invalid-name!".to_string();

        let validator = ConfigValidator::new();
        let result = validator.validate(&config).unwrap();

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_security_issues_detected() {
        let mut config = create_test_config();
        config.database.user = "postgres".to_string();

        let validator = ConfigValidator::new();
        let result = validator.validate(&config).unwrap();

        assert!(!result.security_issues.is_empty());
        assert!(result
            .security_issues
            .iter()
            .any(|s| s.field == "database.user"));
    }

    #[test]
    fn test_database_name_validation() {
        assert!(ConfigValidator::is_valid_database_name("valid_name"));
        assert!(ConfigValidator::is_valid_database_name("valid123"));
        assert!(!ConfigValidator::is_valid_database_name("123invalid"));
        assert!(!ConfigValidator::is_valid_database_name("invalid-name"));
        assert!(!ConfigValidator::is_valid_database_name(""));
    }
}
