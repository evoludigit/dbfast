//! Remote deployment configuration and management

use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

/// Errors that can occur during remote operations
#[derive(Debug, Error)]
pub enum RemoteError {
    /// Database connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Environment variable not found
    #[error("Environment variable not found: {0}")]
    EnvVar(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Auth(String),
}

/// Configuration for remote database deployments
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RemoteConfig {
    /// Remote instance name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Database connection URL
    pub url: String,
    /// Environment variable containing password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_env: Option<String>,
    /// Target environment
    pub environment: String,
    /// Allow destructive operations
    #[serde(default)]
    pub allow_destructive: bool,
    /// Create backup before deployment
    #[serde(default = "default_backup_before_deploy")]
    pub backup_before_deploy: bool,
    /// Require manual confirmation for deployments
    #[serde(default)]
    pub require_confirmation: bool,
}

const fn default_backup_before_deploy() -> bool {
    true
}

impl RemoteConfig {
    /// Create a new remote configuration
    #[must_use]
    pub const fn new(name: String, url: String, environment: String) -> Self {
        Self {
            name: Some(name),
            url,
            password_env: None,
            environment,
            allow_destructive: false,
            backup_before_deploy: true,
            require_confirmation: false,
        }
    }

    /// Get the password from environment variable
    pub fn get_password(&self) -> Result<String, RemoteError> {
        self.password_env.as_ref().map_or_else(
            || Ok(String::new()),
            |password_env| {
                env::var(password_env).map_err(|_| RemoteError::EnvVar(password_env.clone()))
            },
        )
    }

    /// Parse connection URL components
    pub fn parse_connection_url(&self) -> Result<ConnectionParams, RemoteError> {
        use url::Url;

        let url = Url::parse(&self.url)
            .map_err(|e| RemoteError::Config(format!("Invalid URL: {}", e)))?;

        if url.scheme() != "postgresql" && url.scheme() != "postgres" {
            return Err(RemoteError::Config(
                "URL must use postgresql:// or postgres:// scheme".to_string(),
            ));
        }

        let host = url
            .host_str()
            .ok_or_else(|| RemoteError::Config("URL must contain host".to_string()))?
            .to_string();

        let port = url.port().unwrap_or(5432);

        let user = if url.username().is_empty() {
            return Err(RemoteError::Config("URL must contain username".to_string()));
        } else {
            url.username().to_string()
        };

        let database = url.path().trim_start_matches('/');
        if database.is_empty() {
            return Err(RemoteError::Config(
                "URL must contain database name".to_string(),
            ));
        }

        Ok(ConnectionParams {
            host,
            port,
            user,
            database: database.to_string(),
        })
    }
}

/// Parsed connection parameters
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database user
    pub user: String,
    /// Database name
    pub database: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_config_creation() {
        let remote = RemoteConfig::new(
            "staging".to_string(),
            "postgres://test_user@localhost:5432/testdb".to_string(),
            "staging".to_string(),
        );

        assert_eq!(remote.name, Some("staging".to_string()));
        assert_eq!(remote.url, "postgres://test_user@localhost:5432/testdb");
        assert_eq!(remote.environment, "staging");
        assert!(!remote.allow_destructive);
        assert!(remote.backup_before_deploy);
        assert!(!remote.require_confirmation);
    }

    #[test]
    fn test_connection_url_parsing() {
        let remote = RemoteConfig::new(
            "test".to_string(),
            "postgresql://test_user@localhost:5432/testdb".to_string(),
            "local".to_string(),
        );

        let params = remote.parse_connection_url().unwrap();
        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, 5432);
        assert_eq!(params.user, "test_user");
        assert_eq!(params.database, "testdb");
    }

    #[test]
    fn test_invalid_connection_url() {
        let remote = RemoteConfig::new(
            "test".to_string(),
            "invalid://url".to_string(),
            "local".to_string(),
        );

        let result = remote.parse_connection_url();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("postgresql"));
    }
}
