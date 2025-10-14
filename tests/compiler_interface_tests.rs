//! Integration tests for the Bulu compiler interface (langc)

use std::fs;

use std::process::Command;
use tempfile::TempDir;

/// Test helper to create a temporary Bulu source file
fn create_test_file(content: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.bu");
    fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path.to_string_lossy().to_string())
}

/// Test helper to run langc command
fn run_langc(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(&["run", "--bin", "langc", "--"])
        .args(args)
        .output()
        .expect("Failed to execute langc")
}

#[test]
fn test_basic_compilation() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 42
    println("Hello, World!")
}
"#);

    let output = run_langc(&[&file_path]);
    
    // Should succeed (exit code 0)
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Basic compilation should succeed");
}

#[test]
fn test_optimization_levels() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 1 + 2 + 3
    println(x)
}
"#);

    // Test each optimization level
    for opt_level in &["0", "1", "2", "3", "s"] {
        let output = run_langc(&[&file_path, "-O", opt_level]);
        
        if !output.status.success() {
            println!("Optimization level {} failed:", opt_level);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        assert!(output.status.success(), "Optimization level {} should work", opt_level);
    }
}

#[test]
fn test_emit_tokens() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 42
}
"#);

    let output = run_langc(&[&file_path, "--emit", "tokens"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Token emission should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain token information
    assert!(stdout.contains("func") || stdout.contains("Keyword"), "Should emit tokens");
}

#[test]
fn test_emit_ast() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 42
}
"#);

    let output = run_langc(&[&file_path, "--emit", "ast"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "AST emission should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain AST structure
    assert!(stdout.contains("Program") || stdout.contains("Function"), "Should emit AST");
}

#[test]
fn test_emit_ir() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 42
}
"#);

    let output = run_langc(&[&file_path, "--emit", "ir"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "IR emission should succeed");
}

#[test]
fn test_emit_assembly() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 42
}
"#);

    let output = run_langc(&[&file_path, "--emit", "asm"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Assembly emission should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain assembly code
    assert!(stdout.contains(".section") || stdout.contains("Generated assembly"), "Should emit assembly");
}

#[test]
fn test_cross_compilation_targets() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let targets = &[
        "native",
        "linux-amd64",
        "linux-arm64",
        "windows-amd64",
        "darwin-amd64",
        "wasm",
    ];

    for target in targets {
        let output = run_langc(&[&file_path, "--target", target]);
        
        if !output.status.success() {
            println!("Target {} failed:", target);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        assert!(output.status.success(), "Target {} should be supported", target);
    }
}

#[test]
fn test_debug_information() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    let x = 42
    println(x)
}
"#);

    let output = run_langc(&[&file_path, "-g"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Debug information generation should succeed");
}

#[test]
fn test_static_linking() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let output = run_langc(&[&file_path, "--static"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Static linking should succeed");
}

#[test]
fn test_output_file_specification() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("my_program");
    let output_str = output_path.to_string_lossy();

    let output = run_langc(&[&file_path, "-o", &output_str]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Output file specification should succeed");
}

#[test]
fn test_verbose_output() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let output = run_langc(&[&file_path, "-v"]);
    
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "Verbose output should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain verbose information
    assert!(stdout.contains("Bulu Language Compiler") || stdout.contains("Version"), "Should show verbose output");
}

#[test]
fn test_invalid_input_file() {
    let output = run_langc(&["nonexistent.bu"]);
    
    // Should fail with error
    assert!(!output.status.success(), "Should fail for nonexistent file");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("error"), "Should show error message");
}

#[test]
fn test_invalid_extension() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "func main() {}").expect("Failed to write test file");
    
    let output = run_langc(&[&file_path.to_string_lossy()]);
    
    // Should fail with error about extension
    assert!(!output.status.success(), "Should fail for wrong extension");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".bu extension") || stderr.contains("error"), "Should show extension error");
}

#[test]
fn test_invalid_optimization_level() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let output = run_langc(&[&file_path, "-O", "invalid"]);
    
    // Should fail with error
    assert!(!output.status.success(), "Should fail for invalid optimization level");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid optimization level") || stderr.contains("error"), "Should show optimization error");
}

#[test]
fn test_invalid_emit_type() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let output = run_langc(&[&file_path, "--emit", "invalid"]);
    
    // Should fail with error
    assert!(!output.status.success(), "Should fail for invalid emit type");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid emit type") || stderr.contains("error"), "Should show emit type error");
}

#[test]
fn test_invalid_target() {
    let (_temp_dir, file_path) = create_test_file(r#"
func main() {
    println("Hello, World!")
}
"#);

    let output = run_langc(&[&file_path, "--target", "invalid-target"]);
    
    // Should fail with error
    assert!(!output.status.success(), "Should fail for invalid target");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported target") || stderr.contains("error"), "Should show target error");
}

#[test]
fn test_help_output() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "langc", "--", "--help"])
        .output()
        .expect("Failed to execute langc --help");
    
    assert!(output.status.success(), "Help should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Bulu Language Compiler"), "Should show help text");
    assert!(stdout.contains("--emit"), "Should show emit option");
    assert!(stdout.contains("--target"), "Should show target option");
    assert!(stdout.contains("-O"), "Should show optimization option");
}

#[test]
fn test_version_output() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "langc", "--", "--version"])
        .output()
        .expect("Failed to execute langc --version");
    
    assert!(output.status.success(), "Version should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("langc") || stdout.contains("0.1.0"), "Should show version");
}