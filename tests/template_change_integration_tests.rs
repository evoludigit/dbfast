mod common;

use common::TestDatabase;
use dbfast::change_detector::ChangeDetector;
use dbfast::scanner::FileScanner;
use dbfast::template::TemplateManager;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper to create a temporary directory with test SQL files
fn create_test_sql_files(base_dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
    let schema_dir = base_dir.join("0_schema/01_write_side/010_blog/0101_user");
    fs::create_dir_all(&schema_dir)?;

    let user_table_file = schema_dir.join("010111_tb_user.sql");
    fs::write(
        &user_table_file,
        "CREATE TABLE IF NOT EXISTS tb_user (id SERIAL PRIMARY KEY, name VARCHAR(255));",
    )?;

    let user_index_file = schema_dir.join("010112_idx_user.sql");
    fs::write(
        &user_index_file,
        "CREATE INDEX idx_user_name ON tb_user(name);",
    )?;

    Ok(vec![user_table_file, user_index_file])
}

#[tokio::test]
async fn test_template_manager_with_change_detector_creation() {
    let temp_dir = TempDir::new().unwrap();
    let test_db = TestDatabase::create_unique("template_mgr").await.unwrap();

    // This test will fail until we integrate ChangeDetector with TemplateManager
    // We should be able to create a TemplateManager that includes change detection
    let template_manager = TemplateManager::new_with_change_detection(
        test_db.admin_pool.clone(),
        test_db.admin_config(),
        temp_dir.path().to_path_buf(),
    );

    // Should have change detection capabilities
    assert!(template_manager.has_change_detection());
}

#[tokio::test]
#[ignore = "Connection routing issue - connects to postgres instead of template DB"]
async fn test_create_template_with_change_tracking() {
    let temp_dir = TempDir::new().unwrap();
    create_test_sql_files(temp_dir.path()).unwrap();

    let test_db = TestDatabase::create_unique("change_tracking")
        .await
        .unwrap();
    let template_name = format!("tmpl_{}", test_db.name);

    let template_manager = TemplateManager::new_with_change_detection(
        test_db.admin_pool.clone(),
        test_db.admin_config(),
        temp_dir.path().to_path_buf(),
    );

    // Scan for SQL files
    let scanner = FileScanner::new(temp_dir.path());
    let sql_files: Vec<PathBuf> = scanner
        .scan()
        .unwrap()
        .into_iter()
        .map(|f| f.path)
        .collect();

    // Create template - should also store change detection metadata
    let result = template_manager
        .create_template_with_change_tracking(&template_name, &sql_files)
        .await;

    assert!(
        result.is_ok(),
        "Template creation with change tracking should succeed: {:?}",
        result.err()
    );

    // Check that change detection metadata was created
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    let metadata = change_detector
        .get_template_metadata(&template_name)
        .await
        .unwrap();

    assert!(
        metadata.is_some(),
        "Change detection metadata should be stored after template creation"
    );
}

#[tokio::test]
#[ignore = "Connection routing issue - connects to postgres instead of template DB"]
async fn test_template_needs_rebuild_integration() {
    let temp_dir = TempDir::new().unwrap();
    let sql_files = create_test_sql_files(temp_dir.path()).unwrap();

    let test_db = TestDatabase::create_unique("needs_rebuild").await.unwrap();
    let template_name = format!("tmpl_{}", test_db.name);

    let template_manager = TemplateManager::new_with_change_detection(
        test_db.admin_pool.clone(),
        test_db.admin_config(),
        temp_dir.path().to_path_buf(),
    );

    // Initially, template should need rebuilding (doesn't exist)
    let needs_rebuild = template_manager
        .template_needs_rebuild(&template_name)
        .await
        .unwrap();

    assert!(needs_rebuild, "Template should need rebuilding initially");

    // Create template
    template_manager
        .create_template_with_change_tracking(&template_name, &sql_files)
        .await
        .unwrap();

    // Now should not need rebuilding
    let needs_rebuild = template_manager
        .template_needs_rebuild(&template_name)
        .await
        .unwrap();

    assert!(
        !needs_rebuild,
        "Template should not need rebuilding after creation"
    );

    // Modify a file
    let mut content = fs::read_to_string(&sql_files[0]).unwrap();
    content.push_str("\n-- Modified for testing");
    fs::write(&sql_files[0], content).unwrap();

    // Now should need rebuilding
    let needs_rebuild = template_manager
        .template_needs_rebuild(&template_name)
        .await
        .unwrap();

    assert!(
        needs_rebuild,
        "Template should need rebuilding after file modification"
    );
}

#[tokio::test]
#[ignore = "Connection routing issue - connects to postgres instead of template DB"]
async fn test_smart_template_creation() {
    let temp_dir = TempDir::new().unwrap();
    let sql_files = create_test_sql_files(temp_dir.path()).unwrap();

    let test_db = TestDatabase::create_unique("smart_template").await.unwrap();
    let template_name = format!("tmpl_{}", test_db.name);

    let template_manager = TemplateManager::new_with_change_detection(
        test_db.admin_pool.clone(),
        test_db.admin_config(),
        temp_dir.path().to_path_buf(),
    );

    // First call should create template
    let result = template_manager
        .smart_create_template(&template_name, &sql_files)
        .await;

    assert!(result.is_ok());
    assert!(
        result.unwrap(),
        "Should return true when template was actually created"
    );

    // Second call should skip creation (no changes)
    let result = template_manager
        .smart_create_template(&template_name, &sql_files)
        .await;

    assert!(result.is_ok());
    assert!(
        !result.unwrap(),
        "Should return false when template creation was skipped"
    );

    // Modify file and try again - should recreate
    let mut content = fs::read_to_string(&sql_files[0]).unwrap();
    content.push_str("\n-- Modified");
    fs::write(&sql_files[0], content).unwrap();

    let result = template_manager
        .smart_create_template(&template_name, &sql_files)
        .await;

    assert!(result.is_ok());
    assert!(
        result.unwrap(),
        "Should return true when template was rebuilt due to changes"
    );
}
