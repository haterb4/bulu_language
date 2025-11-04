//! Module system for the Bulu language
//!
//! This module provides functionality for loading, resolving, and managing
//! modules and their exports/imports.

use crate::ast::nodes::{ExportStmt, ImportStmt, Program, Statement};
use crate::error::{BuluError, Result};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::primitive::RuntimeValue;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Source information for error reporting
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub file_path: Option<String>,
    pub is_std_lib: bool,
}

/// Represents a loaded module with its exports
#[derive(Debug, Clone)]
pub struct Module {
    pub path: String, // Module identifier (e.g., "std.io", "./utils")
    pub source_info: SourceInfo,
    pub exports: HashMap<String, RuntimeValue>,
    pub ast: Program,
}

/// Module resolver for handling imports and exports
#[derive(Debug)]
pub struct ModuleResolver {
    /// Cache of loaded modules
    modules: HashMap<String, Module>,
    /// Standard library modules
    std_modules: HashMap<String, Module>,
    /// In-memory modules for testing
    memory_modules: HashMap<String, String>,
    /// Current working directory for relative imports
    current_dir: PathBuf,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new() -> Self {
        let mut resolver = Self {
            modules: HashMap::new(),
            std_modules: HashMap::new(),
            memory_modules: HashMap::new(),
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };

        // Initialize standard library modules
        resolver.init_std_modules();
        resolver
    }

    /// Initialize standard library modules
    fn init_std_modules(&mut self) {
        // Create mock standard library modules for now
        let std_modules = vec![
            "io", "fmt", "strings", "arrays", "math", "time", "sync", "os", "path", "http", "net",
            "json", "xml", "csv", "crypto", "db", "test", "random",
        ];

        for module_name in std_modules {
            let mut exports = HashMap::new();

            // Add some basic exports based on module name
            match module_name {
                "io" => {
                    exports.insert("print".to_string(), RuntimeValue::Null);
                    exports.insert("println".to_string(), RuntimeValue::Null);
                    exports.insert("input".to_string(), RuntimeValue::Null);
                }
                "fmt" => {
                    exports.insert("sprintf".to_string(), RuntimeValue::Null);
                    exports.insert("format".to_string(), RuntimeValue::Null);
                }
                "strings" => {
                    exports.insert("len".to_string(), RuntimeValue::Null);
                    exports.insert("substr".to_string(), RuntimeValue::Null);
                    exports.insert("split".to_string(), RuntimeValue::Null);
                    exports.insert("join".to_string(), RuntimeValue::Null);
                }
                "arrays" => {
                    exports.insert("append".to_string(), RuntimeValue::Null);
                    exports.insert("len".to_string(), RuntimeValue::Null);
                    exports.insert("copy".to_string(), RuntimeValue::Null);
                }
                "math" => {
                    exports.insert("abs".to_string(), RuntimeValue::Null);
                    exports.insert("sqrt".to_string(), RuntimeValue::Null);
                    exports.insert("pow".to_string(), RuntimeValue::Null);
                    exports.insert("sin".to_string(), RuntimeValue::Null);
                    exports.insert("cos".to_string(), RuntimeValue::Null);
                }
                "net" => {
                    exports.insert("TcpServer".to_string(), RuntimeValue::String("struct:TcpServer".to_string()));
                    exports.insert("TcpConnection".to_string(), RuntimeValue::String("struct:TcpConnection".to_string()));
                    exports.insert("UdpConnection".to_string(), RuntimeValue::String("struct:UdpConnection".to_string()));
                    exports.insert("NetAddr".to_string(), RuntimeValue::String("struct:NetAddr".to_string()));
                }
                "time" => {
                    exports.insert("sleep".to_string(), RuntimeValue::Null);
                }
                _ => {
                    // Add a generic export for other modules
                    exports.insert("default".to_string(), RuntimeValue::Null);
                }
            }

            // Create a dummy AST for std modules
            let ast = Program {
                statements: vec![],
                position: crate::lexer::token::Position::new(0, 0, 0),
            };

            let module = Module {
                path: format!("std.{}", module_name),
                source_info: SourceInfo {
                    file_path: None,
                    is_std_lib: true,
                },
                exports,
                ast,
            };

            self.std_modules
                .insert(format!("std.{}", module_name), module.clone());
            self.std_modules.insert(module_name.to_string(), module);
        }
    }

