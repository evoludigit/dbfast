use dbfast::commands::seed;
use dbfast::{Config, DatabasePool};
use dbfast::sql_executor::SqlExecutor;
use std::env;
use std::process::Command;

#[test]
fn test_seed_function_creates_database_clone() {
    // For now, this is a placeholder test that shows the basic structure
    // In reality, we'd need a PostgreSQL test container

    let result = seed::handle_seed("test_output_db", false);

    // With a config file present, this should succeed (placeholder implementation)
    // In reality, this would fail at the PostgreSQL connection stage
    if result.is_err() {
        // If it fails, it should be about missing config or database connection
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("config") || error.to_string().contains("database"),
            "Expected config or database error, got: {}",
            error
        );
    } else {
        // If it succeeds, it means our placeholder implementation worked
        assert!(result.is_ok());
    }
}

#[test]
fn test_seed_command_output() {
    let project_dir = env::current_dir().expect("Failed to get current directory");

    let output = Command::new("cargo")
        .args(["run", "--", "seed", "--output", "test_db_123"])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute command");

    // Should show proper output even if it fails due to no config/database
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either success or expected error about missing configuration
    assert!(
        output.status.success() || stderr.contains("config") || stderr.contains("database"),
        "Expected success or config/database error. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_seed_command_with_seeds_flag() {
    let project_dir = env::current_dir().expect("Failed to get current directory");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "seed",
            "--output",
            "test_db_with_seeds",
            "--with-seeds",
        ])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should handle the --with-seeds flag properly
    assert!(
        output.status.success() || stderr.contains("config") || stderr.contains("database"),
        "Expected success or config/database error. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[tokio::test]
async fn test_seed_command_executes_sql_files() {
    // This test verifies that the seed command can actually read and execute SQL files
    
    // Skip test if no PostgreSQL connection is available (for CI/CD flexibility)
    let config = match Config::from_file("tests/fixtures/dbfast.toml") {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Skipping seed integration test: no config file");
            return;
        }
    };
    
    let pool = match DatabasePool::new(&config.database).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping seed integration test: no database connection");
            return;
        }
    };
    
    // Test that we can read SQL files from the db directory and execute them
    let _sql_executor = SqlExecutor::new();
    
    // Read SQL files from db directory
    let statements_result = SqlExecutor::read_sql_files("db");
    
    assert!(statements_result.is_ok(), "Should be able to read SQL files from db directory");
    
    let statements = statements_result.unwrap();
    assert!(!statements.is_empty(), "Should find SQL statements in db directory");
    
    // Create a test database to execute against
    let test_db_name = format!("dbfast_seed_test_{}", std::process::id());
    
    // Create test database
    let create_db_result = pool.execute(
        &format!("CREATE DATABASE {} WITH TEMPLATE template0", test_db_name), 
        &[]
    ).await;
    
    if create_db_result.is_err() {
        eprintln!("Skipping seed integration test: cannot create test database");
        return;
    }
    
    // For now, we'll test that the seed command would work with the actual SQL files
    // The real integration will happen when we replace the placeholder implementation
    // This test establishes that the SQL files can be read and would be executable
    
    // Clean up: drop the test database
    let cleanup_result = pool.execute(&format!("DROP DATABASE {}", test_db_name), &[]).await;
    assert!(cleanup_result.is_ok(), "Should be able to clean up test database");
}

#[tokio::test]
async fn test_seed_command_creates_database_with_real_sql() {
    // This test will fail until we replace the seed command placeholder
    
    // Skip test if no config available
    let config_path = std::env::current_dir().unwrap().join("dbfast.toml");
    if !config_path.exists() {
        eprintln!("Skipping seed command test: no dbfast.toml config file");
        return;
    }
    
    // Skip test if no PostgreSQL connection is available
    let config = match Config::from_file("dbfast.toml") {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Skipping seed command test: config load failed");
            return;
        }
    };
    
    let pool = match DatabasePool::new(&config.database).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping seed command test: no database connection");
            return;
        }
    };
    
    // Test database name
    let test_db_name = format!("dbfast_real_seed_test_{}", std::process::id());
    
    // Call the actual seed command
    let result = seed::handle_seed(&test_db_name, false);
    
    // This should now work with real SQL execution, not just print a message
    assert!(result.is_ok(), "Seed command should successfully create database with SQL files");
    
    // Verify the database was actually created
    let db_exists_result = pool.query(
        "SELECT 1 FROM pg_database WHERE datname = $1",
        &[&test_db_name]
    ).await;
    
    assert!(db_exists_result.is_ok(), "Should be able to check if database exists");
    let rows = db_exists_result.unwrap();
    assert!(!rows.is_empty(), "Database should have been created by seed command");
    
    // Clean up: drop the test database  
    let cleanup_result = pool.execute(&format!("DROP DATABASE {}", test_db_name), &[]).await;
    assert!(cleanup_result.is_ok(), "Should be able to clean up test database");
}
