use crate::change_detector::ChangeDetector;
use crate::config::DatabaseConfig;
/// Template management functionality for `DBFast`
///
/// Templates are created from SQL files and can be used for fast database cloning.
use crate::database::{DatabaseError, DatabasePool};
use crate::scanner::FileScanner;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Template management result type
pub type TemplateResult<T> = Result<T, DatabaseError>;

/// Manager for database template operations
#[derive(Clone)]
pub struct TemplateManager {
    pool: DatabasePool,
    db_config: DatabaseConfig,
    change_detector: Option<ChangeDetector>,
}

impl TemplateManager {
    /// Create a new template manager with the given database pool and config
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing template operations
    /// * `db_config` - Database configuration for creating template-specific connections
    #[must_use]
    pub const fn new(pool: DatabasePool, db_config: DatabaseConfig) -> Self {
        Self {
            pool,
            db_config,
            change_detector: None,
        }
    }

    /// Create a new template manager with change detection capabilities
    ///
    /// # Arguments
    /// * `pool` - Database connection pool for executing template operations
    /// * `db_config` - Database configuration for creating template-specific connections
    /// * `root_path` - Root path to monitor for SQL file changes
    #[must_use]
    pub fn new_with_change_detection(
        pool: DatabasePool,
        db_config: DatabaseConfig,
        root_path: PathBuf,
    ) -> Self {
        Self {
            pool,
            db_config,
            change_detector: Some(ChangeDetector::new(root_path)),
        }
    }

    /// Check if this template manager has change detection capabilities
    #[must_use]
    pub const fn has_change_detection(&self) -> bool {
        self.change_detector.is_some()
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

        // Step 1: Create the template database using admin connection
        self.pool.create_database(template_name).await?;
        println!("📝 Created template database: {template_name}");

        // Step 2: Execute SQL files in order
        // Create connection pool for the template database
        let template_pool = DatabasePool::new_for_database(&self.db_config, template_name)
            .await
            .map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to connect to template database '{template_name}': {e}"
                ))
            })?;

        // Read and concatenate all SQL files
        let mut concatenated_sql = String::new();
        for (i, sql_file) in sql_files.iter().enumerate() {
            let file_path = sql_file.as_ref();
            println!("📄 Reading SQL file {}: {}", i + 1, file_path.display());

            // Read the SQL file content
            let sql_content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to read SQL file {}: {e}",
                    file_path.display()
                ))
            })?;

            // Add file separator comment and content
            concatenated_sql.push_str(&format!("\n-- File: {}\n", file_path.display()));
            concatenated_sql.push_str(&sql_content);
            concatenated_sql.push('\n');
        }

        // Execute all SQL files in a single transaction
        println!(
            "🔄 Executing {} SQL files in a single transaction",
            sql_files.len()
        );
        template_pool
            .execute_sql_content(&concatenated_sql)
            .await
            .map_err(|e| {
                DatabaseError::Config(format!("Failed to execute concatenated SQL files: {e}"))
            })?;

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
        self.pool.database_exists(template_name).await
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
        self.pool.drop_database(template_name).await?;
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
            DatabaseError::Config(format!("Failed to list template databases: {e}"))
        })?;

        let mut templates = Vec::new();
        for row in rows {
            let datname: String = row.get(0);
            templates.push(datname);
        }

        Ok(templates)
    }

    /// Create a database template with change tracking
    ///
    /// This method creates a template and also stores metadata for change detection.
    ///
    /// # Arguments
    /// * `template_name` - Name for the new template database
    /// * `sql_files` - Array of SQL file paths to execute in order
    ///
    /// # Errors
    /// Returns `DatabaseError` if template creation fails or change tracking setup fails
    pub async fn create_template_with_change_tracking<P: AsRef<Path> + Send + Sync>(
        &self,
        template_name: &str,
        sql_files: &[P],
    ) -> TemplateResult<()> {
        // Create the template using existing method
        self.create_template(template_name, sql_files).await?;

        // If we have change detection, store metadata
        if let Some(change_detector) = &self.change_detector {
            // Get the root path from change detector and scan files
            let scanner = FileScanner::new(change_detector.root_path());
            let scanned_files = scanner.scan().map_err(|e| {
                DatabaseError::Config(format!("Failed to scan files for change tracking: {e}"))
            })?;

            // Store metadata for change detection
            change_detector
                .store_template_metadata(template_name, &scanned_files)
                .await
                .map_err(|e| {
                    DatabaseError::Config(format!("Failed to store change detection metadata: {e}"))
                })?;

            println!("📊 Change detection metadata stored for template: {template_name}");
        }

        Ok(())
    }

    /// Check if a template needs rebuilding based on file changes
    ///
    /// # Arguments
    /// * `template_name` - Name of the template to check
    ///
    /// # Returns
    /// `true` if the template needs rebuilding, `false` if it's up to date
    pub async fn template_needs_rebuild(&self, template_name: &str) -> TemplateResult<bool> {
        match &self.change_detector {
            Some(change_detector) => change_detector
                .template_needs_rebuild(template_name)
                .await
                .map_err(|e| {
                    DatabaseError::Config(format!("Failed to check if template needs rebuild: {e}"))
                }),
            None => {
                // Without change detection, always assume rebuild is needed
                // This is the safe default behavior
                Ok(true)
            }
        }
    }

    /// Smart template creation - only creates if template doesn't exist or files have changed
    ///
    /// # Arguments
    /// * `template_name` - Name for the template database
    /// * `sql_files` - Array of SQL file paths to execute in order
    ///
    /// # Returns
    /// `true` if template was created/rebuilt, `false` if creation was skipped
    pub async fn smart_create_template<P: AsRef<Path> + Send + Sync>(
        &self,
        template_name: &str,
        sql_files: &[P],
    ) -> TemplateResult<bool> {
        // Check if template exists and needs rebuilding
        let template_exists = self.template_exists(template_name).await?;

        if template_exists {
            let needs_rebuild = self.template_needs_rebuild(template_name).await?;

            if !needs_rebuild {
                println!("⏩ Template '{template_name}' is up to date, skipping creation");
                return Ok(false);
            }

            println!("🔄 Template '{template_name}' needs rebuilding due to file changes");

            // Drop existing template before recreating
            self.drop_template(template_name).await?;
        }

        // Create (or recreate) template with change tracking
        if self.has_change_detection() {
            self.create_template_with_change_tracking(template_name, sql_files)
                .await?;
        } else {
            self.create_template(template_name, sql_files).await?;
        }

        Ok(true)
    }
}
