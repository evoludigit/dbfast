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

/// Main configuration structure for DBFast
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Database connection configuration
    pub database: DatabaseConfig,
    /// Repository configuration
    pub repository: RepositoryConfig,
    /// Environment-specific configurations
    #[serde(default)]
    pub environments: HashMap<String, Environment>,
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
    /// Create a new configuration with default values
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
            },
            repository: RepositoryConfig {
                path: repo_path.to_string(),
                repo_type: "structured".to_string(),
            },
            environments,
        }
    }

    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
