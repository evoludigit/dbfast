use dbfast::change_detector::ChangeDetector;
use dbfast::scanner::FileScanner;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;
use std::time::Instant;

/// Test helper to create comprehensive test SQL files
fn create_comprehensive_sql_files(base_dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
    // Create a realistic directory structure
    let schema_dir = base_dir.join("0_schema/01_write_side/010_blog/0101_user");
    fs::create_dir_all(&schema_dir)?;
    
    let posts_dir = base_dir.join("0_schema/01_write_side/010_blog/0102_post");
    fs::create_dir_all(&posts_dir)?;
    
    let seed_dir = base_dir.join("1_seed_common");
    fs::create_dir_all(&seed_dir)?;
    
    // Create multiple SQL files
    let user_table_file = schema_dir.join("010111_tb_user.sql");
    fs::write(&user_table_file, 
        "CREATE TABLE tb_user (\n    id SERIAL PRIMARY KEY,\n    name VARCHAR(255) NOT NULL,\n    email VARCHAR(255) UNIQUE\n);")?;
    
    let user_index_file = schema_dir.join("010112_idx_user.sql");
    fs::write(&user_index_file, "CREATE INDEX idx_user_name ON tb_user(name);\nCREATE INDEX idx_user_email ON tb_user(email);")?;
    
    let posts_table_file = posts_dir.join("010211_tb_post.sql");
    fs::write(&posts_table_file,
        "CREATE TABLE tb_post (\n    id SERIAL PRIMARY KEY,\n    user_id INTEGER REFERENCES tb_user(id),\n    title VARCHAR(255) NOT NULL,\n    content TEXT\n);")?;
    
    let seed_file = seed_dir.join("001_users.sql");
    fs::write(&seed_file, 
        "INSERT INTO tb_user (name, email) VALUES\n    ('John Doe', 'john@example.com'),\n    ('Jane Smith', 'jane@example.com');")?;
    
    Ok(vec![user_table_file, user_index_file, posts_table_file, seed_file])
}

#[tokio::test]
async fn test_full_workflow_file_scanning_to_change_detection() {
    let temp_dir = TempDir::new().unwrap();
    let sql_files = create_comprehensive_sql_files(temp_dir.path()).unwrap();
    
    // Step 1: Initial file scanning
    let scanner = FileScanner::new(temp_dir.path());
    let initial_scan = scanner.scan().unwrap();
    
    assert_eq!(initial_scan.len(), 4, "Should find all 4 SQL files");
    
    // Verify files are sorted by path
    for i in 1..initial_scan.len() {
        assert!(initial_scan[i-1].path <= initial_scan[i].path, "Files should be sorted by path");
    }
    
    // Step 2: Change detection setup
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Store initial metadata
    change_detector
        .store_template_metadata("test_template_workflow", &initial_scan)
        .await
        .unwrap();
    
    // Step 3: Verify no changes initially
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template_workflow")
        .await
        .unwrap();
    
    assert!(!needs_rebuild, "Should not need rebuild initially");
    
    // Step 4: Modify one file and verify change detection
    let user_table_file = &sql_files[0];
    let mut content = fs::read_to_string(user_table_file).unwrap();
    content.push_str("\n-- Added comment for testing");
    fs::write(user_table_file, content).unwrap();
    
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_template_workflow")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild after file modification");
    
    // Step 5: Performance verification - change detection should be fast
    let start = Instant::now();
    let _needs_rebuild = change_detector
        .template_needs_rebuild("test_template_workflow")
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 50, "Change detection should be <50ms, was {}ms", duration.as_millis());
}

#[tokio::test]
async fn test_full_workflow_with_file_addition_and_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let _sql_files = create_comprehensive_sql_files(temp_dir.path()).unwrap();
    
    // Initial setup
    let scanner = FileScanner::new(temp_dir.path());
    let initial_scan = scanner.scan().unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    change_detector
        .store_template_metadata("test_workflow_files", &initial_scan)
        .await
        .unwrap();
    
    // Add a new file
    let new_file_dir = temp_dir.path().join("0_schema/01_write_side/010_blog/0103_comment");
    fs::create_dir_all(&new_file_dir).unwrap();
    let new_file = new_file_dir.join("010311_tb_comment.sql");
    fs::write(&new_file, 
        "CREATE TABLE tb_comment (\n    id SERIAL PRIMARY KEY,\n    post_id INTEGER REFERENCES tb_post(id),\n    content TEXT\n);"
    ).unwrap();
    
    // Should detect new file
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_workflow_files")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild after adding new file");
    
    // Update metadata with new file
    let updated_scan = scanner.scan().unwrap();
    assert_eq!(updated_scan.len(), initial_scan.len() + 1, "Should find one more file");
    
    change_detector
        .store_template_metadata("test_workflow_files", &updated_scan)
        .await
        .unwrap();
    
    // Now should not need rebuild
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_workflow_files")
        .await
        .unwrap();
    
    assert!(!needs_rebuild, "Should not need rebuild after metadata update");
    
    // Delete a file
    fs::remove_file(&new_file).unwrap();
    
    // Should detect deleted file
    let needs_rebuild = change_detector
        .template_needs_rebuild("test_workflow_files")
        .await
        .unwrap();
    
    assert!(needs_rebuild, "Should need rebuild after deleting file");
}

