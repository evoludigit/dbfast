use dbfast::cli::{Cli, Commands, RemoteCommands};
use dbfast::commands::{deploy, environments, init, remote, seed, status, validate_env};
use std::process;
use tracing_subscriber::EnvFilter;

// Allow println in main CLI binary
#[allow(clippy::disallowed_methods)]
fn main() {
    // Initialize comprehensive logging
    init_logging();

    let cli = Cli::parse();
    tracing::info!("DBFast CLI initialized");

    match cli.command {
        Some(Commands::Init {
            repo_dir,
            template_name,
        }) => {
            if let Err(e) = init::handle_init(&repo_dir, &template_name) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Seed { output, with_seeds }) => {
            if let Err(e) = seed::handle_seed(&output, with_seeds) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Status { verbose }) => {
            if let Err(e) = status::handle_status_with_options(verbose) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Environments { verbose }) => {
            if let Err(e) = environments::handle_environments(verbose) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::ValidateEnv { env }) => {
            if let Err(e) = validate_env::handle_validate_env(&env) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Remote { command }) => {
            let result = match command {
                RemoteCommands::Add {
                    name,
                    url,
                    env,
                    password_env,
                    allow_destructive,
                    skip_backup,
                } => remote::handle_remote_add(
                    &name,
                    &url,
                    &env,
                    password_env,
                    allow_destructive,
                    skip_backup,
                ),
                RemoteCommands::List { verbose } => remote::handle_remote_list(verbose),
                RemoteCommands::Test { name } => {
                    // Handle async command in sync context
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(remote::handle_remote_test(&name))
                }
                RemoteCommands::Remove { name } => remote::handle_remote_remove(&name),
            };

            if let Err(e) = result {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Deploy {
            remote,
            env,
            yes,
            skip_backup,
            dry_run,
        }) => {
            // Handle async deploy command
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(deploy::handle_deploy(
                remote,
                env,
                yes,
                skip_backup,
                dry_run,
            ));

            if let Err(e) = result {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        None => {
            println!("DBFast - Use --help for available commands");
        }
    }
}

/// Initialize comprehensive logging based on environment variables
fn init_logging() {
    // Default to INFO level, can be overridden by RUST_LOG environment variable
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("dbfast=info,warn"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}
