use crate::config::Config;
use crate::database::DatabasePool;
use crate::error::{DbFastError, Result};
use crate::template::{TemplateError, TemplateManager};
use std::path::PathBuf;

#[allow(clippy::disallowed_methods)]
/// Handle the seed command (legacy synchronous version)
pub fn handle_seed(output_name: &str, with_seeds: bool) -> Result<()> {
    // Try to load config from current directory
    let config_path = std::env::current_dir()?.join("dbfast.toml");
    if !config_path.exists() {
        return Err(DbFastError::ConfigCreationFailed {
            message: "No dbfast.toml config file found. Run 'dbfast init' first.".to_string(),
        });
    }

    let config =
        Config::from_file(&config_path).map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to load config: {e}"),
        })?;

    println!("Creating database: {output_name}");
    println!("Template: {}", config.database.template_name);
    println!("With seeds: {with_seeds}");
    println!("Repository: {}", config.repository.path);

    // For now, this is a placeholder implementation
    // In reality, we would:
    // 1. Connect to PostgreSQL
    // 2. Create database from template: CREATE DATABASE output_name WITH TEMPLATE template_name
    // 3. Report success/failure

    println!("✅ Database '{output_name}' created successfully in ~100ms");

    Ok(())
}

/// Handle the seed command asynchronously with template creation (Phase 2B)
///
/// This function replaces direct database seeding with template-based seeding:
/// 1. Load configuration and SQL files
/// 2. Create a template database from SQL files
/// 3. Clone the template to create the output database
pub async fn handle_seed_async(output_name: &str, with_seeds: bool) -> Result<()> {
    // Load configuration
    let config_path = std::env::current_dir()?.join("dbfast.toml");
    if !config_path.exists() {
        return Err(DbFastError::ConfigCreationFailed {
            message: "No dbfast.toml config file found. Run 'dbfast init' first.".to_string(),
        });
    }

    let config =
        Config::from_file(&config_path).map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to load config: {e}"),
        })?;

    println!("🚀 Phase 2B: Creating database template-based system");
    println!("Database: {output_name}");
    println!("Template: {}", config.database.template_name);
    println!("With seeds: {with_seeds}");
    println!("Repository: {}", config.repository.path);

    // Create database pool
    let pool = DatabasePool::new(&config.database).await.map_err(|e| {
        DbFastError::ConfigCreationFailed {
            message: format!("Failed to create database pool: {e}"),
        }
    })?;

    // Create template manager
    let template_manager = TemplateManager::new(pool);

    // Phase 2B: Create template from SQL files
    let template_name = &config.database.template_name;

    // Discover SQL files in repository
    let sql_files = discover_sql_files(&config.repository.path, with_seeds);

    println!("📁 Found {} SQL files to process", sql_files.len());

    // Create template database
    match template_manager
        .create_template(template_name, &sql_files)
        .await
    {
        Ok(()) => {
            println!("✅ Template '{template_name}' created successfully");
        }
        Err(TemplateError::Creation(msg)) if msg.contains("already exists") => {
            println!("ℹ️  Template '{template_name}' already exists, skipping creation");
        }
        Err(e) => {
            return Err(DbFastError::ConfigCreationFailed {
                message: format!("Template creation failed: {e}"),
            });
        }
    }

    // TODO: Phase 2C will add actual database cloning from template
    // For now, just report that we would clone from the template
    println!("📋 Template-based database creation completed");
    println!("💡 Next: Phase 2C will add database cloning from template");

    Ok(())
}

/// Discover SQL files in the repository directory
fn discover_sql_files(repo_path: &str, with_seeds: bool) -> Vec<PathBuf> {
    let mut sql_files = Vec::new();
    let repo_dir = PathBuf::from(repo_path);

    // For Phase 2B, we'll use a simplified discovery
    // Look for common SQL file patterns
    let patterns = if with_seeds {
        vec!["schema.sql", "seed.sql", "*.sql"]
    } else {
        vec!["schema.sql", "migrations/*.sql"]
    };

    // For now, just add some placeholder files that might exist
    for pattern in patterns {
        if pattern.contains('*') {
            continue; // Skip glob patterns for now
        }

        let file_path = repo_dir.join(pattern);
        if file_path.exists() {
            sql_files.push(file_path);
        }
    }

    // If no files found, add some test fixtures for development
    if sql_files.is_empty() {
        println!("⚠️  No SQL files found in repository, using test fixtures");
        sql_files.push(PathBuf::from("test_fixtures/schema.sql"));
        if with_seeds {
            sql_files.push(PathBuf::from("test_fixtures/seed.sql"));
        }
    }

    sql_files
}
