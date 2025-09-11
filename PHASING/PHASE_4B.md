# Phase 4B - Backup Management

**Goal**: Implement automatic backup creation and restoration for safe remote deployments.

## TDD Approach
Write tests for backup scenarios first, then implement backup operations using pg_dump/pg_restore.

## Deliverables

### 1. Backup Creation
- [ ] pg_dump integration for creating backups
- [ ] Backup file naming and storage
- [ ] Backup metadata (timestamp, size, checksum)
- [ ] Compression support for backup files

**Test**: Backups are created correctly and contain expected data

### 2. Backup Restoration
- [ ] pg_restore integration for restoring backups
- [ ] Backup validation before restoration
- [ ] Atomic restoration (all-or-nothing)
- [ ] Restoration progress and error handling

**Test**: Backups can be restored successfully with data integrity

### 3. Backup Management
- [ ] List available backups
- [ ] Backup cleanup policies (keep last N, older than X)
- [ ] Backup verification and integrity checks
- [ ] Backup metadata storage and retrieval

**Test**: Backup management commands work correctly

## Success Criteria

1. **Reliability**: Backups always capture complete database state
2. **Speed**: Backup/restore operations are reasonably fast for small databases
3. **Safety**: Backup verification ensures data integrity
4. **Storage**: Efficient backup storage with compression

## Key Files to Modify

```
src/
├── backup.rs           # New: Backup creation and management
├── restore.rs          # New: Backup restoration logic
├── commands/
│   └── backup.rs       # New: Backup management commands
├── cli.rs              # Add backup subcommands
└── main.rs             # Wire up backup commands

tests/
├── backup_tests.rs     # Backup creation and restoration tests
└── fixtures/
    └── backups/        # Test backup scenarios
```

## Example Test

```rust
#[tokio::test]
async fn test_backup_creation() {
    let pool = test_db_pool().await;
    let temp_dir = tempfile::tempdir().unwrap();

    // Create test database with data
    setup_test_database(&pool, "test_source_db").await;
    populate_test_data(&pool, "test_source_db").await;

    // Create backup manager
    let backup_manager = BackupManager::new(temp_dir.path());

    let remote_config = RemoteConfig {
        name: "test_remote".to_string(),
        url: get_test_database_url("test_source_db"),
        environment: "local".to_string(),
        ..Default::default()
    };

    // Create backup
    let backup_info = backup_manager.create_backup(&remote_config).await.unwrap();

    // Verify backup file exists
    assert!(backup_info.file_path.exists());
    assert!(backup_info.size_bytes > 0);
    assert!(!backup_info.checksum.is_empty());

    // Verify backup contains expected data
    let backup_content = std::fs::read_to_string(&backup_info.file_path).unwrap();
    assert!(backup_content.contains("blog")); // Should contain our schema
    assert!(backup_content.contains("tb_user")); // Should contain our tables
}

#[tokio::test]
async fn test_backup_restoration() {
    let pool = test_db_pool().await;
    let temp_dir = tempfile::tempdir().unwrap();

    // Create source database with data
    setup_test_database(&pool, "backup_source").await;
    populate_test_data(&pool, "backup_source").await;

    // Create backup
    let backup_manager = BackupManager::new(temp_dir.path());
    let source_config = RemoteConfig {
        name: "source".to_string(),
        url: get_test_database_url("backup_source"),
        environment: "local".to_string(),
        ..Default::default()
    };

    let backup_info = backup_manager.create_backup(&source_config).await.unwrap();

    // Create empty target database
    setup_test_database(&pool, "backup_target").await;

    let target_config = RemoteConfig {
        name: "target".to_string(),
        url: get_test_database_url("backup_target"),
        environment: "local".to_string(),
        ..Default::default()
    };

    // Restore backup to target
    backup_manager.restore_backup(&backup_info, &target_config).await.unwrap();

    // Verify data was restored correctly
    let source_user_count = count_users(&pool, "backup_source").await;
    let target_user_count = count_users(&pool, "backup_target").await;
    assert_eq!(source_user_count, target_user_count);

    // Verify specific data integrity
    let source_users = get_all_users(&pool, "backup_source").await;
    let target_users = get_all_users(&pool, "backup_target").await;
    assert_eq!(source_users, target_users);
}

#[tokio::test]
async fn test_backup_rollback_scenario() {
    let pool = test_db_pool().await;
    let temp_dir = tempfile::tempdir().unwrap();

    // Create database with initial data
    setup_test_database(&pool, "rollback_test").await;
    populate_test_data(&pool, "rollback_test").await;
    let initial_user_count = count_users(&pool, "rollback_test").await;

    // Create backup of initial state
    let backup_manager = BackupManager::new(temp_dir.path());
    let remote_config = RemoteConfig {
        name: "rollback_test".to_string(),
        url: get_test_database_url("rollback_test"),
        environment: "local".to_string(),
        ..Default::default()
    };

    let backup_info = backup_manager.create_backup(&remote_config).await.unwrap();

    // Modify database (simulate failed deployment)
    modify_test_data(&pool, "rollback_test").await;
    let modified_user_count = count_users(&pool, "rollback_test").await;
    assert_ne!(initial_user_count, modified_user_count);

    // Restore from backup (rollback)
    backup_manager.restore_backup(&backup_info, &remote_config).await.unwrap();

    // Verify rollback worked
    let restored_user_count = count_users(&pool, "rollback_test").await;
    assert_eq!(initial_user_count, restored_user_count);
}

#[test]
fn test_backup_commands() {
    let temp_dir = tempfile::tempdir().unwrap();
    setup_test_project(&temp_dir);

    // Test backup create command
    let output = Command::new("cargo")
        .args([
            "run", "--", "backup", "create",
            "--remote", "staging"
        ])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Backup created") || stdout.contains("✅"));
    } else {
        // If command fails, should be due to missing remote database, not command structure
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("connection") || stderr.contains("remote"));
    }

    // Test backup list command
    let output = Command::new("cargo")
        .args(["run", "--", "backup", "list", "--remote", "staging"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute command");

    // Should succeed even if no backups exist
    assert!(output.status.success());
}
```

**Duration**: 3-4 days
**Next**: Phase 4C - Deployment Execution
