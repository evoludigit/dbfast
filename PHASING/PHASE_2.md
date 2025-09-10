# Phase 2 - Template Management

**Goal**: Implement PostgreSQL template database creation and cloning functionality.

## TDD Approach

Write tests for template operations first, then implement the template management system.

## Deliverables

### 1. Template Creation
- [ ] Build template database from SQL files
- [ ] Execute SQL files in correct order (0_schema, 1_seed, etc.)
- [ ] Handle transaction management and rollback
- [ ] Store template metadata (schema hash, created time)

**Test**: Create template from test SQL files, verify tables exist

### 2. Template Validation
- [ ] Check if template exists
- [ ] Validate template has required tables/functions
- [ ] Compare schema hash to detect changes
- [ ] Run validation queries

**Test**: Template validation passes/fails correctly

### 3. Database Cloning
- [ ] Clone template using `CREATE DATABASE WITH TEMPLATE`
- [ ] Handle clone timeouts and errors
- [ ] Generate unique database names
- [ ] Track cloned databases

**Test**: Clone database in <200ms, verify data exists

### 4. Template Rebuilding
- [ ] Detect when rebuild is needed (file changes)
- [ ] Drop and recreate template atomically
- [ ] Handle rebuild failures gracefully

**Test**: Rebuild when files change, skip when unchanged

### 5. Cleanup Operations
- [ ] Drop old/unused databases
- [ ] Clean up failed template builds
- [ ] Manage disk space

**Test**: Cleanup removes correct databases

## Success Criteria

1. **Template creation**: Can build template from SQL files
2. **Fast cloning**: Can clone database in <200ms
3. **Change detection**: Only rebuilds when files actually change
4. **Validation**: Template contains expected data
5. **Error handling**: Graceful failure and recovery

## Key Files to Add/Modify

```
src/
├── template.rs          # Template management
├── executor.rs          # SQL execution engine
├── validator.rs         # Template validation
└── cleanup.rs          # Database cleanup

tests/
├── template_tests.rs    # Template operation tests
└── fixtures/
    └── sql/            # Test SQL files for template
```

## Core Functionality

### Template Manager
```rust
pub struct TemplateManager {
    pool: Pool<PostgresConnectionManager>,
    template_name: String,
    validator: Validator,
}

impl TemplateManager {
    pub async fn needs_rebuild(&self, schema_hash: u64) -> Result<bool>;
    pub async fn rebuild_template(&self, sql_files: &[PathBuf]) -> Result<()>;
    pub async fn clone_database(&self, target_name: &str) -> Result<CloneResult>;
    pub async fn validate_template(&self) -> Result<ValidationResult>;
}
```

### SQL Executor
```rust  
pub struct SqlExecutor {
    pool: Pool<PostgresConnectionManager>,
}

impl SqlExecutor {
    pub async fn execute_file(&self, file_path: &Path) -> Result<ExecutionStats>;
    pub async fn execute_batch(&self, files: &[PathBuf]) -> Result<()>;
    pub async fn create_database(&self, name: &str, template: Option<&str>) -> Result<()>;
}
```

## Example Tests to Write

### Template Creation Test
```rust
#[tokio::test]
async fn test_template_creation() {
    let manager = TemplateManager::new(pool, "test_template".to_string());
    let sql_files = vec![
        PathBuf::from("tests/fixtures/sql/0_schema/tables.sql"),
        PathBuf::from("tests/fixtures/sql/1_seed/users.sql"),
    ];
    
    let result = manager.rebuild_template(&sql_files).await.unwrap();
    assert!(result.success);
    
    // Verify template exists and has data
    let validation = manager.validate_template().await.unwrap();
    assert!(validation.tables_exist);
}
```

### Clone Performance Test
```rust
#[tokio::test] 
async fn test_fast_cloning() {
    let manager = TemplateManager::new(pool, "test_template".to_string());
    
    let start = Instant::now();
    let result = manager.clone_database("test_clone_123").await.unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 200); // <200ms target
    assert_eq!(result.database_name, "test_clone_123");
}
```

### Change Detection Test
```rust
#[tokio::test]
async fn test_change_detection() {
    let scanner = FileScanner::new("tests/fixtures/sql");
    let initial_hash = scanner.calculate_schema_hash().unwrap();
    
    let manager = TemplateManager::new(pool, "test_template".to_string());
    
    // Should not need rebuild initially
    assert!(!manager.needs_rebuild(initial_hash).await.unwrap());
    
    // Modify a file
    std::fs::write("tests/fixtures/sql/1_seed/users.sql", "-- modified").unwrap();
    let new_hash = scanner.calculate_schema_hash().unwrap();
    
    // Should need rebuild after change
    assert!(manager.needs_rebuild(new_hash).await.unwrap());
}
```

## Commands to Implement

### `dbfast rebuild`
```bash
dbfast rebuild --force
```
- Rebuilds template from current SQL files
- Shows progress and timing
- Validates result

### `dbfast seed`
```bash
dbfast seed --output test_db_123
```
- Clones template to new database
- Shows clone time
- Returns database name

## Phase 2 Complete When

- [ ] All tests pass for template operations
- [ ] Can build template from SQL files in ~30-60 seconds
- [ ] Can clone database in <200ms consistently
- [ ] Change detection works (rebuilds only when needed)
- [ ] Template validation catches missing tables/data
- [ ] `dbfast rebuild` and `dbfast seed` commands work

**Next Phase**: Environment-specific file filtering