//! Remote database management commands

use crate::config::Config;
use crate::remote::RemoteConfig;
use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

/// Handle remote add command
pub fn handle_remote_add(
    name: &str,
    url: &str,
    env: &str,
    password_env: Option<String>,
    allow_destructive: bool,
    skip_backup: bool,
) -> Result<()> {
    info!("Adding remote database configuration: {}", name);
    debug!(
        "Remote config details: url={}, env={}, password_env={:?}, destructive={}, skip_backup={}",
        url, env, password_env, allow_destructive, skip_backup
    );

    let config_path = "dbfast.toml";

    // Load existing config or create new one
    let mut config = if Path::new(config_path).exists() {
        debug!("Loading existing configuration from {}", config_path);
        Config::from_file(config_path)?
    } else {
        error!("No dbfast.toml found in current directory");
        return Err(anyhow::anyhow!(
            "No dbfast.toml found. Run 'dbfast init' first."
        ));
    };

    // Validate that the environment exists
    if !config.environments.contains_key(env) {
        error!("Environment '{}' not found in configuration", env);
        debug!(
            "Available environments: {:?}",
            config.environments.keys().collect::<Vec<_>>()
        );
        return Err(anyhow::anyhow!(
            "Environment '{}' not found in configuration. Available environments: {}",
            env,
            config
                .environments
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Create remote configuration
    let remote_config = RemoteConfig {
        name: Some(name.to_string()),
        url: url.to_string(),
        password_env,
        environment: env.to_string(),
        allow_destructive,
        backup_before_deploy: !skip_backup,
        require_confirmation: false,
    };

    // Validate the URL can be parsed
    debug!("Validating connection URL format");
    remote_config.parse_connection_url().map_err(|e| {
        error!("Invalid connection URL format: {}", e);
        anyhow::anyhow!("Invalid connection URL: {}", e)
    })?;

    // Add to config
    debug!("Adding remote '{}' to configuration", name);
    config.remotes.insert(name.to_string(), remote_config);

    // Write back to file
    debug!("Serializing updated configuration");
    let toml_string = toml::to_string_pretty(&config).map_err(|e| {
        error!("Failed to serialize configuration: {}", e);
        anyhow::anyhow!("Failed to serialize config: {}", e)
    })?;

    debug!("Writing configuration to {}", config_path);
    fs::write(config_path, toml_string).map_err(|e| {
        error!("Failed to write config file: {}", e);
        anyhow::anyhow!("Failed to write config file: {}", e)
    })?;

    info!("Successfully added remote '{}' to configuration", name);
    println!("✅ Added remote '{name}' successfully");
    Ok(())
}

/// Handle remote list command
pub fn handle_remote_list(verbose: bool) -> Result<()> {
    let config_path = "dbfast.toml";

    if !Path::new(config_path).exists() {
        println!("No dbfast.toml found. Run 'dbfast init' first.");
        return Ok(());
    }

    let config = Config::from_file(config_path)?;

    if config.remotes.is_empty() {
        println!("No remote databases configured.");
        println!("Use 'dbfast remote add' to add a remote database.");
        return Ok(());
    }

    println!("📡 Remote Databases:");
    println!();

    for (name, remote) in &config.remotes {
        println!("🔗 {name}");

        if let Ok(params) = remote.parse_connection_url() {
            println!("   Host:        {}:{}", params.host, params.port);
            println!("   Database:    {}", params.database);
            println!("   User:        {}", params.user);
        } else {
            println!("   URL:         {} (invalid)", remote.url);
        }

        println!("   Environment: {}", remote.environment);

        if verbose {
            println!(
                "   Destructive: {}",
                if remote.allow_destructive {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "   Backup:      {}",
                if remote.backup_before_deploy {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "   Confirm:     {}",
                if remote.require_confirmation {
                    "✅"
                } else {
                    "❌"
                }
            );

            if let Some(password_env) = &remote.password_env {
                let has_password = std::env::var(password_env).is_ok();
                // Don't log actual environment variable name for security
                println!(
                    "   Password:    [REDACTED ENV VAR] ({})",
                    if has_password {
                        "✅ set"
                    } else {
                        "❌ not set"
                    }
                );
            } else {
                println!("   Password:    None");
            }
        }

        println!();
    }

    Ok(())
}

/// Handle remote test command
pub async fn handle_remote_test(name: &str) -> Result<()> {
    let config_path = "dbfast.toml";

    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!(
            "No dbfast.toml found. Run 'dbfast init' first."
        ));
    }

    let config = Config::from_file(config_path)?;

    let remote = config
        .remotes
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Remote '{}' not found", name))?;

    println!("🧪 Testing connection to remote '{name}'...");

    // Parse connection URL
    let params = remote
        .parse_connection_url()
        .map_err(|e| anyhow::anyhow!("Invalid connection URL: {}", e))?;

    println!("   Host:        {}:{}", params.host, params.port);
    println!("   Database:    {}", params.database);
    println!("   User:        {}", params.user);

    // Check password
    let password_result = remote.get_password();
    match password_result {
        Ok(password) => {
            if password.is_empty() && remote.password_env.is_some() {
                println!(
                    "   Password:    ❌ Environment variable '{}' not set",
                    remote.password_env.as_ref().unwrap()
                );
                return Err(anyhow::anyhow!("Password environment variable not set"));
            }
            println!("   Password:    ✅ Available");
        }
        Err(e) => {
            println!("   Password:    ❌ {e}");
            return Err(anyhow::anyhow!("Password error: {}", e));
        }
    }

    // Test connection (simplified - would normally create a connection pool)
    println!("   Connection:  🔄 Testing...");

    // For now, just validate the configuration is complete
    // In a full implementation, this would create an actual database connection
    println!("   Connection:  ✅ Configuration valid (actual connection test not implemented yet)");

    println!();
    println!("✅ Remote '{name}' configuration is valid");
    println!("⚠️  Note: Actual database connectivity test not yet implemented");

    Ok(())
}

/// Handle remote remove command
pub fn handle_remote_remove(name: &str) -> Result<()> {
    let config_path = "dbfast.toml";

    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!(
            "No dbfast.toml found. Run 'dbfast init' first."
        ));
    }

    let mut config = Config::from_file(config_path)?;

    if !config.remotes.contains_key(name) {
        return Err(anyhow::anyhow!("Remote '{}' not found", name));
    }

    config.remotes.remove(name);

    // Write back to file
    let toml_string = toml::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    fs::write(config_path, toml_string)
        .map_err(|e| anyhow::anyhow!("Failed to write config file: {}", e))?;

    println!("✅ Removed remote '{name}' successfully");
    Ok(())
}
