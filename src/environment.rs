//! Environment filtering for database deployments
//!
//! This module provides environment-specific file filtering to deploy
//! different SQL files to different environments.

use serde::Deserialize;
use std::path::PathBuf;

/// Configuration for environment-specific file filtering
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

impl EnvironmentConfig {
    /// Filter a list of files based on this environment's configuration
    pub fn filter_files(&self, all_files: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut result = Vec::new();

        for file in all_files {
            if self.should_include_file(file) {
                result.push(file.clone());
            }
        }

        Ok(result)
    }

    /// Check if a file should be included based on filtering rules
    fn should_include_file(&self, file: &PathBuf) -> bool {
        let file_str = file.to_string_lossy();
        
        // Extract directory from path - looking for patterns like "tests/fixtures/sql/0_schema/tables.sql"
        // We want to extract "0_schema" part
        let path_parts: Vec<&str> = file_str.split('/').collect();
        let dir = if path_parts.len() >= 2 {
            // Find the directory after "sql" 
            let mut found_sql = false;
            for part in &path_parts {
                if *part == "sql" {
                    found_sql = true;
                    continue;
                }
                if found_sql {
                    return self.apply_filters(part, &file_str);
                }
            }
            ""
        } else {
            ""
        };

        self.apply_filters(dir, &file_str)
    }

    /// Apply include/exclude filters to determine if file should be included
    fn apply_filters(&self, dir: &str, file_str: &str) -> bool {
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
            for pattern in exclude_files {
                if pattern.starts_with("**/") {
                    let pattern_without_glob = &pattern[3..]; // Remove "**/""
                    let filename = file_str.split('/').last().unwrap_or("");
                    
                    // Handle patterns like "prod_*.sql" or "test_*.sql"
                    if pattern_without_glob.ends_with("*.sql") {
                        let prefix = pattern_without_glob.trim_end_matches("*.sql");
                        if filename.starts_with(prefix) && filename.ends_with(".sql") {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }
}