    /// Resolve an import statement and return the imported symbols
    pub fn resolve_import(&mut self, import: &ImportStmt) -> Result<HashMap<String, RuntimeValue>> {
        let module = self.load_module(&import.path)?;
        let mut imported_symbols = HashMap::new();

        if let Some(items) = &import.items {
            // Import specific items: import { item1, item2 } from "path"
            for item in items {
                if let Some(value) = module.exports.get(&item.name) {
                    let symbol_name = item.alias.as_ref().unwrap_or(&item.name);
                    imported_symbols.insert(symbol_name.clone(), value.clone());
                } else {
                    return Err(BuluError::RuntimeError {
                        message: format!(
                            "Module '{}' does not export '{}'",
                            import.path, item.name
                        ),
                        file: module.source_info.file_path.clone(),
                    });
                }
            }
        } else if let Some(alias) = &import.alias {
            // Import entire module with alias: import "path" as alias
            let module_object = RuntimeValue::Map(module.exports.clone());
            imported_symbols.insert(alias.clone(), module_object);
        } else {
            // Import all exports: import "path"
            imported_symbols.extend(module.exports.clone());
        }

        Ok(imported_symbols)
    }

    /// Load a module from the given path
    pub fn load_module(&mut self, path: &str) -> Result<Module> {
        // Check if module is already loaded
        if let Some(module) = self.modules.get(path) {
            return Ok(module.clone());
        }

        // Check if it's a standard library module
        let std_module_key = if path.starts_with("std/") {
            // Convert std/net to std.net format
            path.replace('/', ".")
        } else if path.starts_with("std.") {
            path.to_string()
        } else {
            String::new()
        };

        if !std_module_key.is_empty() {
            if let Some(module) = self.std_modules.get(&std_module_key) {
                return Ok(module.clone());
            } else {
                // Module not found in std_modules, but it's a std module
                return Err(BuluError::RuntimeError {
                    message: format!("Standard library module '{}' not found", path),
                    file: Some(path.to_string()),
                });
            }
        }

        // Check for in-memory modules first
        let (source, actual_file_path) = if let Some(memory_source) = self.memory_modules.get(path) {
            (memory_source.clone(), None)
        } else {
            // Try to load from file system
            let module_path = self.resolve_module_path(path)?;
            let file_path_str = module_path.to_string_lossy().to_string();
            let source = fs::read_to_string(&module_path).map_err(|e| BuluError::RuntimeError {
                message: format!("Failed to read module '{}': {}", path, e),
                file: Some(file_path_str.clone()),
            })?;
            (source, Some(file_path_str))
        };

        // Parse the module
        let file_for_errors = actual_file_path.as_ref().unwrap_or(&path.to_string()).clone();
        let mut lexer = Lexer::with_file(&source, file_for_errors.clone());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::with_file(tokens, file_for_errors.clone());
        let ast = parser.parse()?;

        // Extract exports from the module
        let exports = self.extract_exports(&ast, &file_for_errors)?;

        let module = Module {
            path: path.to_string(),
            source_info: SourceInfo {
                file_path: actual_file_path,
                is_std_lib: false,
            },
            exports,
            ast,
        };

        self.modules.insert(path.to_string(), module.clone());
        Ok(module)
    }

