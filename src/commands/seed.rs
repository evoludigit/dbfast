use crate::clone::CloneManager;
use crate::config::Config;
use crate::database::DatabasePool;
use crate::error::{DbFastError, Result};
use std::time::Instant;

#[allow(clippy::disallowed_methods)]
/// Handle the seed command synchronously (wrapper for async implementation)
pub fn handle_seed(output_name: &str, with_seeds: bool) -> Result<()> {
    // Create a runtime for async operations
    let rt = tokio::runtime::Runtime::new().map_err(|e| DbFastError::ConfigCreationFailed {
        message: format!("Failed to create async runtime: {e}"),
    })?;
    
    rt.block_on(handle_seed_async(output_name, with_seeds))
}

/// Handle the seed command asynchronously with real database cloning
pub async fn handle_seed_async(output_name: &str, with_seeds: bool) -> Result<()> {
    let start = Instant::now();
    
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

    println!("🚀 Starting database creation...");
    println!("📊 Output database: {output_name}");
    println!("📋 Template: {}", config.database.template_name);
    println!("🌱 With seeds: {with_seeds}");
    println!("📁 Repository: {}", config.repository.path);

    // Step 1: Create database connection pool
    println!("🔌 Connecting to PostgreSQL...");
    let pool = DatabasePool::new(&config.database).await.map_err(|e| {
        DbFastError::ConfigCreationFailed {
            message: format!("Failed to connect to database: {e}"),
        }
    })?;

    // Step 2: Create CloneManager and clone database from template
    println!("⚡ Cloning database from template...");
    let clone_manager = CloneManager::new(pool);
    
    let clone_start = Instant::now();
    clone_manager
        .clone_database(&config.database.template_name, output_name)
        .await
        .map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to clone database: {e}"),
        })?;
    
    let clone_duration = clone_start.elapsed();
    let total_duration = start.elapsed();

    // Step 3: Report success with performance metrics
    println!("✅ Database '{}' created successfully!", output_name);
    println!("⚡ Clone operation: {}ms", clone_duration.as_millis());
    println!("🎯 Total time: {}ms", total_duration.as_millis());
    
    // Verify performance target
    if clone_duration.as_millis() < 100 {
        println!("🏆 Performance target achieved: <100ms cloning!");
    } else if clone_duration.as_millis() < 500 {
        println!("⚠️  Performance warning: {}ms (target: <100ms)", clone_duration.as_millis());
    } else {
        println!("🚨 Performance issue: {}ms (target: <100ms)", clone_duration.as_millis());
    }

    Ok(())
}
