# Phase 1 - Core Foundation

**Goal**: Establish the basic Rust project structure with PostgreSQL connectivity and file scanning capabilities.

## TDD Approach

Start with tests first, then implement minimal code to make tests pass.

## Deliverables

### 1. Project Setup
- [ ] `Cargo.toml` with essential dependencies
- [ ] Basic CLI structure with `clap`
- [ ] Logging setup with `tracing`

### 2. Configuration System
- [ ] Config struct that parses `dbfast.toml`
- [ ] Database connection configuration
- [ ] Repository path configuration

**Test**: Load valid config, handle missing config file gracefully

### 3. PostgreSQL Connection
- [ ] Database connection pool setup
- [ ] Connection validation
- [ ] Basic error handling

**Test**: Connect to PostgreSQL, handle connection failures

### 4. File Scanner
- [ ] Walk directory tree to find SQL files
- [ ] Filter files by patterns (include/exclude)
- [ ] Calculate file hash for change detection

**Test**: Scan sample SQL directory, detect file changes

### 5. Basic CLI Commands
- [ ] `dbfast status` - show current state
- [ ] `dbfast init --repo-dir <path>` - initialize from repository

**Test**: Commands run without error, show expected output

## Success Criteria

1. **Tests pass**: All unit tests for core functionality
2. **Basic connectivity**: Can connect to PostgreSQL database
3. **File discovery**: Can scan and list SQL files from repository
4. **Configuration**: Can load and validate configuration
5. **CLI works**: Basic commands execute successfully

## Key Files to Create

```
src/
├── main.rs              # CLI entry point
├── config.rs            # Configuration loading
├── database.rs          # PostgreSQL connection
├── scanner.rs           # SQL file scanning
└── lib.rs              # Library entry point

tests/
├── integration_tests.rs # Integration tests
└── fixtures/           # Test SQL files
    ├── 0_schema/
    └── 1_seed/
```

## Dependencies for Cargo.toml

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
tokio-postgres = "0.7"
bb8 = "0.8"
bb8-postgres = "0.8"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
walkdir = "2.4"
globset = "0.4"
xxhash-rust = { version = "0.8", features = ["xxh3"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
tempfile = "3.8"
testcontainers = "0.15"
```

## Example Tests to Write

### Config Test
```rust
#[test]
fn test_config_loading() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.repository.path, "./db");
}
```

### Scanner Test
```rust
#[test] 
fn test_file_scanning() {
    let scanner = FileScanner::new("tests/fixtures/db");
    let files = scanner.scan().unwrap();
    assert!(files.len() > 0);
    assert!(files.iter().any(|f| f.path.ends_with(".sql")));
}
```

### Database Test
```rust
#[tokio::test]
async fn test_database_connection() {
    let config = DatabaseConfig::default();
    let pool = create_pool(&config).await.unwrap();
    let conn = pool.get().await.unwrap();
    // Test basic query
}
```

## Phase 1 Complete When

- [ ] All tests pass
- [ ] Can run `dbfast status` and see database connection status
- [ ] Can run `dbfast init --repo-dir tests/fixtures/db` successfully
- [ ] File scanner can discover and hash SQL files
- [ ] Configuration loads from `dbfast.toml`

**Next Phase**: Template management and database cloning