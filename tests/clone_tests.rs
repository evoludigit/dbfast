use dbfast::database::DatabasePool;
use dbfast::Config;
use std::time::Instant;

/// Integration test for database cloning functionality
/// 
/// This test verifies that we can clone a PostgreSQL database using
/// the `CREATE DATABASE WITH TEMPLATE` command in under 100ms
#[tokio::test]
async fn test_database_cloning_basic() {
    // Load test configuration
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // Test database cloning with performance measurement
    let start = Instant::now();
    
    // This should clone a database from a template
    // For now, this will fail because CloneManager doesn't exist yet
    let clone_manager = dbfast::clone::CloneManager::new(pool);
    let result = clone_manager.clone_database("blog_template", "test_clone_1").await;
    
    let clone_duration = start.elapsed();

    // Verify the clone operation succeeded
    assert!(result.is_ok(), "Database cloning should succeed");
    
    // Verify performance target: <100ms for small databases
    assert!(
        clone_duration.as_millis() < 100,
        "Clone should complete in <100ms, took {}ms",
        clone_duration.as_millis()
    );
}

/// Test that cloned databases are independent from templates
#[tokio::test]
async fn test_database_clone_independence() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    let clone_manager = dbfast::clone::CloneManager::new(pool.clone());
    
    // Clone database
    clone_manager.clone_database("blog_template", "independence_test").await.unwrap();
    
    // Modify data in clone (this should not affect the template)
    let conn = pool.get().await.unwrap();
    let modify_result = conn.execute_on_database(
        "independence_test",
        "CREATE TABLE test_independence (id SERIAL PRIMARY KEY, name TEXT)",
        &[]
    ).await;
    
    assert!(modify_result.is_ok(), "Should be able to modify clone independently");
    
    // Verify template is unchanged (should not have the new table)
    let template_result = conn.check_table_exists("blog_template", "test_independence").await;
    assert!(template_result.is_ok());
    assert!(!template_result.unwrap(), "Template should not have the test table");
}

/// Test clone cleanup and database removal
#[tokio::test] 
async fn test_database_clone_cleanup() {
    let config = Config::from_file("tests/fixtures/dbfast.toml").unwrap();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    let clone_manager = dbfast::clone::CloneManager::new(pool.clone());
    
    // Create a clone
    clone_manager.clone_database("blog_template", "cleanup_test").await.unwrap();
    
    // Verify clone exists
    let conn = pool.get().await.unwrap();
    let exists_before = conn.database_exists("cleanup_test").await.unwrap();
    assert!(exists_before, "Clone should exist after creation");
    
    // Clean up the clone
    let cleanup_result = clone_manager.drop_database("cleanup_test").await;
    assert!(cleanup_result.is_ok(), "Cleanup should succeed");
    
    // Verify clone is removed
    let exists_after = conn.database_exists("cleanup_test").await.unwrap();
    assert!(!exists_after, "Clone should not exist after cleanup");
}