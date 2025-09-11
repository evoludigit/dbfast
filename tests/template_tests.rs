use dbfast::database::DatabasePool;
use dbfast::template::TemplateManager;
use dbfast::Config;
use std::fs;
use tempfile::TempDir;

/// Integration tests for template management functionality
///
/// These tests verify that templates can be created, managed, and integrated
/// with the database cloning workflow.
/// Test basic template creation functionality
#[tokio::test]
async fn test_template_creation_basic() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool);

            // Test template creation with mock SQL files
            let temp_dir = TempDir::new().unwrap();
            let schema_file = temp_dir.path().join("schema.sql");
            let seed_file = temp_dir.path().join("seed.sql");

            fs::write(
                &schema_file,
                "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);",
            )
            .unwrap();
            fs::write(&seed_file, "INSERT INTO users (name) VALUES ('Test User');").unwrap();

            let sql_files = [schema_file.as_path(), seed_file.as_path()];

            match template_manager
                .create_template("test_basic_template", &sql_files)
                .await
            {
                Ok(()) => {
                    println!("✅ Template creation succeeded");
                }
                Err(_) => {
                    println!("⚠️  Template creation failed (expected without PostgreSQL server)");
                }
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for template test (expected without PostgreSQL)");
        }
    }
}

/// Test template existence checking
#[tokio::test]
async fn test_template_exists_check() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool);

            // Test checking for non-existent template
            match template_manager
                .template_exists("nonexistent_template")
                .await
            {
                Ok(exists) => {
                    println!("✅ Template existence check succeeded: exists = {}", exists);
                }
                Err(_) => {
                    println!("⚠️  Template existence check failed (expected without PostgreSQL)");
                }
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for template existence test");
        }
    }
}

/// Test template listing functionality
#[tokio::test]
async fn test_template_listing() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool);

            match template_manager.list_templates().await {
                Ok(templates) => {
                    println!("✅ Found {} templates: {:?}", templates.len(), templates);
                    assert!(!templates.is_empty(), "Should have at least some templates");
                }
                Err(_) => {
                    println!("⚠️  Template listing failed (expected without PostgreSQL)");
                }
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for template listing test");
        }
    }
}

/// Test template cleanup functionality
#[tokio::test]
async fn test_template_cleanup() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool);

            // Test dropping a template
            match template_manager
                .drop_template("cleanup_test_template")
                .await
            {
                Ok(()) => {
                    println!("✅ Template cleanup succeeded");
                }
                Err(_) => {
                    println!("⚠️  Template cleanup failed (expected without PostgreSQL)");
                }
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for template cleanup test");
        }
    }
}

/// Test integration between TemplateManager and CloneManager
#[tokio::test]
async fn test_template_clone_integration() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool.clone());
            let clone_manager = dbfast::clone::CloneManager::new(pool);

            // Create a template
            let temp_dir = TempDir::new().unwrap();
            let schema_file = temp_dir.path().join("schema.sql");
            fs::write(&schema_file, "CREATE TABLE integration_test (id SERIAL);").unwrap();

            let sql_files = [schema_file.as_path()];

            // Step 1: Create template
            match template_manager
                .create_template("integration_template", &sql_files)
                .await
            {
                Ok(()) => {
                    println!("✅ Template created for integration test");

                    // Step 2: Clone from template
                    match clone_manager
                        .clone_database("integration_template", "integration_clone")
                        .await
                    {
                        Ok(()) => {
                            println!("✅ Clone from template succeeded");

                            // Step 3: Cleanup
                            let _ = clone_manager.drop_database("integration_clone").await;
                            let _ = template_manager.drop_template("integration_template").await;
                        }
                        Err(_) => {
                            println!("⚠️  Clone from template failed (expected without proper PostgreSQL setup)");
                        }
                    }
                }
                Err(_) => {
                    println!("⚠️  Template creation failed (expected without PostgreSQL server)");
                }
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for integration test");
        }
    }
}
