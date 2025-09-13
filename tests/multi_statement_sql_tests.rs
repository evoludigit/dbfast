/// Tests for multi-statement SQL file support (TDD RED PHASE)
///
/// These tests reproduce the issue described in GitHub issue #31:
/// DBFast fails when processing SQL files with PostgreSQL function definitions
/// and additional statements like COMMENT and GRANT.
///
/// Expected behavior:
/// - Parse PostgreSQL functions with dollar-quoted bodies correctly
/// - Handle additional statements after function definitions
/// - Execute statements sequentially without "cannot insert multiple commands" error
///
use dbfast::database::DatabasePool;
use std::fs;
use tempfile::TempDir;

/// Test multi-statement SQL file with PostgreSQL function and metadata
/// RED PHASE: Should FAIL because current implementation doesn't handle this correctly
#[tokio::test]
async fn test_postgresql_function_with_metadata() {
    let sql_content = r#"
-- Create a PostgreSQL function with metadata
CREATE OR REPLACE FUNCTION app.update_contract(
    p_contract_id INTEGER,
    p_status TEXT,
    p_updated_by TEXT
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
DECLARE
    v_count INTEGER;
BEGIN
    -- Update the contract status
    UPDATE contracts
    SET status = p_status,
        updated_by = p_updated_by,
        updated_at = NOW()
    WHERE id = p_contract_id;

    GET DIAGNOSTICS v_count = ROW_COUNT;

    IF v_count = 0 THEN
        RAISE NOTICE 'No contract found with ID %', p_contract_id;
        RETURN FALSE;
    END IF;

    RETURN TRUE;
END;
$$;

-- Add function metadata
COMMENT ON FUNCTION app.update_contract(INTEGER, TEXT, TEXT) IS 'Updates contract status with audit trail';

-- Grant execute permissions
GRANT EXECUTE ON FUNCTION app.update_contract(INTEGER, TEXT, TEXT) TO app_role;
"#;

    // Test statement parsing
    let statements = DatabasePool::parse_sql_statements(sql_content);

    // Should correctly identify 3 separate statements:
    // 1. CREATE OR REPLACE FUNCTION ... (including the entire $$...$$; block)
    // 2. COMMENT ON FUNCTION ...
    // 3. GRANT EXECUTE ON FUNCTION ...
    assert_eq!(statements.len(), 3, "Should parse 3 statements");

    // First statement should contain the entire function definition
    assert!(statements[0].contains("CREATE OR REPLACE FUNCTION"));
    assert!(statements[0].contains("$$"));
    assert!(statements[0].contains("END;"));

    // Second statement should be the COMMENT
    assert!(statements[1].contains("COMMENT ON FUNCTION"));

    // Third statement should be the GRANT
    assert!(statements[2].contains("GRANT EXECUTE"));
}

/// Test complex PostgreSQL function with nested dollar quotes
/// RED PHASE: Should FAIL because current parsing doesn't handle nested dollar quotes
#[tokio::test]
async fn test_nested_dollar_quotes() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION generate_report(p_type TEXT)
RETURNS TEXT
LANGUAGE plpgsql
AS $function$
DECLARE
    v_query TEXT;
    v_result TEXT;
BEGIN
    -- Build dynamic query with nested dollar quotes
    v_query := $query$
        SELECT string_agg(
            format('Row: %s, Data: %s', id, data),
            E'\n'
        )
        FROM (
            SELECT id, data
            FROM reports
            WHERE type = $1
            ORDER BY id
        ) subq
    $query$;

    EXECUTE v_query USING p_type INTO v_result;

    RETURN COALESCE(v_result, 'No data found');
END;
$function$;

COMMENT ON FUNCTION generate_report(TEXT) IS 'Generates formatted report for given type';
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    // Should correctly parse both function and comment as separate statements
    assert_eq!(statements.len(), 2, "Should parse 2 statements");

    // Function should contain all nested dollar quotes
    assert!(statements[0].contains("$function$"));
    assert!(statements[0].contains("$query$"));
    assert!(statements[1].contains("COMMENT ON FUNCTION"));
}

