# DBFast - PostgreSQL Database Cloning Tool

[![Build Status](https://img.shields.io/badge/tests-40%20passing-green)](https://github.com/evoludigit/dbfast/tree/main/tests)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)

A PostgreSQL database template management and cloning tool written in Rust.

## What DBFast Actually Does

DBFast is a command-line tool that helps manage PostgreSQL database templates and create database clones using PostgreSQL's native `CREATE DATABASE WITH TEMPLATE` functionality.

### Core Features

- **Database Template Management**: Initialize and manage PostgreSQL database templates
- **Fast Database Cloning**: Clone databases using PostgreSQL's native template system
- **Environment-Aware Deployments**: Filter SQL files based on environment configurations
- **Remote Database Support**: Deploy templates to remote PostgreSQL instances
- **Change Detection**: Track file changes to determine when template rebuilds are needed
- **Basic Health Monitoring**: Monitor database connection health and basic metrics

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

## Usage

### Initialize Template

```bash
dbfast init --repo-dir ./db --template-name myapp_template
```

### Create Database Clone

```bash
dbfast seed --output test_db_1
dbfast seed --output test_db_2 --with-seeds
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

## Testing

Run the test suite:

```bash
cargo test
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

### Performance

- Database clones use PostgreSQL's native `CREATE DATABASE WITH TEMPLATE` command
- Connection pooling for database operations
- File change detection using xxHash for incremental updates
- Async/await throughout for non-blocking operations

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
