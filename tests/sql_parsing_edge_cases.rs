/// Edge case tests for SQL parsing that demonstrate potential issues
///
/// These tests are designed to reveal edge cases where the current SQL parsing
/// might not work correctly, especially around complex PostgreSQL functions
/// and dollar-quoted strings.
///
use dbfast::database::DatabasePool;

/// Test edge case: Function with complex dollar quote tags
/// This test demonstrates parsing challenges with non-standard dollar quote tags
#[tokio::test]
async fn test_complex_dollar_quote_tags() {
    let sql_content = r#"
-- Function with complex dollar quote tag
CREATE OR REPLACE FUNCTION complex_function()
RETURNS TEXT
LANGUAGE plpgsql
AS $CUSTOM_TAG_123$
DECLARE
    v_sql TEXT := $INNER$
        SELECT format('Generated at %s', now())
    $INNER$;
BEGIN
    EXECUTE v_sql;
    RETURN 'OK';
END;
$CUSTOM_TAG_123$;

COMMENT ON FUNCTION complex_function() IS 'Function with nested dollar quotes';
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    // Should parse into 2 statements
    assert_eq!(statements.len(), 2, "Should parse 2 statements");

    // First statement should contain the function with custom dollar tag
    assert!(statements[0].contains("$CUSTOM_TAG_123$"));
    assert!(statements[0].contains("$INNER$"));

    // Second statement should be the comment
    assert!(statements[1].contains("COMMENT ON FUNCTION"));
}

/// Test edge case: Function with semicolon inside dollar quotes
/// This could confuse statement parsing if not handled correctly
#[tokio::test]
async fn test_semicolon_inside_dollar_quotes() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION test_semicolons()
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    -- This function body contains semicolons that should NOT split statements
    EXECUTE 'INSERT INTO log VALUES (''Action started;'')';
    EXECUTE 'INSERT INTO log VALUES (''Action completed;'')';
    RAISE NOTICE 'Function executed; all done;';
END;
$$;

-- This should be a separate statement
GRANT EXECUTE ON FUNCTION test_semicolons() TO public;
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    // Should parse into 2 statements (function + grant)
    assert_eq!(
        statements.len(),
        2,
        "Should parse 2 statements, not more due to internal semicolons"
    );

    // First statement should contain the entire function including internal semicolons
    assert!(statements[0].contains("CREATE OR REPLACE FUNCTION"));
    assert!(statements[0].contains("Action started;"));
    assert!(statements[0].contains("Action completed;"));
    assert!(statements[0].contains("all done;"));

    // Second statement should be the GRANT
    assert!(statements[1].contains("GRANT EXECUTE"));
}

/// Test edge case: Empty dollar quotes
/// Tests parsing behavior with empty dollar quote blocks
#[tokio::test]
async fn test_empty_dollar_quotes() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION empty_body()
RETURNS VOID
LANGUAGE plpgsql
AS $$
$$;

CREATE TABLE test (id INTEGER);
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    assert_eq!(statements.len(), 2, "Should parse 2 statements");
    assert!(statements[0].contains("CREATE OR REPLACE FUNCTION"));
    assert!(statements[1].contains("CREATE TABLE"));
}

/// Test edge case: Multiple dollar quotes on same line
/// This could confuse parsing logic
#[tokio::test]
async fn test_multiple_dollar_quotes_same_line() {
    let sql_content = r#"
CREATE OR REPLACE FUNCTION inline_function() RETURNS TEXT LANGUAGE plpgsql AS $$ BEGIN RETURN 'test'; END; $$;
CREATE TABLE after_function (id INTEGER);
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    assert_eq!(statements.len(), 2, "Should parse 2 statements");
    assert!(statements[0].contains("CREATE OR REPLACE FUNCTION"));
    assert!(statements[1].contains("CREATE TABLE"));
}

/// Test potential regression: Single statement without dollar quotes
/// This should still work correctly
#[tokio::test]
async fn test_simple_statement_still_works() {
    let sql_content = r#"
CREATE TABLE simple_table (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    assert_eq!(statements.len(), 1, "Should parse 1 statement");
    assert!(statements[0].contains("CREATE TABLE simple_table"));
}

/// Test edge case: Comments mixed with dollar quotes
/// Comments inside dollar quotes should be preserved, comments outside should be ignored
#[tokio::test]
async fn test_comments_with_dollar_quotes() {
    let sql_content = r#"
-- This comment should be ignored
CREATE OR REPLACE FUNCTION comment_test()
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    -- This comment inside dollar quotes should be preserved
    RAISE NOTICE 'Function with comments';
    /* Multi-line comment
       inside dollar quotes
       should also be preserved */
END;
$$;

/* This multi-line comment
   outside dollar quotes
   should be ignored */

CREATE TABLE after_comments (id INTEGER);
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    assert_eq!(statements.len(), 2, "Should parse 2 statements");

    // Function should contain internal comments
    assert!(statements[0].contains("-- This comment inside dollar quotes"));
    assert!(statements[0].contains("/* Multi-line comment"));

    // Table creation should be separate
    assert!(statements[1].contains("CREATE TABLE after_comments"));
}

/// Test current parsing with the exact example from the GitHub issue
/// This reproduces the specific case mentioned in issue #31
#[tokio::test]
async fn test_github_issue_31_example() {
    let sql_content = r#"
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

COMMENT ON FUNCTION app.update_contract IS 'Updates contract status with audit trail';
GRANT EXECUTE ON FUNCTION app.update_contract TO app_role;
"#;

    let statements = DatabasePool::parse_sql_statements(sql_content);

    // This should work correctly with current implementation
    assert_eq!(statements.len(), 3, "Should parse 3 statements");

    // Verify the function definition is complete
    let function_stmt = &statements[0];
    assert!(function_stmt.contains("CREATE OR REPLACE FUNCTION app.update_contract"));
    assert!(function_stmt.contains("DECLARE"));
    assert!(function_stmt.contains("GET DIAGNOSTICS"));
    assert!(function_stmt.contains("RETURN TRUE"));

    // Verify metadata statements
    assert!(statements[1].contains("COMMENT ON FUNCTION"));
    assert!(statements[2].contains("GRANT EXECUTE"));
}
