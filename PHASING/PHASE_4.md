# Phase 4: Micro TDD Cycles

**Goal**: Implement safe remote database deployment using micro TDD cycles with red-green-refactor pattern.

## TDD Cycle Pattern

### Red 🔴
- Write a failing test first
- Test should capture the next small requirement
- Run test to ensure it fails for the right reason

### Green 🟢
- Write minimal code to make the test pass
- Focus on making it work, not making it perfect
- Run test to ensure it passes

### Refactor 🔧
- Improve code quality without changing behavior
- Remove duplication, improve naming, structure
- Run tests to ensure they still pass

### Commit ✅
- Commit each complete red-green-refactor cycle
- Use descriptive commit messages
- Keep commits small and focused

## Core Concept

Deploy database changes safely to remote environments:
- **Dump-based deployment**: Primary method for safety
- **Environment-specific filtering**: Only deploy appropriate files
- **Automatic backups**: Before any destructive operations
- **Atomic operations**: All-or-nothing deployments
- **Rollback on failure**: Restore from backup if deployment fails

## Deliverables

### 1. Remote Configuration
- [ ] Parse remote configs from `dbfast.toml`
- [ ] Validate remote connection strings
- [ ] Environment linking (remote -> environment config)
- [ ] Safety settings (allow_destructive, backup_before_deploy)

**Test**: Load remote configs, validate connections

### 2. Dump-Based Deployment
- [ ] Create pg_dump from local template
- [ ] Compress dumps (LZ4/ZSTD)
- [ ] Transfer dump to remote
- [ ] Restore using pg_restore
- [ ] Handle large dumps efficiently

**Test**: Deploy via dump, verify data integrity

### 3. Backup & Rollback
- [ ] Create backup before deployment
- [ ] Store backup metadata (timestamp, size, hash)
- [ ] Rollback mechanism on deployment failure
- [ ] Cleanup old backups

**Test**: Backup creation, rollback on failure

### 4. Deployment Validation
- [ ] Pre-deployment checks (connectivity, permissions)
- [ ] Post-deployment validation (run test queries)
- [ ] Schema comparison (ensure deployment matches expectations)
- [ ] Data integrity checks

**Test**: Validation catches deployment issues

### 5. Safety Mechanisms
- [ ] Confirmation required for production
- [ ] Environment mismatch detection
- [ ] Destructive operation controls
- [ ] Deployment tracking and logging

**Test**: Safety checks prevent accidents

## Success Criteria

1. **Safe deployments**: No accidental data loss
2. **Atomic operations**: Deployment succeeds completely or fails cleanly
3. **Fast rollback**: Can restore from backup quickly
4. **Environment safety**: Production never gets dev data
5. **Reliable transfers**: Handle network issues gracefully

## Key Files to Add/Modify

```
src/
├── remote.rs           # Remote deployment manager
├── backup.rs           # Backup creation and management
├── dump.rs             # pg_dump/pg_restore operations
├── deployment.rs       # Deployment orchestration
└── validation.rs       # Pre/post deployment validation

tests/
├── deployment_tests.rs # Deployment scenario tests
├── backup_tests.rs     # Backup/restore tests
└── fixtures/
    └── remotes/        # Test remote configurations
```

## Core Data Structures

### Remote Configuration
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub url: String,
    pub user: String,
    pub password_env: String,
    pub environment: String,
    pub allow_destructive: bool,
    pub backup_before_deploy: bool,
    pub require_confirmation: bool,
}

impl RemoteConfig {
    pub async fn validate_connection(&self) -> Result<()>;
    pub async fn create_pool(&self) -> Result<Pool<PostgresConnectionManager>>;
}
```

### Deployment Manager
```rust
pub struct DeploymentManager {
    local_pool: Pool<PostgresConnectionManager>,
    backup_manager: BackupManager,
    validator: DeploymentValidator,
}

