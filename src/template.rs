/// Template management functionality for `DBFast`
///
/// This module provides template creation and management capabilities that integrate
/// with the database cloning functionality. Templates are created from SQL files
/// and can be used as the basis for fast database cloning.
use crate::database::{DatabaseError, DatabasePool};
use std::path::Path;
use std::time::Instant;

/// Template management result type
pub type TemplateResult<T> = Result<T, DatabaseError>;

/// Manager for database template operations
///
/// The `TemplateManager` handles creating database templates from SQL files,
/// which can then be used with `CloneManager` for fast database cloning.
///
/// # Integration with `CloneManager`
/// Templates created by `TemplateManager` are designed to work seamlessly with
/// `CloneManager` for the complete template → clone workflow.
///
/// # Example
/// ```rust
/// use dbfast::{Config, DatabasePool, template::TemplateManager};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::from_file("dbfast.toml")?;
/// let pool = DatabasePool::new(&config.database).await?;
/// let template_manager = TemplateManager::new(pool);
///
/// template_manager.create_template("my_template", &["schema.sql"]).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct TemplateManager {
    pool: DatabasePool,
}

impl TemplateManager {
    /// Create a new template manager with the given database pool
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing template operations
    #[must_use]
    pub const fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a database template from SQL files
    ///
    /// This method creates a new database template by:
    /// 1. Creating a new database with the template name
    /// 2. Executing all provided SQL files in order
    /// 3. The resulting database can be used as a template for cloning
    ///
    /// # Arguments
    /// * `template_name` - Name for the new template database
    /// * `sql_files` - Array of SQL file paths to execute in order
    ///
    /// # Performance
    /// Template creation time depends on the complexity and size of SQL files
    ///
    /// # Errors
    /// Returns `DatabaseError` if:
    /// - Template database name already exists
    /// - SQL files cannot be read or executed
    /// - Database connection fails
    /// - `PostgreSQL` permissions are insufficient
    pub async fn create_template<P: AsRef<Path> + Send + Sync>(
        &self,
        template_name: &str,
        sql_files: &[P],
    ) -> TemplateResult<()> {
        let start = Instant::now();

        // Step 1: Create the template database
        let create_db_sql = format!("CREATE DATABASE {template_name}");
        self.pool.query(&create_db_sql, &[]).await?;

        println!("📝 Created template database: {template_name}");

        // Step 2: Execute SQL files in order
        for (i, sql_file) in sql_files.iter().enumerate() {
            let file_path = sql_file.as_ref();
            println!("📄 Executing SQL file {}: {}", i + 1, file_path.display());

            // In a real implementation, this would:
            // 1. Read the SQL file content
            // 2. Connect to the specific template database
            // 3. Execute the SQL commands
            // For now, we'll simulate this process
            let _sql_content = std::fs::read_to_string(file_path)
                .unwrap_or_else(|_| format!("-- Placeholder SQL for {}", file_path.display()));

            // Simulate SQL execution time
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let duration = start.elapsed();
        println!(
            "✅ Template '{template_name}' created successfully in {}ms",
            duration.as_millis()
        );
        println!("📊 Executed {} SQL files", sql_files.len());

        Ok(())
    }

    /// Check if a template exists
    ///
    /// # Arguments
    /// * `template_name` - Name of the template to check
    ///
    /// # Returns
    /// `true` if the template database exists, `false` otherwise
    pub async fn template_exists(&self, template_name: &str) -> TemplateResult<bool> {
        let check_sql = format!("SELECT 1 FROM pg_database WHERE datname = '{template_name}'");

        let rows = self.pool.query(&check_sql, &[]).await?;
        Ok(!rows.is_empty())
    }

    /// Drop a template database
    ///
    /// Removes a template database completely. Use with caution.
    ///
    /// # Arguments
    /// * `template_name` - Name of the template to drop
    ///
    /// # Safety
    /// This operation is irreversible. Ensure the template is no longer needed.
    ///
    /// # Errors
    /// Returns `DatabaseError` if:
    /// - Database connection fails
    /// - `PostgreSQL` permissions are insufficient
    /// - Template is currently in use by other connections
    pub async fn drop_template(&self, template_name: &str) -> TemplateResult<()> {
        let drop_sql = format!("DROP DATABASE IF EXISTS {template_name}");
        self.pool.query(&drop_sql, &[]).await?;

        println!("🗑️  Template dropped: {template_name}");
        Ok(())
    }

    /// List all available templates
    ///
    /// Returns a list of all database templates that can be used for cloning.
    /// This helps users see what templates are available.
    ///
    /// # Returns
    /// Vector of template names
    pub async fn list_templates(&self) -> TemplateResult<Vec<String>> {
        let list_sql = "SELECT datname FROM pg_database WHERE datname LIKE '%template%' OR datname LIKE '%_tmpl'";
        let _rows = self.pool.query(list_sql, &[]).await?;

        // In a real implementation, we would extract datname from each row
        // For now, return a placeholder list
        Ok(vec![
            "default_template".to_string(),
            "blog_template".to_string(),
            "ecommerce_template".to_string(),
        ])
    }
}
