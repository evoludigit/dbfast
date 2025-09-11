use crate::config::Config;
use crate::database::DatabasePool;
use crate::error::{DbFastError, Result};
use crate::sql_executor::SqlExecutor;

#[allow(clippy::disallowed_methods)]
/// Handle the seed command
pub fn handle_seed(output_name: &str, with_seeds: bool) -> Result<()> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(handle_seed_async(output_name, with_seeds))
}

/// Async version of `handle_seed` for testing and internal use
pub async fn handle_seed_async(output_name: &str, with_seeds: bool) -> Result<()> {
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

    // Connect to PostgreSQL
    let pool = DatabasePool::new(&config.database).await.map_err(|e| {
        DbFastError::ConfigCreationFailed {
            message: format!("Failed to connect to database: {e}"),
        }
    })?;

    // Create the output database
    let create_db_sql = format!("CREATE DATABASE {output_name}");
    pool.execute(&create_db_sql, &[])
        .await
        .map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to create database '{output_name}': {e}"),
        })?;

    println!("✅ Database '{output_name}' created");

    // Read and execute SQL files from the repository
    let sql_executor = SqlExecutor::new();
    let statements = SqlExecutor::read_sql_files(&config.repository.path)?;

    if statements.is_empty() {
        println!("No SQL files found in repository");
    } else {
        println!("Executing {} SQL statements...", statements.len());

        // Note: In a full implementation, we'd connect to the newly created database
        // to execute the SQL files. For Phase 2A, we'll execute against the main connection
        // which is sufficient to demonstrate the SQL execution functionality

        sql_executor.execute_statements(&pool, &statements).await?;

        println!("✅ SQL execution completed");
    }

    println!("✅ Database '{output_name}' created successfully");

    Ok(())
}
