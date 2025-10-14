//! Code formatter for Bulu source files

use crate::project::Project;
use crate::{BuluError, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Formatting configuration that can be loaded from .langfmt.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_max_line_length")]
    pub max_line_length: usize,
    #[serde(default = "default_preserve_comments")]
    pub preserve_comments: bool,
    #[serde(default = "default_space_around_operators")]
    pub space_around_operators: bool,
    #[serde(default = "default_space_after_commas")]
    pub space_after_commas: bool,
    #[serde(default = "default_space_after_keywords")]
    pub space_after_keywords: bool,
    #[serde(default = "default_normalize_whitespace")]
    pub normalize_whitespace: bool,
    #[serde(default = "default_trailing_comma")]
    pub trailing_comma: TrailingCommaStyle,
    #[serde(default = "default_brace_style")]
    pub brace_style: BraceStyle,
    #[serde(default = "default_indent_style")]
    pub indent_style: IndentStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailingCommaStyle {
    Never,
    Always,
    Es5, // Only where valid in ES5 (arrays, objects)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BraceStyle {
    SameLine,         // K&R style: if (condition) {
    NextLine,         // Allman style: if (condition)\n{
    NextLineIndented, // GNU style: if (condition)\n  {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

// Default value functions for serde
fn default_indent_size() -> usize {
    4
}
fn default_max_line_length() -> usize {
    100
}
fn default_preserve_comments() -> bool {
    true
}
fn default_space_around_operators() -> bool {
    true
}
fn default_space_after_commas() -> bool {
    true
}
fn default_space_after_keywords() -> bool {
    true
}
fn default_normalize_whitespace() -> bool {
    true
}
fn default_trailing_comma() -> TrailingCommaStyle {
    TrailingCommaStyle::Es5
}
fn default_brace_style() -> BraceStyle {
    BraceStyle::SameLine
}
fn default_indent_style() -> IndentStyle {
    IndentStyle::Spaces
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: default_indent_size(),
            max_line_length: default_max_line_length(),
            preserve_comments: default_preserve_comments(),
            space_around_operators: default_space_around_operators(),
            space_after_commas: default_space_after_commas(),
            space_after_keywords: default_space_after_keywords(),
            normalize_whitespace: default_normalize_whitespace(),
            trailing_comma: default_trailing_comma(),
            brace_style: default_brace_style(),
            indent_style: default_indent_style(),
        }
    }
}

/// Formatting options (runtime configuration)
#[derive(Debug, Clone)]
pub struct FormatOptions {
    pub check_only: bool,
    pub verbose: bool,
    pub config: FormatConfig,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            check_only: false,
            verbose: false,
            config: FormatConfig::default(),
        }
    }
}

impl FormatOptions {
    /// Create FormatOptions from a FormatConfig
    pub fn from_config(config: FormatConfig) -> Self {
        Self {
            check_only: false,
            verbose: false,
            config,
        }
    }

    /// Get indent size from config
    pub fn indent_size(&self) -> usize {
        self.config.indent_size
    }

    /// Get max line length from config
    pub fn max_line_length(&self) -> usize {
        self.config.max_line_length
    }

    /// Check if comments should be preserved
    pub fn preserve_comments(&self) -> bool {
        self.config.preserve_comments
    }
}

/// Formatting result for a single file
#[derive(Debug)]
pub struct FormatResult {
    pub file: PathBuf,
    pub changed: bool,
    pub original_lines: usize,
    pub formatted_lines: usize,
    pub errors: Vec<String>,
}

/// Code formatter for Bulu projects
pub struct Formatter {
    project: Project,
    options: FormatOptions,
}

impl Formatter {
    pub fn new(project: Project, options: FormatOptions) -> Self {
        Self { project, options }
    }

