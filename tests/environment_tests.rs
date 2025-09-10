use std::path::PathBuf;
use dbfast::environment::EnvironmentConfig;

#[test]
fn test_environment_filtering_basic() {
    let config = EnvironmentConfig {
        name: "local".to_string(),
        include_directories: Some(vec!["0_schema".to_string(), "1_seed_common".to_string()]),
        exclude_directories: Some(vec!["6_migration".to_string()]),
        exclude_files: Some(vec!["**/prod_*.sql".to_string()]),
        ..Default::default()
    };
    
    let all_files = vec![
        PathBuf::from("tests/fixtures/sql/0_schema/tables.sql"),
        PathBuf::from("tests/fixtures/sql/1_seed_common/users.sql"),
        PathBuf::from("tests/fixtures/sql/6_migration/001_add_column.sql"),
        PathBuf::from("tests/fixtures/sql/1_seed_common/prod_data.sql"),
    ];
    
    let filtered = config.filter_files(&all_files).unwrap();
    
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&PathBuf::from("tests/fixtures/sql/0_schema/tables.sql")));
    assert!(filtered.contains(&PathBuf::from("tests/fixtures/sql/1_seed_common/users.sql")));
    assert!(!filtered.contains(&PathBuf::from("tests/fixtures/sql/6_migration/001_add_column.sql")));
    assert!(!filtered.contains(&PathBuf::from("tests/fixtures/sql/1_seed_common/prod_data.sql")));
}

#[test]
fn test_production_safety() {
    let prod_config = EnvironmentConfig {
        name: "production".to_string(),
        include_directories: Some(vec!["0_schema".to_string(), "6_migration".to_string()]),
        exclude_directories: Some(vec!["1_seed_common".to_string(), "2_seed_backend".to_string()]),
        exclude_files: Some(vec!["**/test_*.sql".to_string(), "**/dev_*.sql".to_string()]),
        ..Default::default()
    };
    
    let test_files = vec![
        PathBuf::from("tests/fixtures/sql/0_schema/tables.sql"),        // Should include
        PathBuf::from("tests/fixtures/sql/6_migration/001_add_column.sql"),   // Should include  
        PathBuf::from("tests/fixtures/sql/1_seed_common/users.sql"),    // Should exclude (directory)
        PathBuf::from("tests/fixtures/sql/0_schema/test_data.sql"),     // Should exclude (file pattern)
    ];
    
    let filtered = prod_config.filter_files(&test_files).unwrap();
    
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&PathBuf::from("tests/fixtures/sql/0_schema/tables.sql")));
    assert!(filtered.contains(&PathBuf::from("tests/fixtures/sql/6_migration/001_add_column.sql")));
}