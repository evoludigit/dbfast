//! Environment filtering for database deployments
//!
//! This module provides environment-specific file filtering to deploy
//! different SQL files to different environments.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Configuration for environment-specific file filtering
///
/// Filters are applied in this order:
/// 1. Directory include filter (if specified, only included dirs are processed)
/// 2. Directory exclude filter (remove excluded directories)
/// 3. File include filter (add specifically included files)
/// 4. File exclude filter (remove specifically excluded files - highest priority)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EnvironmentConfig {
    /// Name of the environment
    pub name: String,
    /// Directories to include (if specified, only these are included)
    pub include_directories: Option<Vec<String>>,
    /// Directories to exclude
    pub exclude_directories: Option<Vec<String>>,
    /// Files to include (glob patterns)
    pub include_files: Option<Vec<String>>,
    /// Files to exclude (glob patterns, highest priority)
    pub exclude_files: Option<Vec<String>>,
}

/// Errors that can occur during environment filtering
#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    /// Invalid glob pattern
    #[error("Invalid glob pattern: {pattern}")]
    InvalidPattern {
        /// The invalid pattern that caused the error
        pattern: String,
    },
    /// Path processing error
    #[error("Failed to process path: {path}")]
    PathError {
        /// The path that caused the error
        path: String,
    },
}

impl EnvironmentConfig {
    /// Filter a list of files based on this environment's configuration
    ///
    /// # Errors
    /// Returns `FilterError` if path processing fails or patterns are invalid
    #[allow(clippy::unnecessary_wraps)]
    pub fn filter_files(&self, all_files: &[PathBuf]) -> Result<Vec<PathBuf>, FilterError> {
        Ok(all_files
            .iter()
            .filter(|file| self.should_include_file(file))
            .cloned()
            .collect())
    }

    /// Validate the configuration for a given base path
    pub fn validate(&self, base_path: &Path) -> Result<(), FilterError> {
        // Validate that include directories exist if specified
        if let Some(include_dirs) = &self.include_directories {
            for dir in include_dirs {
                let dir_path = base_path.join(dir);
                if !dir_path.exists() {
                    return Err(FilterError::PathError {
                        path: dir_path.to_string_lossy().to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Check if a file should be included based on filtering rules
    fn should_include_file(&self, file: &Path) -> bool {
        let file_str = file.to_string_lossy();
        let directory = Self::extract_directory(&file_str);
        let filename = Self::extract_filename(&file_str);

        self.apply_filters(&directory, &filename, &file_str)
    }

    /// Extract the relevant directory from a file path
    /// For paths like `"tests/fixtures/sql/0_schema/tables.sql"`, extracts `"0_schema"`
    fn extract_directory(file_str: &str) -> String {
        let path_parts: Vec<&str> = file_str.split('/').collect();

        // Find the directory after "sql"
        let mut found_sql = false;
        for part in &path_parts {
            if *part == "sql" {
                found_sql = true;
                continue;
            }
            if found_sql {
                return (*part).to_string();
            }
        }

        String::new()
    }

    /// Extract filename from path
    fn extract_filename(file_str: &str) -> String {
        file_str.split('/').last().unwrap_or("").to_string()
    }

    /// Apply include/exclude filters to determine if file should be included
    fn apply_filters(
        &self,
        dir: &str,
        filename: &str,
        _file_str: &str,
    ) -> bool {
        // 1. Apply directory include filter
        if let Some(include_dirs) = &self.include_directories {
            if !include_dirs.iter().any(|d| dir == d) {
                return false;
            }
        }

        // 2. Apply directory exclude filter
        if let Some(exclude_dirs) = &self.exclude_directories {
            if exclude_dirs.iter().any(|d| dir == d) {
                return false;
            }
        }

        // 3. Apply file exclude filter (highest priority)
        if let Some(exclude_files) = &self.exclude_files {
            if Self::matches_exclude_pattern(filename, exclude_files) {
                return false;
            }
        }

        true
    }

    /// Check if filename matches any exclude patterns
    fn matches_exclude_pattern(filename: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if let Some(stripped) = pattern.strip_prefix("**/") {
                // Handle patterns like "prod_*.sql" or "test_*.sql"
                if stripped.ends_with("*.sql") {
                    let prefix = stripped.trim_end_matches("*.sql");
                    if filename.starts_with(prefix) && 
                       std::path::Path::new(filename)
                           .extension()
                           .map_or(false, |ext| ext.eq_ignore_ascii_case("sql")) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
