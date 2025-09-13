use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("database fixtures"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("seed"));
    assert!(stdout.contains("status"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dbfast"));
    assert!(stdout.contains("0.2.0"));
}

#[test]
fn test_status_command_has_colored_output() {
    let mut cmd = Command::cargo_bin("dbfast").unwrap();

    // Test status command output with verbose flag
    cmd.arg("status").arg("--verbose");
    let result = cmd.output().unwrap();

    let output_str = String::from_utf8(result.stdout).unwrap();

    // Should have colored output indicators and proper formatting
    assert!(
        output_str.contains("📊 DBFast Status"),
        "Status should have emoji header"
    );
    // Since there's no config file in test environment, check for config error message
    assert!(
        output_str.contains("❌ Configuration") || output_str.contains("Template:"),
        "Should show either template section or config error with emoji"
    );

    // Should exit successfully (status command should not fail even without config)
    assert!(result.status.success(), "Status command should succeed");
}

#[test]
fn test_error_messages_are_helpful() {
    let mut cmd = Command::cargo_bin("dbfast").unwrap();

    // Test with invalid command
    cmd.arg("invalid-command");
    let result = cmd.output().unwrap();

    let error_str = String::from_utf8(result.stderr).unwrap();

    // Should have helpful error message
    assert!(
        error_str.contains("❌") || error_str.contains("error"),
        "Should have error indicator"
    );
    assert!(!result.status.success(), "Invalid command should fail");
}

#[test]
fn test_help_text_is_comprehensive() {
    let mut cmd = Command::cargo_bin("dbfast").unwrap();

    cmd.arg("--help");
    let result = cmd.output().unwrap();

    let output_str = String::from_utf8(result.stdout).unwrap();

    // Should have comprehensive help
    assert!(
        output_str.contains("dbfast") || output_str.contains("database"),
        "Should mention dbfast or database"
    );
    assert!(output_str.contains("status"), "Should list status command");
    assert!(output_str.contains("seed"), "Should list seed command");
    assert!(output_str.contains("init"), "Should list init command");

    assert!(result.status.success(), "Help should succeed");
}
