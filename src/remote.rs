//! Remote deployment configuration and management

/// Configuration for remote database deployments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteConfig {
    /// Remote instance name
    pub name: String,
    /// Database connection URL
    pub url: String,
    /// Database user
    pub user: String,
    /// Target environment
    pub environment: String,
    /// Allow destructive operations
    pub allow_destructive: bool,
    /// Create backup before deployment
    pub backup_before_deploy: bool,
}

impl RemoteConfig {
    /// Create a new remote configuration
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(name: String, url: String, user: String, environment: String) -> Self {
        Self {
            name,
            url,
            user,
            environment,
            allow_destructive: false,
            backup_before_deploy: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_config_creation() {
        let remote = RemoteConfig::new(
            "staging".to_string(),
            "postgres://localhost:5432/testdb".to_string(),
            "test_user".to_string(),
            "staging".to_string(),
        );

        assert_eq!(remote.name, "staging");
        assert_eq!(remote.url, "postgres://localhost:5432/testdb");
        assert_eq!(remote.user, "test_user");
        assert_eq!(remote.environment, "staging");
        assert!(!remote.allow_destructive);
        assert!(remote.backup_before_deploy);
    }
}
