use crate::scanner::ScannedFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

/// Errors that can occur during change detection
#[derive(Debug, Error)]
pub enum ChangeDetectionError {
    /// IO error occurred while reading/writing metadata
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Template metadata stored for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template name
    pub name: String,
    /// Timestamp when template was created/updated
    pub created_at: String,
    /// File hashes at time of template creation
    pub file_hashes: HashMap<PathBuf, String>,
}

/// Change detector for identifying when SQL files have changed and templates need rebuilding
#[derive(Clone)]
pub struct ChangeDetector {
    /// Root path to scan for SQL files
    root_path: PathBuf,
    /// Path where template metadata is stored
    metadata_dir: PathBuf,
}

impl ChangeDetector {
    /// Create a new change detector for the given directory
    ///
    /// # Arguments
    /// * `root_path` - The root directory to scan for SQL files
    #[must_use]
    pub fn new(root_path: PathBuf) -> Self {
        let metadata_dir = root_path.join(".dbfast");
        Self {
            root_path,
            metadata_dir,
        }
    }

    /// Get the root path being monitored
    #[must_use]
    pub fn root_path(&self) -> &std::path::Path {
        &self.root_path
    }

    /// Check if a template needs rebuilding based on file changes
    ///
    /// # Arguments
    /// * `template_name` - Name of the template to check
    ///
    /// # Returns
    /// `true` if the template needs rebuilding, `false` if it's up to date
    pub async fn template_needs_rebuild(
        &self,
        template_name: &str,
    ) -> Result<bool, ChangeDetectionError> {
        // Get stored metadata for the template
        let Some(stored_metadata) = self.get_template_metadata(template_name).await? else {
            // No metadata exists, template needs to be built
            return Ok(true);
        };

        // Scan current files
        let scanner = crate::scanner::FileScanner::new(&self.root_path);
        let current_files = scanner.scan().map_err(|e| {
            ChangeDetectionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Scanner error: {e}"),
            ))
        })?;

        // Compare current files with stored metadata
        Ok(Self::compare_files(&current_files, &stored_metadata))
    }

    /// Store template metadata for change detection
    ///
    /// # Arguments
    /// * `template_name` - Name of the template
    /// * `scanned_files` - Files that were used to create the template
    pub async fn store_template_metadata(
        &self,
        template_name: &str,
        scanned_files: &[ScannedFile],
    ) -> Result<(), ChangeDetectionError> {
        // Ensure metadata directory exists
        fs::create_dir_all(&self.metadata_dir).await?;

        // Create file hash map
        let mut file_hashes = HashMap::new();
        for file in scanned_files {
            file_hashes.insert(file.path.clone(), file.hash.clone());
        }

        let metadata = TemplateMetadata {
            name: template_name.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            file_hashes,
        };

        // Write metadata to file
        let metadata_file = self.metadata_dir.join(format!("{template_name}.json"));
        let json_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_file, json_content).await?;

        Ok(())
    }

    /// Get stored template metadata
    ///
    /// # Arguments
    /// * `template_name` - Name of the template
    ///
    /// # Returns
    /// `Some(Vec<ScannedFile>)` if metadata exists, `None` otherwise
    pub async fn get_template_metadata(
        &self,
        template_name: &str,
    ) -> Result<Option<Vec<ScannedFile>>, ChangeDetectionError> {
        let metadata_file = self.metadata_dir.join(format!("{template_name}.json"));

        if !metadata_file.exists() {
            return Ok(None);
        }

        let json_content = fs::read_to_string(metadata_file).await?;
        let metadata: TemplateMetadata = serde_json::from_str(&json_content)?;

        // Convert back to ScannedFile format
        let mut scanned_files = Vec::new();
        for (path, hash) in metadata.file_hashes {
            scanned_files.push(ScannedFile { path, hash });
        }

        // Sort by path for consistent ordering
        scanned_files.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Some(scanned_files))
    }

    /// Compare current files with stored metadata to determine if rebuild is needed
    fn compare_files(current_files: &[ScannedFile], stored_metadata: &[ScannedFile]) -> bool {
        // Quick check: different number of files
        if current_files.len() != stored_metadata.len() {
            return true; // Files added/deleted, rebuild needed
        }

        // Create hash maps for efficient comparison
        let current_map: HashMap<&PathBuf, &String> =
            current_files.iter().map(|f| (&f.path, &f.hash)).collect();

        let stored_map: HashMap<&PathBuf, &String> =
            stored_metadata.iter().map(|f| (&f.path, &f.hash)).collect();

        // Check if any files have different hashes or are missing
        for (path, current_hash) in &current_map {
            match stored_map.get(path) {
                Some(stored_hash) => {
                    if current_hash != stored_hash {
                        return true; // File modified, rebuild needed
                    }
                }
                None => return true, // New file, rebuild needed
            }
        }

        // Check for deleted files
        for path in stored_map.keys() {
            if !current_map.contains_key(path) {
                return true; // File deleted, rebuild needed
            }
        }

        // All files match, no rebuild needed
        false
    }
}
