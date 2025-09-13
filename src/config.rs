//! # Configuration Management Module
//!
//! Handles loading and parsing of DBFast configuration files in TOML format.
//! Configuration includes database connection details, repository settings,
//! environment-specific configurations, and remote database settings.
//!
//! ## Example Configuration File (dbfast.toml)
//!
//! ```toml
//! [database]
//! host = "localhost"
//! port = 5432
//! user = "postgres"
//! password_env = "DB_PASSWORD"
//! template_name = "my_app_template"
//!
//! [repository]
//! sql_dir = "./sql"
//! exclude_patterns = ["*.backup.sql", "temp_*"]
//!
//! [environments.development]
//! filter_patterns = ["dev_*", "test_*"]
//!
//! [remotes.production]
//! url = "postgresql://user@prod-host:5432/database"
//! env = "production"
//! ```

use crate::remote::RemoteConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    /// IO error occurred while reading config file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error occurred
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Main configuration structure for `DBFast`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Database connection configuration
    pub database: DatabaseConfig,
    /// Repository configuration
    pub repository: RepositoryConfig,
    /// Environment-specific configurations
    #[serde(default)]
    pub environments: HashMap<String, Environment>,
    /// Remote database configurations
    #[serde(default)]
    pub remotes: HashMap<String, RemoteConfig>,
}

/// Database connection configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database user
    pub user: String,
    /// Environment variable containing the password
    pub password_env: Option<String>,
    /// Template database name
    pub template_name: String,
    /// Enable advanced multi-statement SQL parsing for `PostgreSQL` functions
    /// Default: true
    #[serde(default = "default_allow_multi_statement")]
    pub allow_multi_statement: bool,
}

/// Default value for `allow_multi_statement`
const fn default_allow_multi_statement() -> bool {
    true
}

/// Repository configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositoryConfig {
    /// Path to the repository
    pub path: String,
    /// Repository type
    #[serde(rename = "type")]
    pub repo_type: String,
}

/// Environment-specific configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Environment {
    /// Directories to include
    #[serde(default)]
    pub include_directories: Vec<String>,
    /// Directories to exclude
    #[serde(default)]
    pub exclude_directories: Vec<String>,
}

impl Config {
    /// Create a new configuration with sensible default values
    ///
    /// Sets up a default configuration suitable for most `PostgreSQL` database
    /// projects with common environment patterns (local, production).
    ///
    /// # Arguments
    /// * `repo_path` - Path to the SQL repository directory
    /// * `template_name` - Name for the database template
    ///
    /// # Example
    /// ```rust,no_run
    /// use dbfast::Config;
    ///
    /// let config = Config::new("./sql", "my_app_template");
    /// ```
    #[must_use]
    pub fn new(repo_path: &str, template_name: &str) -> Self {
        let mut environments = HashMap::new();
        environments.insert(
            "local".to_string(),
            Environment {
                include_directories: vec![
                    "0_schema".to_string(),
                    "1_seed_common".to_string(),
                    "2_seed_backend".to_string(),
                ],
                exclude_directories: vec![],
            },
        );
        environments.insert(
            "production".to_string(),
            Environment {
                include_directories: vec!["0_schema".to_string(), "6_migration".to_string()],
                exclude_directories: vec![
                    "1_seed_common".to_string(),
                    "2_seed_backend".to_string(),
                ],
            },
        );

        Self {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "postgres".to_string(),
                password_env: Some("POSTGRES_PASSWORD".to_string()),
                template_name: template_name.to_string(),
                allow_multi_statement: true,
            },
            repository: RepositoryConfig {
                path: repo_path.to_string(),
                repo_type: "structured".to_string(),
            },
            environments,
            remotes: HashMap::new(),
        }
    }

    /// Load configuration from a TOML file
    ///
    /// Reads and parses a TOML configuration file, returning a Config instance
    /// with all settings loaded and validated.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Errors
    /// Returns `ConfigError` if the file cannot be read or parsed
    ///
    /// # Example
    /// ```rust,no_run
    /// use dbfast::Config;
    ///
    /// let config = Config::from_file("dbfast.toml")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration from a TOML file (alias for `from_file`)
    ///
    /// This is a convenience method that provides a shorter name for loading configuration.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Example
    /// ```rust,no_run
    /// use dbfast::Config;
    ///
    /// let config = Config::load("dbfast.toml")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        Self::from_file(path)
    }
}
