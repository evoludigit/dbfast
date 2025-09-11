# Phase 4A - Remote Configuration & Validation

**Goal**: Add remote database configuration and connection validation for safe deployments.

## TDD Approach
Write tests for remote configuration first, then implement connection management.

## Deliverables

### 1. Remote Configuration
- [ ] Extend `dbfast.toml` with remote database configs
- [ ] Parse remote connection strings safely
- [ ] Environment-to-remote mapping
- [ ] Connection security (password from env vars)

**Test**: Remote configs are loaded and validated correctly

### 2. Remote Connection Validation
- [ ] Test remote database connectivity
- [ ] Validate remote database permissions
- [ ] Check remote database version compatibility
- [ ] Connection timeout and retry logic

**Test**: Remote connections are validated before deployment

### 3. Remote Management Commands
- [ ] `dbfast remote add` command
- [ ] `dbfast remote list` command
- [ ] `dbfast remote test` command
- [ ] `dbfast remote remove` command

**Test**: Remote management commands work correctly

## Success Criteria

1. **Security**: Remote credentials are handled safely
2. **Validation**: Connection issues are caught early
3. **Safety**: Cannot deploy to unconfigured remotes
4. **Usability**: Clear error messages for connection problems

## Key Files to Modify

```
src/
├── remote.rs           # New: Remote configuration and management
├── config.rs           # Extend with remote configurations
├── cli.rs              # Add remote subcommands
├── commands/
│   └── remote.rs       # New: Remote management commands
└── main.rs             # Wire up remote commands

tests/
├── remote_tests.rs     # Remote configuration tests
└── fixtures/
    └── remote_configs/ # Test remote configurations
```

## Example Test

```rust
#[tokio::test]
async fn test_remote_configuration() {
    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "blog_template"

[repository]
path = "./db"
type = "structured"

[environments.local]
include_directories = ["0_schema", "1_seed_common", "2_seed_backend"]

[environments.production]
include_directories = ["0_schema", "6_migration"]
exclude_directories = ["1_seed_common", "2_seed_backend"]

[remotes.staging]
url = "postgresql://staging_user@staging.example.com:5432/staging_db"
password_env = "STAGING_DB_PASSWORD"
environment = "local"
allow_destructive = true
backup_before_deploy = true

[remotes.production]
url = "postgresql://prod_user@prod.example.com:5432/prod_db"
password_env = "PROD_DB_PASSWORD"
environment = "production"
allow_destructive = false
backup_before_deploy = true
require_confirmation = true
"#;

    let config: Config = toml::from_str(config_content).unwrap();

    // Verify remote configs are parsed
    assert!(config.remotes.is_some());
    let remotes = config.remotes.unwrap();

    let staging = remotes.get("staging").unwrap();
    assert_eq!(staging.environment, "local");
    assert!(staging.allow_destructive);
    assert!(staging.backup_before_deploy);

    let production = remotes.get("production").unwrap();
    assert_eq!(production.environment, "production");
    assert!(!production.allow_destructive);
    assert!(production.require_confirmation);
}

#[tokio::test]
async fn test_remote_connection_validation() {
    let remote_config = RemoteConfig {
        name: "test_remote".to_string(),
        url: "postgresql://test_user@localhost:5432/test_db".to_string(),
        password_env: Some("TEST_DB_PASSWORD".to_string()),
        environment: "local".to_string(),
        allow_destructive: true,
        backup_before_deploy: false,
        require_confirmation: false,
    };

    // Test with valid credentials
    std::env::set_var("TEST_DB_PASSWORD", "correct_password");
    let result = remote_config.validate_connection().await;

    if let Ok(pool) = test_db_pool().await {
        // If we have a test database available
        assert!(result.is_ok());
    } else {
        // If no test database, should get connection error (not config error)
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("connection") || error.to_string().contains("timeout"));
    }

    // Test with missing password env var
    std::env::remove_var("TEST_DB_PASSWORD");
    let result = remote_config.validate_connection().await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("password") || error.to_string().contains("environment"));
}

#[test]
fn test_remote_commands() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");

    // Create basic config
    create_test_config(&config_path);

    // Test remote add
    let output = Command::new("cargo")
        .args([
            "run", "--", "remote", "add",
            "--name", "test_staging",
            "--url", "postgresql://user@staging.example.com:5432/db",
            "--env", "local"
        ])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test remote list
    let output = Command::new("cargo")
        .args(["run", "--", "remote", "list"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_staging"));
}
```

**Duration**: 2-3 days
**Next**: Phase 4B - Backup Management