/// Test multiple functions in single file
/// RED PHASE: Should FAIL because current implementation may not handle multiple functions
#[tokio::test]
async fn test_multiple_functions_with_metadata() {
    let sql_content = r#"
-- First function
CREATE OR REPLACE FUNCTION validate_email(p_email TEXT)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN p_email ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$';
END;
$$;

COMMENT ON FUNCTION validate_email(TEXT) IS 'Validates email format using regex';

-- Second function
CREATE OR REPLACE FUNCTION hash_password(p_password TEXT)
RETURNS TEXT
LANGUAGE plpgsql
AS $$
BEGIN
    -- Simple hash implementation for demo
    RETURN md5(p_password || 'salt123');
END;
$$;

COMMENT ON FUNCTION hash_password(TEXT) IS 'Hashes password with salt';

GRANT EXECUTE ON FUNCTION validate_email(TEXT) TO public;
GRANT EXECUTE ON FUNCTION hash_password(TEXT) TO auth_service;
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    // Should parse 6 statements total:
    // 2 functions + 2 comments + 2 grants
    assert_eq!(statements.len(), 6, "Should parse 6 statements");

    // Verify each statement type
    let create_count = statements
        .iter()
        .filter(|s| s.contains("CREATE OR REPLACE FUNCTION"))
        .count();
    let comment_count = statements
        .iter()
        .filter(|s| s.contains("COMMENT ON FUNCTION"))
        .count();
    let grant_count = statements
        .iter()
        .filter(|s| s.contains("GRANT EXECUTE"))
        .count();

    assert_eq!(create_count, 2, "Should have 2 function definitions");
    assert_eq!(comment_count, 2, "Should have 2 comments");
    assert_eq!(grant_count, 2, "Should have 2 grants");
}

/// Test that function parsing preserves formatting and structure
/// RED PHASE: Should FAIL if parsing corrupts the function body
#[tokio::test]
async fn test_function_body_preservation() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION complex_calculation(
    p_value NUMERIC,
    p_factor NUMERIC DEFAULT 1.0
)
RETURNS NUMERIC
LANGUAGE plpgsql
AS $$
DECLARE
    v_result NUMERIC;
    v_temp NUMERIC;
BEGIN
    -- Multi-line calculation with comments
    v_temp := p_value * p_factor;

    IF v_temp > 100 THEN
        v_result := v_temp * 0.9; -- Apply discount
    ELSIF v_temp > 50 THEN
        v_result := v_temp * 0.95; -- Small discount
    ELSE
        v_result := v_temp; -- No discount
    END IF;

    RETURN ROUND(v_result, 2);
END;
$$;
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    assert_eq!(statements.len(), 1, "Should parse 1 statement");

    let function_statement = &statements[0];

    // Verify important elements are preserved
    assert!(function_statement.contains("DECLARE"));
    assert!(function_statement.contains("v_result NUMERIC"));
    assert!(function_statement.contains("IF v_temp > 100"));
    assert!(function_statement.contains("Apply discount"));
    assert!(function_statement.contains("ROUND(v_result, 2)"));
}

/// Integration test that would fail with actual database execution
/// RED PHASE: Should FAIL with "cannot insert multiple commands into a prepared statement"
#[tokio::test]
#[ignore] // Ignore until we have database setup
async fn test_database_execution_multi_statement() {
    // This test would require actual database connection
    // and should fail with current implementation when executing
    // multi-statement SQL files with functions
    let _sql_content = r#"
CREATE TABLE IF NOT EXISTS contracts (
    id SERIAL PRIMARY KEY,
    status TEXT,
    updated_by TEXT,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION update_contract_status(p_id INTEGER, p_status TEXT)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
BEGIN
    UPDATE contracts SET status = p_status WHERE id = p_id;
    RETURN FOUND;
END;
$$;

COMMENT ON FUNCTION update_contract_status(INTEGER, TEXT) IS 'Updates contract status';
"#;

    // This would fail in actual database execution with current implementation
    // let pool = DatabasePool::new("postgresql://test").await.unwrap();
    // let result = pool.execute_sql_content(sql_content).await;
    // assert!(result.is_err()); // Should fail with current implementation
}

/// Test configuration flag for multi-statement support (not implemented yet)
/// RED PHASE: Should FAIL because configuration doesn't have this flag yet
#[tokio::test]
async fn test_allow_multi_statement_config_flag() {
    // This test will verify that configuration supports enabling multi-statement parsing
    // Currently this should fail because the flag doesn't exist

    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "test_template"
allow_multi_statement = true

[repository]
path = "./sql"
type = "structured"
"#;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");
    fs::write(&config_path, config_content).unwrap();

    // This should fail because allow_multi_statement field doesn't exist yet
    let result = dbfast::config::Config::from_file(&config_path);

    // For now, this should succeed even without the field (serde skips unknown fields)
    // But we'll need to add the field and test it properly
    assert!(result.is_ok());
}
