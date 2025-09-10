use crate::config::Config;
use crate::error::{DbFastError, Result};
use std::fs;
use std::path::Path;

#[allow(clippy::disallowed_methods)]
/// Handle the init command with default output directory
pub fn handle_init(repo_dir: &str, template_name: &str) -> Result<()> {
    handle_init_with_output_dir(repo_dir, template_name, &std::env::current_dir()?)
}

/// Handle the init command with specified output directory
pub fn handle_init_with_output_dir(
    repo_dir: &str,
    template_name: &str,
    output_dir: &std::path::Path,
) -> Result<()> {
    // Check if repository directory exists
    let repo_path = Path::new(repo_dir);
    if !repo_path.exists() {
        return Err(DbFastError::RepoDirectoryNotFound {
            path: repo_dir.to_string(),
        });
    }

    // Create default config
    let config = Config::new(repo_dir, template_name);

    // Write config to dbfast.toml in specified output directory
    let config_content = toml::to_string_pretty(&config)?;
    let config_path = output_dir.join("dbfast.toml");
    fs::write(&config_path, config_content)?;

    println!("Successfully initialized DBFast configuration");
    println!("Repository: {repo_dir}");
    println!("Template: {template_name}");
    println!("Configuration saved to: {}", config_path.display());

    Ok(())
}
