use clap::{Parser, Subcommand};

/// Main CLI interface for `DBFast`
#[derive(Parser)]
#[command(name = "dbfast")]
#[command(version = crate::VERSION)]
#[command(about = "DBFast - Lightning-Fast PostgreSQL Database Seeding")]
#[command(
    long_about = "Transform database fixtures from a 60-second bottleneck into a 100ms delight"
)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize from your existing database repository
    Init {
        /// Repository directory path
        #[arg(long, value_name = "DIR")]
        repo_dir: String,
        /// Template name for the database
        #[arg(long, value_name = "NAME")]
        template_name: String,
    },
    /// Get a seeded test database instantly
    Seed {
        /// Output database name
        #[arg(long, value_name = "NAME")]
        output: String,
        /// Include seed data
        #[arg(long)]
        with_seeds: bool,
    },
    /// Show template and database status
    Status {
        /// Show verbose status information
        #[arg(long)]
        verbose: bool,
    },
}

impl Cli {
    /// Parse command line arguments
    #[must_use]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
