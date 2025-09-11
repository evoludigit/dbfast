use crate::config::Config;
use crate::error::{DbFastError, Result};
use crate::scanner::FileScanner;
use std::path::PathBuf;

/// Handle the environments command synchronously
pub fn handle_environments(verbose: bool) -> Result<()> {
    // Try to load config from current directory
    let config_path = std::env::current_dir()?.join("dbfast.toml");
    if !config_path.exists() {
        return Err(DbFastError::ConfigCreationFailed {
            message: "No dbfast.toml config file found. Run 'dbfast init' first.".to_string(),
        });
    }

    let config = Config::from_file(&config_path).map_err(|e| DbFastError::ConfigCreationFailed {
        message: format!("Failed to load config: {e}"),
    })?;

    println!("🌍 Configured Environments:");
    println!();

    // Get repository path for file scanning
    let repo_path = PathBuf::from(&config.repository.path);
    let scanner = FileScanner::new(&repo_path);

    for (env_name, environment) in &config.environments {
        // Count files that would be included in this environment
        let file_count = count_environment_files(&scanner, environment, verbose)?;
        
        println!("• {} ({} files)", env_name, file_count);
        
        // Always show basic directory info (includes/excludes summary)
        let include_summary = environment.include_directories.join(", ");
        println!("  Includes: {}", include_summary);
        
        if !environment.exclude_directories.is_empty() {
            let exclude_summary = environment.exclude_directories.join(", ");
            println!("  Excludes: {}", exclude_summary);
        }
        
        if verbose {
            println!("  Detailed configuration:");
            println!("    Include directories:");
            for dir in &environment.include_directories {
                println!("      + {}", dir);
            }
            
            if !environment.exclude_directories.is_empty() {
                println!("    Exclude directories:");
                for dir in &environment.exclude_directories {
                    println!("      - {}", dir);
                }
            }
        }
        println!();
    }

    if !verbose {
        println!();
        println!("Use --verbose for detailed environment configuration.");
    }

    Ok(())
}

fn count_environment_files(
    scanner: &FileScanner,
    environment: &crate::config::Environment,
    verbose: bool,
) -> Result<usize> {
    // Scan all files first
    let all_files = scanner.scan().map_err(|e| DbFastError::ConfigCreationFailed {
        message: format!("Failed to scan files: {e}"),
    })?;

    // Filter based on environment configuration
    let filtered_files: Vec<_> = all_files
        .iter()
        .filter(|file| {
            let file_path_str = file.path.to_string_lossy();
            
            // Check if file is in any include directory
            let included = environment.include_directories.iter().any(|include_dir| {
                file_path_str.contains(include_dir)
            });
            
            if !included {
                return false;
            }
            
            // Check if file is excluded
            let excluded = environment.exclude_directories.iter().any(|exclude_dir| {
                file_path_str.contains(exclude_dir)
            });
            
            !excluded
        })
        .collect();

    if verbose && !filtered_files.is_empty() {
        println!("  Files:");
        for file in &filtered_files {
            println!("    - {}", file.path.display());
        }
    }

    Ok(filtered_files.len())
}