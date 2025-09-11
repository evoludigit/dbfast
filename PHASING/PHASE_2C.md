# Phase 2C - Database Cloning

**Goal**: Implement fast database cloning using PostgreSQL's `CREATE DATABASE WITH TEMPLATE`.

## TDD Approach
Write tests for database cloning first, focusing on speed and reliability.

## Deliverables

### 1. Template Cloning
- [ ] `CREATE DATABASE WITH TEMPLATE` implementation
- [ ] Handle clone naming (unique names)
- [ ] Clone timeout and error handling
- [ ] Verify clone completed successfully

**Test**: Clone database from template in <500ms, verify data integrity

### 2. Clone Management
- [ ] Track cloned databases
- [ ] Generate unique clone names
- [ ] Handle concurrent clone operations
- [ ] Clone cleanup and removal

**Test**: Multiple concurrent clones work correctly

### 3. Performance Optimization
- [ ] Connection pool optimization for cloning
- [ ] Parallel clone operations where safe
- [ ] Memory usage during cloning
- [ ] Fast clone validation

**Test**: Clone performance meets target (<200ms for small databases)

## Success Criteria

1. **Speed**: Database clones complete in <200ms for small databases
2. **Reliability**: Clone operations are atomic and always succeed or fail cleanly
3. **Concurrency**: Multiple simultaneous clones work correctly
4. **Data Integrity**: Cloned databases have identical data to template

## Key Files to Modify

```
src/
├── clone.rs            # New: Database cloning logic
├── template.rs         # Add clone methods to template manager
├── database.rs         # Add cloning database operations
└── commands/seed.rs    # Integrate cloning into seed command

tests/
├── clone_tests.rs      # Database cloning tests
├── performance_tests.rs # Clone performance benchmarks
└── fixtures/
    └── clone_scenarios/ # Different cloning test cases
```

## Example Test

```rust
#[tokio::test]
async fn test_database_cloning() {
    let pool = test_db_pool().await;
    let template_manager = TemplateManager::new(pool.clone());

    // First create a template
    let sql_files = get_sql_files_from_db_dir().await;
    template_manager.create_template("blog_template", &sql_files).await.unwrap();

    // Now test cloning
    let clone_manager = CloneManager::new(pool);

    let start = Instant::now();
    let clone_result = clone_manager.clone_database("blog_template", "test_clone_1").await;
    let clone_duration = start.elapsed();

    assert!(clone_result.is_ok());
    assert!(clone_duration.as_millis() < 500); // <500ms for small database

    // Verify clone has same data as template
    let template_user_count = count_rows(&pool, "blog_template", "blog.tb_user").await.unwrap();
    let clone_user_count = count_rows(&pool, "test_clone_1", "blog.tb_user").await.unwrap();
    assert_eq!(template_user_count, clone_user_count);

    // Verify clone is independent (changes don't affect template)
    execute_sql(&pool, "test_clone_1", "INSERT INTO blog.tb_user (username, email, password_hash) VALUES ('test', 'test@test.com', 'hash')").await.unwrap();

    let template_count_after = count_rows(&pool, "blog_template", "blog.tb_user").await.unwrap();
    let clone_count_after = count_rows(&pool, "test_clone_1", "blog.tb_user").await.unwrap();

    assert_eq!(template_user_count, template_count_after); // Template unchanged
    assert_eq!(clone_user_count + 1, clone_count_after); // Clone has new data
}
```

**Duration**: 2-3 days
**Next**: Phase 2D - Change Detection
