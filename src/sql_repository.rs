use crate::database::DatabaseError;
/// SQL Repository functionality for `DBFast`
///
/// This module handles discovery and loading of SQL files from both structured
/// and flat repository layouts, with support for environment-based filtering.
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

/// Result type for SQL repository operations
pub type SqlRepositoryResult<T> = Result<T, DatabaseError>;

/// SQL Repository for managing SQL file discovery and loading
///
/// Supports both:
/// - Structured repositories: `0_schema`/, `1_seed_common`/, `2_seed_dev`/, etc.
/// - Flat repositories: all SQL files in a single directory
#[derive(Debug, Clone)]
pub struct SqlRepository {
    repository_path: PathBuf,
}

impl SqlRepository {
    /// Create a new SQL repository from a directory path
    ///
    /// # Arguments
    /// * `repository_path` - Path to the repository directory
    ///
    /// # Errors
    /// Returns `DatabaseError` if the path doesn't exist or isn't a directory
    pub fn new<P: AsRef<Path>>(repository_path: P) -> SqlRepositoryResult<Self> {
        let path = repository_path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(DatabaseError::Config(format!(
                "Repository path does not exist: {}",
                path.display()
            )));
        }

        if !path.is_dir() {
            return Err(DatabaseError::Config(format!(
                "Repository path is not a directory: {}",
                path.display()
            )));
        }

