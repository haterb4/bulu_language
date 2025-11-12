// std.os module - Operating system interface
// Provides access to command-line arguments, environment variables, and system information

use crate::error::{BuluError, Result};
use crate::types::primitive::RuntimeValue;
use std::collections::HashMap;
use std::sync::Mutex;

// Global storage for command-line arguments
static PROGRAM_ARGS: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// Initialize the program arguments (called from main before execution)
pub fn init_args(args: Vec<String>) {
    if let Ok(mut program_args) = PROGRAM_ARGS.lock() {
        *program_args = Some(args);
    }
}

/// Get all command-line arguments as a Bulu array
/// Returns an array where args[0] is the program name/path
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

/// Get environment variable by name
pub fn get_env(name: &str) -> Result<RuntimeValue> {
    match std::env::var(name) {
        Ok(value) => Ok(RuntimeValue::String(value)),
        Err(_) => Ok(RuntimeValue::Null),
    }
}

/// Set environment variable
pub fn set_env(name: &str, value: &str) -> Result<RuntimeValue> {
    std::env::set_var(name, value);
    Ok(RuntimeValue::Null)
}

/// Get all environment variables as a map
pub fn get_all_env() -> Result<RuntimeValue> {
    let mut env_map = HashMap::new();
    
    for (key, value) in std::env::vars() {
        env_map.insert(key, RuntimeValue::String(value));
    }
    
    Ok(RuntimeValue::Map(env_map))
}

/// Get current working directory
pub fn get_cwd() -> Result<RuntimeValue> {
    match std::env::current_dir() {
        Ok(path) => Ok(RuntimeValue::String(path.to_string_lossy().to_string())),
        Err(e) => Err(BuluError::RuntimeError {
            file: None,
            message: format!("Failed to get current directory: {}", e),
        }),
    }
}

/// Exit the program with a status code
pub fn exit(code: i32) -> Result<RuntimeValue> {
    std::process::exit(code);
}

/// Get the operating system name
pub fn get_os() -> Result<RuntimeValue> {
    Ok(RuntimeValue::String(std::env::consts::OS.to_string()))
}

/// Get the system architecture
pub fn get_arch() -> Result<RuntimeValue> {
    Ok(RuntimeValue::String(std::env::consts::ARCH.to_string()))
}
