use dbfast::cli::{Cli, Commands};
use dbfast::commands::init;
use std::process;

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
            println!("Seeding database to output: {} (with-seeds: {})", output, with_seeds);
        }
        Some(Commands::Status) => {
            println!("DBFast status check");
        }
        None => {
            println!("DBFast - Use --help for available commands");
        }
    }
}