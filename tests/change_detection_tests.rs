use dbfast::change_detector::ChangeDetector;
use dbfast::scanner::FileScanner;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

/// Test helper to create a temporary directory with test SQL files
fn create_test_sql_files(base_dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
    let schema_dir = base_dir.join("0_schema/01_write_side/010_blog/0101_user");
    fs::create_dir_all(&schema_dir)?;
    
    let user_table_file = schema_dir.join("010111_tb_user.sql");
    fs::write(&user_table_file, "CREATE TABLE tb_user (id SERIAL PRIMARY KEY, name VARCHAR(255));")?;
    
    let user_index_file = schema_dir.join("010112_idx_user.sql");
    fs::write(&user_index_file, "CREATE INDEX idx_user_name ON tb_user(name);")?;
    
    Ok(vec![user_table_file, user_index_file])
}

/// Test helper to modify a test SQL file
fn modify_test_sql_file(file_path: &std::path::Path) -> std::io::Result<()> {
    let mut content = fs::read_to_string(file_path)?;
    content.push_str("\n-- Modified for testing");
    fs::write(file_path, content)
}

#[tokio::test]
async fn test_change_detector_creation() {
    let temp_dir = TempDir::new().unwrap();
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Change detector should be created successfully
    assert_eq!(change_detector.root_path(), temp_dir.path());
}

#[tokio::test]
async fn test_change_detection_with_no_template_metadata() {
    let temp_dir = TempDir::new().unwrap();
    create_test_sql_files(temp_dir.path()).unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // When no template metadata exists, should indicate rebuild needed
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild when no template metadata exists");
}

#[tokio::test]
async fn test_change_detection_with_matching_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let _sql_files = create_test_sql_files(temp_dir.path()).unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Scan files to get current hashes
    let scanner = FileScanner::new(temp_dir.path());
    let scanned_files = scanner.scan().unwrap();
    
    // Store metadata as if template was just created
    change_detector
        .store_template_metadata("test_template", &scanned_files)
        .await
        .unwrap();
    
    // Now check - should not need rebuild since nothing changed
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template")
        .await
        .unwrap();
    
    assert!(!needs_rebuild, "Should not need rebuild when files haven't changed");
}

#[tokio::test]
async fn test_change_detection_with_modified_file() {
    let temp_dir = TempDir::new().unwrap();
    let sql_files = create_test_sql_files(temp_dir.path()).unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Scan files to get initial hashes
    let scanner = FileScanner::new(temp_dir.path());
    let initial_scanned_files = scanner.scan().unwrap();
    
    // Store metadata as if template was created
    change_detector
        .store_template_metadata("test_template", &initial_scanned_files)
        .await
        .unwrap();
    
    // Modify one of the files
    modify_test_sql_file(&sql_files[0]).unwrap();
    
    // Now check - should need rebuild since file changed
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild when file has been modified");
}

#[tokio::test]
async fn test_change_detection_with_new_file() {
    let temp_dir = TempDir::new().unwrap();
    create_test_sql_files(temp_dir.path()).unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Scan files to get initial hashes
    let scanner = FileScanner::new(temp_dir.path());
    let initial_scanned_files = scanner.scan().unwrap();
    
    // Store metadata as if template was created
    change_detector
        .store_template_metadata("test_template", &initial_scanned_files)
        .await
        .unwrap();
    
    // Add a new SQL file
    let schema_dir = temp_dir.path().join("0_schema/01_write_side/010_blog/0101_user");
    let new_file = schema_dir.join("010113_new_constraint.sql");
    fs::write(&new_file, "ALTER TABLE tb_user ADD CONSTRAINT ck_name_not_empty CHECK (name != '');").unwrap();
    
    // Now check - should need rebuild since new file was added
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild when new file has been added");
}

#[tokio::test]
async fn test_change_detection_with_deleted_file() {
    let temp_dir = TempDir::new().unwrap();
    let sql_files = create_test_sql_files(temp_dir.path()).unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Scan files to get initial hashes
    let scanner = FileScanner::new(temp_dir.path());
    let initial_scanned_files = scanner.scan().unwrap();
    
    // Store metadata as if template was created
    change_detector
        .store_template_metadata("test_template", &initial_scanned_files)
        .await
        .unwrap();
    
    // Delete one of the files
    fs::remove_file(&sql_files[1]).unwrap();
    
    // Now check - should need rebuild since file was deleted
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild when file has been deleted");
}

#[tokio::test]
async fn test_template_metadata_storage_and_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    create_test_sql_files(temp_dir.path()).unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Scan files to get hashes
    let scanner = FileScanner::new(temp_dir.path());
    let scanned_files = scanner.scan().unwrap();
    
    // Store metadata
    change_detector
        .store_template_metadata("test_template", &scanned_files)
        .await
        .unwrap();
    
    // Retrieve metadata
    let retrieved_metadata = change_detector
        .get_template_metadata("test_template")
        .await
        .unwrap();
    
    assert!(retrieved_metadata.is_some(), "Should be able to retrieve stored metadata");
    
    let metadata = retrieved_metadata.unwrap();
    assert_eq!(metadata.len(), scanned_files.len(), "Retrieved metadata should match stored data");
    
    // Check that paths and hashes match
    for (stored, retrieved) in scanned_files.iter().zip(metadata.iter()) {
        assert_eq!(stored.path, retrieved.path, "File paths should match");
        assert_eq!(stored.hash, retrieved.hash, "File hashes should match");
    }
}

#[tokio::test]
async fn test_performance_change_detection_under_50ms() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create more files to make the test more realistic
    for i in 0..50 {
        let dir_path = temp_dir.path().join(format!("dir_{}", i));
        fs::create_dir_all(&dir_path).unwrap();
        
        for j in 0..5 {
            let file_path = dir_path.join(format!("file_{}.sql", j));
            fs::write(&file_path, format!("-- Test file {} in dir {}", j, i)).unwrap();
        }
    }
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Store metadata first
    let scanner = FileScanner::new(temp_dir.path());
    let scanned_files = scanner.scan().unwrap();
    change_detector
        .store_template_metadata("test_template", &scanned_files)
        .await
        .unwrap();
    
    // Now test performance of change detection
    let start = std::time::Instant::now();
    let _needs_rebuild = change_detector
        .template_needs_rebuild("test_template")
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() < 50,
        "Change detection should complete in <50ms, but took {}ms",
        duration.as_millis()
    );
}