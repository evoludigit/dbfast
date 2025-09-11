use crate::config::Config;
use crate::error::{DbFastError, Result};
use crate::scanner::FileScanner;
use std::path::PathBuf;

/// Handle the validate-env command synchronously
pub fn handle_validate_env(env_name: &str) -> Result<()> {
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

    // Check if environment exists
    let environment = config.environments.get(env_name).ok_or_else(|| {
        DbFastError::ConfigCreationFailed {
            message: format!("Environment '{}' not found in configuration", env_name),
        }
    })?;

    println!("🔍 Validating environment: {}", env_name);
    println!();

    // Get repository path for validation
    let repo_path = PathBuf::from(&config.repository.path);
    let scanner = FileScanner::new(&repo_path);

    // Validate directory existence
    let mut warnings: Vec<String> = Vec::new();
    let errors: Vec<String> = Vec::new();

    for include_dir in &environment.include_directories {
        let dir_path = repo_path.join(include_dir);
        if !dir_path.exists() {
            warnings.push(format!("Include directory '{}' not found", include_dir));
        }
    }

    for exclude_dir in &environment.exclude_directories {
        let dir_path = repo_path.join(exclude_dir);
        if !dir_path.exists() {
            warnings.push(format!("Exclude directory '{}' not found (this might be intentional)", exclude_dir));
        }
    }

    // Check for production safety issues
    if env_name.to_lowercase().contains("production") || env_name.to_lowercase().contains("prod") {
        for include_dir in &environment.include_directories {
            if include_dir.to_lowercase().contains("seed") 
                && !include_dir.to_lowercase().contains("common") {
                warnings.push(format!(
                    "⚠️  Production environment includes seed directory '{}' - this may contain test data",
                    include_dir
                ));
            }
        }
    }

    // Scan for actual files
    let all_files = scanner.scan().map_err(|e| DbFastError::ConfigCreationFailed {
        message: format!("Failed to scan files: {e}"),
    })?;

    // Filter files for this environment
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

    // Report results
    if errors.is_empty() && warnings.is_empty() {
        println!("✅ Environment '{}' is valid", env_name);
        println!("📄 Found {} SQL files", filtered_files.len());
    } else {
        if !warnings.is_empty() {
            for warning in &warnings {
                println!("⚠️  {}", warning);
            }
            println!();
        }
        
        if !errors.is_empty() {
            for error in &errors {
                println!("❌ {}", error);
            }
            return Err(DbFastError::ConfigCreationFailed {
                message: format!("Environment '{}' has validation errors", env_name),
            });
        } else {
            println!("✅ Environment '{}' is valid (with warnings)", env_name);
            println!("📄 Found {} SQL files", filtered_files.len());
        }
    }

    println!();
    println!("Environment Configuration:");
    println!("  Include directories:");
    for dir in &environment.include_directories {
        println!("    + {}", dir);
    }
    
    if !environment.exclude_directories.is_empty() {
        println!("  Exclude directories:");
        for dir in &environment.exclude_directories {
            println!("    - {}", dir);
        }
    }

    Ok(())
}