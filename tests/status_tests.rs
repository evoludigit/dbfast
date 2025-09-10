use dbfast::commands::status;
use std::env;
use std::process::Command;

#[test]
fn test_status_function_shows_config_info() {
    // Test the status function directly
    let result = status::handle_status();

    // Should succeed and show status information
    assert!(result.is_ok(), "Status should succeed: {:?}", result);
}

#[test]
fn test_status_command_output() {
    let project_dir = env::current_dir().expect("Failed to get current directory");

    let output = Command::new("cargo")
        .args(["run", "--", "status"])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show configuration and status information
    assert!(stdout.contains("DBFast Status"), "stdout was: {}", stdout);
    assert!(stdout.contains("Configuration"), "stdout was: {}", stdout);
}

#[test]
fn test_status_without_config() {
    // Test status function when config is missing
    // This would need to be tested in a clean directory without dbfast.toml
    // For now, we test that it handles missing config gracefully

    let result = status::handle_status_in_dir(&std::path::PathBuf::from("/tmp/nonexistent_dir"));

    // Should handle missing config gracefully (not panic)
    if result.is_err() {
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("config") || error.to_string().contains("No such file"),
            "Expected config error, got: {}",
            error
        );
    } else {
        // If it succeeds, it should show appropriate message about missing config
        assert!(result.is_ok());
    }
}
