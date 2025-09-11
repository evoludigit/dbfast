# Phase 2A - Basic SQL File Execution

**Goal**: Replace placeholder seed command with real SQL file execution functionality.

## TDD Approach
Write tests for SQL execution first, then implement minimal functionality to make tests pass.

## Deliverables

### 1. SQL File Reader
- [ ] Read SQL files from directory in order
- [ ] Handle file reading errors gracefully
- [ ] Support basic SQL parsing (split on semicolon)

**Test**: Read SQL files from `db/` directory, parse statements correctly

### 2. Database Execution
- [ ] Execute SQL statements sequentially
- [ ] Handle SQL syntax errors gracefully
- [ ] Log execution progress
- [ ] Return execution results

**Test**: Execute valid/invalid SQL, verify error handling

### 3. Replace Seed Placeholder
- [ ] Remove placeholder implementation in `src/commands/seed.rs`
- [ ] Add real SQL execution logic
- [ ] Connect to existing database pool
- [ ] Use existing config system

**Test**: `dbfast seed` creates real database with tables from SQL files

## Success Criteria

1. **File Reading**: Can scan and read SQL files from db/ directory
2. **SQL Execution**: Can execute SQL statements against PostgreSQL
3. **Error Handling**: Fails gracefully on SQL errors with helpful messages
4. **Integration**: Works with existing config and database pool

## Key Files to Modify

```
src/
├── database.rs         # Add SQL execution methods
├── commands/seed.rs    # Replace placeholder with real implementation
└── sql_executor.rs     # New: SQL file execution logic

tests/
├── sql_execution_tests.rs  # SQL execution tests
└── fixtures/
    └── sql/               # Use existing db/ structure for tests
```

## Example Test

```rust
#[tokio::test]
async fn test_sql_file_execution() {
    let pool = test_db_pool().await;
    let sql_files = vec![
        PathBuf::from("db/0_schema/01_write_side/010_blog/0101_user/010111_tb_user.sql"),
        PathBuf::from("db/1_seed_common/0101_admin_user.sql"),
    ];

    let result = execute_sql_files(&pool, &sql_files).await;

    assert!(result.is_ok());

    // Verify tables exist
    let tables = get_tables(&pool).await.unwrap();
    assert!(tables.contains("tb_user"));

    // Verify data exists
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blog.tb_user")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(user_count > 0);
}
```

**Duration**: 1-2 days
**Next**: Phase 2B - Database Template Creation