    /// Format all source files in the project
    pub fn format_project(&self) -> Result<Vec<FormatResult>> {
        if self.options.verbose {
            println!(
                "{} Formatting project '{}'...",
                "Formatting".green().bold(),
                self.project.config.package.name
            );
        }

        let source_files = self.project.source_files()?;

        if source_files.is_empty() {
            println!("{} No source files found", "Warning".yellow().bold());
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut total_changed = 0;

        for source_file in &source_files {
            if self.options.verbose {
                println!("{} {}", "Formatting".cyan().bold(), source_file.display());
            }

            match self.format_file(source_file) {
                Ok(result) => {
                    if result.changed {
                        total_changed += 1;
                        if !self.options.check_only {
                            println!("  {} {}", "Formatted".green(), source_file.display());
                        } else {
                            println!(
                                "  {} {} (would be formatted)",
                                "Check".yellow(),
                                source_file.display()
                            );
                        }
                    } else if self.options.verbose {
                        println!("  {} {} (no changes)", "OK".green(), source_file.display());
                    }
                    results.push(result);
                }
                Err(e) => {
                    println!(
                        "  {} {} - {}",
                        "Error".red().bold(),
                        source_file.display(),
                        e
                    );
                    results.push(FormatResult {
                        file: source_file.clone(),
                        changed: false,
                        original_lines: 0,
                        formatted_lines: 0,
                        errors: vec![e.to_string()],
                    });
                }
            }
        }

        // Print summary
        if self.options.check_only {
            if total_changed > 0 {
                println!(
                    "{} {} files would be formatted",
                    "Check".yellow().bold(),
                    total_changed
                );
            } else {
                println!(
                    "{} All files are properly formatted",
                    "Check".green().bold()
                );
            }
        } else {
            if total_changed > 0 {
                println!(
                    "{} Formatted {} files",
                    "Finished".green().bold(),
                    total_changed
                );
            } else {
                println!(
                    "{} All files were already formatted",
                    "Finished".green().bold()
                );
            }
        }

        Ok(results)
    }

    /// Format a single source file
    pub fn format_file(&self, file_path: &Path) -> Result<FormatResult> {
        let original_content = fs::read_to_string(file_path)
            .map_err(|e| BuluError::Other(format!("Failed to read file: {}", e)))?;

        let original_lines = original_content.lines().count();

        // Format the content
        let formatted_content = self.format_content(&original_content)?;
        let formatted_lines = formatted_content.lines().count();

        let changed = original_content != formatted_content;

        // Write back if changed and not in check-only mode
        if changed && !self.options.check_only {
            fs::write(file_path, &formatted_content)
                .map_err(|e| BuluError::Other(format!("Failed to write formatted file: {}", e)))?;
        }

        Ok(FormatResult {
            file: file_path.to_path_buf(),
            changed,
            original_lines,
            formatted_lines,
            errors: Vec::new(),
        })
    }

    /// Format the content of a source file
    pub fn format_content(&self, content: &str) -> Result<String> {
        // Handle simple single-line cases first
        if !content.contains('\n') {
            return Ok(self.format_single_line(content));
        }

        let mut formatted_lines = Vec::new();
        let mut indent_level = 0;
        let mut in_multiline_comment = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Handle multiline comments
            if trimmed.starts_with("/*") && !trimmed.ends_with("*/") {
                in_multiline_comment = true;
            }
            if in_multiline_comment {
                formatted_lines.push(line.to_string());
                if trimmed.ends_with("*/") {
                    in_multiline_comment = false;
                }
                continue;
            }

            // Skip empty lines but preserve them
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }

            // Handle comments - preserve but apply indentation
            if trimmed.starts_with("//") {
                let formatted_line = self.format_line(trimmed, indent_level);
                formatted_lines.push(formatted_line);
                continue;
            }

            // Handle closing braces first (reduce indentation before formatting)
            if trimmed.starts_with('}') {
                indent_level = indent_level.saturating_sub(1);
                let formatted_line = self.format_line(trimmed, indent_level);
                formatted_lines.push(formatted_line);

                // Handle } else { pattern
                if trimmed.contains("else") && trimmed.contains('{') {
                    indent_level += 1;
                }
                continue;
            }

            // Check if this line contains braces that need special handling
            if trimmed.contains('{') && trimmed.contains('}') {
                // Handle single-line blocks that need to be expanded
                let formatted_content = self.format_line_with_braces(trimmed, indent_level);
                formatted_lines.extend(formatted_content);
                continue;
            }

            // Format the line content first
            let formatted_content = self.format_line_content(trimmed);

            // Apply indentation
            let formatted_line = self.apply_indentation(&formatted_content, indent_level);
            formatted_lines.push(formatted_line);

            // Handle opening braces (increase indentation after formatting)
            if formatted_content.ends_with('{') {
                indent_level += 1;
            }
        }

