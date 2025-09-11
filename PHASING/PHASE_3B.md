# Phase 3B - Environment Commands & Validation

**Goal**: Add environment management commands and validation to ensure safe deployments.

## TDD Approach
Write tests for environment commands first, then implement command handlers.

## Deliverables

### 1. Environment Listing Command
- [ ] `dbfast environments` command implementation
- [ ] Show configured environments with file counts
- [ ] Display include/exclude patterns
- [ ] Verbose mode with detailed filtering info

**Test**: Environment listing shows correct information

### 2. Environment Validation Command
- [ ] `dbfast validate-env --env <name>` implementation
- [ ] Validate environment configuration syntax
- [ ] Check that filtered files exist
- [ ] Warn about potential production safety issues

**Test**: Environment validation catches configuration errors

### 3. Enhanced Status Command
- [ ] Update `dbfast status` to show environment info
- [ ] Display template status per environment
- [ ] Show file counts and last build times
- [ ] Environment-specific change detection

**Test**: Status command provides comprehensive environment information

## Success Criteria

1. **Visibility**: Users can easily see what each environment includes
2. **Validation**: Configuration errors are caught before deployment
3. **Safety**: Clear warnings about production vs development content
4. **Integration**: Commands work seamlessly with existing functionality

## Key Files to Modify

```
src/
├── cli.rs              # Add new environment subcommands
├── commands/
│   ├── environments.rs # New: Environment listing command
│   ├── validate_env.rs # New: Environment validation command
│   └── status.rs       # Enhance with environment info
└── main.rs             # Wire up new commands

tests/
├── environment_commands_tests.rs # Environment command tests
└── fixtures/
    └── configs/        # Test configuration files
```

## Example Test

```rust
#[test]
fn test_environments_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "environments"])
        .current_dir(test_project_dir())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should list configured environments
    assert!(stdout.contains("local"));
    assert!(stdout.contains("production"));

    // Should show file counts
    assert!(stdout.contains("files:"));

    // Should indicate different file counts for different environments
    let local_line = stdout.lines().find(|line| line.contains("local")).unwrap();
    let prod_line = stdout.lines().find(|line| line.contains("production")).unwrap();

    // Extract file counts (rough parsing for test)
    assert!(local_line.contains("files:"));
    assert!(prod_line.contains("files:"));
}

#[test]
fn test_validate_env_command() {
    // Test valid environment
    let output = Command::new("cargo")
        .args(["run", "--", "validate-env", "--env", "local"])
        .current_dir(test_project_dir())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅") || stdout.contains("valid"));

    // Test invalid environment
    let output = Command::new("cargo")
        .args(["run", "--", "validate-env", "--env", "nonexistent"])
        .current_dir(test_project_dir())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("❌") || stderr.contains("error") || stderr.contains("not found"));
}

#[test]
fn test_enhanced_status_with_environments() {
    let output = Command::new("cargo")
        .args(["run", "--", "status", "--verbose"])
        .current_dir(test_project_dir())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show environment section
    assert!(stdout.contains("Environments:"));
    assert!(stdout.contains("• local"));
    assert!(stdout.contains("• production"));

    // Should show file counts per environment
    assert!(stdout.contains("files)"));

    // Should show template status if templates exist
    if stdout.contains("Templates:") {
        assert!(stdout.contains("blog_template_local") || stdout.contains("template"));
    }
}

#[tokio::test]
async fn test_production_safety_warnings() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_unsafe_production_config(&temp_dir).await;

    let output = Command::new("cargo")
        .args(["run", "--", "validate-env", "--env", "production"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should warn about including seed data in production
    assert!(
        stdout.contains("⚠️") || stderr.contains("warning") ||
        stdout.contains("seed") || stderr.contains("seed")
    );
}
```

**Duration**: 1-2 days
**Next**: Phase 4A - Remote Configuration
