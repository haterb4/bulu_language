// Standard I/O module for Bulu
// Provides functions for reading from stdin and accessing command-line arguments

use crate::error::{BuluError, Result};
use crate::types::primitive::RuntimeValue;
use std::io::{self, BufRead, Write};
use std::sync::Mutex;

// Global storage for command-line arguments
static PROGRAM_ARGS: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// Initialize the program arguments (called from main)
pub fn init_program_args(args: Vec<String>) {
    if let Ok(mut program_args) = PROGRAM_ARGS.lock() {
        *program_args = Some(args);
    }
}

/// Get command-line arguments as a Bulu array of strings
pub fn get_args() -> Result<RuntimeValue> {
    let args_guard = PROGRAM_ARGS.lock().map_err(|e| BuluError::RuntimeError {
        file: None,
        message: format!("Failed to access program arguments: {}", e),
    })?;

    let args = args_guard.as_ref().ok_or_else(|| BuluError::RuntimeError {
        file: None,
        message: "Program arguments not initialized".to_string(),
    })?;

    // Convert Vec<String> to Vec<RuntimeValue>
    let runtime_args: Vec<RuntimeValue> = args
        .iter()
        .map(|s| RuntimeValue::String(s.clone()))
        .collect();

    Ok(RuntimeValue::Array(runtime_args))
}

/// Read a line from stdin
pub fn read_line() -> Result<RuntimeValue> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    handle
        .read_line(&mut line)
        .map_err(|e| BuluError::RuntimeError {
            file: None,
            message: format!("Failed to read from stdin: {}", e),
        })?;

    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(RuntimeValue::String(line))
}

/// Read all input from stdin until EOF
pub fn read_all() -> Result<RuntimeValue> {
    use std::io::Read;
    
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut content = String::new();

    handle
        .read_to_string(&mut content)
        .map_err(|e| BuluError::RuntimeError {
            file: None,
            message: format!("Failed to read from stdin: {}", e),
        })?;

    Ok(RuntimeValue::String(content))
}

/// Print to stdout without newline
pub fn print(text: &str) -> Result<RuntimeValue> {
    print!("{}", text);
    io::stdout().flush().map_err(|e| BuluError::RuntimeError {
        file: None,
        message: format!("Failed to flush stdout: {}", e),
    })?;
    Ok(RuntimeValue::Null)
}

/// Print to stderr
pub fn eprint(text: &str) -> Result<RuntimeValue> {
    eprint!("{}", text);
    io::stderr().flush().map_err(|e| BuluError::RuntimeError {
        file: None,
        message: format!("Failed to flush stderr: {}", e),
    })?;
    Ok(RuntimeValue::Null)
}

/// Print to stderr with newline
pub fn eprintln(text: &str) -> Result<RuntimeValue> {
    eprintln!("{}", text);
    Ok(RuntimeValue::Null)
}