        Ok(Self {
            repository_path: path,
        })
    }

    /// Discover SQL files in the repository with environment filtering
    ///
    /// # Arguments
    /// * `environments` - List of environments to include (e.g., `["dev", "prod"]`)
    ///                   For structured repos, this filters directories like `2_seed_dev`, `3_seed_prod`
    ///                   For flat repos, this parameter is ignored
    ///
    /// # Returns
    /// Vector of SQL file paths in execution order
    ///
    /// # Structured Repository Layout
    /// - `0_schema/` - Database schema files (always included)
    /// - `1_seed_common/` - Common seed data (always included)
    /// - `2_seed_dev/`, `2_seed_test/`, `2_seed_prod/` - Environment-specific data
    /// - Files within directories are sorted alphabetically
    ///
    /// # Flat Repository Layout
    /// - All `.sql` files in the root directory
    /// - Files sorted alphabetically
    pub async fn discover_sql_files(
        &self,
        environments: &[&str],
    ) -> SqlRepositoryResult<Vec<PathBuf>> {
        let is_structured = self.is_structured_repository().await?;

        if is_structured {
            self.discover_structured_files(environments).await
        } else {
            self.discover_flat_files().await
        }
    }

    /// Load SQL content from a file
    ///
    /// # Arguments
    /// * `sql_file` - Path to the SQL file to load
    ///
    /// # Returns
    /// String containing the SQL file content
    pub async fn load_sql_content<P: AsRef<Path> + Send>(
        &self,
        sql_file: P,
    ) -> SqlRepositoryResult<String> {
        let content = async_fs::read_to_string(sql_file.as_ref())
            .await
            .map_err(|e| {
                DatabaseError::Config(format!(
                    "Failed to read SQL file {}: {}",
                    sql_file.as_ref().display(),
                    e
                ))
            })?;

        Ok(content)
    }

    /// Check if this is a structured repository
    ///
    /// A structured repository has directories starting with numbers (0_, 1_, 2_, etc.)
    async fn is_structured_repository(&self) -> SqlRepositoryResult<bool> {
        let mut entries = async_fs::read_dir(&self.repository_path)
            .await
            .map_err(|e| {
                DatabaseError::Config(format!("Failed to read repository directory: {}", e))
            })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DatabaseError::Config(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    // Check for structured directory pattern (starts with number_)
                    if name_str.chars().next().unwrap_or('\0').is_ascii_digit()
                        && name_str.contains('_')
                    {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Discover files in structured repository
    async fn discover_structured_files(
        &self,
        environments: &[&str],
    ) -> SqlRepositoryResult<Vec<PathBuf>> {
        let mut directories = Vec::new();
        let mut entries = async_fs::read_dir(&self.repository_path)
            .await
            .map_err(|e| {
                DatabaseError::Config(format!("Failed to read structured repository: {}", e))
            })?;

        // Collect all structured directories
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DatabaseError::Config(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if self.should_include_structured_directory(&name_str, environments) {
                        directories.push((name_str.to_string(), path));
                    }
                }
            }
        }

        // Sort directories by their numeric prefix
        directories.sort_by(|a, b| a.0.cmp(&b.0));

        let mut sql_files = Vec::new();

        // Collect SQL files from each directory in order
        for (_, dir_path) in directories {
            let mut dir_files = self.collect_sql_files_from_directory(&dir_path).await?;
            sql_files.append(&mut dir_files);
        }

        Ok(sql_files)
    }

    /// Check if a structured directory should be included based on environment filtering
    fn should_include_structured_directory(&self, dir_name: &str, environments: &[&str]) -> bool {
        // Always include schema and common directories
        if dir_name.contains("_schema") || dir_name.contains("_seed_common") {
            return true;
        }

        // For environment-specific directories, check if environment is requested
        for env in environments {
            if dir_name.contains(&format!("_seed_{}", env))
                || dir_name.contains(&format!("_{}", env))
            {
                return true;
            }
        }

        // If no environments specified, include all non-environment directories
        if environments.is_empty() {
            // This is a basic heuristic - include if it doesn't look environment-specific
            let env_keywords = ["_dev", "_test", "_prod", "_staging"];
            return !env_keywords
                .iter()
                .any(|keyword| dir_name.contains(keyword));
        }

        false
    }

    /// Discover files in flat repository
    async fn discover_flat_files(&self) -> SqlRepositoryResult<Vec<PathBuf>> {
        self.collect_sql_files_from_directory(&self.repository_path)
            .await
    }

    /// Collect all SQL files from a directory in alphabetical order
    async fn collect_sql_files_from_directory(
        &self,
        directory: &Path,
    ) -> SqlRepositoryResult<Vec<PathBuf>> {
        let mut sql_files = Vec::new();
        let mut entries = async_fs::read_dir(directory).await.map_err(|e| {
            DatabaseError::Config(format!(
                "Failed to read directory {}: {}",
                directory.display(),
                e
            ))
        })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DatabaseError::Config(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "sql" {
                        sql_files.push(path);
                    }
                }
            }
        }

        // Sort files alphabetically
        sql_files.sort();

        Ok(sql_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_structured_repository_creation() {
        let temp_dir = TempDir::new().unwrap();

        // Create structured directories
        fs::create_dir(temp_dir.path().join("0_schema"))
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("1_seed_common"))
            .await
            .unwrap();

        let repo = SqlRepository::new(temp_dir.path()).unwrap();

        assert!(repo.is_structured_repository().await.unwrap());
    }

    #[tokio::test]
    async fn test_flat_repository_creation() {
        let temp_dir = TempDir::new().unwrap();

        // Create some SQL files (no structured directories)
        fs::write(
            temp_dir.path().join("schema.sql"),
            "CREATE TABLE test (id SERIAL);",
        )
        .await
        .unwrap();

        let repo = SqlRepository::new(temp_dir.path()).unwrap();

        assert!(!repo.is_structured_repository().await.unwrap());
    }

    #[tokio::test]
    async fn test_sql_content_loading() {
        let temp_dir = TempDir::new().unwrap();
        let sql_file = temp_dir.path().join("test.sql");
        let sql_content = "CREATE TABLE users (id SERIAL PRIMARY KEY);";

        fs::write(&sql_file, sql_content).await.unwrap();

        let repo = SqlRepository::new(temp_dir.path()).unwrap();
        let loaded_content = repo.load_sql_content(&sql_file).await.unwrap();

        assert_eq!(loaded_content.trim(), sql_content);
    }
}
