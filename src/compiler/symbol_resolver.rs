//! Symbol resolution for imports and exports in the Bulu language

use crate::ast::*;
use crate::error::{BuluError, Result};
use crate::runtime::module::ModuleResolver;
use crate::types::primitive::RuntimeValue;
use std::collections::HashMap;

/// Symbol table for tracking imported and local symbols
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Local symbols defined in the current module
    pub local_symbols: HashMap<String, SymbolInfo>,
    /// Imported symbols from other modules
    pub imported_symbols: HashMap<String, ImportedSymbolInfo>,
    /// Exported symbols from the current module
    pub exported_symbols: HashMap<String, SymbolInfo>,
}

/// Information about a symbol
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub symbol_type: SymbolType,
    pub is_exported: bool,
    pub position: crate::lexer::token::Position,
    /// Function signature information (only for functions)
    pub function_signature: Option<FunctionSignature>,
}

/// Information about an imported symbol
#[derive(Debug, Clone)]
pub struct ImportedSymbolInfo {
    pub name: String,
    pub original_name: String,
    pub module_path: String,
    pub symbol_type: SymbolType,
    pub position: crate::lexer::token::Position,
    /// Function signature information (only for functions)
    pub function_signature: Option<FunctionSignature>,
}

/// Type of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Variable,
    Constant,
    Struct,
    Interface,
    TypeAlias,
    Module,
}

/// Function signature information
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<Type>,
    pub is_async: bool,
    pub is_variadic: bool,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: Type,
    pub has_default: bool,
    pub is_variadic: bool,
}

/// Symbol resolver that handles import/export resolution
pub struct SymbolResolver {
    module_resolver: ModuleResolver,
    symbol_table: SymbolTable,
    current_module_path: Option<String>,
    /// Stack of local scopes for tracking variables in functions/blocks
    scope_stack: Vec<HashMap<String, SymbolInfo>>,
}

impl SymbolResolver {
    /// Create a new symbol resolver
    pub fn new() -> Self {
        Self {
            module_resolver: ModuleResolver::new(),
            symbol_table: SymbolTable {
                local_symbols: HashMap::new(),
                imported_symbols: HashMap::new(),
                exported_symbols: HashMap::new(),
            },
            current_module_path: None,
            scope_stack: Vec::new(),
        }
    }

    /// Set the current module path for resolution context
    pub fn set_current_module(&mut self, path: String) {
        self.current_module_path = Some(path.clone());
        
        // Also set the current directory for the module resolver
        if let Some(parent_dir) = std::path::Path::new(&path).parent() {
            self.module_resolver.set_current_dir(parent_dir.to_path_buf());
        }
    }

