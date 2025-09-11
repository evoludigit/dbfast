use dbfast::{Config, DatabasePool};
use std::path::PathBuf;
use tempfile::TempDir;

/// Basic test for template creation functionality
/// This test will fail initially because TemplateManager doesn't exist yet
#[tokio::test]
async fn test_template_creation() {
    // Create temporary directory for test config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");

    // Create minimal test config
    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "blog_template"

[repository]
path = "test_fixtures/blog"
type = "structured"
"#;
    std::fs::write(&config_path, config_content).unwrap();

    // Load config
    let config = Config::from_file(&config_path).unwrap();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // This should fail because TemplateManager doesn't exist yet - RED phase
    let template_manager = dbfast::TemplateManager::new(pool);

    let sql_files = vec![
        PathBuf::from("test_fixtures/schema.sql"),
        PathBuf::from("test_fixtures/seed.sql"),
    ];

    let result = template_manager
        .create_template("blog_template", &sql_files)
        .await;
    assert!(result.is_ok());
}

/// Test that we can validate if a template exists
#[tokio::test]
async fn test_template_exists() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");

    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "blog_template"

[repository]
path = "test_fixtures/blog"
type = "structured"
"#;
    std::fs::write(&config_path, config_content).unwrap();

    let config = Config::from_file(&config_path).unwrap();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // This should fail because template_exists function doesn't exist yet - RED phase
    let exists = dbfast::template_exists(&pool, "blog_template")
        .await
        .unwrap();
    assert!(!exists); // Should be false for non-existent template
}

/// Test template metadata tracking
#[tokio::test]
async fn test_template_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dbfast.toml");

    let config_content = r#"
[database]
host = "localhost"
port = 5432
user = "postgres"
password_env = "POSTGRES_PASSWORD"
template_name = "blog_template"

[repository]
path = "test_fixtures/blog"
type = "structured"
"#;
    std::fs::write(&config_path, config_content).unwrap();

    let config = Config::from_file(&config_path).unwrap();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // This should fail because TemplateManager and metadata methods don't exist yet - RED phase
    let template_manager = dbfast::TemplateManager::new(pool);
    let metadata = template_manager
        .get_template_metadata("blog_template")
        .await
        .unwrap();
    assert!(metadata.is_none()); // Should be None for non-existent template
}
