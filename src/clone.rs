/// Database cloning functionality for `DBFast`
///
/// This module provides fast database cloning using `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE`
/// functionality. It's designed to achieve database cloning in under 100ms for optimal performance.
use crate::database::{DatabaseError, DatabasePool};
use std::time::Instant;

/// Database cloning result type
pub type CloneResult<T> = Result<T, DatabaseError>;

/// Manager for database cloning operations
///
/// The `CloneManager` handles creating database clones from templates using `PostgreSQL`'s
/// native `CREATE DATABASE WITH TEMPLATE` command for maximum performance.
///
/// # Performance
/// Target: Database cloning in <100ms for small-to-medium databases
///
/// # Example
/// ```rust
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
pub struct CloneManager {
    pool: DatabasePool,
}

impl CloneManager {
    /// Create a new clone manager with the given database pool
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing clone operations
    #[must_use]
    pub const fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Clone a database from a template using `CREATE DATABASE WITH TEMPLATE`
    ///
    /// This method executes `PostgreSQL`'s `CREATE DATABASE WITH TEMPLATE` command
    /// to create a fast, complete copy of an existing template database.
    ///
    /// # Arguments
    /// * `template_name` - Name of the template database to clone from
    /// * `clone_name` - Name for the new cloned database
    ///
    /// # Performance
    /// Target: <100ms for small-to-medium databases
    ///
    /// # Errors
    /// Returns `DatabaseError` if:
    /// - Template database doesn't exist
    /// - Clone database name already exists
    /// - Database connection fails
    /// - `PostgreSQL` permissions are insufficient
    pub async fn clone_database(&self, template_name: &str, clone_name: &str) -> CloneResult<()> {
        let start = Instant::now();

        // Execute the clone command using PostgreSQL's native template functionality
        let clone_sql = format!("CREATE DATABASE {clone_name} WITH TEMPLATE {template_name}");

        // Execute the clone operation
        self.pool.query(&clone_sql, &[]).await?;

        // Log performance metrics for monitoring
        let duration = start.elapsed();
        println!(
            "Database cloned: {template_name} -> {clone_name} in {}ms",
            duration.as_millis()
        );

        Ok(())
    }

    /// Drop a cloned database
    ///
    /// Removes a database completely. Use with caution.
    ///
    /// # Arguments
    /// * `database_name` - Name of the database to drop
    ///
    /// # Safety
    /// This operation is irreversible. Ensure the database is no longer needed.
    ///
    /// # Errors
    /// Returns `DatabaseError` if:
    /// - Database connection fails
    /// - `PostgreSQL` permissions are insufficient
    /// - Database is currently in use by other connections
    pub async fn drop_database(&self, database_name: &str) -> CloneResult<()> {
        let drop_sql = format!("DROP DATABASE IF EXISTS {database_name}");
        self.pool.query(&drop_sql, &[]).await?;

        println!("Database dropped: {database_name}");
        Ok(())
    }
}
