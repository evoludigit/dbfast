use dbfast::FileScanner;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_scanner_finds_sql_files() {
    // Create a temporary directory with SQL files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create some SQL files
    fs::create_dir_all(temp_path.join("0_schema")).unwrap();
    fs::create_dir_all(temp_path.join("1_seed")).unwrap();
    
    fs::write(temp_path.join("0_schema/01_tables.sql"), "CREATE TABLE users (id SERIAL);").unwrap();
    fs::write(temp_path.join("1_seed/01_data.sql"), "INSERT INTO users (name) VALUES ('test');").unwrap();
    fs::write(temp_path.join("README.md"), "# Documentation").unwrap(); // Non-SQL file
    
    // Test scanner
    let scanner = FileScanner::new(temp_path);
    let files = scanner.scan().unwrap();
    
    // Should find 2 SQL files
    assert_eq!(files.len(), 2);
    
    // Check that all found files are SQL files
    for file in &files {
        assert!(file.path.to_string_lossy().ends_with(".sql"));
        assert!(file.hash.len() > 0);
    }
}

#[test]
fn test_file_scanner_detects_changes() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let sql_file = temp_path.join("test.sql");
    
    // Create a file
    fs::write(&sql_file, "SELECT 1;").unwrap();
    
    let scanner = FileScanner::new(temp_path);
    let files1 = scanner.scan().unwrap();
    assert_eq!(files1.len(), 1);
    let original_hash = &files1[0].hash;
    
    // Modify the file
    fs::write(&sql_file, "SELECT 2;").unwrap();
    
    let files2 = scanner.scan().unwrap();
    assert_eq!(files2.len(), 1);
    let new_hash = &files2[0].hash;
    
    // Hash should be different
    assert_ne!(original_hash, new_hash);
}