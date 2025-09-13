use crate::clone::CloneManager;
use crate::config::Config;
use crate::database::DatabasePool;
use crate::error::{DbFastError, Result};
use crate::scanner::FileScanner;
use crate::template::TemplateManager;
use std::path::PathBuf;
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
#[allow(clippy::too_many_lines)] // Main async function with complex workflow
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
    let pool = DatabasePool::from_config(&config.database)
        .await
        .map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to connect to database: {e}"),
        })?;

    // Step 2: Smart template creation with change detection
    let repo_path = PathBuf::from(&config.repository.path);
    println!("🔍 Scanning for SQL files and checking template state...");

    let template_manager = TemplateManager::new_with_change_detection(
        pool.clone(),
        config.database.clone(),
        repo_path.clone(),
    );

    // Scan for SQL files
    let scanner = FileScanner::new(&repo_path);
    let scanned_files = scanner
        .scan()
        .map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to scan SQL files: {e}"),
        })?;

    if scanned_files.is_empty() {
        println!(
            "⚠️  No SQL files found in repository path: {}",
            repo_path.display()
        );
        return Err(DbFastError::ConfigCreationFailed {
            message: "No SQL files found. Please check your repository path.".to_string(),
        });
    }

    println!("📄 Found {} SQL files", scanned_files.len());

    // Convert scanned files to paths for template creation
    let sql_file_paths: Vec<PathBuf> = scanned_files.into_iter().map(|f| f.path).collect();

    // Smart template creation - only rebuilds if needed
    let template_start = Instant::now();
    let template_was_created = template_manager
        .smart_create_template(&config.database.template_name, &sql_file_paths)
        .await
        .map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to create/update template: {e}"),
        })?;

    let template_duration = template_start.elapsed();

    if template_was_created {
        println!(
            "✅ Template '{}' created/updated in {}ms",
            config.database.template_name,
            template_duration.as_millis()
        );
    } else {
        println!(
            "⏩ Template '{}' is up to date ({}ms check)",
            config.database.template_name,
            template_duration.as_millis()
        );
    }

    // Step 3: Create CloneManager and clone database from template
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

    // Step 4: Report success with performance metrics
    println!("✅ Database '{output_name}' created successfully!");
    if template_was_created {
        println!("🏗️  Template creation: {}ms", template_duration.as_millis());
    } else {
        println!("🔍 Template check: {}ms", template_duration.as_millis());
    }
    println!("⚡ Clone operation: {}ms", clone_duration.as_millis());
    println!("🎯 Total time: {}ms", total_duration.as_millis());

    // Verify performance target
    if clone_duration.as_millis() < 100 {
        println!("🏆 Performance target achieved: <100ms cloning!");
    } else if clone_duration.as_millis() < 500 {
        println!(
            "⚠️  Performance warning: {}ms (target: <100ms)",
            clone_duration.as_millis()
        );
    } else {
        println!(
            "🚨 Performance issue: {}ms (target: <100ms)",
            clone_duration.as_millis()
        );
    }

    Ok(())
}
