//! Module resolution for finding and loading modules

use crate::error::{BuluError, Result};
use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use super::{Module, ResolutionContext, Symbol, SymbolKind, Visibility};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Module resolver for finding and loading modules
pub struct ModuleResolver {
    search_paths: Vec<PathBuf>,
    std_lib_path: Option<PathBuf>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            std_lib_path: None,
        }
    }

    /// Add a search path for modules
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Set the standard library path
    pub fn set_std_lib_path(&mut self, path: PathBuf) {
        self.std_lib_path = Some(path);
    }

    /// Resolve a module path to an actual file path
    pub fn resolve_module_path(&self, module_path: &str, current_file: Option<&Path>) -> Result<PathBuf> {
        // Handle standard library imports
        if module_path.starts_with("std.") {
            return self.resolve_std_module(module_path);
        }

        // Handle relative imports
        if module_path.starts_with("./") || module_path.starts_with("../") {
            if let Some(current) = current_file {
                let base_dir = current.parent().unwrap_or(Path::new("."));
                let resolved = base_dir.join(module_path);
                return self.try_resolve_file(&resolved);
            }
        }

        // Handle absolute imports - search in all search paths
        for search_path in &self.search_paths {
            let candidate = search_path.join(module_path);
            if let Ok(resolved) = self.try_resolve_file(&candidate) {
                return Ok(resolved);
            }
        }

        Err(BuluError::Other(format!("Module not found: {}", module_path)))
    }

    /// Resolve a standard library module
    fn resolve_std_module(&self, module_path: &str) -> Result<PathBuf> {
        if let Some(std_path) = &self.std_lib_path {
            // Convert std.io to std/io.bu
            let module_file = module_path.replace('.', "/") + ".bu";
            let full_path = std_path.join(module_file);
            
            if full_path.exists() {
                Ok(full_path)
            } else {
                Err(BuluError::Other(format!("Standard library module not found: {}", module_path)))
            }
        } else {
            Err(BuluError::Other("Standard library path not configured".to_string()))
        }
    }

    /// Try to resolve a file path, adding .bu extension if needed
    fn try_resolve_file(&self, path: &Path) -> Result<PathBuf> {
        // Try exact path first
        if path.exists() {
            return Ok(path.to_path_buf());
        }

        // Try adding .bu extension
        let with_extension = path.with_extension("bu");
        if with_extension.exists() {
            return Ok(with_extension);
        }

        // Try as directory with index.bu
        let index_file = path.join("index.bu");
        if index_file.exists() {
            return Ok(index_file);
        }

        Err(BuluError::Other(format!("File not found: {}", path.display())))
    }

    /// Load and parse a module from file
    pub fn load_module(&self, file_path: &Path) -> Result<Module> {
        let source = fs::read_to_string(file_path)
            .map_err(|e| BuluError::IoError(format!("Failed to read {}: {}", file_path.display(), e)))?;

        let mut lexer = Lexer::with_file(&source, file_path.to_string_lossy().to_string());
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::with_file(tokens, file_path.to_string_lossy().to_string());
        let ast = parser.parse()?;

        let module_name = file_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut module = Module::new(file_path.to_path_buf(), module_name);

        // Extract symbols from AST
        self.extract_symbols_from_ast(&ast, &mut module)?;

        Ok(module)
    }

    /// Extract symbols from AST and populate module
    fn extract_symbols_from_ast(&self, ast: &Program, module: &mut Module) -> Result<()> {
        for statement in &ast.statements {
            match statement {
                Statement::FunctionDecl(func_decl) => {
                    let visibility = if func_decl.is_exported {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    let symbol = Symbol::new(
                        func_decl.name.clone(),
                        SymbolKind::Function,
                        visibility,
                        func_decl.position,
                    );

                    module.symbols.define(symbol.clone())
                        .map_err(|e| BuluError::Other(e))?;

                    if func_decl.is_exported {
                        module.add_export(func_decl.name.clone(), symbol);
                    }
                }
                Statement::VariableDecl(var_decl) => {
                    let visibility = if var_decl.is_exported {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    let kind = if var_decl.is_const {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };

                    let symbol = Symbol::new(
                        var_decl.name.clone(),
                        kind,
                        visibility,
                        var_decl.position,
                    );

                    module.symbols.define(symbol.clone())
                        .map_err(|e| BuluError::Other(e))?;

                    if var_decl.is_exported {
                        module.add_export(var_decl.name.clone(), symbol);
                    }
                }
                Statement::StructDecl(struct_decl) => {
                    let visibility = if struct_decl.is_exported {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    let symbol = Symbol::new(
                        struct_decl.name.clone(),
                        SymbolKind::Struct,
                        visibility,
                        struct_decl.position,
                    );

                    module.symbols.define(symbol.clone())
                        .map_err(|e| BuluError::Other(e))?;

                    if struct_decl.is_exported {
                        module.add_export(struct_decl.name.clone(), symbol);
                    }
                }
                Statement::InterfaceDecl(interface_decl) => {
                    let visibility = if interface_decl.is_exported {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    let symbol = Symbol::new(
                        interface_decl.name.clone(),
                        SymbolKind::Interface,
                        visibility,
                        interface_decl.position,
                    );

                    module.symbols.define(symbol.clone())
                        .map_err(|e| BuluError::Other(e))?;

                    if interface_decl.is_exported {
                        module.add_export(interface_decl.name.clone(), symbol);
                    }
                }
                Statement::TypeAlias(type_alias) => {
                    // Type aliases are always considered exported for now
                    let symbol = Symbol::new(
                        type_alias.name.clone(),
                        SymbolKind::TypeAlias,
                        Visibility::Public,
                        type_alias.position,
                    );

                    module.symbols.define(symbol.clone())
                        .map_err(|e| BuluError::Other(e))?;
                    module.add_export(type_alias.name.clone(), symbol);
                }
                _ => {
                    // Other statements don't define exportable symbols
                }
            }
        }

        Ok(())
    }
}