impl DeploymentManager {
    pub async fn deploy(&self, remote: &RemoteConfig, environment: &str) -> Result<DeploymentResult>;
    pub async fn rollback(&self, remote: &RemoteConfig, backup_id: &str) -> Result<()>;
    pub async fn validate_deployment(&self, remote: &RemoteConfig) -> Result<ValidationResult>;
}
```

### Backup Manager
```rust
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub async fn create_backup(&self, remote: &RemoteConfig) -> Result<BackupInfo>;
    pub async fn restore_backup(&self, backup_info: &BackupInfo, remote: &RemoteConfig) -> Result<()>;
    pub async fn cleanup_old_backups(&self, keep_count: usize) -> Result<()>;
}
```

## Example Tests to Write

### Successful Deployment Test
```rust
#[tokio::test]
async fn test_successful_deployment() {
    let manager = DeploymentManager::new(local_pool);
    let remote = RemoteConfig {
        name: "test_staging".to_string(),
        url: "postgres://staging:5432/testdb".to_string(),
        environment: "staging".to_string(),
        backup_before_deploy: true,
        ..Default::default()
    };

    let result = manager.deploy(&remote, "staging").await.unwrap();

    assert!(result.success);
    assert!(result.backup_created);
    assert!(result.validation_passed);
    assert!(result.deploy_time_ms < 180000); // < 3 minutes
}
```

### Deployment Failure and Rollback Test
```rust
#[tokio::test]
async fn test_deployment_failure_rollback() {
    let manager = DeploymentManager::new(local_pool);

    // Create backup first
    let backup = manager.backup_manager.create_backup(&remote_config).await.unwrap();

    // Simulate deployment failure (invalid SQL)
    let bad_sql_files = vec![PathBuf::from("tests/fixtures/invalid.sql")];

    let result = manager.deploy_sql_files(&remote_config, &bad_sql_files).await;
    assert!(result.is_err());

    // Verify automatic rollback occurred
    let validation = manager.validate_deployment(&remote_config).await.unwrap();
    assert!(validation.matches_backup(&backup));
}
```

### Environment Safety Test
```rust
#[tokio::test]
async fn test_production_safety() {
    let manager = DeploymentManager::new(local_pool);
    let prod_remote = RemoteConfig {
        environment: "production".to_string(),
        allow_destructive: false,
        require_confirmation: true,
        ..Default::default()
    };

    // Should fail without confirmation
    let result = manager.deploy(&prod_remote, "production").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("confirmation required"));

    // Should succeed with confirmation
    let result = manager.deploy_with_confirmation(&prod_remote, "production", true).await;
    assert!(result.is_ok());
}
```

### Backup and Restore Test
```rust
#[tokio::test]
async fn test_backup_and_restore() {
    let backup_manager = BackupManager::new(temp_dir.path());

    // Create backup
    let backup_info = backup_manager.create_backup(&remote_config).await.unwrap();
    assert!(backup_info.file_path.exists());
    assert!(backup_info.size_bytes > 0);

    // Modify database
    execute_sql(&remote_config, "INSERT INTO test_table VALUES ('test')").await;

    // Restore backup
    backup_manager.restore_backup(&backup_info, &remote_config).await.unwrap();

    // Verify restoration
    let count: i64 = query_scalar(&remote_config, "SELECT COUNT(*) FROM test_table").await;
    assert_eq!(count, 0); // Back to original state
}
```

## Deployment Process Flow

1. **Pre-deployment validation**
   - Check remote connectivity
   - Validate environment configuration
   - Ensure deployment safety settings

2. **Backup creation** (if enabled)
   - Create pg_dump of current remote state
   - Store backup with metadata
   - Verify backup integrity

3. **Template preparation**
   - Filter SQL files for target environment
   - Create local template with filtered files
   - Generate deployment dump

4. **Deployment execution**
   - Transfer dump to remote (with compression)
   - Execute pg_restore on remote
   - Handle errors and timeouts

5. **Post-deployment validation**
   - Run validation queries
   - Compare schema checksums
   - Verify data integrity

6. **Cleanup or rollback**
   - If successful: cleanup temp files, log success
   - If failed: automatic rollback to backup

## Commands to Implement

### Remote Management
```bash
dbfast remote add --name staging --url postgres://staging:5432/db --env staging
dbfast remote list
dbfast remote test --name staging  # Test connectivity
```

### Deployment
```bash
dbfast deploy --remote staging --env staging --confirm
dbfast deploy --remote production --env production --confirm --backup-before
```

### Backup Management
```bash
dbfast backup create --remote staging
dbfast backup list --remote staging
dbfast backup restore --remote staging --backup-id abc123
```

## Phase 4 Complete When

- [ ] All deployment tests pass (success and failure scenarios)
- [ ] Can deploy to remote database safely via dump/restore
- [ ] Automatic backup and rollback on failure works
- [ ] Environment safety prevents production accidents
- [ ] `dbfast deploy` command works with confirmation
- [ ] `dbfast remote` commands manage remote configurations
- [ ] Deployment validation catches common issues
- [ ] Backup management maintains deployment history

**Next Phase**: CLI polish and advanced features
# Phase 4 Complete
