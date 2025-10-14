//! Simple test for bytecode interpreter
//!
//! This test verifies that our bytecode interpreter can execute basic programs.

use std::process::Command;
use std::fs;

#[test]
fn test_hello_world_compilation_and_execution() {
    // Compile the hello world program
    let output = Command::new("./target/release/langc")
        .args(&["hello-world/src/main.bu", "-o", "hello-world/test_main.bulu"])
        .output()
        .expect("Failed to execute langc");
    
    assert!(output.status.success(), "Compilation should succeed");
    
    // Execute the bytecode with our interpreter
    let output = Command::new("./target/release/bulu_bytecode")
        .arg("hello-world/test_main.bulu")
        .output()
        .expect("Failed to execute bulu_bytecode");
    
    assert!(output.status.success(), "Execution should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, Bulu!"), "Should print Hello, Bulu!");
    
    // Clean up
    let _ = fs::remove_file("hello-world/test_main.bulu");
}

#[test]
fn test_bytecode_file_not_found() {
    let output = Command::new("./target/release/bulu_bytecode")
        .arg("nonexistent.bulu")
        .output()
        .expect("Failed to execute bulu_bytecode");
    
    assert!(!output.status.success(), "Should fail for nonexistent file");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"), "Should report file not found");
}