# DBFast v0.1.0 - Lightning-Fast PostgreSQL Database Seeding

[![Build Status](https://img.shields.io/badge/tests-40%20passing-green)](https://github.com/evoludigit/dbfast/tree/main/tests)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Transform database fixtures from a 60-second bottleneck into a 100ms delight.**

DBFast is a high-performance PostgreSQL database seeding and template management tool that accelerates your development workflow by providing instant database fixtures through intelligent template-based caching.

## 🚀 Quick Start

Get up and running in 60 seconds:

```bash
# 1. Clone and build
git clone https://github.com/evoludigit/dbfast
cd dbfast && cargo build --release

# 2. Initialize your database template
./target/release/dbfast init --repo-dir ./sql --template-name myapp_template

# 3. Get instant database fixtures
./target/release/dbfast seed --output test_db_1        # Empty schema (~100ms)
./target/release/dbfast seed --output test_db_2 --with-seeds  # With data (~150ms)

# 4. Status check
./target/release/dbfast status --verbose
```

## 🎯 Why DBFast?

**Before DBFast:**
- 🐌 60+ seconds to set up test databases
- 🔄 Running migrations and seeds repeatedly
- 💸 Expensive CI pipeline time
- 😤 Developer frustration waiting for tests

**After DBFast:**
- ⚡ ~100ms for clean database fixtures
- 🎯 Template-based approach with intelligent caching
- 🔄 Only rebuilds when SQL files actually change
- 🚀 Instant test database provisioning

## ✨ Core Features

- **Template-Based Fixtures**: Pre-built database images for instant seeding
- **Intelligent Change Detection**: xxHash-based rebuilding only when SQL files change
- **Environment-Aware Deployments**: Filter SQL files based on environment configurations
- **Remote Database Support**: Deploy templates to remote PostgreSQL instances with safety features
- **Backup Integration**: Automatic backup before destructive operations
- **Production-Ready**: Connection pooling, retry logic, and comprehensive error handling

## Architecture

```
┌─────────────────────────────────────────┐
│ DBFast CLI                              │
│ - init, seed, deploy, status commands   │
│ - Configuration management              │
│ - Environment filtering                 │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│ Core Components                         │
│ - Template Manager                      │
│ - Database Clone Manager                │
│ - SQL Repository Scanner                │
│ - Change Detection                      │
└─────────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────┐
│ PostgreSQL Database                     │
│ - Template databases                    │
│ - Native template cloning              │
└─────────────────────────────────────────┘
```

## Installation

### From Source

```bash
git clone https://github.com/evoludigit/dbfast
cd dbfast
cargo build --release

# Binary available at target/release/dbfast
```

## Configuration

Create a `dbfast.toml` configuration file:

```toml
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "myapp_template"

# Environment definitions
[environments.local]
include_directories = ["0_schema", "1_seed_common", "2_seed_backend"]

[environments.production]
include_directories = ["0_schema", "6_migration"]
exclude_directories = ["1_seed_common", "2_seed_backend"]

# Remote configurations
[remotes.production]
url = "postgres://prod-server:5432/myapp"
environment = "production"
```

## 📖 Detailed Usage

### Initialize Template

Set up your database template from existing SQL files:

```bash
# Initialize from a SQL repository
dbfast init --repo-dir ./sql --template-name myapp_template

# This scans your SQL files and creates a template database
# Template will include schema, indexes, functions, etc.
```

### Create Database Fixtures

Get instant database copies from your template:

```bash
# Clean schema only (fastest - ~100ms)
dbfast seed --output test_db_1

# Schema + seed data (~150ms)
dbfast seed --output test_db_2 --with-seeds

# Multiple parallel fixtures for testing
dbfast seed --output integration_test_db
dbfast seed --output unit_test_db --with-seeds
dbfast seed --output feature_test_db
```

### Real-World Integration Examples

**In your test suite:**
```bash
#!/bin/bash
# test-setup.sh
export TEST_DB="test_$(date +%s)"
dbfast seed --output "$TEST_DB" --with-seeds
export DATABASE_URL="postgresql://localhost:5432/$TEST_DB"
npm test
```

**CI/CD Pipeline:**
```yaml
# .github/workflows/test.yml
- name: Setup Test Database
  run: |
    dbfast seed --output "ci_test_${{ github.run_id }}" --with-seeds
    echo "DATABASE_URL=postgresql://localhost:5432/ci_test_${{ github.run_id }}" >> $GITHUB_ENV
```

**Docker Development:**
```dockerfile
RUN cargo build --release
RUN ./target/release/dbfast init --repo-dir ./sql --template-name app_template
CMD ./target/release/dbfast seed --output dev_db --with-seeds && npm run dev
```

### Check Status

```bash
dbfast status
dbfast status --verbose
```

### Environment Management

```bash
dbfast environments
dbfast validate-env production
```

### Remote Deployment

```bash
# Add remote
dbfast remote add --name production --url $DATABASE_URL --env production

# List remotes
dbfast remote list

# Test connection
dbfast remote test production

# Deploy to remote
dbfast deploy --remote production --env production --yes
```

## Project Structure

```
your-project/
├── db/                          # SQL repository
│   ├── 0_schema/               # Schema files
│   ├── 1_seed_common/          # Common seed data
│   ├── 2_seed_backend/         # Backend-specific seeds
│   └── 6_migration/            # Migrations
├── dbfast.toml                 # Configuration
└── target/release/dbfast       # Binary
```

## 📚 Documentation

### API Documentation

Generate and view the complete API documentation:

```bash
# Generate documentation
cargo doc --no-deps --document-private-items

# Open in browser
cargo doc --open --no-deps

# Documentation available at: target/doc/dbfast/index.html
```

The generated documentation includes:
- Complete module documentation with examples
- All public APIs with usage patterns
- Internal architecture details
- Cross-referenced code examples

### Library Usage

DBFast can also be used as a Rust library:

```rust
use dbfast::{Config, DatabasePool, FileScanner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load("dbfast.toml")?;

    // Create database pool
    let pool = DatabasePool::new("postgresql://localhost:5432/mydb").await?;

    // Scan SQL files for changes
    let scanner = FileScanner::new("./sql");
    let files = scanner.scan_sql_files()?;

    println!("Found {} SQL files", files.len());
    Ok(())
}
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run with coverage
cargo test --coverage

# Run specific test categories
cargo test --test integration_test
cargo test scanner_tests
cargo test cli_tests
```

**Test Coverage:**

- 40 unit and integration tests passing
- Error handling system tests
- Health monitoring data structure tests
- Metrics collection framework tests
- Retry/circuit breaker pattern tests
- Basic database cloning tests
- Configuration management tests

## Technical Details

### Dependencies

- **tokio**: Async runtime
- **tokio-postgres**: PostgreSQL driver
- **bb8/bb8-postgres**: Connection pooling
- **clap**: CLI argument parsing
- **serde/toml**: Configuration serialization
- **walkdir/globset**: File system operations
- **xxhash-rust**: File change detection

### 🚀 Performance Details

**Speed Comparison:**
```
Traditional Approach (migrations + seeds):
├── DROP/CREATE database:     ~500ms
├── Run 50 migrations:        ~45s
├── Load seed data:           ~15s
└── Total:                    ~60s

DBFast Template Approach:
├── CREATE DATABASE WITH TEMPLATE: ~80ms
├── Template cache check:           ~20ms
└── Total:                          ~100ms

Performance Improvement: 600x faster! 🚀
```

**How it works:**
- **Template Caching**: Pre-built database images stored as PostgreSQL templates
- **Smart Rebuilding**: xxHash-based change detection only rebuilds when SQL files change
- **Native Speed**: Uses PostgreSQL's `CREATE DATABASE WITH TEMPLATE` (copy-on-write)
- **Connection Pooling**: bb8 connection pooling for optimal database performance
- **Async Everything**: tokio-based async operations for maximum throughput

**Real-world impact:**
- CI pipelines: 10+ minute savings per test run
- Development: Instant test database setup
- Parallel testing: Provision dozens of databases simultaneously
- Resource efficiency: Lower CPU/IO usage vs repeated migrations

### Code Quality

- ~9,000 lines of Rust code
- Comprehensive error handling with structured error types
- CLI interface with proper argument validation
- Configuration validation and environment filtering
- Modular architecture with separation of concerns

## Limitations

- Requires PostgreSQL (uses PostgreSQL-specific template functionality)
- No built-in security features beyond PostgreSQL's native security
- No web interface - CLI only
- No real-time monitoring beyond basic health checks
- Configuration changes require restart

## Contributing

```bash
# Setup development environment
cargo build
cargo test
cargo clippy
cargo fmt --check
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

**Lionel Hamayon**
Email: <lionel.hamayon@evolution-digitale.fr>
GitHub: [@evoludigit](https://github.com/evoludigit)
