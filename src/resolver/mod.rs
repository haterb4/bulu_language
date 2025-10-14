//! Symbol resolution and module system for Bulu language

pub mod symbol_table;
pub mod module_resolver;
pub mod import_resolver;

pub use symbol_table::{Symbol, SymbolTable, SymbolKind, Visibility};
pub use module_resolver::ModuleResolver;
pub use import_resolver::ImportResolver;

use crate::error::{BuluError, Result};
use crate::ast::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Module information
#[derive(Debug, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub name: String,
    pub symbols: SymbolTable,
    pub dependencies: Vec<String>,
    pub exports: HashMap<String, Symbol>,
}

impl Module {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            symbols: SymbolTable::new(),
            dependencies: Vec::new(),
            exports: HashMap::new(),
        }
    }

    /// Check if a symbol is exported from this module
    pub fn is_exported(&self, name: &str) -> bool {
        self.exports.contains_key(name)
    }

    /// Get an exported symbol
    pub fn get_exported_symbol(&self, name: &str) -> Option<&Symbol> {
        self.exports.get(name)
    }

    /// Add an exported symbol
    pub fn add_export(&mut self, name: String, symbol: Symbol) {
        self.exports.insert(name, symbol);
    }
}

/// Resolution context for tracking imports and exports
#[derive(Debug)]
pub struct ResolutionContext {
    pub modules: HashMap<String, Module>,
    pub current_module: Option<String>,
    pub import_paths: HashMap<String, PathBuf>,
}

impl ResolutionContext {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            current_module: None,
            import_paths: HashMap::new(),
        }
    }

    /// Register a module
    pub fn register_module(&mut self, module: Module) {
        let name = module.name.clone();
        self.modules.insert(name, module);
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Get a mutable reference to a module
    pub fn get_module_mut(&mut self, name: &str) -> Option<&mut Module> {
        self.modules.get_mut(name)
    }

    /// Set the current module being processed
    pub fn set_current_module(&mut self, name: String) {
        self.current_module = Some(name);
    }

    /// Get the current module
    pub fn current_module(&self) -> Option<&Module> {
        self.current_module.as_ref().and_then(|name| self.get_module(name))
    }

    /// Get the current module mutably
    pub fn current_module_mut(&mut self) -> Option<&mut Module> {
        let name = self.current_module.clone()?;
        self.get_module_mut(&name)
    }
}