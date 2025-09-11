# Phase 3A - Environment Configuration & File Filtering

**Goal**: Implement environment-specific file filtering to deploy different files to different environments.

## TDD Approach
Write tests for various filtering scenarios first, then implement the filtering engine.

## Deliverables

### 1. Environment Configuration Loading
- [ ] Parse environment configs from existing `dbfast.toml`
- [ ] Validate environment configuration syntax
- [ ] Handle include/exclude directories and files
- [ ] Support glob pattern matching

**Test**: Load environment configs, validate patterns work correctly

### 2. File Filtering Engine
- [ ] Apply directory filters (include then exclude)
- [ ] Apply file filters (include then exclude)
- [ ] Resolve glob patterns efficiently using existing `globset`
- [ ] Maintain execution order within filtered results

**Test**: Filter produces correct file lists for each environment

### 3. Environment Integration
- [ ] Add `--env` flag to seed command
- [ ] Filter SQL files before template creation
- [ ] Create environment-specific templates
- [ ] Handle template naming per environment

**Test**: Different environments produce different templates with correct files

## Success Criteria

1. **Correct Filtering**: Each environment includes only intended files
2. **Performance**: Filtering is fast even with many files (existing scanner is already fast)
3. **Validation**: Can detect invalid environment configurations
4. **Safety**: Production never accidentally gets dev/test files

## Key Files to Modify

```
src/
├── environment.rs      # New: Environment filtering logic
├── filter.rs           # New: File filtering engine
├── template.rs         # Add environment support to templates
├── commands/seed.rs    # Add --env flag and filtering
└── config.rs           # Already has environment config support

tests/
├── environment_tests.rs # Environment filtering tests (already exists as stub)
└── fixtures/
    └── environments/   # Test environment configs
```

## Example Test

```rust
#[test]
fn test_environment_filtering() {
    let config = EnvironmentConfig {
        name: "local".to_string(),
        include_directories: Some(vec!["0_schema".to_string(), "1_seed_common".to_string()]),
        exclude_directories: Some(vec!["6_migration".to_string()]),
        exclude_files: Some(vec!["**/prod_*.sql".to_string()]),
        ..Default::default()
    };

    let all_files = vec![
        PathBuf::from("db/0_schema/01_write_side/010_blog/0101_user/010111_tb_user.sql"),
        PathBuf::from("db/1_seed_common/0101_admin_user.sql"),
        PathBuf::from("db/6_migration/001_add_column.sql"),
        PathBuf::from("db/1_seed_common/prod_data.sql"),
    ];

    let filter = FileFilter::new(&config).unwrap();
    let filtered = filter.filter_files(&all_files);

    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&PathBuf::from("db/0_schema/01_write_side/010_blog/0101_user/010111_tb_user.sql")));
    assert!(filtered.contains(&PathBuf::from("db/1_seed_common/0101_admin_user.sql")));
    assert!(!filtered.contains(&PathBuf::from("db/6_migration/001_add_column.sql")));
    assert!(!filtered.contains(&PathBuf::from("db/1_seed_common/prod_data.sql")));
}

#[tokio::test]
async fn test_environment_specific_templates() {
    let pool = test_db_pool().await;
    let template_manager = TemplateManager::new(pool.clone());

    // Load environments from config
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    let local_env = config.environments.get("local").unwrap();
    let prod_env = config.environments.get("production").unwrap();

    // Get all SQL files
    let all_files = get_sql_files_from_db_dir().await;

    // Filter for local environment
    let local_filter = FileFilter::new(local_env).unwrap();
    let local_files = local_filter.filter_files(&all_files);

    // Filter for production environment
    let prod_filter = FileFilter::new(prod_env).unwrap();
    let prod_files = prod_filter.filter_files(&all_files);

    // Local should have more files (includes seed data)
    assert!(local_files.len() > prod_files.len());

    // Create environment-specific templates
    template_manager.create_template("blog_template_local", &local_files).await.unwrap();
    template_manager.create_template("blog_template_production", &prod_files).await.unwrap();

    // Verify different content
    let local_tables = get_tables_in_template(&pool, "blog_template_local").await.unwrap();
    let prod_tables = get_tables_in_template(&pool, "blog_template_production").await.unwrap();

    // Both should have schema tables
    assert!(local_tables.contains("tb_user"));
    assert!(prod_tables.contains("tb_user"));

    // Local should have seed data, production should not
    let local_user_count = count_rows_in_template(&pool, "blog_template_local", "blog.tb_user").await.unwrap();
    let prod_user_count = count_rows_in_template(&pool, "blog_template_production", "blog.tb_user").await.unwrap();

    assert!(local_user_count > 0); // Has seed users
    assert_eq!(prod_user_count, 0); // No seed data in production
}
```

**Duration**: 2-3 days
**Next**: Phase 3B - Environment Commands
