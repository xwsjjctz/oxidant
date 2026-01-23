// CLI configuration
use clap::Parser;
use std::path::PathBuf;

/// Oxidant - Audio metadata CLI tool
#[derive(Parser, Debug)]
#[command(name = "oxidant")]
#[command(about = "A high-performance audio metadata command-line tool", long_about = None)]
#[command(version)]
#[command(author = "xwsjjctz <xwsjjctz@icloud.com>")]
pub struct Config {
    /// Audio file path (for single file operations)
    #[arg(value_name = "FILE")]
    pub file: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Quiet mode (suppress progress messages)
    #[arg(short, long)]
    pub quiet: bool,

    /// Verbose mode (show more details)
    #[arg(short, long)]
    pub verbose: bool,

    /// Subcommand
    #[command(subcommand)]
    pub command: Commands,
}

/// Output format for metadata
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Pretty-printed JSON
    #[default]
    Pretty,
    /// Compact JSON
    Json,
    /// Key-value pairs
    KeyValue,
    /// Table format
    Table,
}

/// CLI subcommands
#[derive(Parser, Debug)]
pub enum Commands {
    /// Read metadata from audio file(s)
    Read {
        /// Audio file path(s)
        #[arg(value_name = "FILE")]
        files: Vec<String>,

        /// Metadata fields to display (comma-separated)
        #[arg(short, long)]
        fields: Option<String>,

        /// Output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Write metadata to audio file(s)
    Write {
        /// Audio file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Metadata JSON string
        #[arg(short, long)]
        metadata: String,

        /// Read metadata from JSON file
        #[arg(long)]
        from_file: Option<String>,
    },

    /// Copy metadata between files
    Copy {
        /// Source audio file
        #[arg(value_name = "SOURCE")]
        source: String,

        /// Target audio file(s)
        #[arg(value_name = "TARGET")]
        targets: Vec<String>,
    },

    /// Batch process multiple files
    Batch {
        /// Directory path
        #[arg(short, long)]
        directory: String,

        /// File pattern (e.g., "*.mp3", "*.flac")
        #[arg(short, long)]
        pattern: String,

        /// Operation: read or write
        #[arg(value_enum)]
        operation: BatchOperation,

        /// Metadata JSON to write (required for write operation)
        #[arg(long)]
        metadata: Option<String>,
    },

    /// Detect file format
    Detect {
        /// Audio file path(s)
        #[arg(value_name = "FILE")]
        files: Vec<String>,
    },

    /// Export cover art
    ExportCover {
        /// Audio file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Output directory for cover images
        #[arg(short, long)]
        output: String,

        /// Cover index (for files with multiple covers)
        #[arg(short, long)]
        index: Option<usize>,
    },

    /// Set cover art
    SetCover {
        /// Audio file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Image file path
        #[arg(short, long)]
        image: String,

        /// MIME type (auto-detected if not specified)
        #[arg(short, long)]
        mime_type: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Remove cover art
    RemoveCover {
        /// Audio file path
        #[arg(value_name = "FILE")]
        files: Vec<String>,
    },

    /// Show file information
    Info {
        /// Audio file path(s)
        #[arg(value_name = "FILE")]
        files: Vec<String>,

        /// Show detailed technical information
        #[arg(short, long)]
        detailed: bool,
    },
}

/// Batch operation type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOperation {
    Read,
    Write,
}

impl std::fmt::Display for BatchOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchOperation::Read => write!(f, "read"),
            BatchOperation::Write => write!(f, "write"),
        }
    }
}

impl Config {
    /// Parse field list into vector
    pub fn parse_fields(&self) -> Option<Vec<String>> {
        // This will be called from commands that have fields option
        None
    }
}
