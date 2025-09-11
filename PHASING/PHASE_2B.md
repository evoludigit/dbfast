# Phase 2B - Database Template Creation

**Goal**: Create PostgreSQL template databases from executed SQL files.

## TDD Approach
Write tests for template creation first, then implement template management.

## Deliverables

### 1. Template Database Creation
- [ ] Create template database with specific name
- [ ] Execute SQL files against template database
- [ ] Handle template creation errors
- [ ] Validate template was created successfully

**Test**: Create template database, verify it exists and has correct schema

### 2. Template Metadata
- [ ] Store template metadata (created time, file hash)
- [ ] Track which files were used to build template
- [ ] Template validation queries
- [ ] Template naming conventions

**Test**: Template metadata is stored and retrieved correctly

### 3. Template Management
- [ ] Check if template exists before creating
- [ ] Drop existing template if rebuilding
- [ ] Handle concurrent template operations
- [ ] Clean up failed template creation

**Test**: Template creation is atomic, handles failures gracefully

## Success Criteria

1. **Template Creation**: Can create template database from SQL files
2. **Validation**: Can verify template has correct schema and data
3. **Metadata**: Tracks template creation info for change detection
4. **Error Handling**: Atomic operations, clean rollback on failure

## Key Files to Modify

```
src/
├── template.rs         # New: Template management logic
├── database.rs         # Add template-specific database operations
└── commands/
    └── init.rs         # Add template creation to init command

tests/
├── template_tests.rs   # Template creation and management tests
└── fixtures/
    └── templates/      # Template test scenarios
```

## Example Test

```rust
#[tokio::test]
async fn test_template_creation() {
    let pool = test_db_pool().await;
    let template_manager = TemplateManager::new(pool);

    let sql_files = get_sql_files_from_db_dir().await;
    let result = template_manager.create_template("blog_template", &sql_files).await;

    assert!(result.is_ok());

    // Verify template database exists
    assert!(template_exists(&pool, "blog_template").await.unwrap());

    // Verify template has correct schema
    let tables = get_tables_in_template(&pool, "blog_template").await.unwrap();
    assert!(tables.contains("tb_user"));
    assert!(tables.contains("tb_post"));
    assert!(tables.contains("tb_comment"));

    // Verify template has seed data
    let user_count = count_rows_in_template(&pool, "blog_template", "blog.tb_user").await.unwrap();
    assert!(user_count > 0);
}
```

**Duration**: 2-3 days
**Next**: Phase 2C - Database Cloning
