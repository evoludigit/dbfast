use crate::config::DatabaseConfig;
/// Template management functionality for DBFast
///
/// Templates are created from SQL files and can be used for fast database cloning.
use crate::database::{DatabaseError, DatabasePool};
use std::path::Path;
use std::time::Instant;

/// Template management result type
pub type TemplateResult<T> = Result<T, DatabaseError>;

/// Manager for database template operations
#[derive(Clone)]
pub struct TemplateManager {
    pool: DatabasePool,
    db_config: DatabaseConfig,
}

impl TemplateManager {
    /// Create a new template manager with the given database pool and config
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing template operations
    /// * `db_config` - Database configuration for creating template-specific connections
    #[must_use]
    pub const fn new(pool: DatabasePool, db_config: DatabaseConfig) -> Self {
        Self { pool, db_config }
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
        self.pool.query(&create_db_sql, &[]).await.map_err(|e| {
            DatabaseError::Config(format!(
                "Failed to create template database '{}': {}",
                template_name, e
            ))
        })?;

        println!("📝 Created template database: {template_name}");

        // Step 2: Execute SQL files in order
        // Create connection pool for the template database
        let template_pool = DatabasePool::new_for_database(&self.db_config, template_name)
            .await
            .map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to connect to template database '{}': {}",
                    template_name, e
                ))
            })?;

        for (i, sql_file) in sql_files.iter().enumerate() {
            let file_path = sql_file.as_ref();
            println!("📄 Executing SQL file {}: {}", i + 1, file_path.display());

            // 1. Read the SQL file content
            let sql_content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to read SQL file {}: {}",
                    file_path.display(),
                    e
                ))
            })?;

            // 2. Execute the SQL commands on the template database
            template_pool
                .execute_sql_content(&sql_content)
                .await
                .map_err(|e| {
                    DatabaseError::Config(format!(
                        "Failed to execute SQL file {}: {}",
                        file_path.display(),
                        e
                    ))
                })?;
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
        let check_sql = "SELECT 1 FROM pg_database WHERE datname = $1";

        let rows = self
            .pool
            .query(&check_sql, &[&template_name])
            .await
            .map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to check if template '{}' exists: {}",
                    template_name, e
                ))
            })?;
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
        self.pool.query(&drop_sql, &[]).await.map_err(|e| {
            DatabaseError::Config(format!(
                "Failed to drop template database '{}': {}",
                template_name, e
            ))
        })?;

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
        let rows = self.pool.query(list_sql, &[]).await.map_err(|e| {
            DatabaseError::Config(format!("Failed to list template databases: {}", e))
        })?;

        let mut templates = Vec::new();
        for row in rows {
            let datname: String = row.get(0);
            templates.push(datname);
        }

        Ok(templates)
    }
}
