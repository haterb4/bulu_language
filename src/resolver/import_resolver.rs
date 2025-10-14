//! Import resolution and validation

use crate::error::{BuluError, Result};
use crate::ast::*;
use crate::lexer::token::Position;
use super::{ModuleResolver, ResolutionContext, Symbol};
use std::path::Path;

/// Import resolver for validating and resolving imports
pub struct ImportResolver {
    module_resolver: ModuleResolver,
}

impl ImportResolver {
    pub fn new(module_resolver: ModuleResolver) -> Self {
        Self { module_resolver }
    }

    /// Get a mutable reference to the module resolver
    pub fn module_resolver_mut(&mut self) -> &mut ModuleResolver {
        &mut self.module_resolver
    }

    /// Resolve all imports in a program
    pub fn resolve_imports(
        &mut self,
        ast: &Program,
        current_file: Option<&Path>,
        context: &mut ResolutionContext,
    ) -> Result<()> {
        for statement in &ast.statements {
            if let Statement::Import(import_stmt) = statement {
                self.resolve_import(import_stmt, current_file, context)?;
            }
        }
        Ok(())
    }

    /// Resolve a single import statement
    fn resolve_import(
        &mut self,
        import_stmt: &ImportStmt,
        current_file: Option<&Path>,
        context: &mut ResolutionContext,
    ) -> Result<()> {
        // Resolve the module path
        let module_path = self.module_resolver.resolve_module_path(&import_stmt.path, current_file)?;

        // Load the module if not already loaded
        let module_name = self.get_module_name(&import_stmt.path);
        if !context.modules.contains_key(&module_name) {
            let module = self.module_resolver.load_module(&module_path)?;
            context.register_module(module);
        }

        // Validate the import
        self.validate_import(import_stmt, &module_name, context)?;

        Ok(())
    }

    /// Validate that imported symbols exist and are exported
    fn validate_import(
        &self,
        import_stmt: &ImportStmt,
        module_name: &str,
        context: &ResolutionContext,
    ) -> Result<()> {
        let module = context.get_module(module_name)
            .ok_or_else(|| BuluError::Other(format!("Module not found: {}", module_name)))?;

        if let Some(items) = &import_stmt.items {
            // Validate each imported item
            for item in items {
                self.validate_import_item(item, module_name, module, import_stmt)?;
            }
        } else {
            // Importing the entire module - no specific validation needed
            // The module itself exists, which is sufficient
        }

        Ok(())
    }

    /// Validate a single import item
    fn validate_import_item(
        &self,
        item: &ImportItem,
        module_name: &str,
        module: &super::Module,
        import_stmt: &ImportStmt,
    ) -> Result<()> {
        // Check if the symbol exists in the module
        if let Some(symbol) = module.symbols.lookup(&item.name) {
            // Check if the symbol is exported
            if !symbol.is_exported() {
                return Err(BuluError::parse_error(
                    format!(
                        "Symbol '{}' is not exported from module '{}'",
                        item.name, module_name
                    ),
                    item.position.line,
                    item.position.column,
                    Some(import_stmt.path.clone()),
                ));
            }
        } else {
            // Symbol doesn't exist in the module
            return Err(BuluError::parse_error(
                format!(
                    "Symbol '{}' does not exist in module '{}'",
                    item.name, module_name
                ),
                item.position.line,
                item.position.column,
                Some(import_stmt.path.clone()),
            ));
        }

        Ok(())
    }

    /// Get module name from import path
    fn get_module_name(&self, path: &str) -> String {
        if path.starts_with("std.") {
            path.to_string()
        } else {
            // Extract filename without extension
            Path::new(path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        }
    }

    /// Resolve symbol usage in expressions
    pub fn resolve_symbol_usage(
        &self,
        identifier: &str,
        position: Position,
        context: &ResolutionContext,
    ) -> Result<Option<Symbol>> {
        // First check current module
        if let Some(current_module) = context.current_module() {
            if let Some(symbol) = current_module.symbols.lookup(identifier) {
                return Ok(Some(symbol.clone()));
            }
        }

        // Then check imported symbols
        // This would require tracking which symbols were imported into the current scope
        // For now, we'll return None if not found locally
        Ok(None)
    }

    /// Check if a symbol is accessible in the current context
    pub fn is_symbol_accessible(
        &self,
        identifier: &str,
        context: &ResolutionContext,
    ) -> bool {
        // Check current module
        if let Some(current_module) = context.current_module() {
            if current_module.symbols.contains(identifier) {
                return true;
            }
        }

        // Check imported symbols
        // This would require more sophisticated tracking of imports
        false
    }

    /// Get all accessible symbols in the current context
    pub fn get_accessible_symbols(&self, context: &ResolutionContext) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        // Add symbols from current module
        if let Some(current_module) = context.current_module() {
            for symbol in current_module.symbols.symbols().values() {
                symbols.push(symbol.clone());
            }
        }

        // Add imported symbols
        // This would require tracking imports more carefully

        symbols
    }
}