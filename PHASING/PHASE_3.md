# Phase 3 - Environment Filtering

**Goal**: Implement environment-specific file filtering to deploy different SQL files to different environments.

## TDD Approach

Write tests for various filtering scenarios first, then implement the filtering engine.

## Core Concept

The same repository can deploy different files to different environments:

- **Local**: Schema + backend seeds + debug data
- **Staging**: Schema + frontend data (no debug/test files)  
- **Production**: Schema + migrations only (no seed data)

## Deliverables

### 1. Environment Configuration
- [ ] Parse environment configs from `dbfast.toml`
- [ ] Support include/exclude directories and files
- [ ] Glob pattern matching for file filtering
- [ ] Environment validation

**Test**: Load environment configs, validate patterns work

### 2. File Filtering Engine
- [ ] Apply directory filters (include then exclude)
- [ ] Apply file filters (include then exclude)
- [ ] Resolve glob patterns efficiently
- [ ] Maintain execution order within filtered results

**Test**: Filter produces correct file lists for each environment

### 3. Environment Commands
- [ ] `dbfast environments` - list configured environments
- [ ] `dbfast validate-env --env <name>` - validate environment config
- [ ] `--env` flag for seed/deploy commands

**Test**: Environment commands work correctly

### 4. Template per Environment
- [ ] Build different templates for different environments
- [ ] Cache multiple templates efficiently
- [ ] Handle template naming (template_local, template_production)

**Test**: Each environment produces different template content

## Success Criteria

1. **Correct filtering**: Each environment includes only intended files
2. **Performance**: Filtering is fast even with many files
3. **Validation**: Can detect invalid environment configurations
4. **Safety**: Production never accidentally gets dev/test files
5. **Flexibility**: Glob patterns work as expected

## Key Files to Add/Modify

```
src/
├── environment.rs       # Environment filtering logic
├── filter.rs           # File filtering engine
├── glob_matcher.rs     # Glob pattern matching
└── template.rs         # Modify for multi-environment templates

tests/
├── environment_tests.rs # Environment filtering tests
└── fixtures/
    ├── environments/   # Test environment configs
    └── sql/           # Test SQL files with different purposes
        ├── 0_schema/
        ├── 1_seed_common/
        ├── 2_seed_backend/
        ├── debug_*.sql
        └── prod_*.sql
```

## Core Data Structures

### Environment Configuration
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub include_directories: Option<Vec<String>>,
    pub exclude_directories: Option<Vec<String>>,
    pub include_files: Option<Vec<String>>,
    pub exclude_files: Option<Vec<String>>,
}

impl EnvironmentConfig {
    pub fn filter_files(&self, all_files: &[PathBuf]) -> Result<Vec<PathBuf>>;
    pub fn validate(&self, base_path: &Path) -> Result<()>;
}
```

### File Filter Engine
```rust
pub struct FileFilter {
    include_dirs: GlobSet,
    exclude_dirs: GlobSet,
    include_files: GlobSet,
    exclude_files: GlobSet,
}

impl FileFilter {
    pub fn new(config: &EnvironmentConfig) -> Result<Self>;
    pub fn matches(&self, file_path: &Path) -> bool;
    pub fn filter_files(&self, files: &[PathBuf]) -> Vec<PathBuf>;
}
```

## Example Tests to Write

### Basic Filtering Test
```rust
#[test]
fn test_environment_filtering() {
    let config = EnvironmentConfig {
        name: "local".to_string(),
        include_directories: Some(vec!["0_schema".to_string(), "1_seed_common".to_string()]),
        exclude_directories: Some(vec!["6_migration".to_string()]),
        exclude_files: Some(vec!["**/prod_*.sql".to_string()]),
        ..Default::default()
    };
    
    let all_files = vec![
        PathBuf::from("0_schema/tables.sql"),
        PathBuf::from("1_seed_common/users.sql"),
        PathBuf::from("6_migration/001_add_column.sql"),
        PathBuf::from("1_seed_common/prod_data.sql"),
    ];
    
    let filtered = config.filter_files(&all_files).unwrap();
    
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&PathBuf::from("0_schema/tables.sql")));
    assert!(filtered.contains(&PathBuf::from("1_seed_common/users.sql")));
    assert!(!filtered.contains(&PathBuf::from("6_migration/001_add_column.sql")));
    assert!(!filtered.contains(&PathBuf::from("1_seed_common/prod_data.sql")));
}
```

### Production Safety Test
```rust
#[test]
fn test_production_safety() {
    let prod_config = EnvironmentConfig {
        name: "production".to_string(),
        include_directories: Some(vec!["0_schema".to_string(), "6_migration".to_string()]),
        exclude_directories: Some(vec!["1_seed_common".to_string(), "2_seed_backend".to_string()]),
        exclude_files: Some(vec!["**/test_*.sql".to_string(), "**/dev_*.sql".to_string()]),
        ..Default::default()
    };
    
    let test_files = vec![
        PathBuf::from("0_schema/tables.sql"),        // Should include
        PathBuf::from("6_migration/001_prod.sql"),   // Should include  
        PathBuf::from("1_seed_common/users.sql"),    // Should exclude (directory)
        PathBuf::from("0_schema/test_data.sql"),     // Should exclude (file pattern)
    ];
    
    let filtered = prod_config.filter_files(&test_files).unwrap();
    
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&PathBuf::from("0_schema/tables.sql")));
    assert!(filtered.contains(&PathBuf::from("6_migration/001_prod.sql")));
}
```

### Multi-Environment Template Test
```rust
#[tokio::test]
async fn test_environment_specific_templates() {
    let manager = TemplateManager::new(pool);
    
    // Build local template (with seeds)
    let local_files = env_config_local.filter_files(&all_sql_files).unwrap();
    manager.build_template("test_local", &local_files).await.unwrap();
    
    // Build production template (schema only)
    let prod_files = env_config_prod.filter_files(&all_sql_files).unwrap();
    manager.build_template("test_prod", &prod_files).await.unwrap();
    
    // Verify different content
    let local_tables = count_tables("test_local").await;
    let prod_tables = count_tables("test_prod").await;
    
    assert!(local_tables > prod_tables); // Local has seed data
}
```

## Filter Resolution Logic

1. **Start with all SQL files** from repository scan
2. **Apply directory include filter**: If specified, only keep files in included directories  
3. **Apply directory exclude filter**: Remove files from excluded directories
4. **Apply file include filter**: Add any specifically included files
5. **Apply file exclude filter**: Remove any specifically excluded files (highest priority)

## Commands to Implement

### `dbfast environments`
```bash  
dbfast environments --verbose
```
Shows all configured environments and their file counts.

### `dbfast validate-env`
```bash
dbfast validate-env --env production
```
Validates environment configuration and shows which files would be included.

### Environment-aware seeding
```bash
dbfast seed --output test_local --env local
dbfast seed --output test_prod --env production  
```

## Phase 3 Complete When

- [ ] All environment filtering tests pass
- [ ] Can build different templates for different environments
- [ ] Production environment excludes all dev/test files
- [ ] `dbfast environments` shows configured environments
- [ ] `dbfast validate-env` validates configurations
- [ ] `--env` flag works with seed command
- [ ] File filtering is performant (handles 1000+ files quickly)

**Next Phase**: Remote deployment with environment safety