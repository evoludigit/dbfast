use dbfast::commands::init;
use std::env;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_init_function_directly() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("db");
    fs::create_dir_all(&repo_path).expect("Failed to create repo directory");

    // Test init function directly
    let result = init::handle_init_with_output_dir(
        repo_path.to_str().unwrap(),
        "test_template",
        temp_dir.path(),
    );

    assert!(result.is_ok(), "Init should succeed: {:?}", result);

    // Check that dbfast.toml was created
    let config_path = temp_dir.path().join("dbfast.toml");
    assert!(config_path.exists());

    // Check config contents
    let config_content = fs::read_to_string(config_path).expect("Failed to read config");
    assert!(config_content.contains("test_template"));
    assert!(config_content.contains("[database]"));
    assert!(config_content.contains("[repository]"));
}

#[test]
fn test_init_command_creates_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("db");
    fs::create_dir_all(&repo_path).expect("Failed to create repo directory");

    // Get the current project directory and run the CLI from there
    let project_dir = env::current_dir().expect("Failed to get current directory");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--repo-dir",
            repo_path.to_str().unwrap(),
            "--template-name",
            "test_template",
        ])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Successfully initialized"),
        "stdout was: {}",
        stdout
    );

    // Check that dbfast.toml was created in project directory
    let config_path = project_dir.join("dbfast.toml");
    assert!(config_path.exists());

    // Clean up
    let _ = fs::remove_file(&config_path);
}

#[test]
fn test_init_function_with_invalid_repo_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalid_path = temp_dir.path().join("nonexistent");

    // Test init function directly with invalid repo path
    let result = init::handle_init_with_output_dir(
        invalid_path.to_str().unwrap(),
        "test_template",
        temp_dir.path(),
    );

    assert!(result.is_err(), "Init should fail with invalid repo dir");
    let error = result.unwrap_err();
    assert!(error
        .to_string()
        .contains("Repository directory does not exist"));
}

#[test]
fn test_init_command_with_invalid_repo_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalid_path = temp_dir.path().join("nonexistent");
    let project_dir = env::current_dir().expect("Failed to get current directory");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--repo-dir",
            invalid_path.to_str().unwrap(),
            "--template-name",
            "test_template",
        ])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Repository directory does not exist"),
        "stderr was: {}",
        stderr
    );
}
