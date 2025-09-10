use std::fs;
use std::io;
/// File scanning and hash calculation for change detection
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

/// Scanner-related errors
#[derive(Debug, Error)]
pub enum ScannerError {
    /// IO error occurred while scanning files
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// File walking error
    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
}

/// Represents a scanned SQL file with its hash
#[derive(Debug, Clone)]
pub struct ScannedFile {
    /// Path to the file
    pub path: PathBuf,
    /// Hash of the file contents
    pub hash: String,
}

/// File scanner for SQL files
pub struct FileScanner {
    /// Root directory to scan
    root_path: PathBuf,
}

impl FileScanner {
    /// Create a new file scanner for the given directory
    ///
    /// # Arguments
    /// * `path` - The root directory to scan for SQL files
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            root_path: path.as_ref().to_path_buf(),
        }
    }

    /// Scan for SQL files and calculate their hashes for change detection
    ///
    /// This method walks the directory tree recursively, finds all `.sql` files,
    /// and calculates a hash for each file to enable change detection.
    ///
    /// # Returns
    /// A vector of `ScannedFile` structs containing file paths and their hashes,
    /// sorted by path for consistent ordering.
    pub fn scan(&self) -> Result<Vec<ScannedFile>, ScannerError> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .follow_links(false)
            .into_iter()
        {
            let entry = entry?;
            let path = entry.path();

            // Only include SQL files
            if let Some(extension) = path.extension() {
                if extension == "sql" {
                    let contents = fs::read(path)?;
                    let hash = xxh3_64(&contents);

                    files.push(ScannedFile {
                        path: path.to_path_buf(),
                        hash: format!("{:016x}", hash),
                    });
                }
            }
        }

        // Sort by path for consistent ordering
        files.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(files)
    }
}
