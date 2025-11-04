//! Module resolution for finding and loading modules

use crate::error::{BuluError, Result};
use crate::ast::*;
use crate::lexer::{Lexer, token::Position};
use crate::parser::Parser;
use super::{Module, Symbol, SymbolKind, Visibility};
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
        if module_path.starts_with("std/") || module_path.starts_with("std.") {
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
            // Handle both std/net and std.net formats
            let module_file = if module_path.starts_with("std/") {
                // std/net -> net.bu
                module_path.strip_prefix("std/").unwrap().to_string() + ".bu"
            } else {
                // std.net -> net.bu
                module_path.strip_prefix("std.").unwrap().replace('.', "/") + ".bu"
            };
            
            let full_path = std_path.join(module_file);
            
            if full_path.exists() {
                Ok(full_path)
            } else {
                Err(BuluError::Other(format!("Standard library module not found: {} (looked in {})", module_path, full_path.display())))
            }
        } else {
            // Try to find std library in src/std directory as fallback
            let fallback_std_path = PathBuf::from("src/std");
            let module_file = if module_path.starts_with("std/") {
                module_path.strip_prefix("std/").unwrap().to_string() + ".bu"
            } else {
                module_path.strip_prefix("std.").unwrap().replace('.', "/") + ".bu"
            };
            
            let full_path = fallback_std_path.join(module_file);
            
            if full_path.exists() {
                Ok(full_path)
            } else {
                Err(BuluError::Other(format!("Standard library module not found: {} (looked in {} and no std_lib_path configured)", module_path, full_path.display())))
            }
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

    /// Load and parse a module from file or create a standard library module
    pub fn load_module(&self, module_path: &str) -> Result<Module> {
        // Handle standard library modules
        if module_path.starts_with("std/") || module_path.starts_with("std.") {
            return self.create_std_module(module_path);
        }

        // Handle regular file modules
        let file_path = self.resolve_module_path(module_path, None)?;
        let source = fs::read_to_string(&file_path)
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

    /// Create a virtual standard library module
    fn create_std_module(&self, module_path: &str) -> Result<Module> {
        let module_name = if module_path.starts_with("std/") {
            module_path.strip_prefix("std/").unwrap()
        } else {
            module_path.strip_prefix("std.").unwrap()
        };

        match module_name {
            "net" => self.create_net_module(),
            "time" => self.create_time_module(),
            "io" => self.create_io_module(),
            "math" => self.create_math_module(),
            _ => Err(BuluError::Other(format!("Unknown standard library module: {}", module_path)))
        }
    }

    /// Create the std/net module
    fn create_net_module(&self) -> Result<Module> {
        let mut module = Module::new(PathBuf::from("std/net"), "net".to_string());
        
        // Add exports for networking types and functions
        let position = Position::new(0, 0, 0);
        
        let tcp_server_symbol = Symbol::new("TcpServer".to_string(), SymbolKind::Struct, Visibility::Public, position);
        module.add_export("TcpServer".to_string(), tcp_server_symbol);
        
        let tcp_connection_symbol = Symbol::new("TcpConnection".to_string(), SymbolKind::Struct, Visibility::Public, position);
        module.add_export("TcpConnection".to_string(), tcp_connection_symbol);
        
        let udp_connection_symbol = Symbol::new("UdpConnection".to_string(), SymbolKind::Struct, Visibility::Public, position);
        module.add_export("UdpConnection".to_string(), udp_connection_symbol);
        
        let net_addr_symbol = Symbol::new("NetAddr".to_string(), SymbolKind::Struct, Visibility::Public, position);
        module.add_export("NetAddr".to_string(), net_addr_symbol);

        Ok(module)
    }

    /// Create the std/time module
    fn create_time_module(&self) -> Result<Module> {
        let mut module = Module::new(PathBuf::from("std/time"), "time".to_string());
        
        // Add exports for time functions
        let position = Position::new(0, 0, 0);
        
        let sleep_symbol = Symbol::new("sleep".to_string(), SymbolKind::Function, Visibility::Public, position);
        module.add_export("sleep".to_string(), sleep_symbol);

        Ok(module)
    }

    /// Create the std/io module
    fn create_io_module(&self) -> Result<Module> {
        let mut module = Module::new(PathBuf::from("std/io"), "io".to_string());
        
        // Add exports for IO functions
        let position = Position::new(0, 0, 0);
        
        let print_symbol = Symbol::new("print".to_string(), SymbolKind::Function, Visibility::Public, position);
        module.add_export("print".to_string(), print_symbol);
        
        let println_symbol = Symbol::new("println".to_string(), SymbolKind::Function, Visibility::Public, position);
        module.add_export("println".to_string(), println_symbol);

        Ok(module)
    }

    /// Create the std/math module
    fn create_math_module(&self) -> Result<Module> {
        let mut module = Module::new(PathBuf::from("std/math"), "math".to_string());
        
        // Add exports for math functions
        let position = Position::new(0, 0, 0);
        
        let abs_symbol = Symbol::new("abs".to_string(), SymbolKind::Function, Visibility::Public, position);
        module.add_export("abs".to_string(), abs_symbol);
        
        let sqrt_symbol = Symbol::new("sqrt".to_string(), SymbolKind::Function, Visibility::Public, position);
        module.add_export("sqrt".to_string(), sqrt_symbol);
        
        let pow_symbol = Symbol::new("pow".to_string(), SymbolKind::Function, Visibility::Public, position);
        module.add_export("pow".to_string(), pow_symbol);

        Ok(module)
    }
}