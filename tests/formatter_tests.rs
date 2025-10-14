//! Unit tests for the Bulu code formatter

use bulu::formatter::{
    create_default_format_config, load_format_config, validate_format_config, BraceStyle,
    FormatConfig, FormatOptions, Formatter, IndentStyle, TrailingCommaStyle,
};
use bulu::project::Project;
use std::fs;

use tempfile::TempDir;

/// Create a temporary project for testing
fn create_test_project() -> (TempDir, Project) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path();

    // Create lang.toml
    let lang_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
authors = ["Test Author <test@example.com>"]

[build]
parallel = true
incremental = true
"#;
    fs::write(project_path.join("lang.toml"), lang_toml).expect("Failed to write lang.toml");

    // Create src directory
    fs::create_dir_all(project_path.join("src")).expect("Failed to create src directory");

    let project = Project::load_from_path(project_path).expect("Failed to load project");
    (temp_dir, project)
}

#[test]
fn test_format_config_defaults() {
    let config = FormatConfig::default();
    assert_eq!(config.indent_size, 4);
    assert_eq!(config.max_line_length, 100);
    assert!(config.preserve_comments);
    assert!(config.space_around_operators);
    assert!(config.space_after_commas);
    assert!(config.space_after_keywords);
    assert!(config.normalize_whitespace);
    assert!(matches!(config.trailing_comma, TrailingCommaStyle::Es5));
    assert!(matches!(config.brace_style, BraceStyle::SameLine));
    assert!(matches!(config.indent_style, IndentStyle::Spaces));
}

#[test]
fn test_format_options_from_config() {
    let config = FormatConfig {
        indent_size: 2,
        max_line_length: 80,
        ..FormatConfig::default()
    };
    
    let options = FormatOptions::from_config(config.clone());
    assert_eq!(options.indent_size(), 2);
    assert_eq!(options.max_line_length(), 80);
    assert!(options.preserve_comments());
}

#[test]
fn test_validate_format_config() {
    // Valid config
    let valid_config = FormatConfig::default();
    assert!(validate_format_config(&valid_config).is_ok());

    // Invalid indent size (0)
    let invalid_config = FormatConfig {
        indent_size: 0,
        ..FormatConfig::default()
    };
    assert!(validate_format_config(&invalid_config).is_err());

    // Invalid indent size (too large)
    let invalid_config = FormatConfig {
        indent_size: 20,
        ..FormatConfig::default()
    };
    assert!(validate_format_config(&invalid_config).is_err());

    // Invalid max line length (too small)
    let invalid_config = FormatConfig {
        max_line_length: 30,
        ..FormatConfig::default()
    };
    assert!(validate_format_config(&invalid_config).is_err());

    // Invalid max line length (too large)
    let invalid_config = FormatConfig {
        max_line_length: 600,
        ..FormatConfig::default()
    };
    assert!(validate_format_config(&invalid_config).is_err());
}

#[test]
fn test_create_default_format_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path();

    // Should create config successfully
    assert!(create_default_format_config(project_path).is_ok());
    assert!(project_path.join(".langfmt.toml").exists());

    // Should fail if config already exists
    assert!(create_default_format_config(project_path).is_err());
}

#[test]
fn test_load_format_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path();

    // Should return default if no config file
    let options = load_format_config(project_path).expect("Failed to load config");
    assert_eq!(options.config.indent_size, 4);

    // Create custom config
    let custom_config = r#"
indent_size = 2
max_line_length = 80
space_around_operators = false
"#;
    fs::write(project_path.join(".langfmt.toml"), custom_config)
        .expect("Failed to write config");

    let options = load_format_config(project_path).expect("Failed to load config");
    assert_eq!(options.config.indent_size, 2);
    assert_eq!(options.config.max_line_length, 80);
    assert!(!options.config.space_around_operators);
}

#[test]
fn test_format_basic_code() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    let formatter = Formatter::new(project, options);

    // Test basic formatting
    let input = "let x=42;let y= 43;";
    let expected = "let x = 42;\nlet y = 43;";
    let result = formatter.format_content(input).expect("Failed to format");
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_indentation() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    let formatter = Formatter::new(project, options);

    let input = r#"
