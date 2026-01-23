// CLI module for oxidant
//
// This module provides command-line interface functionality for the oxidant library.
// It is only compiled when building the binary, not when building the Python extension.

pub mod commands;
pub mod config;
pub mod output;

pub use commands::Commands;
pub use config::Config;
pub use output::OutputFormat;

// Re-export core library types for CLI use
pub use oxidant::{AudioFile, Metadata, CoverArt};

// Error type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    FileNotFound(String),
    InvalidFormat(String),
    IoError(std::io::Error),
    ParseError(String),
    Other(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::FileNotFound(path) => write!(f, "File not found: {}", path),
            CliError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            CliError::IoError(e) => write!(f, "I/O error: {}", e),
            CliError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CliError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::IoError(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::ParseError(e.to_string())
    }
}
