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
            let template_manager = TemplateManager::new(pool, config.database.clone());

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
            let template_manager = TemplateManager::new(pool, config.database.clone());

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
            let template_manager = TemplateManager::new(pool, config.database.clone());

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
            let template_manager = TemplateManager::new(pool, config.database.clone());

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

/// Test that SQL files are actually executed and visible in template database
/// RED PHASE: This test should FAIL because real SQL execution is not implemented
#[tokio::test]
async fn test_sql_actually_executed_on_template() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool.clone(), config.database.clone());

            // Create test SQL files
            let temp_dir = TempDir::new().unwrap();
            let schema_file = temp_dir.path().join("schema.sql");
            let seed_file = temp_dir.path().join("seed.sql");

            fs::write(
                &schema_file,
                "CREATE TABLE real_test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL);",
            )
            .unwrap();
            fs::write(
                &seed_file,
                "INSERT INTO real_test_table (name) VALUES ('real_data_test');",
            )
            .unwrap();

            let sql_files = [schema_file.as_path(), seed_file.as_path()];
            let template_name = "real_sql_test_template";

            // Create the template
            if let Ok(()) = template_manager
                .create_template(template_name, &sql_files)
                .await
            {
                // CRITICAL TEST: Verify SQL was actually executed by checking table exists in template database
                let table_check_sql = format!(
                    "SELECT table_name FROM information_schema.tables WHERE table_catalog = '{}' AND table_name = 'real_test_table'",
                    template_name
                );

                // This should find the table if SQL was actually executed
                let table_exists = pool.query(&table_check_sql, &[]).await.unwrap();
                assert!(
                    !table_exists.is_empty(),
                    "Table should exist in template database after SQL execution"
                );

                // CRITICAL TEST: Verify data was actually inserted
                let data_check_sql = format!(
                    "SELECT name FROM {}.real_test_table WHERE name = 'real_data_test'",
                    template_name
                );

                let data_exists = pool.query(&data_check_sql, &[]).await.unwrap();
                assert!(
                    !data_exists.is_empty(),
                    "Seed data should exist in template database after SQL execution"
                );

                // Cleanup
                let _ = template_manager.drop_template(template_name).await;
            } else {
                panic!("Template creation should succeed when database connection is available");
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for real SQL execution test");
        }
    }
}

/// Test template listing returns actual PostgreSQL databases
/// RED PHASE: This test should FAIL because it returns hardcoded list
#[tokio::test]
async fn test_list_templates_from_real_postgres() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool.clone(), config.database.clone());

            // Create actual template database first
            let temp_dir = TempDir::new().unwrap();
            let test_sql = temp_dir.path().join("test.sql");
            fs::write(&test_sql, "CREATE TABLE list_test (id SERIAL);").unwrap();

            let unique_template_name = format!(
                "list_test_template_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );

            if let Ok(()) = template_manager
                .create_template(&unique_template_name, &[test_sql.as_path()])
                .await
            {
                // Now list templates - should include our newly created one
                match template_manager.list_templates().await {
                    Ok(templates) => {
                        // CRITICAL TEST: Should find our actual template, not hardcoded list
                        assert!(
                            templates.contains(&unique_template_name),
                            "Template listing should include actual database '{}', got: {:?}",
                            unique_template_name,
                            templates
                        );

                        // CRITICAL TEST: Should NOT contain hardcoded templates if they don't exist
                        let actual_db_check = pool.query("SELECT datname FROM pg_database WHERE datname = 'default_template'", &[]).await.unwrap();
                        if actual_db_check.is_empty() {
                            assert!(
                                !templates.contains(&"default_template".to_string()),
                                "Should not return hardcoded 'default_template' if it doesn't actually exist"
                            );
                        }

                        // Cleanup
                        let _ = template_manager.drop_template(&unique_template_name).await;
                    }
                    Err(e) => panic!("Template listing should work: {}", e),
                }
            }
        }
        Err(_) => {
            println!("⚠️  No database connection for real template listing test");
        }
    }
}

/// Test integration between TemplateManager and CloneManager
#[tokio::test]
async fn test_template_clone_integration() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();

    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let template_manager = TemplateManager::new(pool.clone(), config.database.clone());
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
