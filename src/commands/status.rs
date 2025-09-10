use crate::config::Config;
use crate::error::{DbFastError, Result};
use std::path::Path;

#[allow(clippy::disallowed_methods)]

/// Handle the status command using current working directory
pub fn handle_status() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    handle_status_in_dir(&current_dir)
}

/// Handle the status command in a specific directory  
pub fn handle_status_in_dir(dir: &Path) -> Result<()> {
    println!("🔍 DBFast Status");
    println!("================");

    // Check for config file
    let config_path = dir.join("dbfast.toml");
    if !config_path.exists() {
        println!("❌ Configuration: No dbfast.toml found");
        println!("   Run 'dbfast init --repo-dir <path> --template-name <name>' to initialize");
        return Ok(());
    }

    println!("✅ Configuration: {}", config_path.display());

    // Load and display config information
    let config =
        Config::from_file(&config_path).map_err(|e| DbFastError::ConfigCreationFailed {
            message: format!("Failed to load config: {}", e),
        })?;

    println!("\n📋 Configuration Details:");
    println!(
        "   Database Host: {}:{}",
        config.database.host, config.database.port
    );
    println!("   Database User: {}", config.database.user);
    println!("   Template Name: {}", config.database.template_name);
    println!(
        "   Repository: {} ({})",
        config.repository.path, config.repository.repo_type
    );

    // Check if repository path exists
    let repo_path = Path::new(&config.repository.path);
    if repo_path.exists() {
        println!("✅ Repository: Directory exists");

        // Check for common directories
        let common_dirs = ["0_schema", "1_seed_common", "2_seed_backend", "6_migration"];
        for dir_name in &common_dirs {
            let dir_path = repo_path.join(dir_name);
            if dir_path.exists() {
                println!("   📁 {}/", dir_name);
            }
        }
    } else {
        println!(
            "❌ Repository: Directory not found at {}",
            config.repository.path
        );
    }

    // Show environment configurations
    if !config.environments.is_empty() {
        println!("\n🌍 Environments:");
        for (name, env) in &config.environments {
            println!("   {} - includes: {:?}", name, env.include_directories);
            if !env.exclude_directories.is_empty() {
                println!("     excludes: {:?}", env.exclude_directories);
            }
        }
    }

    println!("\n✨ Ready to seed databases!");

    Ok(())
}
