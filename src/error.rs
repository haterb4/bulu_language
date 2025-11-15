//! Error handling for the Bulu language

use std::fmt;

/// Result type alias for Bulu operations
pub type Result<T> = std::result::Result<T, BuluError>;

/// Stack frame for error tracing
#[derive(Debug, Clone)]
pub struct ErrorFrame {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub context: String,
}

/// Main error type for the Bulu language
#[derive(Debug, Clone)]
pub enum BuluError {
    /// Lexical analysis errors
    LexError {
        message: String,
        line: usize,
        column: usize,
        file: Option<String>,
        #[allow(dead_code)]
        token: Option<String>,
        #[allow(dead_code)]
        stack: Vec<ErrorFrame>,
    },
    /// Syntax parsing errors
    ParseError {
        message: String,
        line: usize,
        column: usize,
        file: Option<String>,
        #[allow(dead_code)]
        token: Option<String>,
        #[allow(dead_code)]
        stack: Vec<ErrorFrame>,
    },
    /// Type checking errors
    TypeError {
        message: String,
        line: usize,
        column: usize,
        file: Option<String>,
        #[allow(dead_code)]
        stack: Vec<ErrorFrame>,
    },
    /// Runtime errors
    RuntimeError {
        message: String,
        file: Option<String>,
    },
    /// I/O errors
    IoError(String),
    /// Break statement (control flow)
    Break,
    /// Continue statement (control flow)
    Continue,
    /// Return statement (control flow)
    Return(crate::types::primitive::RuntimeValue),
    /// Generic error
    Other(String),
}

impl fmt::Display for BuluError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuluError::LexError { message, line, column, file, token, stack } => {
                if let Some(file_path) = file {
                    write!(f, "Lexical Error: {}\n", message)?;
                    write!(f, "  --> {}:{}:{}\n", file_path, line, column)?;
                    if let Some(tok) = token {
                        write!(f, "  Token: '{}'\n", tok)?;
                    }
                    if !stack.is_empty() {
                        write!(f, "\nStack trace:\n")?;
                        for frame in stack {
                            write!(f, "  at {}:{}:{} in {}\n", frame.file, frame.line, frame.column, frame.context)?;
                        }
                    }
                    Ok(())
                } else {
                    write!(f, "Lexical error at {}:{}: {}", line, column, message)
                }
            }
            BuluError::ParseError { message, line, column, file, token, stack } => {
                if let Some(file_path) = file {
                    write!(f, "Parse Error: {}\n", message)?;
                    write!(f, "  --> {}:{}:{}\n", file_path, line, column)?;
                    if let Some(tok) = token {
                        write!(f, "  Token: '{}'\n", tok)?;
                    }
                    if !stack.is_empty() {
                        write!(f, "\nStack trace:\n")?;
                        for frame in stack {
                            write!(f, "  at {}:{}:{} in {}\n", frame.file, frame.line, frame.column, frame.context)?;
                        }
                    }
                    Ok(())
                } else {
                    write!(f, "Parse error at {}:{}: {}", line, column, message)
                }
            }
            BuluError::TypeError { message, line, column, file, stack } => {
                if let Some(file_path) = file {
                    write!(f, "Type Error: {}\n", message)?;
                    write!(f, "  --> {}:{}:{}\n", file_path, line, column)?;
                    if !stack.is_empty() {
                        write!(f, "\nStack trace:\n")?;
                        for frame in stack {
                            write!(f, "  at {}:{}:{} in {}\n", frame.file, frame.line, frame.column, frame.context)?;
                        }
                    }
                    Ok(())
                } else {
                    write!(f, "Type error at {}:{}: {}", line, column, message)
                }
            }
            BuluError::RuntimeError { message, file } => {
                if let Some(file_path) = file {
                    write!(f, "Runtime error in {}: {}", file_path, message)
                } else {
                    write!(f, "Runtime error: {}", message)
                }
            }
            BuluError::IoError(message) => {
                write!(f, "I/O error: {}", message)
            }
            BuluError::Break => {
                write!(f, "Break statement outside of loop")
            }
            BuluError::Continue => {
                write!(f, "Continue statement outside of loop")
            }
            BuluError::Return(_) => {
                write!(f, "Return statement outside of function")
            }
            BuluError::Other(message) => {
                write!(f, "Error: {}", message)
            }
        }
    }
}

impl BuluError {
    /// Create a new lexical error with file information
    pub fn lex_error(message: String, line: usize, column: usize, file: Option<String>) -> Self {
        BuluError::LexError { message, line, column, file, token: None, stack: Vec::new() }
    }