        Ok(formatted_lines.join("\n"))
    }

    /// Format a line that contains braces and may need to be split into multiple lines
    fn format_line_with_braces(&self, line: &str, mut indent_level: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut chars = line.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    current_line.push(ch);
                    // Add the current line and start a new indented block
                    result.push(self.apply_indentation(&current_line.trim(), indent_level));
                    indent_level += 1;
                    current_line.clear();
                }
                '}' => {
                    // Finish current line if it has content
                    if !current_line.trim().is_empty() {
                        result.push(self.apply_indentation(&current_line.trim(), indent_level));
                        current_line.clear();
                    }
                    indent_level = indent_level.saturating_sub(1);
                    
                    // Check if this is } else {
                    let remaining: String = chars.collect();
                    if remaining.trim().starts_with("else") {
                        let else_part = remaining.trim();
                        if else_part.contains('{') {
                            result.push(self.apply_indentation("}", indent_level));
                            result.push(self.apply_indentation("else {", indent_level));
                            indent_level += 1;
                            // Handle any remaining content after else {
                            let after_brace = else_part.split('{').nth(1).unwrap_or("").trim();
                            if !after_brace.is_empty() {
                                result.push(self.apply_indentation(after_brace, indent_level));
                            }
                        } else {
                            result.push(self.apply_indentation(&format!("}} {}", else_part), indent_level));
                        }
                        break;
                    } else {
                        current_line.push(ch);
                        current_line.push_str(&remaining);
                        if !current_line.trim().is_empty() {
                            result.push(self.apply_indentation(&current_line.trim(), indent_level));
                        }
                        break;
                    }
                }
                _ => {
                    current_line.push(ch);
                }
            }
        }
        
        // Add any remaining content
        if !current_line.trim().is_empty() {
            result.push(self.apply_indentation(&current_line.trim(), indent_level));
        }
        
        result
    }

    /// Format a single line of code
    fn format_single_line(&self, content: &str) -> String {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Split on semicolons and format each statement
        let statements: Vec<&str> = trimmed.split(';').collect();
        let mut formatted_statements = Vec::new();

        for (i, statement) in statements.iter().enumerate() {
            let stmt = statement.trim();
            if !stmt.is_empty() {
                let formatted = self.format_line_content(stmt);
                formatted_statements.push(formatted);
            } else if i < statements.len() - 1 {
                // Keep empty statements that aren't at the end
                formatted_statements.push(String::new());
            }
        }

        let result = formatted_statements.join(";\n");
        // Add semicolon at the end if the original had one
        if trimmed.ends_with(';') && !result.ends_with(';') {
            result + ";"
        } else {
            result
        }
    }

    /// Format a single line with proper indentation
    fn format_line(&self, line: &str, indent_level: usize) -> String {
        let formatted = self.format_line_content(line);
        self.apply_indentation(&formatted, indent_level)
    }

    /// Apply indentation to a formatted line
    fn apply_indentation(&self, content: &str, indent_level: usize) -> String {
        if content.is_empty() {
            String::new()
        } else {
            let indent = match self.options.config.indent_style {
                IndentStyle::Spaces => " ".repeat(indent_level * self.options.config.indent_size),
                IndentStyle::Tabs => "\t".repeat(indent_level),
            };
            format!("{}{}", indent, content)
        }
    }

    /// Format the content of a line (spacing, operators, etc.)
    fn format_line_content(&self, line: &str) -> String {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Handle special cases for brace formatting
        let formatted = self.format_braces_and_keywords(trimmed);
        let formatted = self.format_operators(&formatted);
        let formatted = self.format_punctuation(&formatted);

        formatted
    }

    /// Format braces and keywords with proper spacing
    fn format_braces_and_keywords(&self, line: &str) -> String {
        // Handle special patterns first
        let mut line = line.to_string();

        // Fix }else patterns - add space between } and else
        line = line.replace("}else", "} else");

        // Fix else{ patterns - add space between else and {
        line = line.replace("else{", "else {");

        // Fix }else{ patterns
        line = line.replace("}else{", "} else {");

        // Fix keyword( patterns - add space between keyword and (
        line = line.replace("if(", "if (");
        line = line.replace("while(", "while (");
        line = line.replace("for(", "for (");
        line = line.replace("match(", "match (");
        line = line.replace("switch(", "switch (");
        line = line.replace("select(", "select (");

        // Fix keyword{ patterns - add space between keyword and {
        line = line.replace("if{", "if {");
        line = line.replace("while{", "while {");
        line = line.replace("for{", "for {");
        line = line.replace("match{", "match {");
        line = line.replace("switch{", "switch {");
        line = line.replace("select{", "select {");

        // Normalize multiple spaces to single spaces
        let normalized = self.normalize_spaces(&line);

        // Then handle keywords and braces
        let mut result = String::new();
        let tokens = self.tokenize_line(&normalized);

        for (i, token) in tokens.iter().enumerate() {
            match token.as_str() {
                // Keywords that need space after them
                "struct" | "func" | "interface" | "const" | "let" | "if" | "else" | "while"
                | "for" | "return" | "match" | "switch" | "select" => {
                    result.push_str(token);
                    // Add space after keyword if not already present
                    if i + 1 < tokens.len() {
                        let next_token = &tokens[i + 1];
                        if next_token != " " {
                            result.push(' ');
                        }
                    }
                }
                // Opening brace - ensure space before it
                "{" => {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push_str(token);
                }
                // Closing brace - handle special cases
                "}" => {
                    result.push_str(token);
                    // Add space after } if followed by else
                    if i + 1 < tokens.len() {
                        let next_token = &tokens[i + 1];
                        if next_token == "else" && !result.ends_with(' ') {
                            result.push(' ');
                        }
                    }
                }
                // Skip redundant spaces
                " " => {
                    if !result.ends_with(' ') {
                        result.push_str(token);
                    }
                }
                // Other tokens
                _ => {
                    result.push_str(token);
                }
            }
        }

        result
    }

    /// Normalize multiple spaces to single spaces
    fn normalize_spaces(&self, line: &str) -> String {
        let mut result = String::new();
        let mut prev_was_space = false;

        for ch in line.chars() {
            if ch == ' ' {
                if !prev_was_space {
                    result.push(ch);
                }
                prev_was_space = true;
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }

        result
    }

    /// Simple tokenizer to split line into tokens
    fn tokenize_line(&self, line: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in line.chars() {
            match ch {
                ' ' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(" ".to_string());
                }
                '{' | '}' | '(' | ')' | '[' | ']' | ',' | ':' | ';' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// Check if a word is a keyword
    fn is_keyword(&self, word: &str) -> bool {
        matches!(
            word,
            "if" | "else"
                | "while"
                | "for"
                | "func"
                | "let"
                | "const"
                | "struct"
                | "interface"
                | "return"
                | "match"
                | "import"
                | "export"
                | "type"
                | "enum"
                | "trait"
                | "impl"
        )
    }

    /// Format operators with proper spacing
    fn format_operators(&self, line: &str) -> String {
        // First, protect special operators that should not have spaces inside them
        let mut line = line.to_string();

        // Protect special compound operators by temporarily replacing them
        line = line.replace("=>", "ARROW_RIGHT_TEMP");
        line = line.replace("->", "ARROW_TEMP");
        line = line.replace("<-", "ARROW_LEFT_TEMP");
        line = line.replace("<=", "LESS_EQUAL_TEMP");
        line = line.replace(">=", "GREATER_EQUAL_TEMP");
        line = line.replace("==", "EQUAL_EQUAL_TEMP");
        line = line.replace("!=", "NOT_EQUAL_TEMP");
        line = line.replace("&&", "AND_AND_TEMP");
        line = line.replace("||", "OR_OR_TEMP");
        line = line.replace("<<", "SHIFT_LEFT_TEMP");
        line = line.replace(">>", "SHIFT_RIGHT_TEMP");
        line = line.replace("+=", "PLUS_EQUAL_TEMP");
        line = line.replace("-=", "MINUS_EQUAL_TEMP");
        line = line.replace("*=", "MULT_EQUAL_TEMP");
        line = line.replace("/=", "DIV_EQUAL_TEMP");

        if !self.options.config.space_around_operators {
            // Restore protected operators without spacing
            line = line.replace("ARROW_RIGHT_TEMP", "=>");
            line = line.replace("ARROW_TEMP", "->");
            line = line.replace("ARROW_LEFT_TEMP", "<-");
            line = line.replace("LESS_EQUAL_TEMP", "<=");
            line = line.replace("GREATER_EQUAL_TEMP", ">=");
            line = line.replace("EQUAL_EQUAL_TEMP", "==");
            line = line.replace("NOT_EQUAL_TEMP", "!=");
            line = line.replace("AND_AND_TEMP", "&&");
            line = line.replace("OR_OR_TEMP", "||");
            line = line.replace("SHIFT_LEFT_TEMP", "<<");
            line = line.replace("SHIFT_RIGHT_TEMP", ">>");
            line = line.replace("PLUS_EQUAL_TEMP", "+=");
            line = line.replace("MINUS_EQUAL_TEMP", "-=");
            line = line.replace("MULT_EQUAL_TEMP", "*=");
            line = line.replace("DIV_EQUAL_TEMP", "/=");
            return line;
        }

        let mut result = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            match ch {
                '=' | '+' | '-' | '*' | '/' | '%' | '<' | '>' | '!' | '&' | '|' => {
                    // Single operator - add spaces around it
                    if !result.is_empty()
                        && !result.ends_with(' ')
                        && !self.is_operator_context(&result)
                    {
                        result.push(' ');
                    }
                    result.push(ch);
                    result.push(' ');
                }
                _ => {
                    result.push(ch);
                }
            }

            i += 1;
        }

        // Restore protected operators with proper spacing
        result = result.replace("ARROW_RIGHT_TEMP", " => ");
        result = result.replace("ARROW_TEMP", " -> ");
        result = result.replace("ARROW_LEFT_TEMP", " <- ");
        result = result.replace("LESS_EQUAL_TEMP", " <= ");
        result = result.replace("GREATER_EQUAL_TEMP", " >= ");
        result = result.replace("EQUAL_EQUAL_TEMP", " == ");
        result = result.replace("NOT_EQUAL_TEMP", " != ");
        result = result.replace("AND_AND_TEMP", " && ");
        result = result.replace("OR_OR_TEMP", " || ");
        result = result.replace("SHIFT_LEFT_TEMP", " << ");
        result = result.replace("SHIFT_RIGHT_TEMP", " >> ");
        result = result.replace("PLUS_EQUAL_TEMP", " += ");
        result = result.replace("MINUS_EQUAL_TEMP", " -= ");
        result = result.replace("MULT_EQUAL_TEMP", " *= ");
        result = result.replace("DIV_EQUAL_TEMP", " /= ");

        result
    }

    /// Check if we're in an operator context where we shouldn't add spaces
    fn is_operator_context(&self, result: &str) -> bool {
        result.ends_with('(')
            || result.ends_with('[')
            || result.ends_with(',')
            || result.ends_with(' ')
    }

    /// Format punctuation (commas, colons, etc.)
    fn format_punctuation(&self, line: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            match ch {
                ',' => {
                    result.push(ch);
                    if self.options.config.space_after_commas {
                        // Only add space if not already present and not at end
                        if i + 1 < chars.len() && chars[i + 1] != ' ' {
                            result.push(' ');
                        }
                    }
                }
                ':' => {
                    result.push(ch);
                    // Add space after colon in type annotations
                    if i + 1 < chars.len() && chars[i + 1] != ' ' && chars[i + 1] != ':' {
                        result.push(' ');
                    }
                }
                ' ' => {
                    // Normalize whitespace
                    if self.options.config.normalize_whitespace {
                        if !result.ends_with(' ') {
                            result.push(ch);
                        }
                    } else {
                        result.push(ch);
                    }
                }
                _ => {
                    result.push(ch);
                }
            }

            i += 1;
        }

        result.trim_end().to_string()
    }

    /// Check if files need formatting
    pub fn check_formatting(&self) -> Result<bool> {
        let mut options = self.options.clone();
        options.check_only = true;

        let formatter = Formatter::new(self.project.clone(), options);
        let results = formatter.format_project()?;

        Ok(results.iter().any(|r| r.changed))
    }
}

