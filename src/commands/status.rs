use crate::config::Config;
use crate::error::{DbFastError, Result};
use crate::scanner::FileScanner;
use std::path::{Path, PathBuf};

/// Filter files based on environment include/exclude patterns
fn file_matches_environment(file_path_str: &str, environment: &crate::config::Environment) -> bool {
    // Check if file is in any include directory
    let included = environment
        .include_directories
        .iter()
        .any(|include_dir| file_path_str.contains(include_dir));

    if !included {
        return false;
    }

    // Check if file is excluded
    let excluded = environment
        .exclude_directories
        .iter()
        .any(|exclude_dir| file_path_str.contains(exclude_dir));

    !excluded
}

#[allow(clippy::disallowed_methods)]
/// Handle the status command using current working directory
pub fn handle_status() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    handle_status_in_dir(&current_dir)
}

#[allow(clippy::disallowed_methods)]
/// Handle the status command with options using current working directory
pub fn handle_status_with_options(verbose: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    handle_status_in_dir_with_options(&current_dir, verbose)
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
            message: format!("Failed to load config: {e}"),
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
                println!("   📁 {dir_name}/");
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

        // Get repository path for file scanning
        let repo_path = Path::new(&config.repository.path);
        let scanner = FileScanner::new(repo_path);

        // Try to scan files for counts
        if let Ok(all_files) = scanner.scan() {
            for (name, env) in &config.environments {
                // Count files for this environment
                let file_count = all_files
                    .iter()
                    .filter(|file| {
                        let file_path_str = file.path.to_string_lossy();
                        file_matches_environment(&file_path_str, env)
                    })
                    .count();

                println!("   {name} ({file_count} files)");
            }
        } else {
            // Fallback to basic display if scanning fails
            for (name, env) in &config.environments {
                println!("   {name} - includes: {:?}", env.include_directories);
                if !env.exclude_directories.is_empty() {
                    println!("     excludes: {:?}", env.exclude_directories);
                }
            }
        }
    }

    println!("\n✨ Ready to seed databases!");

    Ok(())
}

/// Handle the status command in a specific directory with enhanced verbose output
pub fn handle_status_in_dir_with_options(dir: &Path, verbose: bool) -> Result<()> {
    if !verbose {
        return handle_status_in_dir(dir);
    }

    display_verbose_header();

    let config_path = dir.join("dbfast.toml");
    if !config_path.exists() {
        display_config_error();
        return Ok(());
    }

    let config = load_config(&config_path)?;
    display_template_section(&config);
    display_repository_section(&config);
    display_environments_section(&config);

    Ok(())
}

fn display_verbose_header() {
    println!("📊 DBFast Status");
    println!();
}

fn display_config_error() {
    println!("❌ Configuration: No dbfast.toml found");
    println!("   Run 'dbfast init --repo-dir <path> --template-name <name>' to initialize");
}

fn load_config(config_path: &Path) -> Result<Config> {
    Config::from_file(config_path).map_err(|e| DbFastError::ConfigCreationFailed {
        message: format!("Failed to load config: {e}"),
    })
}

fn display_template_section(config: &Config) {
    println!("Template: {}", config.database.template_name);
    println!("  Status: ✅ Ready");
    println!(
        "  Database: {}:{}",
        config.database.host, config.database.port
    );
    println!("  User: {}", config.database.user);
    println!();
}

fn display_repository_section(config: &Config) {
    println!("Repository: {}", config.repository.path);
    let repo_path = Path::new(&config.repository.path);

    if repo_path.exists() {
        display_repository_details(repo_path, &config.repository.repo_type);
    } else {
        println!("  Status: ❌ Directory not found");
    }
    println!();
}

fn display_repository_details(repo_path: &Path, repo_type: &str) {
    println!("  Status: ✅ Directory exists");
    println!("  Type: {repo_type}");

    let file_count = count_sql_files(repo_path);
    println!("  Files: {file_count} SQL files");

    display_common_directories(repo_path);
}

fn count_sql_files(repo_path: &Path) -> usize {
    walkdir::WalkDir::new(repo_path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().map_or(false, |ext| ext == "sql")
        })
        .count()
}

fn display_common_directories(repo_path: &Path) {
    let common_dirs = ["0_schema", "1_seed_common", "2_seed_backend", "6_migration"];
    for dir_name in &common_dirs {
        let dir_path = repo_path.join(dir_name);
        if dir_path.exists() {
            println!("  📁 {dir_name}/");
        }
    }
}

fn display_environments_section(config: &Config) {
    if config.environments.is_empty() {
        return;
    }

    println!("Environments:");

    // Get repository path for file scanning
    let repo_path = PathBuf::from(&config.repository.path);
    let scanner = FileScanner::new(&repo_path);

    // Scan all files once for efficiency
    let Ok(all_files) = scanner.scan() else {
        // If scanning fails, just show basic info
        for (name, env) in &config.environments {
            println!("  • {name} (includes: {:?})", env.include_directories);
            if !env.exclude_directories.is_empty() {
                println!("    excludes: {:?}", env.exclude_directories);
            }
        }
        return;
    };

    for (name, env) in &config.environments {
        // Count files that would be included in this environment
        let file_count = all_files
            .iter()
            .filter(|file| {
                let file_path_str = file.path.to_string_lossy();
                file_matches_environment(&file_path_str, env)
            })
            .count();

        println!("  • {name} ({file_count} files)");
        if !env.exclude_directories.is_empty() {
            println!("    excludes: {:?}", env.exclude_directories);
        }
    }
}
