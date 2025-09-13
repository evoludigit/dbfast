/// Shared test utilities for dbfast integration tests
use dbfast::config::DatabaseConfig;
use dbfast::database::DatabasePool;
use uuid::Uuid;

/// Test database manager that ensures unique database names and automatic cleanup
pub struct TestDatabase {
    pub name: String,
    #[allow(dead_code)]
    pub pool: DatabasePool,
    pub admin_pool: DatabasePool,
}

impl TestDatabase {
    /// Create a unique test database with automatic cleanup
    ///
    /// # Arguments
    /// * `base_name` - Base name for the test database (will be made unique)
    ///
    /// # Returns
    /// A TestDatabase instance with a unique name and connection pool
    pub async fn create_unique(base_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Generate unique database name using UUID
        let unique_id = Uuid::new_v4().simple().to_string();
        let unique_name = format!("test_{}_{}", base_name, unique_id);

        // Create admin connection to postgres database for database management
        let admin_config = create_admin_db_config();
        let admin_pool = DatabasePool::from_config(&admin_config).await?;

        // Create the unique test database
        admin_pool.create_database(&unique_name).await?;

        // Create connection pool for the test database
        let test_config = create_test_db_config_with_name(&unique_name);
        let test_pool = DatabasePool::new_for_database(&test_config, &unique_name).await?;

        Ok(Self {
            name: unique_name,
            pool: test_pool,
            admin_pool,
        })
    }

    /// Get an admin database configuration for template operations
    /// This returns a config that connects to postgres database for admin operations
    pub fn admin_config(&self) -> DatabaseConfig {
        create_admin_db_config()
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Schedule cleanup of the test database
        // We use tokio::spawn to handle async cleanup in Drop
        let name = self.name.clone();
        let admin_pool = self.admin_pool.clone();

        tokio::spawn(async move {
            if let Err(e) = admin_pool.drop_database(&name).await {
                eprintln!("Warning: Failed to cleanup test database '{}': {}", name, e);
            }
        });
    }
}

/// Create a database config for admin operations (connecting to postgres database)
fn create_admin_db_config() -> DatabaseConfig {
    DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        user: "postgres".to_string(),
        password_env: Some("POSTGRES_PASSWORD".to_string()),
        template_name: "postgres".to_string(), // Connect to postgres database for admin operations
    }
}

/// Create a test database config with a specific database name
fn create_test_db_config_with_name(database_name: &str) -> DatabaseConfig {
    DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        user: "postgres".to_string(),
        password_env: Some("POSTGRES_PASSWORD".to_string()),
        template_name: database_name.to_string(),
    }
}
