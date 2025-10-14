//! Code linter for Bulu source files

use crate::project::Project;
use crate::{BuluError, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Lint severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    Allow,
    Warn,
    Error,
}

/// A single lint issue
#[derive(Debug, Clone)]
pub struct LintIssue {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub level: LintLevel,
    pub rule: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Linting options
#[derive(Debug, Clone)]
pub struct LintOptions {
    pub verbose: bool,
    pub fix: bool,
    pub max_warnings: Option<usize>,
    pub rules: LintRules,
}

/// Configurable lint rules that can be loaded from .langlint.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintRules {
    #[serde(default = "default_unused_variables")]
    pub unused_variables: LintLevel,
    #[serde(default = "default_unused_imports")]
    pub unused_imports: LintLevel,
    #[serde(default = "default_unused_functions")]
    pub unused_functions: LintLevel,
    #[serde(default = "default_unreachable_code")]
    pub unreachable_code: LintLevel,
    #[serde(default = "default_missing_docs")]
    pub missing_docs: LintLevel,
    #[serde(default = "default_long_lines")]
    pub long_lines: LintLevel,
    #[serde(default = "default_naming_convention")]
    pub naming_convention: LintLevel,
    #[serde(default = "default_complexity")]
    pub complexity: LintLevel,
    #[serde(default = "default_performance")]
    pub performance: LintLevel,
    #[serde(default = "default_security")]
    pub security: LintLevel,
    #[serde(default = "default_max_line_length")]
    pub max_line_length: usize,
    #[serde(default = "default_max_complexity")]
    pub max_complexity: usize,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            fix: false,
            max_warnings: None,
            rules: LintRules::default(),
        }
    }
}

// Default value functions for serde
fn default_unused_variables() -> LintLevel {
    LintLevel::Warn
}
fn default_unused_imports() -> LintLevel {
    LintLevel::Warn
}
fn default_unused_functions() -> LintLevel {
    LintLevel::Warn
}
fn default_unreachable_code() -> LintLevel {
    LintLevel::Warn
}
fn default_missing_docs() -> LintLevel {
    LintLevel::Allow
}
fn default_long_lines() -> LintLevel {
    LintLevel::Warn
}
fn default_naming_convention() -> LintLevel {
    LintLevel::Warn
}
fn default_complexity() -> LintLevel {
    LintLevel::Warn
}
fn default_performance() -> LintLevel {
    LintLevel::Warn
}
fn default_security() -> LintLevel {
    LintLevel::Error
}
fn default_max_line_length() -> usize {
    100
}
fn default_max_complexity() -> usize {
    4
}

impl Default for LintRules {
    fn default() -> Self {
        Self {
            unused_variables: default_unused_variables(),
            unused_imports: default_unused_imports(),
            unused_functions: default_unused_functions(),
            unreachable_code: default_unreachable_code(),
            missing_docs: default_missing_docs(),
            long_lines: default_long_lines(),
            naming_convention: default_naming_convention(),
            complexity: default_complexity(),
            performance: default_performance(),
            security: default_security(),
            max_line_length: default_max_line_length(),
            max_complexity: default_max_complexity(),
        }
    }
}

/// Lint results for the entire project
#[derive(Debug)]
pub struct LintResult {
    pub files_checked: usize,
    pub issues: Vec<LintIssue>,
    pub errors: usize,
    pub warnings: usize,
    pub fixed: usize,
}

/// Code linter for Bulu projects
pub struct Linter {
    project: Project,
    options: LintOptions,
}

impl Linter {
    pub fn new(project: Project, options: LintOptions) -> Self {
        Self { project, options }
    }

    /// Lint all source files in the project
    pub fn lint_project(&self) -> Result<LintResult> {
        if self.options.verbose {
            println!(
                "{} Linting project '{}'...",
                "Linting".green().bold(),
                self.project.config.package.name
            );
        }

        let source_files = self.project.source_files()?;

        if source_files.is_empty() {
            println!("{} No source files found", "Warning".yellow().bold());
            return Ok(LintResult {
                files_checked: 0,
                issues: Vec::new(),
                errors: 0,
                warnings: 0,
                fixed: 0,
            });
        }

        let mut all_issues = Vec::new();
        let mut fixed_count = 0;

        for source_file in &source_files {
            if self.options.verbose {
                println!("{} {}", "Checking".cyan().bold(), source_file.display());
            }

            let (issues, fixed) = self.lint_file(source_file)?;
            all_issues.extend(issues);
            fixed_count += fixed;
        }

        // Sort issues by severity and location
        all_issues.sort_by(|a, b| {
            a.level
                .cmp(&b.level)
                .then(a.file.cmp(&b.file))
                .then(a.line.cmp(&b.line))
                .then(a.column.cmp(&b.column))
        });

        let errors = all_issues
            .iter()
            .filter(|i| i.level == LintLevel::Error)
            .count();
        let warnings = all_issues
            .iter()
            .filter(|i| i.level == LintLevel::Warn)
            .count();

        // Print issues
        for issue in &all_issues {
            self.print_issue(issue);
        }

        // Print summary
        self.print_summary(source_files.len(), errors, warnings, fixed_count);

        Ok(LintResult {
            files_checked: source_files.len(),
            issues: all_issues,
            errors,
            warnings,
            fixed: fixed_count,
        })
    }

