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
    /// List configured environments
    Environments {
        /// Show verbose environment information
        #[arg(long)]
        verbose: bool,
    },
    /// Validate environment configuration
    ValidateEnv {
        /// Environment name to validate
        #[arg(long, value_name = "NAME")]
        env: String,
    },
    /// Remote database management
    Remote {
        /// Remote subcommand
        #[command(subcommand)]
        command: RemoteCommands,
    },
    /// Deploy to remote database
    Deploy {
        /// Remote name to deploy to
        #[arg(value_name = "REMOTE")]
        remote: String,
        /// Environment to deploy
        #[arg(long, value_name = "ENV")]
        env: Option<String>,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
        /// Skip backup before deployment
        #[arg(long)]
        skip_backup: bool,
        /// Dry run - validate only, don't deploy
        #[arg(long)]
        dry_run: bool,
    },
}

/// Remote database management commands
#[derive(Subcommand)]
pub enum RemoteCommands {
    /// Add a new remote database configuration
    Add {
        /// Remote name
        #[arg(long, value_name = "NAME")]
        name: String,
        /// Connection URL (postgresql://user@host:port/database)
        #[arg(long, value_name = "URL")]
        url: String,
        /// Target environment
        #[arg(long, value_name = "ENV")]
        env: String,
        /// Environment variable for password
        #[arg(long, value_name = "PASSWORD_ENV")]
        password_env: Option<String>,
        /// Allow destructive operations
        #[arg(long)]
        allow_destructive: bool,
        /// Skip backup before deployment
        #[arg(long)]
        skip_backup: bool,
    },
    /// List configured remote databases
    List {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,
    },
    /// Test remote database connection
    Test {
        /// Remote name to test
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Remove a remote database configuration
    Remove {
        /// Remote name to remove
        #[arg(value_name = "NAME")]
        name: String,
    },
}

impl Cli {
    /// Parse command line arguments
    #[must_use]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
