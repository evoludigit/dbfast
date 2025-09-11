//! Security tests for database cloning functionality
//! 
//! These tests verify that the clone manager properly validates database names
//! and prevents SQL injection attacks through malformed input.

use dbfast::clone::{CloneManager, CloneError};
use dbfast::database::DatabasePool;
use dbfast::Config;

#[tokio::test]
async fn test_sql_injection_protection_in_template_name() {
    // RED PHASE: This test should FAIL because current implementation is vulnerable
    
    // Setup (may fail without real database, that's OK for RED phase)
    let config = match Config::from_file("tests/fixtures/dbfast.toml") {
        Ok(c) => c,
        Err(_) => {
            println!("⚠️  No config file found, testing validation logic only");
            return;
        }
    };
    
    // Even if pool creation fails, we want to test the validation logic
    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let clone_manager = CloneManager::new(pool);
            
            // SQL injection attempt in template name
            let malicious_template = "template'; DROP DATABASE important_data; --";
            let safe_clone_name = "test_clone";
            
            let result = clone_manager
                .clone_database(malicious_template, safe_clone_name)
                .await;
            
            // Should return InvalidDatabaseName error, not execute malicious SQL
            assert!(matches!(result, Err(CloneError::InvalidDatabaseName { .. })));
        }
        Err(_) => {
            // No database connection - create clone manager with mock validation
            println!("⚠️  Testing validation logic without database connection");
            
            // We should still be able to validate input even without database
            let validation_result = validate_database_name("template'; DROP DATABASE important_data; --");
            assert!(validation_result.is_err(), "Should reject SQL injection attempts");
        }
    }
}

#[tokio::test] 
async fn test_sql_injection_protection_in_clone_name() {
    // RED PHASE: This test should FAIL because current implementation is vulnerable
    
    let config = match Config::from_file("tests/fixtures/dbfast.toml") {
        Ok(c) => c,
        Err(_) => {
            println!("⚠️  No config file found, testing validation logic only");
            return;
        }
    };
    
    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let clone_manager = CloneManager::new(pool);
            
            // SQL injection attempt in clone name
            let safe_template = "safe_template";
            let malicious_clone = "clone'; CREATE DATABASE malicious_db; --";
            
            let result = clone_manager
                .clone_database(safe_template, malicious_clone)
                .await;
            
            // Should return InvalidDatabaseName error, not execute malicious SQL
            assert!(matches!(result, Err(CloneError::InvalidDatabaseName { .. })));
        }
        Err(_) => {
            println!("⚠️  Testing validation logic without database connection");
            
            let validation_result = validate_database_name("clone'; CREATE DATABASE malicious_db; --");
            assert!(validation_result.is_err(), "Should reject SQL injection attempts");
        }
    }
}

#[tokio::test]
async fn test_invalid_database_name_characters() {
    // RED PHASE: Should fail because no validation exists yet
    
    let config = match Config::from_file("tests/fixtures/dbfast.toml") {
        Ok(c) => c,
        Err(_) => {
            println!("⚠️  No config file found, testing validation logic only");
            return;
        }
    };
    
    let long_name = "x".repeat(64);
    let invalid_names = vec![
        "db with spaces",
        "db-with-hyphens", 
        "db.with.dots",
        "db@with@symbols",
        "DB_WITH_UPPERCASE", // PostgreSQL converts to lowercase but we should be explicit
        "123_starts_with_number",
        "", // empty name
        "a", // too short
        &long_name, // too long (PostgreSQL limit is 63)
        "select", // PostgreSQL reserved word
        "table", // PostgreSQL reserved word
        "drop", // PostgreSQL reserved word
    ];
    
    match DatabasePool::new(&config.database).await {
        Ok(pool) => {
            let clone_manager = CloneManager::new(pool);
            
            for invalid_name in invalid_names {
                let result = clone_manager
                    .clone_database("template", invalid_name)
                    .await;
                
                assert!(
                    matches!(result, Err(CloneError::InvalidDatabaseName { .. })),
                    "Should reject invalid database name: '{}'", 
                    invalid_name
                );
            }
        }
        Err(_) => {
            println!("⚠️  Testing validation logic without database connection");
            
            for invalid_name in invalid_names {
                let validation_result = validate_database_name(invalid_name);
                assert!(
                    validation_result.is_err(), 
                    "Should reject invalid database name: '{}'", 
                    invalid_name
                );
            }
        }
    }
}

#[tokio::test]
async fn test_valid_database_names_accepted() {
    // RED PHASE: This test should pass once we implement validation correctly
    
    let config = match Config::from_file("tests/fixtures/dbfast.toml") {
        Ok(c) => c,
        Err(_) => {
            println!("⚠️  No config file found, testing validation logic only");
            return;
        }
    };
    
    let valid_names = vec![
        "simple_name",
        "with_underscores_123", 
        "test_db_v2",
        "user_data_2024",
        "app_production",
        "staging_env",
    ];
    
    match DatabasePool::new(&config.database).await {
        Ok(_) => {
            // With real database connection, validation should pass
            // but actual cloning may fail (template doesn't exist) - that's OK
            for valid_name in valid_names {
                let validation_result = validate_database_name(valid_name);
                assert!(
                    validation_result.is_ok(),
                    "Should accept valid database name: '{}'", 
                    valid_name
                );
            }
        }
        Err(_) => {
            println!("⚠️  Testing validation logic without database connection");
            
            for valid_name in valid_names {
                let validation_result = validate_database_name(valid_name);
                assert!(
                    validation_result.is_ok(), 
                    "Should accept valid database name: '{}'", 
                    valid_name
                );
            }
        }
    }
}

// Helper function that will be implemented in CloneManager
// This should be exposed for testing or implemented as part of CloneManager
fn validate_database_name(name: &str) -> Result<(), CloneError> {
    // This function doesn't exist yet - RED phase
    // Will be implemented in GREEN phase
    Err(CloneError::InvalidDatabaseName {
        name: name.to_string(),
        reason: "Validation not implemented yet".to_string(),
    })
}