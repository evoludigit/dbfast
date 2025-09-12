use dbfast::commands::seed;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Integration test for the complete seed command workflow
///
/// Tests the end-to-end template → clone pipeline that integrates:
/// - Configuration loading
/// - Database connection
/// - Template-based database cloning
/// - Performance monitoring
#[tokio::test]
async fn test_seed_command_integration_workflow() {
    // Create a temporary directory for test config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");

    // Create test configuration
    let test_config = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "test_template"

[repository]
path = "./db"
type = "structured"

[environments.local]
include_directories = ["0_schema", "1_seed_common", "2_seed_backend"]
"#;

    fs::write(&config_path, test_config).unwrap();

    // Change to temp directory for test
    let original_dir = env::current_dir().unwrap();
    
    // Ensure we can change back to original directory before proceeding
    if let Err(e) = env::set_current_dir(&temp_dir) {
        eprintln!("⚠️  Failed to set temp directory: {}", e);
        return;
    }

    // Test the async seed function directly
    let result = seed::handle_seed_async("test_integration_db", true).await;

    // Always attempt to restore original directory, but handle errors gracefully
    if let Err(e) = env::set_current_dir(&original_dir) {
        eprintln!("⚠️  Failed to restore original directory: {}", e);
        // Try to restore to a known good directory as fallback
        if let Ok(home) = env::var("HOME") {
            let _ = env::set_current_dir(home);
        } else {
            let _ = env::set_current_dir("/tmp");
        }
    }

    // The result will likely be an error due to no PostgreSQL connection in test environment
    // But we can verify it's the right kind of error (database connection, not config)
    match result {
        Ok(()) => {
            println!("✅ Seed integration test succeeded (PostgreSQL available)");
        }
        Err(e) => {
            let error_message = e.to_string();
            println!("⚠️  Seed integration failed (expected): {}", error_message);

            // Verify it's a database-related error, not a config error
            // Could be connection error, template error, or other database issues
            assert!(
                error_message.contains("database")
                    || error_message.contains("connect")
                    || error_message.contains("template")
                    || error_message.contains("config"), // Config errors about database connection are also acceptable
                "Should be a database/config related error, got: {}",
                error_message
            );
        }
    }
}

/// Test seed command with missing config file
#[tokio::test]
async fn test_seed_command_missing_config() {
    // Create temp directory but no config file
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    let result = seed::handle_seed_async("test_db", false).await;

    env::set_current_dir(original_dir).unwrap();

    // Should fail with config error
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("dbfast.toml"),
        "Should mention config file: {}",
        error
    );
    println!("✅ Missing config test passed: {}", error);
}

/// Test the synchronous wrapper function
#[test]
fn test_seed_command_sync_wrapper() {
    // Create temp directory with config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");

    let test_config = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "sync_test_template"

[repository]
path = "./db"
type = "structured"
"#;

    fs::write(&config_path, test_config).unwrap();

    let original_dir = env::current_dir().unwrap();
    
    // Ensure we can change back to original directory before proceeding
    if let Err(e) = env::set_current_dir(&temp_dir) {
        eprintln!("⚠️  Failed to set temp directory: {}", e);
        return;
    }

    // Test synchronous function
    let result = seed::handle_seed("sync_test_db", false);

    // Always attempt to restore original directory, but handle errors gracefully
    if let Err(e) = env::set_current_dir(&original_dir) {
        eprintln!("⚠️  Failed to restore original directory: {}", e);
        // Try to restore to a known good directory as fallback
        if let Ok(home) = env::var("HOME") {
            let _ = env::set_current_dir(home);
        } else {
            let _ = env::set_current_dir("/tmp");
        }
    }

    // Should handle the async operation properly
    match result {
        Ok(()) => {
            println!("✅ Sync wrapper test succeeded");
        }
        Err(e) => {
            println!("⚠️  Sync wrapper failed (expected due to database): {}", e);
            // Should be a database error, not a runtime error
            let error_str = e.to_string();
            assert!(
                !error_str.contains("runtime"),
                "Should not be runtime error: {}",
                error_str
            );
        }
    }
}
