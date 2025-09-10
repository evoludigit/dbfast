use dbfast::cli::{Cli, Commands};
use dbfast::commands::{init, seed, status};
use std::process;

// Allow println in main CLI binary
#[allow(clippy::disallowed_methods)]
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { repo_dir, template_name }) => {
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
        Some(Commands::Status) => {
            if let Err(e) = status::handle_status() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        None => {
            println!("DBFast - Use --help for available commands");
        }
    }
}