    /// Create a new parse error with file information
    pub fn parse_error(message: String, line: usize, column: usize, file: Option<String>) -> Self {
        BuluError::ParseError { message, line, column, file, token: None, stack: Vec::new() }
    }

    /// Create a new type error with file information
    pub fn type_error(message: String, line: usize, column: usize, file: Option<String>) -> Self {
        BuluError::TypeError { message, line, column, file, stack: Vec::new() }
    }

    /// Create a new runtime error with file information
    pub fn runtime_error(message: String, file: Option<String>) -> Self {
        BuluError::RuntimeError { message, file }
    }
    
    /// Add token information to an error
    pub fn with_token(mut self, token: String) -> Self {
        match &mut self {
            BuluError::LexError { token: tok, .. } => *tok = Some(token),
            BuluError::ParseError { token: tok, .. } => *tok = Some(token),
            _ => {}
        }
        self
    }
    
    /// Add a stack frame to an error
    pub fn with_frame(mut self, file: String, line: usize, column: usize, context: String) -> Self {
        let frame = ErrorFrame { file, line, column, context };
        match &mut self {
            BuluError::LexError { stack, .. } => stack.push(frame),
            BuluError::ParseError { stack, .. } => stack.push(frame),
            BuluError::TypeError { stack, .. } => stack.push(frame),
            _ => {}
        }
        self
    }

    /// Get the file path associated with this error, if any
    pub fn file_path(&self) -> Option<&String> {
        match self {
            BuluError::LexError { file, .. } => file.as_ref(),
            BuluError::ParseError { file, .. } => file.as_ref(),
            BuluError::TypeError { file, .. } => file.as_ref(),
            BuluError::RuntimeError { file, .. } => file.as_ref(),
            _ => None,
        }
    }

    /// Get the line number associated with this error, if any
    pub fn line(&self) -> Option<usize> {
        match self {
            BuluError::LexError { line, .. } => Some(*line),
            BuluError::ParseError { line, .. } => Some(*line),
            BuluError::TypeError { line, .. } => Some(*line),
            _ => None,
        }
    }

    /// Get the column number associated with this error, if any
    pub fn column(&self) -> Option<usize> {
        match self {
            BuluError::LexError { column, .. } => Some(*column),
            BuluError::ParseError { column, .. } => Some(*column),
            BuluError::TypeError { column, .. } => Some(*column),
            _ => None,
        }
    }
    
    /// Get the token associated with this error, if any
    pub fn token(&self) -> Option<&String> {
        match self {
            BuluError::LexError { token, .. } => token.as_ref(),
            BuluError::ParseError { token, .. } => token.as_ref(),
            _ => None,
        }
    }

    /// Format error with source code context for better debugging
    pub fn format_with_context(&self, source_lines: &[String]) -> String {
        let mut output = String::new();
        
        // Add the main error message
        output.push_str(&format!("{}\n", self));
        
        // Add source context if we have line information
        if let (Some(line), Some(column)) = (self.line(), self.column()) {
            if line > 0 && line <= source_lines.len() {
                let line_idx = line - 1;
                
                // Show a few lines of context
                let start = line_idx.saturating_sub(2);
                let end = (line_idx + 3).min(source_lines.len());
                
                output.push('\n');
                for (i, source_line) in source_lines[start..end].iter().enumerate() {
                    let current_line = start + i + 1;
                    let is_error_line = current_line == line;
                    
                    if is_error_line {
                        output.push_str(&format!(" --> {}: {}\n", current_line, source_line));
                        
                        // Add pointer to the exact column
                        let pointer_line = format!("     {}{}", " ".repeat(column.saturating_sub(1)), "^");
                        output.push_str(&format!("{}\n", pointer_line));
                    } else {
                        output.push_str(&format!("     {}: {}\n", current_line, source_line));
                    }
                }
            }
        }
        
        output
    }
}

impl std::error::Error for BuluError {}

impl From<std::io::Error> for BuluError {
    fn from(err: std::io::Error) -> Self {
        BuluError::IoError(err.to_string())
    }
}

impl From<String> for BuluError {
    fn from(message: String) -> Self {
        BuluError::Other(message)
    }
}

impl From<&str> for BuluError {
    fn from(message: &str) -> Self {
        BuluError::Other(message.to_string())
    }
}

impl From<serde_json::Error> for BuluError {
    fn from(err: serde_json::Error) -> Self {
        BuluError::Other(format!("JSON error: {}", err))
    }
}