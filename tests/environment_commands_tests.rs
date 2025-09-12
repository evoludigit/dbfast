use assert_cmd::prelude::*;
use dbfast::config::{Config, DatabaseConfig, Environment, RepositoryConfig};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test helper to create a test project with environment configuration
fn create_test_project_with_environments() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create SQL directory structure
    let schema_dir = temp_dir.path().join("0_schema");
    fs::create_dir_all(&schema_dir).unwrap();

    let seed_local_dir = temp_dir.path().join("1_seed_local");
    fs::create_dir_all(&seed_local_dir).unwrap();

    let seed_common_dir = temp_dir.path().join("1_seed_common");
    fs::create_dir_all(&seed_common_dir).unwrap();

    // Create some test SQL files
    fs::write(
        schema_dir.join("001_users.sql"),
        "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);",
    )
    .unwrap();

    fs::write(
        seed_common_dir.join("001_common_data.sql"),
        "INSERT INTO users (name) VALUES ('system');",
    )
    .unwrap();

    fs::write(
        seed_local_dir.join("001_test_users.sql"),
        "INSERT INTO users (name) VALUES ('test_user_1'), ('test_user_2');",
    )
    .unwrap();

    // Create config with multiple environments
    let mut environments = HashMap::new();

    environments.insert(
        "local".to_string(),
        Environment {
            include_directories: vec![
                "0_schema".to_string(),
                "1_seed_common".to_string(),
                "1_seed_local".to_string(),
            ],
            exclude_directories: vec![],
        },
    );

    environments.insert(
        "production".to_string(),
        Environment {
            include_directories: vec!["0_schema".to_string(), "1_seed_common".to_string()],
            exclude_directories: vec!["1_seed_local".to_string()],
        },
    );

    let config = Config {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password_env: Some("POSTGRES_PASSWORD".to_string()),
            template_name: "blog_template".to_string(),
        },
        repository: RepositoryConfig {
            path: temp_dir.path().display().to_string(),
            repo_type: "structured".to_string(),
        },
        environments,
        remotes: HashMap::new(),
    };

    let config_content = toml::to_string(&config).unwrap();
    fs::write(temp_dir.path().join("dbfast.toml"), config_content).unwrap();

    temp_dir
}

/// Test helper to create unsafe production config (includes seed data in production)
fn create_unsafe_production_config() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create directories
    let schema_dir = temp_dir.path().join("0_schema");
    fs::create_dir_all(&schema_dir).unwrap();

    let seed_dir = temp_dir.path().join("1_seed_backend");
    fs::create_dir_all(&seed_dir).unwrap();

    // Create SQL files
    fs::write(
        schema_dir.join("001_users.sql"),
        "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);",
    )
    .unwrap();

    fs::write(
        seed_dir.join("001_test_data.sql"),
        "INSERT INTO users (name) VALUES ('test_user');",
    )
    .unwrap();

    // Create UNSAFE production config that includes seed data
    let mut environments = HashMap::new();

    environments.insert(
        "production".to_string(),
        Environment {
            include_directories: vec![
                "0_schema".to_string(),
                "1_seed_backend".to_string(), // DANGEROUS: includes test data in production
            ],
            exclude_directories: vec![],
        },
    );

    let config = Config {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password_env: Some("POSTGRES_PASSWORD".to_string()),
            template_name: "unsafe_template".to_string(),
        },
        repository: RepositoryConfig {
            path: temp_dir.path().display().to_string(),
            repo_type: "structured".to_string(),
        },
        environments,
        remotes: HashMap::new(),
    };

    let config_content = toml::to_string(&config).unwrap();
    fs::write(temp_dir.path().join("dbfast.toml"), config_content).unwrap();

    temp_dir
}

#[test]
fn test_environments_command_lists_configured_environments() {
    let temp_dir = create_test_project_with_environments();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .arg("environments")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should list configured environments
    assert!(stdout.contains("local"), "Should show local environment");
    assert!(
        stdout.contains("production"),
        "Should show production environment"
    );

    // Should show file counts for each environment
    assert!(stdout.contains("files"), "Should show file counts");

    // Should show directory information
    assert!(
        stdout.contains("0_schema"),
        "Should mention schema directory"
    );
    assert!(stdout.contains("1_seed"), "Should mention seed directories");
}

