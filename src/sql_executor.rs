/// SQL file execution functionality for DBFast
use crate::error::{DbFastError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// SQL file executor that can read and execute SQL files
pub struct SqlExecutor {
    // Will contain database connection and configuration
}

impl SqlExecutor {
    /// Create a new SQL executor
    pub fn new() -> Self {
        Self {}
    }

    /// Read SQL files from a directory and return ordered list of SQL statements
    pub fn read_sql_files<P: AsRef<Path>>(db_path: P) -> Result<Vec<String>> {
        let db_path = db_path.as_ref();
        
        if !db_path.exists() {
            return Err(DbFastError::ConfigCreationFailed {
                message: format!("Database path does not exist: {}", db_path.display()),
            });
        }

        let mut sql_files = Vec::new();
        
        // Recursively find all SQL files
        fn find_sql_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            if dir.is_dir() {
                let entries = fs::read_dir(dir).map_err(|e| DbFastError::ConfigCreationFailed {
                    message: format!("Failed to read directory {}: {}", dir.display(), e),
                })?;
                
                for entry in entries {
                    let entry = entry.map_err(|e| DbFastError::ConfigCreationFailed {
                        message: format!("Failed to read directory entry: {}", e),
                    })?;
                    let path = entry.path();
                    
                    if path.is_dir() {
                        find_sql_files(&path, files)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }
        
        find_sql_files(db_path, &mut sql_files)?;
        
        // Sort files by path to ensure consistent ordering
        sql_files.sort();
        
        let mut all_statements = Vec::new();
        
        // Read each file and parse statements
        for file_path in sql_files {
            let content = fs::read_to_string(&file_path).map_err(|e| DbFastError::ConfigCreationFailed {
                message: format!("Failed to read file {}: {}", file_path.display(), e),
            })?;
            
            let statements = Self::parse_sql_statements(&content);
            all_statements.extend(statements);
        }
        
        Ok(all_statements)
    }

    /// Parse SQL content into individual statements
    pub fn parse_sql_statements(content: &str) -> Vec<String> {
        // Simple implementation: split on semicolon and clean up
        content
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}