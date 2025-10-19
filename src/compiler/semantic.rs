//! Semantic analysis for closure capture detection and other semantic checks

use crate::ast::*;
use crate::error::{BuluError, Result};
// Import resolution is handled by the symbol resolver
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Semantic analyzer that performs closure analysis and other semantic checks
pub struct SemanticAnalyzer {
    /// Stack of scopes for variable resolution
    scopes: Vec<Scope>,
    /// Current function depth (for detecting captures)
    function_depth: usize,
    /// Current file path
    current_file: Option<String>,
}

/// Represents a lexical scope
#[derive(Debug, Clone)]
struct Scope {
    /// Variables defined in this scope
    variables: HashMap<String, VariableInfo>,
    /// Function depth when this scope was created
    function_depth: usize,
    /// Whether this scope is a function scope
    is_function_scope: bool,
}

/// Information about a variable
#[derive(Debug, Clone)]
struct VariableInfo {
    /// Whether the variable is mutable
    is_mutable: bool,
    /// Function depth where the variable was defined
    function_depth: usize,
    /// Whether the variable has been captured by a closure
    is_captured: bool,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            function_depth: 0,
            current_file: None,
        }
    }

    /// Create a new semantic analyzer with a specific file path
    pub fn with_file(file_path: String) -> Self {
        let mut analyzer = Self::new();
        analyzer.current_file = Some(file_path);
        analyzer
    }

    /// Set the current file path
    pub fn set_current_file(&mut self, file_path: String) {
        self.current_file = Some(file_path);
    }

    /// Analyze the entire program
    pub fn analyze(&mut self, program: &mut Program) -> Result<()> {
        // Import resolution is handled by the symbol resolver
        // This analyzer focuses on semantic analysis only

        // Create global scope
        self.push_scope(false);

        // Analyze all statements
        for statement in &mut program.statements {
            self.analyze_statement(statement)?;
        }

        self.pop_scope();
        Ok(())
    }

    /// Push a new scope onto the scope stack
    fn push_scope(&mut self, is_function_scope: bool) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
            function_depth: self.function_depth,
            is_function_scope,
        });
    }

    /// Pop the current scope from the scope stack
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable in the current scope
    fn define_variable(&mut self, name: String, is_mutable: bool) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.variables.insert(
                name,
                VariableInfo {
                    is_mutable,
                    function_depth: self.function_depth,
                    is_captured: false,
                },
            );
        }
    }

    /// Resolve a variable and return capture information if needed
    fn resolve_variable(&mut self, name: &str) -> Option<(bool, bool)> {
        // Search from innermost to outermost scope
        for (_scope_index, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(var_info) = scope.variables.get(name) {
                // Check if this is a capture (variable from outer function scope)
                let is_capture = var_info.function_depth < self.function_depth;

                // Mark variable as captured if it's from an outer function
                if is_capture {
                    // We need to mark the variable as captured in its original scope
                    // This is a bit tricky with the current borrow checker, so we'll handle it differently
                }

                return Some((var_info.is_mutable, is_capture));
            }
        }
        None
    }

    /// Analyze a statement
    fn analyze_statement(&mut self, statement: &mut Statement) -> Result<()> {
        match statement {
            Statement::VariableDecl(decl) => {
                // Analyze initializer first (if any)
                if let Some(ref mut initializer) = decl.initializer {
                    self.analyze_expression(initializer)?;
                }

                // Define the variable in current scope
                self.define_variable(decl.name.clone(), !decl.is_const);
            }

            Statement::FunctionDecl(decl) => {
                // Define function name in current scope
                self.define_variable(decl.name.clone(), false);

                // Enter function scope
                self.function_depth += 1;
                self.push_scope(true);

                // Define parameters
                for param in &decl.params {
                    self.define_variable(param.name.clone(), true);
                }

                // Analyze function body
                self.analyze_block_statement(&mut decl.body)?;

                // Exit function scope
                self.pop_scope();
                self.function_depth -= 1;
            }

            Statement::Block(block) => {
                self.push_scope(false);
                self.analyze_block_statement(block)?;
                self.pop_scope();
            }

            Statement::If(if_stmt) => {
                self.analyze_expression(&mut if_stmt.condition)?;
                self.analyze_block_statement(&mut if_stmt.then_branch)?;

                if let Some(ref mut else_branch) = if_stmt.else_branch {
                    self.analyze_statement(else_branch)?;
                }
            }

            Statement::While(while_stmt) => {
                self.analyze_expression(&mut while_stmt.condition)?;
                self.analyze_block_statement(&mut while_stmt.body)?;
            }

            Statement::For(for_stmt) => {
                self.push_scope(false);

                // Define loop variables
                self.define_variable(for_stmt.variable.clone(), true);
                if let Some(ref index_var) = for_stmt.index_variable {
                    self.define_variable(index_var.clone(), true);
                }

                self.analyze_expression(&mut for_stmt.iterable)?;
                self.analyze_block_statement(&mut for_stmt.body)?;

                self.pop_scope();
            }

            Statement::Return(return_stmt) => {
                if let Some(ref mut value) = return_stmt.value {
                    self.analyze_expression(value)?;
                }
            }

            Statement::Expression(expr_stmt) => {
                self.analyze_expression(&mut expr_stmt.expr)?;
            }

            // Add other statement types as needed
            _ => {
                // For now, just continue without analysis for other statement types
            }
        }

        Ok(())
    }

    /// Analyze a block statement
    fn analyze_block_statement(&mut self, block: &mut BlockStmt) -> Result<()> {
        for statement in &mut block.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    /// Analyze an expression and detect captures for lambda expressions
    fn analyze_expression(&mut self, expression: &mut Expression) -> Result<()> {
        match expression {
            Expression::Lambda(lambda) => {
                // Enter lambda scope (new function)
                self.function_depth += 1;
                self.push_scope(true);

                // Define parameters
                for param in &lambda.params {
                    self.define_variable(param.name.clone(), true);
                }

                // Collect variables referenced in the lambda body
                let mut captured_vars = HashSet::new();
                self.collect_variable_references(&lambda.body, &mut captured_vars)?;

                // Analyze the body
                self.analyze_expression(&mut lambda.body)?;

                // Determine which variables are captures
                let mut captures = Vec::new();
                for var_name in captured_vars {
                    if let Some((_is_mutable, is_capture)) = self.resolve_variable(&var_name) {
                        if is_capture {
                            // For now, determine capture type based on usage in the lambda body
                            let capture_type = if self
                                .is_variable_mutated_in_expression(&lambda.body, &var_name)
                            {
                                CaptureType::ByReference
                            } else {
                                CaptureType::ByValue
                            };

                            captures.push(Capture {
                                name: var_name,
                                capture_type,
                                position: lambda.position, // Use lambda position for now
                            });
                        }
                    }
                }

                // Update lambda with capture information
                lambda.captures = captures;

                // Exit lambda scope
                self.pop_scope();
                self.function_depth -= 1;
            }

            Expression::Identifier(_ident) => {
                // Skip identifier validation here - it's handled by the TypeChecker
                // which has proper access to imported symbols from the SymbolResolver
                //
                // The original code was:
                // if !self.is_identifier_defined(&ident.name) {
                //     return Err(BuluError::type_error(...));
                // }
            }

            Expression::Call(call) => {
                self.analyze_expression(&mut call.callee)?;
                for arg in &mut call.args {
                    self.analyze_expression(arg)?;
                }
            }

            Expression::Binary(binary) => {
                self.analyze_expression(&mut binary.left)?;
                self.analyze_expression(&mut binary.right)?;
            }

            Expression::Unary(unary) => {
                self.analyze_expression(&mut unary.operand)?;
            }

            Expression::Assignment(assignment) => {
                self.analyze_expression(&mut assignment.target)?;
                self.analyze_expression(&mut assignment.value)?;
            }

            Expression::Block(block) => {
                self.push_scope(false);
                for statement in &mut block.statements {
                    self.analyze_statement(statement)?;
                }
                self.pop_scope();
            }

            Expression::Array(array) => {
                for element in &mut array.elements {
                    self.analyze_expression(element)?;
                }
            }

            Expression::If(if_expr) => {
                self.analyze_expression(&mut if_expr.condition)?;
                self.analyze_expression(&mut if_expr.then_expr)?;
                self.analyze_expression(&mut if_expr.else_expr)?;
            }

            // Add other expression types as needed
            _ => {
                // For now, just continue without analysis for other expression types
            }
        }

        Ok(())
    }

    /// Collect all variable references in an expression (for capture detection)
    fn collect_variable_references(
        &self,
        expression: &Expression,
        vars: &mut HashSet<String>,
    ) -> Result<()> {
        match expression {
            Expression::Identifier(ident) => {
                vars.insert(ident.name.clone());
            }

            Expression::Binary(binary) => {
                self.collect_variable_references(&binary.left, vars)?;
                self.collect_variable_references(&binary.right, vars)?;
            }

            Expression::Unary(unary) => {
                self.collect_variable_references(&unary.operand, vars)?;
            }

            Expression::Call(call) => {
                self.collect_variable_references(&call.callee, vars)?;
                for arg in &call.args {
                    self.collect_variable_references(arg, vars)?;
                }
            }

            Expression::Assignment(assignment) => {
                self.collect_variable_references(&assignment.target, vars)?;
                self.collect_variable_references(&assignment.value, vars)?;
            }

            Expression::Block(block) => {
                // For block expressions, we need to be careful about scope
                // For now, just collect all references
                for statement in &block.statements {
                    self.collect_statement_references(statement, vars)?;
                }
            }

            Expression::Array(array) => {
                for element in &array.elements {
                    self.collect_variable_references(element, vars)?;
                }
            }

            Expression::If(if_expr) => {
                self.collect_variable_references(&if_expr.condition, vars)?;
                self.collect_variable_references(&if_expr.then_expr, vars)?;
                self.collect_variable_references(&if_expr.else_expr, vars)?;
            }

            // Add other expression types as needed
            _ => {
                // For literals and other expressions that don't reference variables
            }
        }

        Ok(())
    }

    /// Collect variable references in a statement
    fn collect_statement_references(
        &self,
        statement: &Statement,
        vars: &mut HashSet<String>,
    ) -> Result<()> {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.collect_variable_references(&expr_stmt.expr, vars)?;
            }

            Statement::Return(return_stmt) => {
                if let Some(ref value) = return_stmt.value {
                    self.collect_variable_references(value, vars)?;
                }
            }

            Statement::VariableDecl(decl) => {
                if let Some(ref initializer) = decl.initializer {
                    self.collect_variable_references(initializer, vars)?;
                }
            }

            // Add other statement types as needed
            _ => {}
        }

        Ok(())
    }

    /// Check if a variable is mutated (assigned to) in an expression
    fn is_variable_mutated_in_expression(&self, expression: &Expression, var_name: &str) -> bool {
        match expression {
            Expression::Assignment(assignment) => {
                // Check if the target of assignment is our variable
                if let Expression::Identifier(ident) = &*assignment.target {
                    if ident.name == var_name {
                        return true;
                    }
                }
                // Also check the value expression
                self.is_variable_mutated_in_expression(&assignment.value, var_name)
            }

            Expression::Binary(binary) => {
                self.is_variable_mutated_in_expression(&binary.left, var_name)
                    || self.is_variable_mutated_in_expression(&binary.right, var_name)
            }

            Expression::Unary(unary) => {
                self.is_variable_mutated_in_expression(&unary.operand, var_name)
            }

            Expression::Call(call) => {
                if self.is_variable_mutated_in_expression(&call.callee, var_name) {
                    return true;
                }
                for arg in &call.args {
                    if self.is_variable_mutated_in_expression(arg, var_name) {
                        return true;
                    }
                }
                false
            }

            Expression::Block(block) => {
                for statement in &block.statements {
                    if self.is_variable_mutated_in_statement(statement, var_name) {
                        return true;
                    }
                }
                false
            }

            Expression::Array(array) => {
                for element in &array.elements {
                    if self.is_variable_mutated_in_expression(element, var_name) {
                        return true;
                    }
                }
                false
            }

            Expression::If(if_expr) => {
                self.is_variable_mutated_in_expression(&if_expr.condition, var_name)
                    || self.is_variable_mutated_in_expression(&if_expr.then_expr, var_name)
                    || self.is_variable_mutated_in_expression(&if_expr.else_expr, var_name)
            }

            // For other expressions, assume no mutation
            _ => false,
        }
    }

    /// Check if a variable is mutated in a statement
    fn is_variable_mutated_in_statement(&self, statement: &Statement, var_name: &str) -> bool {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.is_variable_mutated_in_expression(&expr_stmt.expr, var_name)
            }

            Statement::VariableDecl(decl) => {
                if let Some(ref initializer) = decl.initializer {
                    self.is_variable_mutated_in_expression(initializer, var_name)
                } else {
                    false
                }
            }

            Statement::Return(return_stmt) => {
                if let Some(ref value) = return_stmt.value {
                    self.is_variable_mutated_in_expression(value, var_name)
                } else {
                    false
                }
            }

            Statement::If(if_stmt) => {
                if self.is_variable_mutated_in_expression(&if_stmt.condition, var_name) {
                    return true;
                }

                for stmt in &if_stmt.then_branch.statements {
                    if self.is_variable_mutated_in_statement(stmt, var_name) {
                        return true;
                    }
                }

                if let Some(ref else_branch) = if_stmt.else_branch {
                    if self.is_variable_mutated_in_statement(else_branch, var_name) {
                        return true;
                    }
                }

                false
            }

            // Add other statement types as needed
            _ => false,
        }
    }

    /// Check if an identifier is defined in the current scope or imported
    fn is_identifier_defined(&self, name: &str) -> bool {
        // Check local scopes first
        for scope in self.scopes.iter().rev() {
            if scope.variables.contains_key(name) {
                return true;
            }
        }

        // Import resolution is handled by the symbol resolver

        // Check built-in functions and types
        self.is_builtin_symbol(name)
    }

    /// Check if a name is a built-in symbol
    fn is_builtin_symbol(&self, name: &str) -> bool {
        matches!(
            name,
            // Built-in functions
            "print" | "println" | "printf" | "input" |
            "len" | "cap" | "append" | "make" | "copy" | "clone" |
            "panic" | "recover" | "assert" |
            "typeof" | "instanceof" |
            // Type conversion functions
            "int8" | "int16" | "int32" | "int64" |
            "uint8" | "uint16" | "uint32" | "uint64" |
            "float32" | "float64" | "string" | "bool" |
            // Built-in types
            "any" |
            // Channel type identifiers (generated by make() parser)
            "chan_int8" | "chan_int16" | "chan_int32" | "chan_int64" |
            "chan_uint8" | "chan_uint16" | "chan_uint32" | "chan_uint64" |
            "chan_float32" | "chan_float64" | "chan_bool" | "chan_char" |
            "chan_string" | "chan_any" | "chan_unknown" | "chan"
        )
    }
}