    /// Resolve module path from import string
    fn resolve_module_path(&self, path: &str) -> Result<PathBuf> {
        // Handle different import path formats
        if path.starts_with("./") || path.starts_with("../") {
            // Relative import
            let mut full_path = self.current_dir.join(path);
            
            // Normalize the path to resolve . and .. components manually
            let mut components = Vec::new();
            for component in full_path.components() {
                match component {
                    std::path::Component::CurDir => {
                        // Skip current directory components
                    }
                    std::path::Component::ParentDir => {
                        // Pop the last component for parent directory
                        components.pop();
                    }
                    _ => {
                        components.push(component);
                    }
                }
            }
            full_path = components.iter().collect();
            
            if !full_path.extension().map_or(false, |ext| ext == "bu") {
                full_path.set_extension("bu");
            }
            Ok(full_path)
        } else if path.starts_with("/") {
            // Absolute import
            let mut full_path = PathBuf::from(path);
            if !full_path.extension().map_or(false, |ext| ext == "bu") {
                full_path.set_extension("bu");
            }
            Ok(full_path)
        } else if path.starts_with("std.") {
            // Standard library import - these are built-in
            Err(BuluError::RuntimeError {
                message: format!("Standard library module '{}' not found", path),
                file: Some(path.to_string()),
            })
        } else {
            // Package import or bare module name
            let mut full_path = self.current_dir.join(format!("{}.bu", path));
            if !full_path.exists() {
                // Try in src directory
                full_path = self.current_dir.join("src").join(format!("{}.bu", path));
            }
            Ok(full_path)
        }
    }

    /// Extract exports from a module's AST by analyzing declarations
    fn extract_exports(&mut self, ast: &Program, _module_path: &str) -> Result<HashMap<String, RuntimeValue>> {
        let mut exports = HashMap::new();
        
        // Extract exports from the module's AST
        for statement in &ast.statements {
            match statement {
                Statement::Export(export_stmt) => {
                    match export_stmt.item.as_ref() {
                        Statement::FunctionDecl(func) => {
                            // Export function - create a function value
                            exports.insert(func.name.clone(), RuntimeValue::String(format!("function:{}", func.name)));
                        }
                        Statement::VariableDecl(var) => {
                            // Export variable/constant - create placeholder value
                            if var.is_const {
                                // For constants, try to extract the value from the initializer
                                if let Some(ref initializer) = var.initializer {
                                    let value = self.extract_literal_value(initializer);
                                    exports.insert(var.name.clone(), value);
                                } else {
                                    exports.insert(var.name.clone(), RuntimeValue::Null);
                                }
                            } else {
                                exports.insert(var.name.clone(), RuntimeValue::Null);
                            }
                        }
                        Statement::StructDecl(struct_decl) => {
                            // Export struct - create a struct constructor
                            exports.insert(struct_decl.name.clone(), RuntimeValue::String(format!("struct:{}", struct_decl.name)));
                        }
                        Statement::InterfaceDecl(interface) => {
                            // Export interface
                            exports.insert(interface.name.clone(), RuntimeValue::String(format!("interface:{}", interface.name)));
                        }
                        Statement::TypeAlias(type_alias) => {
                            // Export type alias
                            exports.insert(type_alias.name.clone(), RuntimeValue::String(format!("type:{}", type_alias.name)));
                        }
                        Statement::Import(_import_stmt) => {
                            // Re-export: this would need recursive handling
                            // For now, skip
                        }
                        _ => {
                            return Err(BuluError::RuntimeError {
                                message: "Only declarations can be exported".to_string(),
                                file: None,
                            });
                        }
                    }
                }
                Statement::FunctionDecl(func) if func.is_exported => {
                    // Implicitly exported function
                    exports.insert(func.name.clone(), RuntimeValue::String(format!("function:{}", func.name)));
                }
                Statement::VariableDecl(var) if var.is_exported => {
                    // Implicitly exported variable
                    if var.is_const {
                        // For constants, try to extract the value from the initializer
                        if let Some(ref initializer) = var.initializer {
                            let value = self.extract_literal_value(initializer);
                            exports.insert(var.name.clone(), value);
                        } else {
                            exports.insert(var.name.clone(), RuntimeValue::Null);
                        }
                    } else {
                        exports.insert(var.name.clone(), RuntimeValue::Null);
                    }
                }
                Statement::StructDecl(struct_decl) if struct_decl.is_exported => {
                    // Implicitly exported struct
                    exports.insert(struct_decl.name.clone(), RuntimeValue::String(format!("struct:{}", struct_decl.name)));
                }
                Statement::InterfaceDecl(interface) if interface.is_exported => {
                    // Implicitly exported interface
                    exports.insert(interface.name.clone(), RuntimeValue::String(format!("interface:{}", interface.name)));
                }
                _ => {
                    // Non-exported statement, skip
                }
            }
        }

        Ok(exports)
    }

