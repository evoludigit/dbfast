//! Backup creation and management for database deployments

use std::path::PathBuf;

/// Manages database backups for safe deployments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupManager {
    /// Directory where backups are stored
    pub backup_dir: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_manager_creation() {
        let backup_dir = PathBuf::from("/tmp/dbfast-backups");
        let manager = BackupManager::new(backup_dir.clone());

        assert_eq!(manager.backup_dir, backup_dir);
    }
}