    /// Push a new local scope
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pop the current local scope
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Define a symbol in the current local scope
    fn define_local_symbol(&mut self, name: String, symbol: SymbolInfo) {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.insert(name, symbol);
        }
    }

    /// Check if a symbol exists in any local scope
    fn is_in_local_scope(&self, name: &str) -> bool {
        for scope in self.scope_stack.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    /// Resolve all imports and exports in a program
    pub fn resolve_program(&mut self, program: &mut Program) -> Result<()> {
        // First pass: collect all local declarations
        self.collect_local_symbols(program)?;

        // Second pass: resolve imports
        self.resolve_imports(program)?;

        // Third pass: validate symbol usage (with scope tracking)
        self.validate_symbol_usage(program)?;

        Ok(())
    }

    /// Collect all local symbol declarations
    fn collect_local_symbols(&mut self, program: &Program) -> Result<()> {
        for statement in &program.statements {
            match statement {
                Statement::FunctionDecl(func) => {
                    let function_signature = Some(FunctionSignature {
                        parameters: func.params.iter().map(|p| ParameterInfo {
                            name: p.name.clone(),
                            param_type: p.param_type.clone(),
                            has_default: p.default_value.is_some(),
                            is_variadic: p.is_variadic,
                        }).collect(),
                        return_type: func.return_type.clone(),
                        is_async: func.is_async,
                        is_variadic: func.params.iter().any(|p| p.is_variadic),
                    });
                    
                    let symbol = SymbolInfo {
                        name: func.name.clone(),
                        symbol_type: SymbolType::Function,
                        is_exported: func.is_exported,
                        position: func.position,
                        function_signature,
                    };
                    self.symbol_table.local_symbols.insert(func.name.clone(), symbol.clone());
                    
                    if func.is_exported {
                        self.symbol_table.exported_symbols.insert(func.name.clone(), symbol);
                    }
                }
                Statement::VariableDecl(var) => {
                    let symbol_type = if var.is_const {
                        SymbolType::Constant
                    } else {
                        SymbolType::Variable
                    };
                    
                    let symbol = SymbolInfo {
                        name: var.name.clone(),
                        symbol_type,
                        is_exported: var.is_exported,
                        position: var.position,
                        function_signature: None,
                    };
                    self.symbol_table.local_symbols.insert(var.name.clone(), symbol.clone());
                    
                    if var.is_exported {
                        self.symbol_table.exported_symbols.insert(var.name.clone(), symbol);
                    }
                }
                Statement::StructDecl(struct_decl) => {
                    let symbol = SymbolInfo {
                        name: struct_decl.name.clone(),
                        symbol_type: SymbolType::Struct,
                        is_exported: struct_decl.is_exported,
                        position: struct_decl.position,
                        function_signature: None,
                    };
                    self.symbol_table.local_symbols.insert(struct_decl.name.clone(), symbol.clone());
                    
                    if struct_decl.is_exported {
                        self.symbol_table.exported_symbols.insert(struct_decl.name.clone(), symbol);
                    }
                }
                Statement::InterfaceDecl(interface) => {
                    let symbol = SymbolInfo {
                        name: interface.name.clone(),
                        symbol_type: SymbolType::Interface,
                        is_exported: interface.is_exported,
                        position: interface.position,
                        function_signature: None,
                    };
                    self.symbol_table.local_symbols.insert(interface.name.clone(), symbol.clone());
                    
                    if interface.is_exported {
                        self.symbol_table.exported_symbols.insert(interface.name.clone(), symbol);
                    }
                }
                Statement::TypeAlias(type_alias) => {
                    let symbol = SymbolInfo {
                        name: type_alias.name.clone(),
                        symbol_type: SymbolType::TypeAlias,
                        is_exported: false, // Type aliases don't have explicit export in current AST
                        position: type_alias.position,
                        function_signature: None,
                    };
                    self.symbol_table.local_symbols.insert(type_alias.name.clone(), symbol);
                }
                Statement::Export(export_stmt) => {
                    // Handle explicit exports
                    self.handle_export_statement(export_stmt)?;
                }
                _ => {
                    // Other statements don't declare symbols
                }
            }
        }
        Ok(())
    }

    /// Handle explicit export statements
    fn handle_export_statement(&mut self, export_stmt: &ExportStmt) -> Result<()> {
        match export_stmt.item.as_ref() {
            Statement::Import(import_stmt) => {
                // Re-export: export { item1, item2 } from "module"
                self.handle_reexport(import_stmt)?;
            }
            Statement::FunctionDecl(func) => {
                let function_signature = Some(FunctionSignature {
                    parameters: func.params.iter().map(|p| ParameterInfo {
                        name: p.name.clone(),
                        param_type: p.param_type.clone(),
                        has_default: p.default_value.is_some(),
                        is_variadic: p.is_variadic,
                    }).collect(),
                    return_type: func.return_type.clone(),
                    is_async: func.is_async,
                    is_variadic: func.params.iter().any(|p| p.is_variadic),
                });
                
                let symbol = SymbolInfo {
                    name: func.name.clone(),
                    symbol_type: SymbolType::Function,
                    is_exported: true,
                    position: func.position,
                    function_signature,
                };
                self.symbol_table.exported_symbols.insert(func.name.clone(), symbol);
            }
            Statement::VariableDecl(var) => {
                let symbol_type = if var.is_const {
                    SymbolType::Constant
                } else {
                    SymbolType::Variable
                };
                
                let symbol = SymbolInfo {
                    name: var.name.clone(),
                    symbol_type,
                    is_exported: true,
                    position: var.position,
                    function_signature: None,
                };
                self.symbol_table.exported_symbols.insert(var.name.clone(), symbol);
            }
            _ => {
                return Err(BuluError::TypeError {
                    message: "Only functions, variables, and re-exports can be exported".to_string(),
                    line: export_stmt.position.line,
                    column: export_stmt.position.column,
                    file: self.current_module_path.clone(),
                });
            }
        }
        Ok(())
    }

    /// Handle re-export statements
    fn handle_reexport(&mut self, import_stmt: &ImportStmt) -> Result<()> {
        // Load the module to get its exports
        let module = self.module_resolver.load_module(&import_stmt.path)?;
        
        if let Some(items) = &import_stmt.items {
            // Re-export specific items
            for item in items {
                if module.exports.contains_key(&item.name) {
                    let symbol = SymbolInfo {
                        name: item.alias.as_ref().unwrap_or(&item.name).clone(),
                        symbol_type: self.infer_symbol_type_from_value(&module.exports[&item.name]),
                        is_exported: true,
                        position: item.position,
                        function_signature: None, // TODO: Extract function signature from module exports
                    };
                    self.symbol_table.exported_symbols.insert(symbol.name.clone(), symbol);
                } else {
                    return Err(BuluError::TypeError {
                        message: format!(
                            "Module '{}' does not export '{}'",
                            import_stmt.path, item.name
                        ),
                        line: item.position.line,
                        column: item.position.column,
                        file: self.current_module_path.clone(),
                    });
                }
            }
        } else {
            // Re-export all items from the module
            for (name, value) in &module.exports {
                let symbol = SymbolInfo {
                    name: name.clone(),
                    symbol_type: self.infer_symbol_type_from_value(value),
                    is_exported: true,
                    position: import_stmt.position,
                    function_signature: None, // TODO: Extract function signature from module exports
                };
                self.symbol_table.exported_symbols.insert(name.clone(), symbol);
            }
        }
        
        Ok(())
    }

    /// Resolve all import statements
    fn resolve_imports(&mut self, program: &Program) -> Result<()> {
        for statement in &program.statements {
            if let Statement::Import(import_stmt) = statement {
                self.resolve_import_statement(import_stmt)?;
            }
        }
        Ok(())
    }

    /// Resolve a single import statement
    fn resolve_import_statement(&mut self, import_stmt: &ImportStmt) -> Result<()> {
        // Load the module
        let module = self.module_resolver.load_module(&import_stmt.path)?;

        if let Some(items) = &import_stmt.items {
            // Import specific items: import { item1, item2 } from "path"
            for item in items {
                if let Some(value) = module.exports.get(&item.name) {
                    let symbol_name = item.alias.as_ref().unwrap_or(&item.name);
                    let symbol_type = self.infer_symbol_type_from_value(value);
                    let function_signature = if symbol_type == SymbolType::Function {
                        self.extract_function_signature_from_module(&module, &item.name)
                    } else {
                        None
                    };
                    
                    let imported_symbol = ImportedSymbolInfo {
                        name: symbol_name.clone(),
                        original_name: item.name.clone(),
                        module_path: import_stmt.path.clone(),
                        symbol_type,
                        position: item.position,
                        function_signature,
                    };
                    self.symbol_table.imported_symbols.insert(symbol_name.clone(), imported_symbol);
                } else {
                    return Err(BuluError::TypeError {
                        message: format!(
                            "Module '{}' does not export '{}'",
                            import_stmt.path, item.name
                        ),
                        line: item.position.line,
                        column: item.position.column,
                        file: self.current_module_path.clone(),
                    });
                }
            }
        } else if let Some(alias) = &import_stmt.alias {
            // Import entire module with alias: import "path" as alias
            let imported_symbol = ImportedSymbolInfo {
                name: alias.clone(),
                original_name: import_stmt.path.clone(),
                module_path: import_stmt.path.clone(),
                symbol_type: SymbolType::Module,
                position: import_stmt.position,
                function_signature: None,
            };
            self.symbol_table.imported_symbols.insert(alias.clone(), imported_symbol);
        } else {
            // Import all exports: import "path"
            for (name, value) in &module.exports {
                let symbol_type = self.infer_symbol_type_from_value(value);
                let function_signature = if symbol_type == SymbolType::Function {
                    self.extract_function_signature_from_module(&module, name)
                } else {
                    None
                };
                
                let imported_symbol = ImportedSymbolInfo {
                    name: name.clone(),
                    original_name: name.clone(),
                    module_path: import_stmt.path.clone(),
                    symbol_type,
                    position: import_stmt.position,
                    function_signature,
                };
                self.symbol_table.imported_symbols.insert(name.clone(), imported_symbol);
            }
        }

        Ok(())
    }

    /// Extract function signature from module AST
    fn extract_function_signature_from_module(&self, module: &crate::runtime::module::Module, function_name: &str) -> Option<FunctionSignature> {
        // Search through the module's AST for the function declaration
        for statement in &module.ast.statements {
            match statement {
                Statement::FunctionDecl(func) if func.name == function_name => {
                    return Some(FunctionSignature {
                        parameters: func.params.iter().map(|p| ParameterInfo {
                            name: p.name.clone(),
                            param_type: p.param_type.clone(),
                            has_default: p.default_value.is_some(),
                            is_variadic: p.is_variadic,
                        }).collect(),
                        return_type: func.return_type.clone(),
                        is_async: func.is_async,
                        is_variadic: func.params.iter().any(|p| p.is_variadic),
                    });
                }
                Statement::Export(export_stmt) => {
                    // Check if this is an exported function
                    if let Statement::FunctionDecl(func) = export_stmt.item.as_ref() {
                        if func.name == function_name {
                            return Some(FunctionSignature {
                                parameters: func.params.iter().map(|p| ParameterInfo {
                                    name: p.name.clone(),
                                    param_type: p.param_type.clone(),
                                    has_default: p.default_value.is_some(),
                                    is_variadic: p.is_variadic,
                                }).collect(),
                                return_type: func.return_type.clone(),
                                is_async: func.is_async,
                                is_variadic: func.params.iter().any(|p| p.is_variadic),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Get the symbol table (for integration with other components)
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get all loaded modules for compilation
    pub fn get_loaded_modules(&self) -> Vec<&crate::runtime::module::Module> {
        self.module_resolver.get_loaded_modules()
    }

    /// Infer symbol type from runtime value
    fn infer_symbol_type_from_value(&self, value: &RuntimeValue) -> SymbolType {
        match value {
            RuntimeValue::String(s) => {
                if s.starts_with("function:") {
                    SymbolType::Function
                } else if s.starts_with("struct:") {
                    SymbolType::Struct
                } else if s.starts_with("interface:") {
                    SymbolType::Interface
                } else if s.starts_with("type:") {
                    SymbolType::TypeAlias
                } else {
                    SymbolType::Variable
                }
            }
            RuntimeValue::Map(_) => SymbolType::Module,
            _ => SymbolType::Variable,
        }
    }

    /// Validate that all used symbols are properly imported or defined
    fn validate_symbol_usage(&mut self, program: &Program) -> Result<()> {
        for statement in &program.statements {
            self.validate_statement_symbols(statement)?;
        }
        Ok(())
    }

    /// Validate symbols in a statement
    fn validate_statement_symbols(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.validate_expression_symbols(&expr_stmt.expr)?;
            }
            Statement::VariableDecl(var_decl) => {
                // First validate the initializer
                if let Some(ref initializer) = var_decl.initializer {
                    self.validate_expression_symbols(initializer)?;
                }
                
                // Then add the variable to the current scope
                let symbol = SymbolInfo {
                    name: var_decl.name.clone(),
                    symbol_type: if var_decl.is_const { SymbolType::Constant } else { SymbolType::Variable },
                    is_exported: var_decl.is_exported,
                    position: var_decl.position,
                    function_signature: None,
                };
                self.define_local_symbol(var_decl.name.clone(), symbol);
            }
            Statement::FunctionDecl(func_decl) => {
                // Create a new scope for the function
                self.push_scope();
                
                // Add parameters to the function scope
                for param in &func_decl.params {
                    let symbol = SymbolInfo {
                        name: param.name.clone(),
                        symbol_type: SymbolType::Variable,
                        is_exported: false,
                        position: param.position,
                        function_signature: None,
                    };
                    self.define_local_symbol(param.name.clone(), symbol);
                }
                
                // Validate the function body
                self.validate_block_symbols(&func_decl.body)?;
                
                // Pop the function scope
                self.pop_scope();
            }
            Statement::If(if_stmt) => {
                self.validate_expression_symbols(&if_stmt.condition)?;
                
                // Create new scope for then branch
                self.push_scope();
                self.validate_block_symbols(&if_stmt.then_branch)?;
                self.pop_scope();
                
                if let Some(ref else_branch) = if_stmt.else_branch {
                    self.push_scope();
                    self.validate_statement_symbols(else_branch)?;
                    self.pop_scope();
                }
            }
            Statement::While(while_stmt) => {
                self.validate_expression_symbols(&while_stmt.condition)?;
                
                self.push_scope();
                self.validate_block_symbols(&while_stmt.body)?;
                self.pop_scope();
            }
            Statement::For(for_stmt) => {
                self.validate_expression_symbols(&for_stmt.iterable)?;
                
                // Create new scope for the loop
                self.push_scope();
                
                // Add loop variables to scope
                let var_symbol = SymbolInfo {
                    name: for_stmt.variable.clone(),
                    symbol_type: SymbolType::Variable,
                    is_exported: false,
                    position: crate::lexer::token::Position::new(0, 0, 0), // TODO: get actual position
                    function_signature: None,
                };
                self.define_local_symbol(for_stmt.variable.clone(), var_symbol);
                
                if let Some(ref index_var) = for_stmt.index_variable {
                    let index_symbol = SymbolInfo {
                        name: index_var.clone(),
                        symbol_type: SymbolType::Variable,
                        is_exported: false,
                        position: crate::lexer::token::Position::new(0, 0, 0), // TODO: get actual position
                        function_signature: None,
                    };
                    self.define_local_symbol(index_var.clone(), index_symbol);
                }
                
                self.validate_block_symbols(&for_stmt.body)?;
                self.pop_scope();
            }
            Statement::Return(return_stmt) => {
                if let Some(ref value) = return_stmt.value {
                    self.validate_expression_symbols(value)?;
                }
            }
            Statement::Block(block) => {
                self.push_scope();
                self.validate_block_symbols(block)?;
                self.pop_scope();
            }
            _ => {
                // Other statements don't use symbols that need validation
            }
        }
        Ok(())
    }

    /// Validate symbols in a block
    fn validate_block_symbols(&mut self, block: &BlockStmt) -> Result<()> {
        for statement in &block.statements {
            self.validate_statement_symbols(statement)?;
        }
        Ok(())
    }

    /// Validate symbols in an expression
    fn validate_expression_symbols(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::Identifier(ident) => {
                self.validate_identifier(&ident.name, ident.position)?;
            }
            Expression::MemberAccess(member_access) => {
                // For member access, we need to validate the object but not the member
                // The member will be validated at runtime or by the type checker
                self.validate_expression_symbols(&member_access.object)?;
            }
            Expression::Call(call) => {
                self.validate_expression_symbols(&call.callee)?;
                for arg in &call.args {
                    self.validate_expression_symbols(arg)?;
                }
            }
            Expression::Binary(binary) => {
                self.validate_expression_symbols(&binary.left)?;
                self.validate_expression_symbols(&binary.right)?;
            }
            Expression::Unary(unary) => {
                self.validate_expression_symbols(&unary.operand)?;
            }
            Expression::Assignment(assignment) => {
                self.validate_expression_symbols(&assignment.target)?;
                self.validate_expression_symbols(&assignment.value)?;
            }
            Expression::Array(array) => {
                for element in &array.elements {
                    self.validate_expression_symbols(element)?;
                }
            }
            Expression::If(if_expr) => {
                self.validate_expression_symbols(&if_expr.condition)?;
                self.validate_expression_symbols(&if_expr.then_expr)?;
                self.validate_expression_symbols(&if_expr.else_expr)?;
            }
            _ => {
                // Other expressions don't reference symbols
            }
        }
        Ok(())
    }

    /// Validate that an identifier is properly defined or imported
    fn validate_identifier(&self, name: &str, position: crate::lexer::token::Position) -> Result<()> {
        // Check if it's in a local scope (function/block variables)
        if self.is_in_local_scope(name) {
            return Ok(());
        }

        // Check if it's a module-level local symbol
        if self.symbol_table.local_symbols.contains_key(name) {
            return Ok(());
        }

        // Check if it's an imported symbol
        if self.symbol_table.imported_symbols.contains_key(name) {
            return Ok(());
        }

        // Check if it's a built-in function or keyword
        if self.is_builtin(name) {
            return Ok(());
        }

        // Symbol not found
        Err(BuluError::TypeError {
            message: format!("Undefined symbol '{}'", name),
            line: position.line,
            column: position.column,
            file: self.current_module_path.clone(),
        })
    }

    /// Check if a name is a built-in function or keyword
    fn is_builtin(&self, name: &str) -> bool {
        // Check exact matches first
        if matches!(
            name,
            "print" | "println" | "len" | "append" | "make" | "clone" | "typeof" | "instanceof"
                | "panic" | "recover" | "assert" | "input" | "sleep" | "timer" | "lock" | "toString"
                // Primitive type identifiers
                | "int8" | "int16" | "int32" | "int64"
                | "uint8" | "uint16" | "uint32" | "uint64"
                | "float32" | "float64" | "bool" | "char"
                | "string" | "any" | "unknown" | "chan"
        ) {
            return true;
        }
        
        // Check patterns for generated type identifiers
        if name.starts_with("chan_") || name.starts_with("slice_") || name.starts_with("array_") || name.starts_with("map_") {
            return true;
        }
        
        false
    }

    /// Get the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get the module resolver
    pub fn module_resolver(&mut self) -> &mut ModuleResolver {
        &mut self.module_resolver
    }

    /// Check if a symbol is available in the current scope
    pub fn is_symbol_available(&self, name: &str) -> bool {
        self.symbol_table.local_symbols.contains_key(name)
            || self.symbol_table.imported_symbols.contains_key(name)
            || self.is_builtin(name)
    }

    /// Get information about a symbol
    pub fn get_symbol_info(&self, name: &str) -> Option<SymbolType> {
        if let Some(local) = self.symbol_table.local_symbols.get(name) {
            Some(local.symbol_type.clone())
        } else if let Some(imported) = self.symbol_table.imported_symbols.get(name) {
            Some(imported.symbol_type.clone())
        } else {
            None
        }
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}