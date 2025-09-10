use dbfast::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { repo_dir, template_name }) => {
            println!("Initializing DBFast with repo-dir: {} and template-name: {}", repo_dir, template_name);
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