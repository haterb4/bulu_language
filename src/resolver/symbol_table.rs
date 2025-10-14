//! Symbol table implementation for tracking symbols and their visibility

use crate::lexer::token::Position;
use std::collections::HashMap;

/// Symbol visibility
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,    // Exported symbol
    Private,   // Not exported symbol
}

/// Symbol kind
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Variable,
    Constant,
    Struct,
    Interface,
    TypeAlias,
    Module,
}

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub position: Position,
    pub module_path: Option<String>,
}

impl Symbol {
    pub fn new(
        name: String,
        kind: SymbolKind,
        visibility: Visibility,
        position: Position,
    ) -> Self {
        Self {
            name,
            kind,
            visibility,
            position,
            module_path: None,
        }
    }

    pub fn with_module_path(mut self, module_path: String) -> Self {
        self.module_path = Some(module_path);
        self
    }

    /// Check if this symbol is exported (public)
    pub fn is_exported(&self) -> bool {
        self.visibility == Visibility::Public
    }
}

/// Symbol table for tracking symbols in a scope
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: SymbolTable) -> Self {
        Self {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a symbol in this scope
    pub fn define(&mut self, symbol: Symbol) -> Result<(), String> {
        if self.symbols.contains_key(&symbol.name) {
            return Err(format!("Symbol '{}' is already defined", symbol.name));
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol in this scope or parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        if let Some(symbol) = self.symbols.get(name) {
            Some(symbol)
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Look up a symbol only in this scope (not parent scopes)
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Get all symbols in this scope
    pub fn symbols(&self) -> &HashMap<String, Symbol> {
        &self.symbols
    }

    /// Get all exported symbols
    pub fn exported_symbols(&self) -> HashMap<String, Symbol> {
        self.symbols
            .iter()
            .filter(|(_, symbol)| symbol.is_exported())
            .map(|(name, symbol)| (name.clone(), symbol.clone()))
            .collect()
    }

    /// Check if a symbol exists in this scope or parent scopes
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Check if a symbol exists only in this scope
    pub fn contains_local(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}