#[test]
fn test_environments_command_shows_different_file_counts() {
    let temp_dir = create_test_project_with_environments();

    // Ensure files are flushed to disk before running command
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .arg("environments")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Local should have more files than production (includes local seed data)
    let local_line = stdout
        .lines()
        .find(|line| line.starts_with("• local"))
        .expect("Should find local environment in output");
    let production_line = stdout
        .lines()
        .find(|line| line.starts_with("• production"))
        .expect("Should find production environment in output");

    assert!(
        local_line.contains("files"),
        "Local environment should show file count. Line: '{}', Full output: '{}'",
        local_line,
        stdout
    );
    assert!(
        production_line.contains("files"),
        "Production environment should show file count"
    );

    // Both should have file counts, but they should be different
    // (this is a basic check - we'll verify exact counts in unit tests)
    assert_ne!(
        local_line, production_line,
        "Local and production should have different configurations"
    );
}

#[test]
fn test_environments_command_verbose_mode() {
    let temp_dir = create_test_project_with_environments();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .args(["environments", "--verbose"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verbose mode should show include/exclude patterns
    assert!(
        stdout.contains("include") || stdout.contains("Include"),
        "Should show include patterns"
    );
    assert!(
        stdout.contains("exclude") || stdout.contains("Exclude"),
        "Should show exclude patterns"
    );

    // Should show more detailed information
    assert!(stdout.len() > 200, "Verbose output should be more detailed");
}

#[test]
fn test_validate_env_command_with_valid_environment() {
    let temp_dir = create_test_project_with_environments();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .args(["validate-env", "--env", "local"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Valid environment should pass validation"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✅") || stdout.contains("valid") || stdout.contains("Valid"),
        "Should indicate successful validation"
    );
}

#[test]
fn test_validate_env_command_with_invalid_environment() {
    let temp_dir = create_test_project_with_environments();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .args(["validate-env", "--env", "nonexistent"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Invalid environment should fail validation"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("not found") || stderr.contains("invalid"),
        "Should show error for nonexistent environment"
    );
}

#[test]
fn test_validate_env_command_production_safety_warnings() {
    let temp_dir = create_unsafe_production_config();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .args(["validate-env", "--env", "production"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    // Should warn about including seed data in production
    assert!(
        all_output.contains("⚠️")
            || all_output.contains("warning")
            || all_output.contains("seed")
            || all_output.contains("production") && all_output.contains("unsafe"),
        "Should warn about unsafe production configuration. Output: {}",
        all_output
    );
}

#[test]
fn test_enhanced_status_command_shows_environment_info() {
    let temp_dir = create_test_project_with_environments();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .arg("status")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show environment section
    assert!(
        stdout.contains("Environment") || stdout.contains("environment"),
        "Should show environment information"
    );

    // Should show configured environments
    assert!(stdout.contains("local"), "Should list local environment");
    assert!(
        stdout.contains("production"),
        "Should list production environment"
    );
}

#[test]
fn test_enhanced_status_verbose_shows_detailed_environment_info() {
    let temp_dir = create_test_project_with_environments();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .args(["status", "--verbose"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show detailed environment information
    assert!(
        stdout.contains("files"),
        "Should show file counts per environment"
    );
    assert!(
        stdout.contains("local"),
        "Should show local environment details"
    );
    assert!(
        stdout.contains("production"),
        "Should show production environment details"
    );

    // Should be more verbose than regular status
    assert!(
        stdout.len() > 300,
        "Verbose status should provide detailed information"
    );
}

#[test]
fn test_environments_command_with_no_config_file() {
    let temp_dir = TempDir::new().unwrap();
    // Don't create dbfast.toml

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .arg("environments")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success(), "Should fail without config file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("dbfast.toml") || stderr.contains("config"),
        "Should mention missing config file"
    );
}

#[test]
fn test_validate_env_command_checks_file_existence() {
    let temp_dir = TempDir::new().unwrap();

    // Create config but don't create the actual SQL files
    let mut environments = HashMap::new();
    environments.insert(
        "test".to_string(),
        Environment {
            include_directories: vec!["nonexistent_dir".to_string()],
            exclude_directories: vec![],
        },
    );

    let config = Config {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password_env: Some("POSTGRES_PASSWORD".to_string()),
            template_name: "test_template".to_string(),
        },
        repository: RepositoryConfig {
            path: temp_dir.path().display().to_string(),
            repo_type: "structured".to_string(),
        },
        environments,
        remotes: HashMap::new(),
    };

    let config_content = toml::to_string(&config).unwrap();
    fs::write(temp_dir.path().join("dbfast.toml"), config_content).unwrap();

    let mut cmd = Command::cargo_bin("dbfast").unwrap();
    let output = cmd
        .args(["validate-env", "--env", "test"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    // Should warn about missing directories/files
    assert!(
        all_output.contains("warning")
            || all_output.contains("not found")
            || all_output.contains("missing")
            || all_output.contains("nonexistent"),
        "Should warn about missing directories. Output: {}",
        all_output
    );
}
