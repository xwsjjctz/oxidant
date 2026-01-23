// CLI binary entry point for oxidant
//
// This is the main entry point for the oxidant command-line tool.

use clap::{Parser, Subcommand, ValueEnum};
use std::process;

/// Oxidant - Audio metadata CLI tool
#[derive(Parser, Debug)]
#[command(name = "oxidant")]
#[command(about = "A high-performance audio metadata command-line tool", long_about = None)]
#[command(version)]
#[command(author = "xwsjjctz <xwsjjctz@icloud.com>")]
struct Config {
    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    format: OutputFormat,

    /// Quiet mode (suppress progress messages)
    #[arg(short, long)]
    quiet: bool,

    /// Subcommand
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Read metadata from audio file(s)
    Read {
        /// Audio file path(s)
        files: Vec<String>,

        /// Output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Detect file format
    Detect {
        /// Audio file path(s)
        files: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Pretty,
    Json,
}

fn main() {
    let config = Config::parse();

    match &config.command {
        Commands::Read { files, output } => {
            command_read(files.clone(), output.clone(), &config);
        }
        Commands::Detect { files } => {
            command_detect(files.clone(), &config);
        }
    }
}

fn command_read(files: Vec<String>, _output: Option<String>, config: &Config) {
    if files.is_empty() {
        eprintln!("Error: No files specified");
        process::exit(1);
    }

    for file_path in files {
        match oxidant::AudioFile::new(file_path.clone()) {
            Ok(audio) => {
                match audio.get_metadata() {
                    Ok(metadata) => {
                        if !config.quiet {
                            println!("{}", metadata);
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ {}: {}", file_path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ {}: {}", file_path, e);
            }
        }
    }
}

fn command_detect(files: Vec<String>, config: &Config) {
    if files.is_empty() {
        eprintln!("Error: No files specified");
        process::exit(1);
    }

    for file_path in files {
        match oxidant::AudioFile::new(file_path.clone()) {
            Ok(audio) => {
                if !config.quiet {
                    println!("  {}: {} (version: {})", file_path, audio.file_type,
                        audio.get_version().unwrap_or_else(|_| "N/A".to_string()));
                }
            }
            Err(e) => {
                eprintln!("✗ {}: Unknown format ({})", file_path, e);
            }
        }
    }
}