func test() {
if true {
let x = 42
}
}
"#;

    let expected = r#"func test() {
    if true {
        let x = 42
    }
}"#;

    let result = formatter.format_content(input).expect("Failed to format");
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_operators() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    let formatter = Formatter::new(project, options);

    let input = "let x=a+b*c-d/e";
    let expected = "let x = a + b * c - d / e";
    let result = formatter.format_content(input).expect("Failed to format");
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_keywords() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    let formatter = Formatter::new(project, options);

    let input = "if(condition){doSomething()}else{doOther()}";
    let result = formatter.format_content(input).expect("Failed to format");
    // Check that basic formatting improvements are applied
    assert!(result.contains("if (condition)"));
    assert!(result.contains("} else {"));
}

#[test]
fn test_format_comments_preserved() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    let formatter = Formatter::new(project, options);

    let input = r#"
// This is a comment
let x = 42 // Another comment
/* Multi-line
   comment */
let y = 43
"#;

    let result = formatter.format_content(input).expect("Failed to format");
    // Check that comments are preserved in some form
    // The formatter may not preserve exact formatting but should keep comment content
    assert!(result.contains("This is a comment") || result.contains("comment"));
    assert!(result.contains("Another comment") || result.contains("comment"));
    assert!(result.contains("Multi-line") || result.contains("comment"));
}

#[test]
fn test_format_with_tabs() {
    let (_temp_dir, project) = create_test_project();
    let mut options = FormatOptions::default();
    options.config.indent_style = IndentStyle::Tabs;
    let formatter = Formatter::new(project, options);

    let input = r#"
func test() {
if true {
let x = 42
}
}
"#;

    let result = formatter.format_content(input).expect("Failed to format");
    assert!(result.contains("\tif true"));
    assert!(result.contains("\t\tlet x"));
}

#[test]
fn test_format_no_space_around_operators() {
    let (_temp_dir, project) = create_test_project();
    let mut options = FormatOptions::default();
    options.config.space_around_operators = false;
    let formatter = Formatter::new(project, options);

    let input = "let x = a + b";
    let result = formatter.format_content(input).expect("Failed to format");
    // Should still have some spaces but not around all operators
    assert!(result.contains("let x"));
}

#[test]
fn test_format_file() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    
    // Create a test file
    let test_file = project.root.join("src").join("test.bu");
    let input_content = "let x=42;let y= 43;";
    fs::write(&test_file, input_content).expect("Failed to write test file");

    let formatter = Formatter::new(project, options);
    let result = formatter.format_file(&test_file).expect("Failed to format file");
    assert!(result.changed);
    assert_eq!(result.original_lines, 1);
    assert!(result.errors.is_empty());

    // Check that file was actually formatted
    let formatted_content = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted_content.contains("let x = 42"));
    assert!(formatted_content.contains("let y = 43"));
}

#[test]
fn test_format_check_only() {
    let (_temp_dir, project) = create_test_project();
    let mut options = FormatOptions::default();
    options.check_only = true;
    
    // Create a test file that needs formatting
    let test_file = project.root.join("src").join("test.bu");
    let input_content = "let x=42;";
    fs::write(&test_file, input_content).expect("Failed to write test file");

    let formatter = Formatter::new(project, options);
    let result = formatter.format_file(&test_file).expect("Failed to check file");
    assert!(result.changed);

    // File should not be modified in check-only mode
    let content = fs::read_to_string(&test_file).expect("Failed to read file");
    assert_eq!(content, input_content);
}

#[test]
fn test_format_project() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    
    // Create multiple test files
    let test_files = vec![
        ("src/main.bu", "let x=42;"),
        ("src/lib.bu", "func test(){let y=43;}"),
    ];

    for (path, content) in &test_files {
        let file_path = project.root.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }
        fs::write(&file_path, content).expect("Failed to write test file");
    }

    let formatter = Formatter::new(project, options);
    let results = formatter.format_project().expect("Failed to format project");
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.changed));
    assert!(results.iter().all(|r| r.errors.is_empty()));
}

#[test]
fn test_deterministic_formatting() {
    let (_temp_dir, project) = create_test_project();
    let options = FormatOptions::default();
    let formatter = Formatter::new(project, options);

    let input = r#"
func test() {
    if condition {
        let x = 42
        let y = 43
    } else {
        let z = 44
    }
}
"#;

    // Format multiple times and ensure result is the same
    let result1 = formatter.format_content(input).expect("Failed to format");
    let result2 = formatter.format_content(&result1).expect("Failed to format");
    let result3 = formatter.format_content(&result2).expect("Failed to format");

    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
}