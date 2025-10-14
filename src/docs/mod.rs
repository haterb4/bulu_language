//! Documentation generation for Bulu projects

use crate::Result;
use crate::project::Project;
// Imports will be added as needed by the extractor
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use colored::*;
use serde::{Serialize, Deserialize};

pub mod extractor;
pub mod html_generator;
pub mod server;

use extractor::DocExtractor;
use html_generator::HtmlGenerator;
use server::DocServer;

/// Documentation format
#[derive(Debug, Clone)]
pub enum DocFormat {
    Html,
    Markdown,
    Json,
}

/// Documentation options
#[derive(Debug, Clone)]
pub struct DocOptions {
    pub output_dir: PathBuf,
    pub format: DocFormat,
    pub serve: bool,
    pub port: u16,
    pub verbose: bool,
}

impl Default for DocOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("docs"),
            format: DocFormat::Html,
            serve: false,
            port: 8080,
            verbose: false,
        }
    }
}

/// Represents a documentation comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocComment {
    pub content: String,
    pub params: HashMap<String, String>,
    pub returns: Option<String>,
    pub examples: Vec<String>,
    pub since: Option<String>,
    pub deprecated: Option<String>,
}

impl DocComment {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            params: HashMap::new(),
            returns: None,
            examples: Vec::new(),
            since: None,
            deprecated: None,
        }
    }

    /// Parse a documentation comment from raw text
    pub fn parse(text: &str) -> Self {
        let mut doc = DocComment::new();
        let mut current_section = String::new();
        let mut in_example = false;
        let mut example_content = String::new();

        for line in text.lines() {
            let line = line.trim().trim_start_matches("*").trim();
            
            if line.starts_with("@param") {
                if let Some(rest) = line.strip_prefix("@param") {
                    let rest = rest.trim();
                    if let Some((param_name, param_desc)) = rest.split_once(" - ") {
                        doc.params.insert(param_name.trim().to_string(), param_desc.trim().to_string());
                    }
                }
            } else if line.starts_with("@return") {
                if let Some(rest) = line.strip_prefix("@return") {
                    doc.returns = Some(rest.trim().to_string());
                }
            } else if line.starts_with("@example") {
                if in_example && !example_content.is_empty() {
                    doc.examples.push(example_content.trim().to_string());
                }
                in_example = true;
                example_content.clear();
            } else if line.starts_with("@since") {
                if let Some(rest) = line.strip_prefix("@since") {
                    doc.since = Some(rest.trim().to_string());
                }
            } else if line.starts_with("@deprecated") {
                if let Some(rest) = line.strip_prefix("@deprecated") {
                    doc.deprecated = Some(rest.trim().to_string());
                }
            } else if line.starts_with("@") {
                // End of example or other section
                if in_example && !example_content.is_empty() {
                    doc.examples.push(example_content.trim().to_string());
                    example_content.clear();
                }
                in_example = false;
            } else {
                if in_example {
                    if !example_content.is_empty() {
                        example_content.push('\n');
                    }
                    example_content.push_str(line);
                } else {
                    if !current_section.is_empty() {
                        current_section.push('\n');
                    }
                    current_section.push_str(line);
                }
            }
        }

        // Handle final example
        if in_example && !example_content.is_empty() {
            doc.examples.push(example_content.trim().to_string());
        }

        doc.content = current_section.trim().to_string();
        doc
    }
}

