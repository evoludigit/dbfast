use thiserror::Error;

/// Custom error types for `DBFast`
#[derive(Error, Debug)]
pub enum DbFastError {
    /// Repository directory not found
    #[error("Repository directory does not exist: {path}")]
    RepoDirectoryNotFound {
        /// The path that was not found
        path: String,
    },

    /// Configuration file creation failed
    #[error("Failed to create configuration file: {message}")]
    ConfigCreationFailed {
        /// Error message details
        message: String,
    },

    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML serialization error wrapper
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

/// Result type alias for `DBFast` operations
pub type Result<T> = std::result::Result<T, DbFastError>;