/// Load formatting configuration from .langfmt.toml
pub fn load_format_config(project_root: &Path) -> Result<FormatOptions> {
    let config_path = project_root.join(".langfmt.toml");

    if !config_path.exists() {
        return Ok(FormatOptions::default());
    }

    let _config_content = fs::read_to_string(&config_path)
        .map_err(|e| BuluError::Other(format!("Failed to read .langfmt.toml: {}", e)))?;

    // Parse TOML configuration
    let format_config: FormatConfig = toml::from_str(&_config_content)
        .map_err(|e| BuluError::Other(format!("Failed to parse .langfmt.toml: {}", e)))?;

    Ok(FormatOptions::from_config(format_config))
}

/// Create a default .langfmt.toml configuration file
pub fn create_default_format_config(project_root: &Path) -> Result<()> {
    let config_path = project_root.join(".langfmt.toml");

    if config_path.exists() {
        return Err(BuluError::Other(
            ".langfmt.toml already exists. Remove it first if you want to recreate it.".to_string(),
        ));
    }

    let default_config = FormatConfig::default();
    let _config_content = toml::to_string_pretty(&default_config)
        .map_err(|e| BuluError::Other(format!("Failed to serialize default config: {}", e)))?;

    // Add comments to make the config file more user-friendly
    let trailing_comma_str = match default_config.trailing_comma {
        TrailingCommaStyle::Never => "never",
        TrailingCommaStyle::Always => "always",
        TrailingCommaStyle::Es5 => "es5",
    };

    let brace_style_str = match default_config.brace_style {
        BraceStyle::SameLine => "same_line",
        BraceStyle::NextLine => "next_line",
        BraceStyle::NextLineIndented => "next_line_indented",
    };

    let indent_style_str = match default_config.indent_style {
        IndentStyle::Spaces => "spaces",
        IndentStyle::Tabs => "tabs",
    };

    let commented_config = format!(
        r#"# Bulu Language Formatter Configuration
# This file configures how the 'lang fmt' command formats your code.

# Number of spaces per indentation level (ignored if indent_style = "tabs")
indent_size = {}

# Maximum line length before wrapping
max_line_length = {}

# Whether to preserve existing comments
preserve_comments = {}

# Whether to add spaces around operators (=, +, -, etc.)
space_around_operators = {}

# Whether to add spaces after commas
space_after_commas = {}

# Whether to add spaces after keywords (if, while, func, etc.)
space_after_keywords = {}

# Whether to normalize whitespace (remove extra spaces)
normalize_whitespace = {}

# Trailing comma style: "never", "always", or "es5"
trailing_comma = "{}"

# Brace style: "same_line", "next_line", or "next_line_indented"
brace_style = "{}"

# Indentation style: "spaces" or "tabs"
indent_style = "{}"
"#,
        default_config.indent_size,
        default_config.max_line_length,
        default_config.preserve_comments,
        default_config.space_around_operators,
        default_config.space_after_commas,
        default_config.space_after_keywords,
        default_config.normalize_whitespace,
        trailing_comma_str,
        brace_style_str,
        indent_style_str,
    );

    fs::write(&config_path, commented_config)
        .map_err(|e| BuluError::Other(format!("Failed to write .langfmt.toml: {}", e)))?;

    println!("Created default .langfmt.toml configuration file");
    Ok(())
}

/// Validate a format configuration
pub fn validate_format_config(config: &FormatConfig) -> Result<()> {
    if config.indent_size == 0 {
        return Err(BuluError::Other(
            "indent_size must be greater than 0".to_string(),
        ));
    }

    if config.indent_size > 16 {
        return Err(BuluError::Other(
            "indent_size should not exceed 16 spaces".to_string(),
        ));
    }

    if config.max_line_length < 40 {
        return Err(BuluError::Other(
            "max_line_length should be at least 40 characters".to_string(),
        ));
    }

    if config.max_line_length > 500 {
        return Err(BuluError::Other(
            "max_line_length should not exceed 500 characters".to_string(),
        ));
    }

    Ok(())
}
