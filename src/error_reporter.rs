//! Enhanced error reporting with source code context

use crate::error::BuluError;
use colored::*;
use std::fs;
use std::path::Path;

/// Enhanced error reporter that provides rich error messages with source context
pub struct ErrorReporter {
    source_lines: Vec<String>,
    file_path: Option<String>,
}

impl ErrorReporter {
    /// Create a new error reporter for a source file
    pub fn new(file_path: &Path) -> Result<Self, BuluError> {
        let source = fs::read_to_string(file_path)
            .map_err(|e| BuluError::IoError(format!("Failed to read {}: {}", file_path.display(), e)))?;
        
        let source_lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
        
        Ok(Self {
            source_lines,
            file_path: Some(file_path.to_string_lossy().to_string()),
        })
    }

    /// Create a new error reporter from source code string
    pub fn from_source(source: &str, file_path: Option<String>) -> Self {
        let source_lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
        
        Self {
            source_lines,
            file_path,
        }
    }

    /// Format an error with rich source context
    pub fn format_error(&self, error: &BuluError) -> String {
        let mut output = String::new();
        
        // Add the main error message with color
        match error {
            BuluError::LexError { .. } => {
                output.push_str(&format!("{}: {}\n", "Lexical Error".red().bold(), error));
            }
            BuluError::ParseError { .. } => {
                output.push_str(&format!("{}: {}\n", "Parse Error".red().bold(), error));
            }
            BuluError::TypeError { .. } => {
                output.push_str(&format!("{}: {}\n", "Type Error".red().bold(), error));
            }
            BuluError::RuntimeError { .. } => {
                output.push_str(&format!("{}: {}\n", "Runtime Error".red().bold(), error));
            }
            _ => {
                output.push_str(&format!("{}: {}\n", "Error".red().bold(), error));
            }
        }
        
        // Add source context if we have line information
        if let (Some(line), Some(column)) = (error.line(), error.column()) {
            if line > 0 && line <= self.source_lines.len() {
                output.push('\n');
                output.push_str(&self.format_source_context(line, column));
            }
        }
        
        output
    }

    /// Format source code context around an error location
    fn format_source_context(&self, error_line: usize, error_column: usize) -> String {
        let mut output = String::new();
        
        let line_idx = error_line - 1;
        
        // Show a few lines of context
        let context_lines = 2;
        let start = line_idx.saturating_sub(context_lines);
        let end = (line_idx + context_lines + 1).min(self.source_lines.len());
        
        // Calculate the width needed for line numbers
        let max_line_num = end;
        let line_num_width = max_line_num.to_string().len();
        
        for (i, source_line) in self.source_lines[start..end].iter().enumerate() {
            let current_line = start + i + 1;
            let is_error_line = current_line == error_line;
            
            if is_error_line {
                // Highlight the error line
                output.push_str(&format!(
                    " {} {} {}\n",
                    "-->".blue().bold(),
                    format!("{:width$}", current_line, width = line_num_width).blue().bold(),
                    source_line
                ));
                
                // Add pointer to the exact column
                let pointer_prefix = format!("     {}", " ".repeat(line_num_width));
                let pointer_spaces = " ".repeat(error_column.saturating_sub(1));
                let pointer = "^".red().bold();
                output.push_str(&format!("{}{}{}\n", pointer_prefix, pointer_spaces, pointer));
            } else {
                // Show context lines in a muted color
                output.push_str(&format!(
                    "     {} {}\n",
                    format!("{:width$}", current_line, width = line_num_width).bright_black(),
                    source_line.bright_black()
                ));
            }
        }
        
        output
    }

    /// Format multiple errors with source context
    pub fn format_errors(&self, errors: &[BuluError]) -> String {
        let mut output = String::new();
        
        for (i, error) in errors.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&self.format_error(error));
        }
        
        output
    }

    /// Create a summary of compilation results
    pub fn format_summary(&self, errors: &[BuluError], warnings: &[BuluError]) -> String {
        let mut output = String::new();
        
        let error_count = errors.len();
        let warning_count = warnings.len();
        
        if error_count > 0 || warning_count > 0 {
            output.push('\n');
            output.push_str(&"=".repeat(50));
            output.push('\n');
            
            if error_count > 0 {
                output.push_str(&format!(
                    "{} {} compilation error{}\n",
                    "Found".red().bold(),
                    error_count,
                    if error_count == 1 { "" } else { "s" }
                ));
            }
            
            if warning_count > 0 {
                output.push_str(&format!(
                    "{} {} warning{}\n",
                    "Found".yellow().bold(),
                    warning_count,
                    if warning_count == 1 { "" } else { "s" }
                ));
            }
            
            output.push_str(&"=".repeat(50));
            output.push('\n');
        }
        
        output
    }
}

/// Helper function to report a single error with context
pub fn report_error(error: &BuluError, source: Option<&str>) {
    if let Some(source_code) = source {
        let reporter = ErrorReporter::from_source(source_code, error.file_path().cloned());
        eprintln!("{}", reporter.format_error(error));
    } else {
        eprintln!("{}: {}", "Error".red().bold(), error);
    }
}

/// Helper function to report multiple errors with context
pub fn report_errors(errors: &[BuluError], source: Option<&str>) {
    if let Some(source_code) = source {
        let reporter = ErrorReporter::from_source(source_code, None);
        eprintln!("{}", reporter.format_errors(errors));
    } else {
        for error in errors {
            eprintln!("{}: {}", "Error".red().bold(), error);
        }
    }
}