/// Represents documented item (function, struct, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentedItem {
    pub name: String,
    pub kind: ItemKind,
    pub signature: String,
    pub doc_comment: Option<DocComment>,
    pub visibility: Visibility,
    pub file_path: PathBuf,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemKind {
    Function,
    Struct,
    Interface,
    Constant,
    Variable,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

/// Documentation generator
pub struct DocGenerator {
    project: Project,
    options: DocOptions,
}

impl DocGenerator {
    pub fn new(project: Project, options: DocOptions) -> Self {
        Self { project, options }
    }

    /// Generate documentation
    pub fn generate(&self) -> Result<()> {
        if self.options.verbose {
            println!("{} Generating documentation for '{}'...", "Documenting".green().bold(), self.project.config.package.name);
        }

        // Create output directory
        fs::create_dir_all(&self.options.output_dir)?;

        // Extract documentation from source files
        let extractor = DocExtractor::new();
        let documented_items = self.extract_documentation(&extractor)?;

        // Generate documentation based on format
        match self.options.format {
            DocFormat::Html => {
                let generator = HtmlGenerator::new(&self.options.output_dir);
                generator.generate(&documented_items, &self.project)?;
            }
            DocFormat::Markdown => {
                self.generate_markdown(&documented_items)?;
            }
            DocFormat::Json => {
                self.generate_json(&documented_items)?;
            }
        }

        if self.options.verbose {
            println!("{} Documentation generated in '{}'", "Success".green().bold(), self.options.output_dir.display());
        }

        // Start local server if requested
        if self.options.serve {
            let server = DocServer::new(self.options.output_dir.clone(), self.options.port);
            server.start()?;
        }

        Ok(())
    }

    fn extract_documentation(&self, extractor: &DocExtractor) -> Result<Vec<DocumentedItem>> {
        let mut documented_items = Vec::new();
        
        // Find all .bu files in the project
        let source_files = self.find_source_files()?;
        
        for file_path in source_files {
            if self.options.verbose {
                println!("Processing {}", file_path.display());
            }
            
            let content = fs::read_to_string(&file_path)?;
            let items = extractor.extract_from_file(&content, &file_path)?;
            documented_items.extend(items);
        }
        
        Ok(documented_items)
    }

    fn find_source_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.find_source_files_recursive(&self.project.root.join("src"), &mut files)?;
        Ok(files)
    }

    fn find_source_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.find_source_files_recursive(&path, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("bu") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    fn generate_markdown(&self, items: &[DocumentedItem]) -> Result<()> {
        let mut content = String::new();
        content.push_str(&format!("# {} API Documentation\n\n", self.project.config.package.name));

        // Group items by kind
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut interfaces = Vec::new();
        let mut constants = Vec::new();

        for item in items {
            match item.kind {
                ItemKind::Function => functions.push(item),
                ItemKind::Struct => structs.push(item),
                ItemKind::Interface => interfaces.push(item),
                ItemKind::Constant => constants.push(item),
                _ => {}
            }
        }

        // Generate sections
        if !functions.is_empty() {
            content.push_str("## Functions\n\n");
            for func in functions {
                self.generate_markdown_item(&mut content, func);
            }
        }

        if !structs.is_empty() {
            content.push_str("## Structs\n\n");
            for struct_item in structs {
                self.generate_markdown_item(&mut content, struct_item);
            }
        }

        if !interfaces.is_empty() {
            content.push_str("## Interfaces\n\n");
            for interface in interfaces {
                self.generate_markdown_item(&mut content, interface);
            }
        }

        if !constants.is_empty() {
            content.push_str("## Constants\n\n");
            for constant in constants {
                self.generate_markdown_item(&mut content, constant);
            }
        }

        let output_path = self.options.output_dir.join("README.md");
        fs::write(output_path, content)?;
        Ok(())
    }

    fn generate_markdown_item(&self, content: &mut String, item: &DocumentedItem) {
        content.push_str(&format!("### {}\n\n", item.name));
        content.push_str(&format!("```bulu\n{}\n```\n\n", item.signature));
        
        if let Some(doc) = &item.doc_comment {
            if !doc.content.is_empty() {
                content.push_str(&format!("{}\n\n", doc.content));
            }
            
            if !doc.params.is_empty() {
                content.push_str("**Parameters:**\n\n");
                for (param, desc) in &doc.params {
                    content.push_str(&format!("- `{}`: {}\n", param, desc));
                }
                content.push('\n');
            }
            
            if let Some(returns) = &doc.returns {
                content.push_str(&format!("**Returns:** {}\n\n", returns));
            }
            
            if !doc.examples.is_empty() {
                content.push_str("**Examples:**\n\n");
                for example in &doc.examples {
                    content.push_str(&format!("```bulu\n{}\n```\n\n", example));
                }
            }
        }
        
        content.push_str("---\n\n");
    }

    fn generate_json(&self, items: &[DocumentedItem]) -> Result<()> {
        let json = serde_json::to_string_pretty(items)?;
        let output_path = self.options.output_dir.join("api.json");
        fs::write(output_path, json)?;
        Ok(())
    }
}