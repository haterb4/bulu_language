//! Unit tests for the Bulu code linter

use bulu::linter::{
    create_default_lint_config, load_lint_config, validate_lint_config, LintLevel, LintOptions,
    LintRules, Linter,
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

/// Helper function to create a linter and test file
fn create_linter_and_file(project: &Project, content: &str) -> (Linter, std::path::PathBuf) {
    let options = LintOptions::default();
    let linter = Linter::new(project.clone(), options);

    let test_file = project.root.join("src").join("test.bu");
    fs::write(&test_file, content).expect("Failed to write test file");

    (linter, test_file)
}

#[test]
fn test_lint_rules_defaults() {
    let rules = LintRules::default();
    assert_eq!(rules.unused_variables, LintLevel::Warn);
    assert_eq!(rules.unused_imports, LintLevel::Warn);
    assert_eq!(rules.unused_functions, LintLevel::Warn);
    assert_eq!(rules.unreachable_code, LintLevel::Warn);
    assert_eq!(rules.missing_docs, LintLevel::Allow);
    assert_eq!(rules.long_lines, LintLevel::Warn);
    assert_eq!(rules.naming_convention, LintLevel::Warn);
    assert_eq!(rules.complexity, LintLevel::Warn);
    assert_eq!(rules.performance, LintLevel::Warn);
    assert_eq!(rules.security, LintLevel::Error);
    assert_eq!(rules.max_line_length, 100);
    assert_eq!(rules.max_complexity, 4);
}

#[test]
fn test_lint_options_defaults() {
    let options = LintOptions::default();
    assert!(!options.verbose);
    assert!(!options.fix);
    assert!(options.max_warnings.is_none());
}

#[test]
fn test_validate_lint_config() {
    // Valid config
    let valid_rules = LintRules::default();
    assert!(validate_lint_config(&valid_rules).is_ok());

    // Invalid max_line_length (too small)
    let invalid_rules = LintRules {
        max_line_length: 30,
        ..LintRules::default()
    };
    assert!(validate_lint_config(&invalid_rules).is_err());

    // Invalid max_line_length (too large)
    let invalid_rules = LintRules {
        max_line_length: 600,
        ..LintRules::default()
    };
    assert!(validate_lint_config(&invalid_rules).is_err());

    // Invalid max_complexity (zero)
    let invalid_rules = LintRules {
        max_complexity: 0,
        ..LintRules::default()
    };
    assert!(validate_lint_config(&invalid_rules).is_err());

    // Invalid max_complexity (too large)
    let invalid_rules = LintRules {
        max_complexity: 25,
        ..LintRules::default()
    };
    assert!(validate_lint_config(&invalid_rules).is_err());
}

#[test]
fn test_create_default_lint_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path();

    // Should create config successfully
    assert!(create_default_lint_config(project_path).is_ok());
    assert!(project_path.join(".langlint.toml").exists());

    // Should fail if config already exists
    assert!(create_default_lint_config(project_path).is_err());
}

#[test]
fn test_load_lint_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path();

    // Should return default if no config file
    let options = load_lint_config(project_path).expect("Failed to load config");
    assert_eq!(options.rules.max_line_length, 100);

    // Create custom config
    let custom_config = r#"
unused_variables = "error"
max_line_length = 80
max_complexity = 6
"#;
    fs::write(project_path.join(".langlint.toml"), custom_config).expect("Failed to write config");

    let options = load_lint_config(project_path).expect("Failed to load config");
    assert_eq!(options.rules.unused_variables, LintLevel::Error);
    assert_eq!(options.rules.max_line_length, 80);
    assert_eq!(options.rules.max_complexity, 6);
}

#[test]
fn test_detect_unused_variables() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func test() {
    let unused_var = 42
    let used_var = 43
    print(used_var)
}
"#;
    let (linter, test_file) = create_linter_and_file(&project, content);

    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect unused variable
    let unused_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "unused-variable")
        .collect();
    assert!(!unused_issues.is_empty());
    assert!(unused_issues[0].message.contains("unused_var"));
}

#[test]
fn test_detect_unused_imports() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
import unused_module
import used_module

func test() {
    used_module.function()
}
"#;
    let (linter, test_file) = create_linter_and_file(&project, content);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect unused import
    let unused_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "unused-import")
        .collect();
    assert!(!unused_issues.is_empty());
    assert!(unused_issues[0].message.contains("unused_module"));
}

#[test]
fn test_detect_unreachable_code() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func test() {
    return 42
    let unreachable = 43  // This should be detected
}
"#;
    let (linter, test_file) = create_linter_and_file(&project, content);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect unreachable code
    let unreachable_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "unreachable-code")
        .collect();
    assert!(!unreachable_issues.is_empty());
}

#[test]
fn test_detect_long_lines() {
    let (_temp_dir, project) = create_test_project();
    let long_line = "let very_long_variable_name = very_long_function_name_that_exceeds_the_maximum_line_length_limit_and_should_be_detected_by_the_linter()";
    let (linter, test_file) = create_linter_and_file(&project, long_line);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect long line
    let long_line_issues: Vec<_> = issues.iter().filter(|i| i.rule == "long-line").collect();
    assert!(!long_line_issues.is_empty());
}

