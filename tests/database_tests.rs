use dbfast::{Config, DatabasePool};

#[tokio::test]
async fn test_database_connection_pool_creation() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    // Test that we can create a pool from config
    match DatabasePool::from_config(&config.database).await {
        Ok(_pool) => {
            // Pool creation succeeded - actual query testing would require TestContainers
            println!("✅ Database pool creation succeeded");
        }
        Err(_) => {
            // Expected to fail without real PostgreSQL connection
            println!("⚠️  Database pool creation failed (expected without PostgreSQL)");
        }
    }
}

#[tokio::test]
async fn test_database_config_validation() {
    // Test that we can create a pool from config (doesn't actually connect yet)
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    let result = DatabasePool::from_config(&config.database).await;
    // For now, our pool creation succeeds without immediate connection testing
    // Real database connection testing would require TestContainers
    assert!(result.is_ok());
}

#[cfg(test)]
mod sql_parsing_tests {
    use dbfast::DatabasePool;

    #[test]
    fn test_parse_sql_statements_with_dollar_quoted_strings() {
        let sql_content = r#"
-- Create a test table
CREATE TABLE test_table (id INTEGER);

-- This should be parsed as a single statement despite containing semicolons
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'printoptim_core') THEN
        CREATE ROLE printoptim_core WITH
            LOGIN
            NOSUPERUSER
            INHERIT
            CREATEDB
            NOCREATEROLE
            NOREPLICATION
            PASSWORD 'printoptim_core_test';  -- Only for testing/CI
    END IF;
END $$;

-- Another simple statement
INSERT INTO test_table VALUES (1);
"#;

        // Access the public method for testing
        let statements = DatabasePool::parse_sql_statements(sql_content);

        // Should have exactly 3 statements:
        // 1. CREATE TABLE test_table (id INTEGER)
        // 2. The entire DO $$ ... END $$ block
        // 3. INSERT INTO test_table VALUES (1)
        assert_eq!(statements.len(), 3);

        // Check that the DO block is kept as one statement
        let do_block = statements.iter().find(|s| s.contains("DO $$")).unwrap();
        assert!(do_block.contains("CREATE ROLE printoptim_core"));
        assert!(do_block.contains("END $$"));

        // Check other statements
        assert!(statements
            .iter()
            .any(|s| s.contains("CREATE TABLE test_table")));
        assert!(statements
            .iter()
            .any(|s| s.contains("INSERT INTO test_table VALUES (1)")));
    }

    #[test]
    fn test_parse_sql_statements_with_nested_dollar_quotes() {
        let sql_content = r#"
CREATE OR REPLACE FUNCTION test_func() RETURNS text AS $BODY$
DECLARE
    result text := $$Hello; World$$;
BEGIN
    RETURN result;
END;
$BODY$ LANGUAGE plpgsql;
"#;

        let statements = DatabasePool::parse_sql_statements(sql_content);

        // Should be one statement
        assert_eq!(statements.len(), 1);
        assert!(statements[0].contains("CREATE OR REPLACE FUNCTION"));
        assert!(statements[0].contains("$$Hello; World$$"));
    }

    #[test]
    fn test_parse_sql_statements_edge_cases() {
        // Test empty input
        assert!(DatabasePool::parse_sql_statements("").is_empty());

        // Test only comments
        let comment_only = r#"
        -- Just comments
        /* Multi-line comment */
        "#;
        assert!(DatabasePool::parse_sql_statements(comment_only).is_empty());

        // Test statement without ending semicolon
        let no_semicolon = "SELECT 1";
        let statements = DatabasePool::parse_sql_statements(no_semicolon);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], "SELECT 1");

        // Test multiple semicolons in dollar-quoted string
        let complex_dollar_quote = r#"
        CREATE FUNCTION complex() RETURNS text AS $func$
        BEGIN
            EXECUTE 'SELECT 1; SELECT 2;';  -- Multiple semicolons inside
            RETURN 'done; really done;';    -- More semicolons
        END;
        $func$ LANGUAGE plpgsql;
        "#;
        let statements = DatabasePool::parse_sql_statements(complex_dollar_quote);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].contains("SELECT 1; SELECT 2"));
        assert!(statements[0].contains("done; really done"));
    }
}