    /// Extract literal value from an expression (for constants)
    fn extract_literal_value(&self, expr: &crate::ast::Expression) -> RuntimeValue {
        match expr {
            crate::ast::Expression::Literal(literal) => {
                match &literal.value {
                    crate::ast::LiteralValue::Integer(i) => RuntimeValue::Int64(*i),
                    crate::ast::LiteralValue::Float(f) => RuntimeValue::Float64(*f),
                    crate::ast::LiteralValue::String(s) => RuntimeValue::String(s.clone()),
                    crate::ast::LiteralValue::Boolean(b) => RuntimeValue::Bool(*b),
                    crate::ast::LiteralValue::Char(c) => RuntimeValue::Char(*c),
                    crate::ast::LiteralValue::Null => RuntimeValue::Null,
                }
            }
            _ => RuntimeValue::Null, // For non-literal expressions, return null for now
        }
    }

    /// Check if a symbol is exported from a module
    pub fn is_exported(&self, module_path: &str, symbol: &str) -> bool {
        if let Some(module) = self.modules.get(module_path) {
            module.exports.contains_key(symbol)
        } else if let Some(module) = self.std_modules.get(module_path) {
            module.exports.contains_key(symbol)
        } else {
            false
        }
    }

    /// Get all exported symbols from a module
    pub fn get_exports(&self, module_path: &str) -> Option<&HashMap<String, RuntimeValue>> {
        if let Some(module) = self.modules.get(module_path) {
            Some(&module.exports)
        } else if let Some(module) = self.std_modules.get(module_path) {
            Some(&module.exports)
        } else {
            None
        }
    }

    /// Set the current directory for relative imports
    pub fn set_current_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }

    /// Add an in-memory module for testing
    pub fn add_memory_module(&mut self, path: String, source: String) {
        self.memory_modules.insert(path, source);
    }

    /// Get all loaded modules for compilation
    pub fn get_loaded_modules(&self) -> Vec<&Module> {
        self.modules.values().collect()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Position;

    #[test]
    fn test_std_module_loading() {
        let mut resolver = ModuleResolver::new();

        // Test loading std.io module
        let module = resolver.load_module("std.io").unwrap();
        assert_eq!(module.path, "std.io");
        assert!(module.exports.contains_key("print"));
        assert!(module.exports.contains_key("println"));
    }

    #[test]
    fn test_import_resolution() {
        let mut resolver = ModuleResolver::new();

        let import = ImportStmt {
            path: "std.io".to_string(),
            alias: Some("io".to_string()),
            items: None,
            position: Position::new(1, 1, 0),
        };

        let symbols = resolver.resolve_import(&import).unwrap();
        assert!(symbols.contains_key("io"));
    }

    #[test]
    fn test_selective_import() {
        let mut resolver = ModuleResolver::new();

        let import = ImportStmt {
            path: "std.io".to_string(),
            alias: None,
            items: Some(vec![crate::ast::nodes::ImportItem {
                name: "print".to_string(),
                alias: None,
                position: Position::new(1, 1, 0),
            }]),
            position: Position::new(1, 1, 0),
        };

        let symbols = resolver.resolve_import(&import).unwrap();
        assert!(symbols.contains_key("print"));
        assert!(!symbols.contains_key("println"));
    }
}
