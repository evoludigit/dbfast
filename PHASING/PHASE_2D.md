# Phase 2D - Change Detection & Auto-Rebuild

**Goal**: Integrate file scanning with template management for automatic rebuild detection.

## TDD Approach
Write tests for change detection scenarios first, then implement smart rebuilding.

## Deliverables

### 1. Template Change Detection
- [ ] Compare current files hash with template metadata
- [ ] Detect when SQL files have changed
- [ ] Handle new/deleted SQL files
- [ ] Cache template metadata for fast checks

**Test**: Change detection correctly identifies when rebuild is needed

### 2. Smart Rebuilding
- [ ] Rebuild template only when files change
- [ ] Skip rebuild when no changes detected
- [ ] Handle rebuild failures gracefully
- [ ] Update template metadata after rebuild

**Test**: Templates rebuild when needed, skip when unchanged

### 3. Integration with Existing Commands
- [ ] `dbfast seed` checks for changes before cloning
- [ ] `dbfast init` creates initial template
- [ ] `dbfast status` shows template state
- [ ] Optional `--force-rebuild` flag

**Test**: Commands integrate seamlessly with change detection

## Success Criteria

1. **Smart Detection**: Only rebuilds when files actually change
2. **Fast Checks**: Change detection completes in <50ms
3. **Reliability**: Never uses stale templates, always detects changes
4. **User Experience**: Clear feedback on what's happening

## Key Files to Modify

```
src/
├── change_detector.rs  # New: Change detection logic
├── template.rs         # Add change detection integration
├── commands/
│   ├── seed.rs         # Add change detection to seed
│   ├── init.rs         # Create initial template
│   └── status.rs       # Show template status
└── scanner.rs          # Enhance with metadata support

tests/
├── change_detection_tests.rs  # Change detection scenarios
└── fixtures/
    └── change_scenarios/       # File modification test cases
```

## Example Test

```rust
#[tokio::test]
async fn test_change_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("db");
    create_test_sql_files(&db_path).await;

    let pool = test_db_pool().await;
    let template_manager = TemplateManager::new(pool.clone());

    // Initial template creation
    let sql_files = get_sql_files(&db_path).await;
    template_manager.create_template("test_template", &sql_files).await.unwrap();

    // Check: no changes detected initially
    let change_detector = ChangeDetector::new(db_path.clone());
    let needs_rebuild = change_detector.template_needs_rebuild("test_template").await.unwrap();
    assert!(!needs_rebuild);

    // Modify a file
    let user_file = db_path.join("0_schema/01_write_side/010_blog/0101_user/010111_tb_user.sql");
    let mut content = fs::read_to_string(&user_file).unwrap();
    content.push_str("\n-- Modified file");
    fs::write(&user_file, content).unwrap();

    // Check: changes detected
    let needs_rebuild = change_detector.template_needs_rebuild("test_template").await.unwrap();
    assert!(needs_rebuild);

    // Rebuild template
    let new_sql_files = get_sql_files(&db_path).await;
    template_manager.rebuild_template("test_template", &new_sql_files).await.unwrap();

    // Check: no changes after rebuild
    let needs_rebuild = change_detector.template_needs_rebuild("test_template").await.unwrap();
    assert!(!needs_rebuild);
}

#[tokio::test]
async fn test_seed_with_change_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    setup_test_db_directory(&temp_dir).await;

    // First seed should create template
    let result = handle_seed_with_change_detection("test_db_1", false, &temp_dir.path()).await;
    assert!(result.is_ok());
    assert!(template_exists("test_template").await);

    // Second seed should use existing template (no rebuild)
    let start = Instant::now();
    let result = handle_seed_with_change_detection("test_db_2", false, &temp_dir.path()).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_millis() < 100); // Fast because no rebuild needed

    // Modify files and seed again - should trigger rebuild
    modify_test_sql_file(&temp_dir).await;
    let start = Instant::now();
    let result = handle_seed_with_change_detection("test_db_3", false, &temp_dir.path()).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_millis() > 200); // Slower because rebuild happened
}
```

**Duration**: 2-3 days
**Next**: Phase 3A - Environment Configuration
