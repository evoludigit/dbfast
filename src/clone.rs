/// Database cloning functionality for `DBFast`
use crate::database::{DatabaseError, DatabasePool};
use std::time::Instant;

/// Database cloning result
pub type CloneResult<T> = Result<T, DatabaseError>;

/// Manager for database cloning operations
pub struct CloneManager {
    pool: DatabasePool,
}

impl CloneManager {
    /// Create a new clone manager with the given database pool
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Clone a database from a template using `CREATE DATABASE WITH TEMPLATE`
    pub async fn clone_database(
        &self,
        template_name: &str,
        clone_name: &str,
    ) -> CloneResult<()> {
        let start = Instant::now();

        // Execute the clone command
        let clone_sql = format!(
            "CREATE DATABASE {} WITH TEMPLATE {}",
            clone_name, template_name
        );

        // Try to execute the clone command
        // For testing, this will work with the mock database pool
        self.pool.query(&clone_sql, &[]).await?;

        // Log performance for monitoring
        let duration = start.elapsed();
        println!(
            "Database cloned: {} -> {} in {}ms",
            template_name,
            clone_name,
            duration.as_millis()
        );

        Ok(())
    }

    /// Drop a cloned database
    pub async fn drop_database(&self, database_name: &str) -> CloneResult<()> {
        let drop_sql = format!("DROP DATABASE IF EXISTS {}", database_name);
        self.pool.query(&drop_sql, &[]).await?;

        println!("Database dropped: {}", database_name);
        Ok(())
    }
}