#[tokio::test]
async fn test_workflow_performance_with_many_files() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create many files to test performance
    for dir_num in 0..10 {
        for file_num in 0..10 {
            let dir_path = temp_dir.path().join(format!("dir_{:02}", dir_num));
            fs::create_dir_all(&dir_path).unwrap();
            
            let file_path = dir_path.join(format!("file_{:02}.sql", file_num));
            fs::write(&file_path, format!(
                "-- File {} in directory {}\nCREATE TABLE table_{}_{} (id SERIAL PRIMARY KEY);",
                file_num, dir_num, dir_num, file_num
            )).unwrap();
        }
    }
    
    // Performance test: scanning should be fast
    let scanner = FileScanner::new(temp_dir.path());
    
    let scan_start = Instant::now();
    let scan_result = scanner.scan().unwrap();
    let scan_duration = scan_start.elapsed();
    
    assert_eq!(scan_result.len(), 100, "Should find all 100 files");
    assert!(scan_duration.as_millis() < 100, "File scanning should be <100ms, was {}ms", scan_duration.as_millis());
    
    // Performance test: change detection should be fast
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    change_detector
        .store_template_metadata("perf_test_template", &scan_result)
        .await
        .unwrap();
    
    let change_start = Instant::now();
    let needs_rebuild = change_detector
        .template_needs_rebuild("perf_test_template")
        .await
        .unwrap();
    let change_duration = change_start.elapsed();
    
    assert!(!needs_rebuild, "Should not need rebuild");
    assert!(change_duration.as_millis() < 50, "Change detection should be <50ms, was {}ms", change_duration.as_millis());
    
    // Test change detection after file modification
    let first_file = temp_dir.path().join("dir_00/file_00.sql");
    let mut content = fs::read_to_string(&first_file).unwrap();
    content.push_str("\n-- Modified");
    fs::write(&first_file, content).unwrap();
    
    let change_start = Instant::now();
    let needs_rebuild = change_detector
        .template_needs_rebuild("perf_test_template")
        .await
        .unwrap();
    let change_duration = change_start.elapsed();
    
    assert!(needs_rebuild, "Should need rebuild after modification");
    assert!(change_duration.as_millis() < 50, "Change detection should be <50ms even with changes, was {}ms", change_duration.as_millis());
}

#[tokio::test]
async fn test_workflow_metadata_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let _sql_files = create_comprehensive_sql_files(temp_dir.path()).unwrap();
    
    let scanner = FileScanner::new(temp_dir.path());
    let scan_result = scanner.scan().unwrap();
    
    let change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Store metadata
    change_detector
        .store_template_metadata("persistence_test", &scan_result)
        .await
        .unwrap();
    
    // Create a new change detector (simulating restart)
    let new_change_detector = ChangeDetector::new(temp_dir.path().to_path_buf());
    
    // Should be able to retrieve stored metadata
    let retrieved_metadata = new_change_detector
        .get_template_metadata("persistence_test")
        .await
        .unwrap();
    
    assert!(retrieved_metadata.is_some(), "Should be able to retrieve stored metadata");
    
    let metadata = retrieved_metadata.unwrap();
    assert_eq!(metadata.len(), scan_result.len(), "Retrieved metadata should match original");
    
    // Verify metadata accuracy
    for (original, retrieved) in scan_result.iter().zip(metadata.iter()) {
        assert_eq!(original.path, retrieved.path, "File paths should match");
        assert_eq!(original.hash, retrieved.hash, "File hashes should match");
    }
    
    // Should not need rebuild with matching metadata
    let needs_rebuild = new_change_detector
        .template_needs_rebuild("persistence_test")
        .await
        .unwrap();
    
    assert!(!needs_rebuild, "Should not need rebuild with matching metadata");
}