    /// Lint a single source file
    pub fn lint_file(&self, file_path: &Path) -> Result<(Vec<LintIssue>, usize)> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| BuluError::Other(format!("Failed to read file: {}", e)))?;

        let mut issues = Vec::new();
        let mut fixed_count = 0;

        // Run various lint checks
        issues.extend(self.check_unused_variables(file_path, &content));
        issues.extend(self.check_unused_imports(file_path, &content));
        issues.extend(self.check_unreachable_code(file_path, &content));
        issues.extend(self.check_long_lines(file_path, &content));
        issues.extend(self.check_naming_conventions(file_path, &content));
        issues.extend(self.check_missing_docs(file_path, &content));
        issues.extend(self.check_complexity(file_path, &content));
        issues.extend(self.check_performance(file_path, &content));
        issues.extend(self.check_security(file_path, &content));

        // Apply fixes if requested
        if self.options.fix {
            fixed_count = self.apply_fixes(file_path, &content, &issues)?;
        }

        Ok((issues, fixed_count))
    }

    /// Check for unused variables
    fn check_unused_variables(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.unused_variables == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Simple pattern matching for unused variables
        // In a real implementation, this would use semantic analysis
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Look for variable declarations
            if trimmed.starts_with("let ") {
                if let Some(var_name) = self.extract_variable_name(trimmed) {
                    // Skip variables that start with underscore (intentionally unused)
                    if var_name.starts_with('_') {
                        continue;
                    }
                    // Check if variable is used later in the file
                    if !self.is_variable_used(&var_name, content, line_num) {
                        issues.push(LintIssue {
                            file: file_path.to_path_buf(),
                            line: line_num + 1,
                            column: trimmed.find(&var_name).unwrap_or(0) + 1,
                            level: self.options.rules.unused_variables.clone(),
                            rule: "unused-variable".to_string(),
                            message: format!("Variable '{}' is declared but never used", var_name),
                            suggestion: Some(format!("Consider removing the variable or prefixing with '_' if intentionally unused")),
                        });
                    }
                }
            }
        }

        issues
    }

    /// Check for unused imports
    fn check_unused_imports(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.unused_imports == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("import ") {
                // Extract imported names and check if they're used
                if let Some(import_name) = self.extract_import_name(trimmed) {
                    if !self.is_import_used(&import_name, content, line_num) {
                        issues.push(LintIssue {
                            file: file_path.to_path_buf(),
                            line: line_num + 1,
                            column: 1,
                            level: self.options.rules.unused_imports.clone(),
                            rule: "unused-import".to_string(),
                            message: format!("Import '{}' is not used", import_name),
                            suggestion: Some("Consider removing this import".to_string()),
                        });
                    }
                }
            }
        }

        issues
    }

    /// Check for unreachable code
    fn check_unreachable_code(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.unreachable_code == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();
        let mut after_return = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("return ") || trimmed == "return" {
                after_return = true;
                continue;
            }

            if after_return
                && !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("}")
            {
                issues.push(LintIssue {
                    file: file_path.to_path_buf(),
                    line: line_num + 1,
                    column: 1,
                    level: self.options.rules.unreachable_code.clone(),
                    rule: "unreachable-code".to_string(),
                    message: "Code after return statement is unreachable".to_string(),
                    suggestion: Some("Remove unreachable code".to_string()),
                });
                break; // Only report the first unreachable line
            }

            if trimmed.starts_with("}") {
                after_return = false;
            }
        }

        issues
    }

    /// Check for long lines
    fn check_long_lines(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.long_lines == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();
        let max_line_length = self.options.rules.max_line_length;

        for (line_num, line) in content.lines().enumerate() {
            if line.len() > max_line_length {
                issues.push(LintIssue {
                    file: file_path.to_path_buf(),
                    line: line_num + 1,
                    column: max_line_length + 1,
                    level: self.options.rules.long_lines.clone(),
                    rule: "long-line".to_string(),
                    message: format!(
                        "Line is {} characters long, exceeds maximum of {}",
                        line.len(),
                        max_line_length
                    ),
                    suggestion: Some("Consider breaking this line into multiple lines".to_string()),
                });
            }
        }

        issues
    }

    /// Check naming conventions
    fn check_naming_conventions(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.naming_convention == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check function names (should be camelCase)
            if trimmed.starts_with("func ") {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    if !self.is_camel_case(&func_name) && !func_name.starts_with("Test") {
                        issues.push(LintIssue {
                            file: file_path.to_path_buf(),
                            line: line_num + 1,
                            column: trimmed.find(&func_name).unwrap_or(0) + 1,
                            level: self.options.rules.naming_convention.clone(),
                            rule: "naming-convention".to_string(),
                            message: format!(
                                "Function '{}' should use camelCase naming",
                                func_name
                            ),
                            suggestion: Some(format!(
                                "Consider renaming to '{}'",
                                self.to_camel_case(&func_name)
                            )),
                        });
                    }
                }
            }

            // Check struct names (should be PascalCase)
            if trimmed.starts_with("struct ") {
                if let Some(struct_name) = self.extract_struct_name(trimmed) {
                    if !self.is_pascal_case(&struct_name) {
                        issues.push(LintIssue {
                            file: file_path.to_path_buf(),
                            line: line_num + 1,
                            column: trimmed.find(&struct_name).unwrap_or(0) + 1,
                            level: self.options.rules.naming_convention.clone(),
                            rule: "naming-convention".to_string(),
                            message: format!(
                                "Struct '{}' should use PascalCase naming",
                                struct_name
                            ),
                            suggestion: Some(format!(
                                "Consider renaming to '{}'",
                                self.to_pascal_case(&struct_name)
                            )),
                        });
                    }
                }
            }
        }

        issues
    }

    /// Check for missing documentation
    fn check_missing_docs(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.missing_docs == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check for public functions without documentation
            if trimmed.starts_with("func ") && !trimmed.contains("test") {
                // Check if previous line has documentation comment
                let has_doc = line_num > 0
                    && (lines[line_num - 1].trim().starts_with("/**")
                        || lines[line_num - 1].trim().starts_with("//"));

                if !has_doc {
                    if let Some(func_name) = self.extract_function_name(trimmed) {
                        issues.push(LintIssue {
                            file: file_path.to_path_buf(),
                            line: line_num + 1,
                            column: 1,
                            level: self.options.rules.missing_docs.clone(),
                            rule: "missing-docs".to_string(),
                            message: format!("Function '{}' is missing documentation", func_name),
                            suggestion: Some(
                                "Add a documentation comment above the function".to_string(),
                            ),
                        });
                    }
                }
            }
        }

        issues
    }

    /// Check code complexity
    fn check_complexity(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.complexity == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();
        let mut nesting_level = 0;
        let max_nesting = self.options.rules.max_complexity;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Count nesting level
            if trimmed.contains('{') {
                nesting_level += 1;
                if nesting_level > max_nesting {
                    issues.push(LintIssue {
                        file: file_path.to_path_buf(),
                        line: line_num + 1,
                        column: 1,
                        level: self.options.rules.complexity.clone(),
                        rule: "high-complexity".to_string(),
                        message: format!(
                            "Code nesting level {} exceeds maximum of {}",
                            nesting_level, max_nesting
                        ),
                        suggestion: Some(
                            "Consider extracting nested code into separate functions".to_string(),
                        ),
                    });
                }
            }

            if trimmed.contains('}') {
                nesting_level = nesting_level.saturating_sub(1);
            }
        }

        issues
    }

    /// Check for performance issues
    fn check_performance(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.performance == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for string concatenation in loops
            if (trimmed.contains("for ") || trimmed.contains("while "))
                && content
                    .lines()
                    .skip(line_num)
                    .take(10)
                    .any(|l| l.contains(" + ") && l.contains("string"))
            {
                issues.push(LintIssue {
                    file: file_path.to_path_buf(),
                    line: line_num + 1,
                    column: 1,
                    level: self.options.rules.performance.clone(),
                    rule: "performance-string-concat".to_string(),
                    message: "String concatenation in loop may cause performance issues"
                        .to_string(),
                    suggestion: Some(
                        "Consider using a string builder or collecting into an array".to_string(),
                    ),
                });
            }
        }

        issues
    }

    /// Check for security issues
    fn check_security(&self, file_path: &Path, content: &str) -> Vec<LintIssue> {
        if self.options.rules.security == LintLevel::Allow {
            return Vec::new();
        }

        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for potential SQL injection
            if trimmed.contains("query") && trimmed.contains("+") && trimmed.contains("\"") {
                issues.push(LintIssue {
                    file: file_path.to_path_buf(),
                    line: line_num + 1,
                    column: 1,
                    level: self.options.rules.security.clone(),
                    rule: "security-sql-injection".to_string(),
                    message: "Potential SQL injection vulnerability detected".to_string(),
                    suggestion: Some(
                        "Use parameterized queries instead of string concatenation".to_string(),
                    ),
                });
            }

            // Check for hardcoded secrets
            if trimmed.contains("password") || trimmed.contains("secret") || trimmed.contains("key")
            {
                if trimmed.contains("=") && trimmed.contains("\"") {
                    issues.push(LintIssue {
                        file: file_path.to_path_buf(),
                        line: line_num + 1,
                        column: 1,
                        level: self.options.rules.security.clone(),
                        rule: "security-hardcoded-secret".to_string(),
                        message: "Potential hardcoded secret detected".to_string(),
                        suggestion: Some(
                            "Use environment variables or secure configuration for secrets"
                                .to_string(),
                        ),
                    });
                }
            }
        }

        issues
    }

    /// Apply automatic fixes to issues
    fn apply_fixes(
        &self,
        _file_path: &Path,
        _content: &str,
        _issues: &[LintIssue],
    ) -> Result<usize> {
        // In a real implementation, this would apply automatic fixes
        // For now, just return 0 fixes applied
        Ok(0)
    }

    /// Print a single lint issue
    fn print_issue(&self, issue: &LintIssue) {
        let level_str = match issue.level {
            LintLevel::Error => "error".red().bold(),
            LintLevel::Warn => "warning".yellow().bold(),
            LintLevel::Allow => return, // Don't print allowed issues
        };

        println!(
            "{}:{}:{}: {}: {} [{}]",
            issue.file.display(),
            issue.line,
            issue.column,
            level_str,
            issue.message,
            issue.rule.cyan()
        );

        if let Some(suggestion) = &issue.suggestion {
            println!("  {} {}", "help:".cyan().bold(), suggestion);
        }
    }

    /// Print summary of lint results
    fn print_summary(&self, files_checked: usize, errors: usize, warnings: usize, fixed: usize) {
        println!();

        if errors == 0 && warnings == 0 {
            println!(
                "{} Checked {} files, no issues found",
                "Finished".green().bold(),
                files_checked
            );
        } else {
            let mut summary = format!("Checked {} files", files_checked);

            if errors > 0 {
                summary.push_str(&format!(", {} errors", errors));
            }

            if warnings > 0 {
                summary.push_str(&format!(", {} warnings", warnings));
            }

            if fixed > 0 {
                summary.push_str(&format!(", {} fixed", fixed));
            }

            let status = if errors > 0 { "Failed" } else { "Finished" };
            let color = if errors > 0 { "red" } else { "yellow" };

            match color {
                "red" => println!("{} {}", status.red().bold(), summary),
                "yellow" => println!("{} {}", status.yellow().bold(), summary),
                _ => println!("{} {}", status.green().bold(), summary),
            }
        }
    }

    // Helper methods for parsing and checking
    fn extract_variable_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("let ") {
            let after_let = &line[start + 4..];
            if let Some(end) = after_let.find(|c: char| c == ':' || c == '=' || c.is_whitespace()) {
                Some(after_let[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("func ") {
            let after_func = &line[start + 5..];
            if let Some(end) = after_func.find('(') {
                Some(after_func[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn extract_struct_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("struct ") {
            let after_struct = &line[start + 7..];
            if let Some(end) = after_struct.find(|c: char| c == '{' || c.is_whitespace()) {
                Some(after_struct[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn extract_import_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("import ") {
            let after_import = &line[start + 7..];
            Some(after_import.trim().to_string())
        } else {
            None
        }
    }

    fn is_variable_used(&self, var_name: &str, content: &str, declaration_line: usize) -> bool {
        for (line_num, line) in content.lines().enumerate() {
            if line_num > declaration_line && line.contains(var_name) {
                return true;
            }
        }
        false
    }

    fn is_import_used(&self, import_name: &str, content: &str, import_line: usize) -> bool {
        for (line_num, line) in content.lines().enumerate() {
            if line_num > import_line && line.contains(import_name) {
                return true;
            }
        }
        false
    }

    fn is_camel_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        first_char.is_lowercase() && !name.contains('_')
    }

    fn is_pascal_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        first_char.is_uppercase() && !name.contains('_')
    }

    fn to_camel_case(&self, name: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for ch in name.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                capitalize_next = false;
            } else {
                result.push(ch.to_lowercase().next().unwrap_or(ch));
            }
        }

        result
    }

    fn to_pascal_case(&self, name: &str) -> String {
        let camel = self.to_camel_case(name);
        if let Some(first_char) = camel.chars().next() {
            first_char.to_uppercase().collect::<String>() + &camel[1..]
        } else {
            camel
        }
    }
}
// Load linting configuration from .langlint.toml
pub fn load_lint_config(project_root: &Path) -> Result<LintOptions> {
    let config_path = project_root.join(".langlint.toml");

    if !config_path.exists() {
        return Ok(LintOptions::default());
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| BuluError::Other(format!("Failed to read .langlint.toml: {}", e)))?;

    // Parse TOML configuration
    let lint_rules: LintRules = toml::from_str(&config_content)
        .map_err(|e| BuluError::Other(format!("Failed to parse .langlint.toml: {}", e)))?;

    Ok(LintOptions {
        rules: lint_rules,
        ..LintOptions::default()
    })
}

/// Create a default .langlint.toml configuration file
pub fn create_default_lint_config(project_root: &Path) -> Result<()> {
    let config_path = project_root.join(".langlint.toml");

    if config_path.exists() {
        return Err(BuluError::Other(
            ".langlint.toml already exists. Remove it first if you want to recreate it."
                .to_string(),
        ));
    }

    let default_rules = LintRules::default();
    let _config_content = toml::to_string_pretty(&default_rules)
        .map_err(|e| BuluError::Other(format!("Failed to serialize default config: {}", e)))?;

    // Add comments to make the config file more user-friendly
    let commented_config = format!(
        r#"# Bulu Language Linter Configuration
# This file configures how the 'lang lint' command checks your code.

# Unused variable detection: "allow", "warn", or "error"
unused_variables = "{}"

# Unused import detection: "allow", "warn", or "error"
unused_imports = "{}"

# Unused function detection: "allow", "warn", or "error"
unused_functions = "{}"

# Unreachable code detection: "allow", "warn", or "error"
unreachable_code = "{}"

# Missing documentation detection: "allow", "warn", or "error"
missing_docs = "{}"

# Long line detection: "allow", "warn", or "error"
long_lines = "{}"

# Naming convention checking: "allow", "warn", or "error"
naming_convention = "{}"

# Code complexity checking: "allow", "warn", or "error"
complexity = "{}"

# Performance issue detection: "allow", "warn", or "error"
performance = "{}"

# Security issue detection: "allow", "warn", or "error"
security = "{}"

# Maximum line length before warning
max_line_length = {}

# Maximum nesting level before warning
max_complexity = {}
"#,
        format!("{:?}", default_rules.unused_variables).to_lowercase(),
        format!("{:?}", default_rules.unused_imports).to_lowercase(),
        format!("{:?}", default_rules.unused_functions).to_lowercase(),
        format!("{:?}", default_rules.unreachable_code).to_lowercase(),
        format!("{:?}", default_rules.missing_docs).to_lowercase(),
        format!("{:?}", default_rules.long_lines).to_lowercase(),
        format!("{:?}", default_rules.naming_convention).to_lowercase(),
        format!("{:?}", default_rules.complexity).to_lowercase(),
        format!("{:?}", default_rules.performance).to_lowercase(),
        format!("{:?}", default_rules.security).to_lowercase(),
        default_rules.max_line_length,
        default_rules.max_complexity,
    );

    fs::write(&config_path, commented_config)
        .map_err(|e| BuluError::Other(format!("Failed to write .langlint.toml: {}", e)))?;

    println!("Created default .langlint.toml configuration file");
    Ok(())
}

/// Validate a lint configuration
pub fn validate_lint_config(rules: &LintRules) -> Result<()> {
    if rules.max_line_length < 40 {
        return Err(BuluError::Other(
            "max_line_length should be at least 40 characters".to_string(),
        ));
    }

    if rules.max_line_length > 500 {
        return Err(BuluError::Other(
            "max_line_length should not exceed 500 characters".to_string(),
        ));
    }

    if rules.max_complexity == 0 {
        return Err(BuluError::Other(
            "max_complexity must be greater than 0".to_string(),
        ));
    }

    if rules.max_complexity > 20 {
        return Err(BuluError::Other(
            "max_complexity should not exceed 20 levels".to_string(),
        ));
    }

    Ok(())
}
