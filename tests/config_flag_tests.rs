/// Tests for the allow_multi_statement configuration flag
use dbfast::database::DatabasePool;

/// Test that the configuration flag works correctly
#[tokio::test]
async fn test_allow_multi_statement_flag() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION inline_function() RETURNS TEXT LANGUAGE plpgsql AS $$ BEGIN RETURN 'test'; END; $$;
CREATE TABLE after_function (id INTEGER);
"#;

    // Test with allow_multi_statement = true (advanced parsing)
    let advanced_statements = DatabasePool::parse_sql_statements_with_config(sql_content, true);
    assert_eq!(
        advanced_statements.len(),
        2,
        "Advanced parsing should handle inline functions"
    );
    assert!(advanced_statements[0].contains("CREATE OR REPLACE FUNCTION"));
    assert!(advanced_statements[1].contains("CREATE TABLE"));

    // Test with allow_multi_statement = false (simple parsing)
    let simple_statements = DatabasePool::parse_sql_statements_with_config(sql_content, false);

    // Simple parsing will split on semicolons and might not handle this correctly
    // but it should at least attempt to split the statements
    assert!(
        !simple_statements.is_empty(),
        "Simple parsing should produce some statements"
    );

    // The exact behavior of simple parsing depends on the implementation,
    // but it should be different from advanced parsing for this case
    println!("Advanced parsing: {} statements", advanced_statements.len());
    println!("Simple parsing: {} statements", simple_statements.len());
}

/// Test simple parsing behavior
#[tokio::test]
async fn test_simple_parsing_basic_sql() {
    let sql_content = r#"
CREATE TABLE users (id INTEGER);
INSERT INTO users VALUES (1);
INSERT INTO users VALUES (2);
"#;

    let statements = DatabasePool::parse_sql_statements_simple(sql_content);

    assert_eq!(
        statements.len(),
        3,
        "Simple parsing should split on semicolons"
    );
    assert!(statements[0].contains("CREATE TABLE users"));
    assert!(statements[1].contains("INSERT INTO users VALUES (1)"));
    assert!(statements[2].contains("INSERT INTO users VALUES (2)"));
}

/// Test that default parsing method maintains backward compatibility
#[tokio::test]
async fn test_backward_compatibility() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION test_func() RETURNS TEXT AS $$ BEGIN RETURN 'test'; END; $$;
CREATE TABLE test_table (id INTEGER);
"#;

    // Default method should use advanced parsing for backward compatibility
    let default_statements = DatabasePool::parse_sql_statements(sql_content);
    let advanced_statements = DatabasePool::parse_sql_statements_advanced(sql_content);

    assert_eq!(
        default_statements.len(),
        advanced_statements.len(),
        "Default parsing should match advanced parsing"
    );

    for (default, advanced) in default_statements.iter().zip(advanced_statements.iter()) {
        assert_eq!(default, advanced, "Statements should be identical");
    }
}

/// Test configuration parsing for the new flag
#[tokio::test]
async fn test_config_file_parsing() {
    use dbfast::config::Config;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    // Test config with explicit allow_multi_statement = false
    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "test_user"
password_env = "TEST_PASSWORD"
template_name = "test_template"
allow_multi_statement = false

[repository]
path = "./sql"
type = "structured"
"#;

    fs::write(&config_path, config_content).unwrap();
    let config = Config::from_file(&config_path).unwrap();

    assert!(
        !config.database.allow_multi_statement,
        "Config should parse allow_multi_statement = false"
    );

    // Test config without the flag (should default to true)
    let default_config_content = r#"
[database]
host = "localhost"
port = 5432
user = "test_user"
password_env = "TEST_PASSWORD"
template_name = "test_template"

[repository]
path = "./sql"
type = "structured"
"#;

    fs::write(&config_path, default_config_content).unwrap();
    let default_config = Config::from_file(&config_path).unwrap();

    assert!(
        default_config.database.allow_multi_statement,
        "Config should default allow_multi_statement to true"
    );
}

/// Test that the new Config::new method includes the flag
#[tokio::test]
async fn test_config_new_includes_flag() {
    let config = dbfast::config::Config::new("./sql", "test_template");

    assert!(
        config.database.allow_multi_statement,
        "Config::new should set allow_multi_statement to true by default"
    );
}
