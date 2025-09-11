use dbfast::sql_executor::SqlExecutor;
use std::path::PathBuf;

#[test]
fn test_sql_file_reading() {
    // Test reading SQL files from the db directory
    let db_path = PathBuf::from("db");
    
    // This test should pass when we implement SQL file reading
    let result = SqlExecutor::read_sql_files(&db_path);
    
    // We expect this to work and return SQL statements
    assert!(result.is_ok(), "SQL file reading should work");
    
    let statements = result.unwrap();
    assert!(!statements.is_empty(), "Should find SQL files in db directory");
    
    // Should find the user table creation statement
    let has_user_table = statements.iter().any(|stmt| stmt.contains("CREATE TABLE blog.tb_user"));
    assert!(has_user_table, "Should find user table creation statement");
}

#[test]
fn test_sql_statement_parsing() {
    // Test parsing SQL content into individual statements
    let sql_content = "CREATE SCHEMA blog; CREATE TABLE blog.users (id INT);";
    
    let statements = SqlExecutor::parse_sql_statements(sql_content);
    
    // Should parse into 2 statements
    assert_eq!(statements.len(), 2, "Should parse 2 SQL statements");
    assert!(statements[0].contains("CREATE SCHEMA blog"));
    assert!(statements[1].contains("CREATE TABLE blog.users"));
}

#[test]
fn test_sql_file_ordering() {
    // This will test that SQL files are read in the correct order
    let db_path = PathBuf::from("db");
    
    let result = SqlExecutor::read_sql_files(&db_path);
    
    // Should succeed and files should be in order
    assert!(result.is_ok(), "Should be able to read SQL files");
    
    let statements = result.unwrap();
    
    // Schema should come before tables
    let schema_pos = statements.iter().position(|stmt| stmt.contains("CREATE SCHEMA"));
    let table_pos = statements.iter().position(|stmt| stmt.contains("CREATE TABLE"));
    
    if let (Some(schema), Some(table)) = (schema_pos, table_pos) {
        assert!(schema < table, "Schema creation should come before table creation");
    }
}