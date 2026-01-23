// Output formatting for CLI

use crate::cli::{CliError, CliResult};
use serde::Serialize;
use std::io::{self, Write};

/// Output format options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Pretty,
    Json,
    KeyValue,
    Table,
}

/// Format and output data
pub struct OutputFormatter {
    format: OutputFormat,
    quiet: bool,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat, quiet: bool) -> Self {
        Self { format, quiet }
    }

    /// Output metadata
    pub fn output_metadata(&self, metadata: &serde_json::Value, writer: &mut impl Write) -> CliResult<()> {
        match self.format {
            OutputFormat::Pretty => {
                writeln!(writer, "{}", serde_json::to_string_pretty(metadata).map_err(|e| CliError::ParseError(e.to_string()))?)?;
            }
            OutputFormat::Json => {
                writeln!(writer, "{}", serde_json::to_string(metadata).map_err(|e| CliError::ParseError(e.to_string()))?)?;
            }
            OutputFormat::KeyValue => {
                self.output_key_value(metadata, writer)?;
            }
            OutputFormat::Table => {
                self.output_table(metadata, writer)?;
            }
        }
        Ok(())
    }

    /// Output as key-value pairs
    fn output_key_value(&self, metadata: &serde_json::Value, writer: &mut impl Write) -> CliResult<()> {
        if let Some(obj) = metadata.as_object() {
            let mut items: Vec<_> = obj.iter().collect();
            items.sort_by(|a, b| a.0.cmp(b.0));

            for (key, value) in items {
                writeln!(writer, "{}: {}", key, self.format_value(value))?;
            }
        }
        Ok(())
    }

    /// Output as table
    fn output_table(&self, metadata: &serde_json::Value, writer: &mut impl Write) -> CliResult<()> {
        if let Some(obj) = metadata.as_object() {
            let max_key_len = obj.keys().map(|k| k.len()).max().unwrap_or(0);

            writeln!(writer, "{}", "=".repeat(max_key_len + 30))?;

            for (key, value) in obj {
                writeln!(writer, "{:<width$}: {}", format!("{}:", key), self.format_value(value), width = max_key_len + 2)?;
            }

            writeln!(writer, "{}", "=".repeat(max_key_len + 30))?;
        }
        Ok(())
    }

    /// Format a JSON value for display
    fn format_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => "(null)".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    "[]".to_string()
                } else {
                    format!("[{} items]", arr.len())
                }
            }
            serde_json::Value::Object(obj) => {
                if obj.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{{}} items}", obj.len())
                }
            }
        }
    }

    /// Print success message
    pub fn print_success(&self, message: &str) {
        if !self.quiet {
            println!("✓ {}", message);
        }
    }

    /// Print error message
    pub fn print_error(&self, message: &str) {
        eprintln!("✗ {}", message);
    }

    /// Print info message
    pub fn print_info(&self, message: &str) {
        if !self.quiet {
            println!("  {}", message);
        }
    }
}

/// Progress indicator for batch operations
pub struct ProgressBar {
    total: usize,
    current: usize,
    show: bool,
    prefix: String,
}

impl ProgressBar {
    pub fn new(total: usize, show: bool) -> Self {
        Self {
            total,
            current: 0,
            show,
            prefix: String::new(),
        }
    }

    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }

    pub fn increment(&mut self) {
        self.current += 1;
        if self.show && self.total > 0 {
            let percent = (self.current * 100) / self.total;
            print!("\r{} [{}/{}] ({}%)", self.prefix, self.current, self.total, percent);
            if self.current == self.total {
                println!();
            }
            use std::io::Write;
            std::io::stdout().flush().ok();
        }
    }
}
