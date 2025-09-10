# DBFast ⚡ - Lightning-Fast PostgreSQL Database Seeding

*Transform database fixtures from a 60-second bottleneck into a 100ms delight*

## What is DBFast?

DBFast is a PostgreSQL-native database seeding tool that creates "template" databases once, then clones them instantly using `CREATE DATABASE WITH TEMPLATE`. Perfect for fast test setup and environment deployment.

**Core Magic**: Build once, clone infinitely in ~100ms

## Quick Start

```bash
# 1. Initialize from your existing database repository
dbfast init --repo-dir ./db --template-name myapp_template

# 2. Get a seeded test database instantly
dbfast seed --output test_db_$(date +%s)

# 3. Deploy to remote environments safely
dbfast deploy --remote staging --env staging --confirm
```

## Key Features

### 🚀 **Blazing Fast**
- **~100ms database clones** using PostgreSQL's native templating
- **Smart change detection** - only rebuilds when SQL files change
- **PostgreSQL 17 ZSTD compression** for faster transfers

### 🎯 **Environment-Aware**
Deploy different files to different environments:
```toml
[environments.local]
include_directories = ["0_schema", "1_seed_common", "2_seed_backend"]

[environments.production]  
include_directories = ["0_schema", "6_migration"]
exclude_directories = ["1_seed_common", "2_seed_backend"]
```

### 🔒 **Production-Safe**
- **Atomic deployments** with automatic backups
- **Environment-specific filtering** prevents accidents
- **Rollback on failure** - your data stays safe

## Repository Structure

Works with structured database repositories:
```
db/
├── 0_schema/          # Tables, views, functions
├── 1_seed_common/     # Essential seed data  
├── 2_seed_backend/    # Backend-specific data
├── 6_migration/       # Production migrations
└── 99_finalize/       # Cleanup, grants
```

## Installation

```bash
# Build from source
git clone https://github.com/yourusername/dbfast
cd dbfast
cargo build --release

# Binary will be at target/release/dbfast
```

## Configuration

Create `dbfast.toml`:
```toml
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "myapp_template"

[repository]
path = "./db"
type = "structured"

[environments.local]
include_directories = ["0_schema", "1_seed_common", "2_seed_backend"]

[environments.production]
include_directories = ["0_schema", "6_migration"]
exclude_directories = ["1_seed_common", "2_seed_backend"]
```

## Commands

```bash
# Template management
dbfast init --repo-dir ./db --template-name myapp_template
dbfast rebuild --force
dbfast status

# Database seeding
dbfast seed --output test_db --with-seeds
dbfast seed --output test_schema  # Schema only

# Remote deployment  
dbfast remote add --name staging --url postgres://staging-server/db
dbfast deploy --remote staging --env staging --confirm

# Repository management
dbfast repo add --name shared --path ~/shared_db
dbfast repo sync --name myapp
```

## Real-World Impact

### Before DBFast 😢
```python
@pytest.fixture(scope="session")  
def database():
    # 🐌 Takes 30-60 seconds
    # 💥 Often hangs mysteriously  
    setup_complex_database_fixture()
```

### After DBFast 🎉
```python  
@pytest.fixture(scope="session")
def database():
    # ⚡ Takes <500ms, always works
    subprocess.run(["dbfast", "seed", "--output", "test_session"])
```

## Performance

- **File scanning**: ~5ms for 1000+ SQL files
- **Template detection**: ~0.5ms (cache hit)
- **Database clone**: **~100ms** 
- **Remote deployment**: ~2-3min with ZSTD compression

## Safety Features

- **Environment-specific rules** prevent production accidents
- **Automatic backups** before destructive operations  
- **Atomic deployments** via single transactions
- **Validation framework** ensures deployments work

## License

MIT License - see LICENSE file for details.