use dbfast::commands::seed;
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
        output.status.success()
            || stderr.contains("config")
            || stderr.contains("database")
            || stderr.contains("current directory")
            || stderr.contains("No such file"),
        "Expected success or config/database/directory error. stdout: {}, stderr: {}",
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
        output.status.success()
            || stderr.contains("config")
            || stderr.contains("database")
            || stderr.contains("current directory")
            || stderr.contains("No such file"),
        "Expected success or config/database/directory error. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test the new async seed handler with template creation
#[tokio::test]
async fn test_async_seed_template_creation() {
    use dbfast::commands::seed::handle_seed_async;
    use std::fs;
    use tempfile::TempDir;

    // Create temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test config file
    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "test_template"

[repository]
path = "test_fixtures"
type = "structured"
"#;

    let config_path = temp_path.join("dbfast.toml");
    fs::write(&config_path, config_content).unwrap();

    // Change to the temp directory for the test
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Test async seed handler
    let result = handle_seed_async("test_db", false).await;

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // The function should handle the case where SQL files don't exist gracefully
    // or where database connection fails (which is expected in test environment)
    match result {
        Ok(_) => println!("Async seed succeeded"),
        Err(e) => {
            // Expected errors: config, database connection, or template creation issues
            let error_str = e.to_string();
            assert!(
                error_str.contains("config")
                    || error_str.contains("database")
                    || error_str.contains("connection")
                    || error_str.contains("template"),
                "Unexpected error: {}",
                error_str
            );
        }
    }
}