#[test]
fn test_detect_naming_conventions() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func BadFunctionName() {  // Should be camelCase
    // function body
}

struct bad_struct_name {  // Should be PascalCase
    field: int32
}
"#;
    let (linter, test_file) = create_linter_and_file(&project, content);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect naming convention issues
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "naming-convention")
        .collect();
    assert!(!naming_issues.is_empty());
}

#[test]
fn test_detect_missing_docs() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func undocumented_function() {
    // This function has no documentation
}

// This function has documentation
func documented_function() {
    // This function is documented
}
"#;
    let mut options = LintOptions::default();
    options.rules.missing_docs = LintLevel::Warn;
    let linter = Linter::new(project.clone(), options);
    
    let test_file = project.root.join("src").join("test.bu");
    fs::write(&test_file, content).expect("Failed to write test file");
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect missing documentation
    let doc_issues: Vec<_> = issues.iter().filter(|i| i.rule == "missing-docs").collect();
    assert!(!doc_issues.is_empty());
    assert!(doc_issues[0].message.contains("undocumented_function"));
}

#[test]
fn test_detect_high_complexity() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func complex_function() {
    if condition1 {
        if condition2 {
            if condition3 {  // This should exceed max_complexity of 2
                do_something()
            }
        }
    }
}
"#;
    let mut options = LintOptions::default();
    options.rules.max_complexity = 2; // Set low threshold for testing
    let linter = Linter::new(project.clone(), options);
    
    let test_file = project.root.join("src").join("test.bu");
    fs::write(&test_file, content).expect("Failed to write test file");
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect high complexity
    let complexity_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "high-complexity")
        .collect();
    assert!(!complexity_issues.is_empty());
}

#[test]
fn test_detect_performance_issues() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func performance_issue() {
    for i in 0..<100 {
        let result = "prefix" + string(i)  // String concatenation in loop
    }
}
"#;
    let (linter, test_file) = create_linter_and_file(&project, content);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect performance issue
    let perf_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "performance-string-concat")
        .collect();
    assert!(!perf_issues.is_empty());
}

#[test]
fn test_detect_security_issues() {
    let (_temp_dir, project) = create_test_project();
    let content = r#"
func security_issues() {
    let password = "hardcoded_secret"  // Hardcoded secret
    let query = "SELECT * FROM users WHERE id = " + user_id  // SQL injection
}
"#;
    let (linter, test_file) = create_linter_and_file(&project, content);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect security issues
    let security_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule.starts_with("security-"))
        .collect();
    assert!(!security_issues.is_empty());
}

#[test]
fn test_lint_project() {
    let (_temp_dir, project) = create_test_project();
    let options = LintOptions::default();

    // Create multiple test files with various issues
    let test_files = vec![
        ("src/main.bu", "let unused = 42\nprint(\"hello\")"),
        ("src/lib.bu", "func BadName() { return 42 }"),
    ];

    for (path, content) in &test_files {
        let file_path = project.root.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }
        fs::write(&file_path, content).expect("Failed to write test file");
    }

    let linter = Linter::new(project.clone(), options);
    let result = linter.lint_project().expect("Failed to lint project");
    assert_eq!(result.files_checked, 2);
    assert!(!result.issues.is_empty());
    assert!(result.warnings > 0);
}

#[test]
fn test_lint_level_ordering() {
    // Test that lint levels are ordered correctly
    assert!(LintLevel::Allow < LintLevel::Warn);
    assert!(LintLevel::Warn < LintLevel::Error);
}

#[test]
fn test_lint_with_disabled_rules() {
    let (_temp_dir, project) = create_test_project();
    let content = "let unused = 42";
    let mut options = LintOptions::default();
    options.rules.unused_variables = LintLevel::Allow;
    let linter = Linter::new(project.clone(), options);
    
    let test_file = project.root.join("src").join("test.bu");
    fs::write(&test_file, content).expect("Failed to write test file");
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should not detect unused variable when rule is disabled
    let unused_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "unused-variable")
        .collect();
    assert!(unused_issues.is_empty());
}

#[test]
fn test_lint_issue_suggestions() {
    let (_temp_dir, project) = create_test_project();
    let content = "let unused_var = 42";
    let (linter, test_file) = create_linter_and_file(&project, content);
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Check that issues have suggestions
    let unused_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "unused-variable")
        .collect();
    assert!(!unused_issues.is_empty());
    assert!(unused_issues[0].suggestion.is_some());
}

#[test]
fn test_custom_max_line_length() {
    let (_temp_dir, project) = create_test_project();
    let content = "let variable_with_a_very_long_name_that_exceeds_fifty_characters = 42";
    let mut options = LintOptions::default();
    options.rules.max_line_length = 50; // Set custom limit
    let linter = Linter::new(project.clone(), options);
    
    let test_file = project.root.join("src").join("test.bu");
    fs::write(&test_file, content).expect("Failed to write test file");
    let (issues, _) = linter.lint_file(&test_file).expect("Failed to lint file");

    // Should detect long line with custom limit
    let long_line_issues: Vec<_> = issues.iter().filter(|i| i.rule == "long-line").collect();
    assert!(!long_line_issues.is_empty());
    assert!(long_line_issues[0].message.contains("50"));
}
