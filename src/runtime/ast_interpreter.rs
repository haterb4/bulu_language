//! AST-based interpreter for Bulu programs
//!
//! This interpreter directly executes AST nodes and handles
//! import/export statements, module resolution, and symbol management.

use crate::ast::nodes::*;
use crate::error::{BuluError, Result};
use crate::runtime::module::ModuleResolver;
use crate::types::primitive::RuntimeValue;
use std::collections::HashMap;

/// Environment for variable and function storage
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variables in current scope
    variables: HashMap<String, RuntimeValue>,
    /// Parent environment for nested scopes
    parent: Option<Box<Environment>>,
}

impl Environment {
    /// Create a new environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent
    pub fn with_parent(parent: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, value: RuntimeValue) {
        self.variables.insert(name, value);
    }

    /// Get a variable from the current scope or parent scopes
    pub fn get(&self, name: &str) -> Option<&RuntimeValue> {
        if let Some(value) = self.variables.get(name) {
            Some(value)
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    /// Set a variable in the current scope or parent scopes
    pub fn set(&mut self, name: &str, value: RuntimeValue) -> Result<()> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value)
        } else {
            Err(BuluError::RuntimeError {
                message: format!("Undefined variable '{}'", name),
                file: None,
            })
        }
    }

    /// Check if a variable exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name) || 
        self.parent.as_ref().map_or(false, |p| p.contains(name))
    }
}

/// AST-based interpreter
pub struct AstInterpreter {
    /// Current environment
    environment: Environment,
    /// Module resolver for imports/exports
    pub module_resolver: ModuleResolver,
    /// Global environment for exported symbols
    globals: Environment,
    /// Current file being executed (for relative imports)
    current_file: Option<String>,
    /// Struct definitions for type checking and default values
    struct_definitions: HashMap<String, StructDecl>,
}

impl AstInterpreter {
    /// Create a new AST interpreter
    pub fn new() -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            module_resolver: ModuleResolver::new(),
            globals: Environment::new(),
            current_file: None,
            struct_definitions: HashMap::new(),
        };
        
        // Add built-in identifiers
        interpreter.environment.define("chan".to_string(), RuntimeValue::String("chan".to_string()));
        
        // Add primitive type identifiers for make() calls
        let primitive_types = vec![
            "int8", "int16", "int32", "int64",
            "uint8", "uint16", "uint32", "uint64", 
            "float32", "float64", "bool", "string",
            "char", "byte", "rune", "any"
        ];
        
        for prim_type in primitive_types {
            interpreter.environment.define(prim_type.to_string(), RuntimeValue::String(prim_type.to_string()));
        }
        
        // Add channel type identifiers
        let channel_types = vec![
            "chan_int8", "chan_int16", "chan_int32", "chan_int64",
            "chan_uint8", "chan_uint16", "chan_uint32", "chan_uint64",
            "chan_float32", "chan_float64", "chan_bool", "chan_char",
            "chan_string", "chan_any", "chan_unknown"
        ];
        
        for chan_type in channel_types {
            interpreter.environment.define(chan_type.to_string(), RuntimeValue::String(chan_type.to_string()));
        }
        
        interpreter
    }

    /// Create a new AST interpreter with a specific file context
    pub fn with_file(file_path: String) -> Self {
        let mut interpreter = Self::new();
        interpreter.current_file = Some(file_path);
        interpreter
    }

    /// Execute a program
    pub fn execute_program(&mut self, program: &Program) -> Result<RuntimeValue> {
        let mut last_value = RuntimeValue::Null;

        for statement in &program.statements {
            last_value = self.execute_statement(statement)?;
        }

        Ok(last_value)
    }

    /// Execute a statement
    pub fn execute_statement(&mut self, statement: &Statement) -> Result<RuntimeValue> {
        match statement {
            Statement::VariableDecl(decl) => self.execute_variable_decl(decl),
            Statement::FunctionDecl(decl) => self.execute_function_decl(decl),
            Statement::StructDecl(decl) => self.execute_struct_decl(decl),
            Statement::InterfaceDecl(decl) => self.execute_interface_decl(decl),
            Statement::TypeAlias(decl) => self.execute_type_alias_decl(decl),
            Statement::If(stmt) => self.execute_if_stmt(stmt),
            Statement::While(stmt) => self.execute_while_stmt(stmt),
            Statement::For(stmt) => self.execute_for_stmt(stmt),
            Statement::Match(stmt) => self.execute_match_stmt(stmt),
            Statement::Select(stmt) => self.execute_select_stmt(stmt),
            Statement::Return(stmt) => self.execute_return_stmt(stmt),
            Statement::Break(stmt) => self.execute_break_stmt(stmt),
            Statement::Continue(stmt) => self.execute_continue_stmt(stmt),
            Statement::Defer(stmt) => self.execute_defer_stmt(stmt),
            Statement::Try(stmt) => self.execute_try_stmt(stmt),
            Statement::Fail(stmt) => self.execute_fail_stmt(stmt),
            Statement::Import(stmt) => self.execute_import_stmt(stmt),
            Statement::Export(stmt) => self.execute_export_stmt(stmt),
            Statement::Expression(stmt) => self.execute_expression_stmt(stmt),
            Statement::Block(stmt) => self.execute_block_stmt(stmt),
        }
    }

    /// Execute variable declaration
    fn execute_variable_decl(&mut self, decl: &VariableDecl) -> Result<RuntimeValue> {
        let value = if let Some(initializer) = &decl.initializer {
            self.execute_expression(initializer)?
        } else {
            RuntimeValue::Null
        };

        self.environment.define(decl.name.clone(), value.clone());

        // If exported, also add to globals
        if decl.is_exported {
            self.globals.define(decl.name.clone(), value.clone());
        }

        Ok(RuntimeValue::Null)
    }

    /// Execute function declaration
    fn execute_function_decl(&mut self, decl: &FunctionDecl) -> Result<RuntimeValue> {
        // For now, just store function as a placeholder
        let function_value = RuntimeValue::String(format!("function:{}", decl.name));
        
        self.environment.define(decl.name.clone(), function_value.clone());

        // If exported, also add to globals
        if decl.is_exported {
            self.globals.define(decl.name.clone(), function_value);
        }

        Ok(RuntimeValue::Null)
    }

    /// Execute struct declaration
    fn execute_struct_decl(&mut self, decl: &StructDecl) -> Result<RuntimeValue> {
        // Store the complete struct definition for later use
        self.struct_definitions.insert(decl.name.clone(), decl.clone());
        
        // Store struct as a type identifier in the environment
        let struct_value = RuntimeValue::String(format!("struct:{}", decl.name));
        self.environment.define(decl.name.clone(), struct_value.clone());

        // If exported, also add to globals
        if decl.is_exported {
            self.globals.define(decl.name.clone(), struct_value);
        }

        Ok(RuntimeValue::Null)
    }

    /// Execute interface declaration
    fn execute_interface_decl(&mut self, decl: &InterfaceDecl) -> Result<RuntimeValue> {
        // For now, just store interface as a placeholder
        let interface_value = RuntimeValue::String(format!("interface:{}", decl.name));
        
        self.environment.define(decl.name.clone(), interface_value.clone());

        // If exported, also add to globals
        if decl.is_exported {
            self.globals.define(decl.name.clone(), interface_value);
        }

        Ok(RuntimeValue::Null)
    }

    /// Execute type alias declaration
    fn execute_type_alias_decl(&mut self, decl: &TypeAliasDecl) -> Result<RuntimeValue> {
        // For now, just store type alias as a placeholder
        let type_value = RuntimeValue::String(format!("type:{}", decl.name));
        
        self.environment.define(decl.name.clone(), type_value);

        Ok(RuntimeValue::Null)
    }

    /// Execute import statement
    fn execute_import_stmt(&mut self, stmt: &ImportStmt) -> Result<RuntimeValue> {
        // Set the current directory for the module resolver
        if let Some(current_file) = &self.current_file {
            if let Some(parent) = std::path::Path::new(current_file).parent() {
                self.module_resolver.set_current_dir(parent.to_path_buf());
            }
        }

        let imported_symbols = self.module_resolver.resolve_import(stmt)?;

        // Add imported symbols to current environment
        for (name, value) in imported_symbols {
            self.environment.define(name, value);
        }

        Ok(RuntimeValue::Null)
    }

    /// Execute export statement
    fn execute_export_stmt(&mut self, stmt: &ExportStmt) -> Result<RuntimeValue> {
        // Execute the exported item
        let result = self.execute_statement(&stmt.item)?;

        // Handle re-exports
        if let Statement::Import(import_stmt) = stmt.item.as_ref() {
            // This is a re-export: export { items } from "path"
            let imported_symbols = self.module_resolver.resolve_import(import_stmt)?;
            
            // Add re-exported symbols to globals
            for (name, value) in imported_symbols {
                self.globals.define(name, value);
            }
        }

        Ok(result)
    }

    /// Execute expression statement
    fn execute_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Result<RuntimeValue> {
        self.execute_expression(&stmt.expr)
    }

    /// Execute block statement
    fn execute_block_stmt(&mut self, stmt: &BlockStmt) -> Result<RuntimeValue> {
        // Create new scope
        let parent_env = self.environment.clone();
        self.environment = Environment::with_parent(parent_env.clone());
        
        let mut last_value = RuntimeValue::Null;
        for statement in &stmt.statements {
            last_value = self.execute_statement(statement)?;
        }

        // Restore previous scope
        self.environment = parent_env;
        
        Ok(last_value)
    }

    /// Execute expression (stub implementations for now)
    fn execute_expression(&mut self, expr: &Expression) -> Result<RuntimeValue> {
        match expr {
            Expression::Literal(lit) => self.execute_literal_expr(lit),
            Expression::Identifier(id) => self.execute_identifier_expr(id),
            Expression::Binary(bin) => self.execute_binary_expr(bin),
            Expression::Unary(un) => self.execute_unary_expr(un),
            Expression::Call(call) => self.execute_call_expr(call),
            Expression::MemberAccess(member) => self.execute_member_access_expr(member),
            Expression::Index(index) => self.execute_index_expr(index),
            Expression::Assignment(assign) => self.execute_assignment_expr(assign),
            Expression::If(if_expr) => self.execute_if_expr(if_expr),
            Expression::Match(match_expr) => self.execute_match_expr(match_expr),
            Expression::Array(array) => self.execute_array_expr(array),
            Expression::Map(map) => self.execute_map_expr(map),
            Expression::Lambda(lambda) => self.execute_lambda_expr(lambda),
            Expression::Async(async_expr) => self.execute_async_expr(async_expr),
            Expression::Await(await_expr) => self.execute_await_expr(await_expr),
            Expression::Run(run) => self.execute_run_expr(run),
            Expression::Channel(channel) => self.execute_channel_expr(channel),
            Expression::Select(select) => self.execute_select_expr(select),
            Expression::Cast(cast) => self.execute_cast_expr(cast),
            Expression::TypeOf(typeof_expr) => self.execute_typeof_expr(typeof_expr),
            Expression::Range(range) => self.execute_range_expr(range),
            Expression::Yield(yield_expr) => self.execute_yield_expr(yield_expr),
            Expression::Parenthesized(paren) => self.execute_expression(&paren.expr),
            Expression::Block(block) => self.execute_block_expr(block),
            Expression::Tuple(tuple) => self.execute_tuple_expr(tuple),
            Expression::StructLiteral(struct_lit) => self.execute_struct_literal_expr(struct_lit),
        }
    }

    /// Execute literal expression
    fn execute_literal_expr(&mut self, expr: &LiteralExpr) -> Result<RuntimeValue> {
        match &expr.value {
            LiteralValue::Integer(i) => Ok(RuntimeValue::Integer(*i)),
            LiteralValue::Float(f) => Ok(RuntimeValue::Float64(*f)),
            LiteralValue::String(s) => Ok(RuntimeValue::String(s.clone())),
            LiteralValue::Char(c) => Ok(RuntimeValue::Char(*c)),
            LiteralValue::Boolean(b) => Ok(RuntimeValue::Bool(*b)),
            LiteralValue::Null => Ok(RuntimeValue::Null),
        }
    }

    /// Execute identifier expression
    fn execute_identifier_expr(&mut self, expr: &IdentifierExpr) -> Result<RuntimeValue> {
        if let Some(value) = self.environment.get(&expr.name) {
            Ok(value.clone())
        } else {
            Err(BuluError::RuntimeError {
                message: format!("Undefined variable '{}'", expr.name),
                file: self.current_file.clone(),
            })
        }
    }

    // Stub implementations for other expressions
    fn execute_binary_expr(&mut self, expr: &BinaryExpr) -> Result<RuntimeValue> {
        let left = self.execute_expression(&expr.left)?;
        let right = self.execute_expression(&expr.right)?;
        
        match expr.operator {
            BinaryOperator::Add => {
                match (left, right) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => Ok(RuntimeValue::Integer(a + b)),
                    (RuntimeValue::Float64(a), RuntimeValue::Float64(b)) => Ok(RuntimeValue::Float64(a + b)),
                    (RuntimeValue::String(a), RuntimeValue::String(b)) => Ok(RuntimeValue::String(a + &b)),
                    (RuntimeValue::String(a), RuntimeValue::Integer(b)) => Ok(RuntimeValue::String(a + &b.to_string())),
                    (RuntimeValue::Integer(a), RuntimeValue::String(b)) => Ok(RuntimeValue::String(a.to_string() + &b)),
                    _ => Ok(RuntimeValue::Null),
                }
            }
            BinaryOperator::Subtract => {
                match (left, right) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => Ok(RuntimeValue::Integer(a - b)),
                    (RuntimeValue::Float64(a), RuntimeValue::Float64(b)) => Ok(RuntimeValue::Float64(a - b)),
                    _ => Ok(RuntimeValue::Null),
                }
            }
            BinaryOperator::Multiply => {
                match (left, right) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => Ok(RuntimeValue::Integer(a * b)),
                    (RuntimeValue::Float64(a), RuntimeValue::Float64(b)) => Ok(RuntimeValue::Float64(a * b)),
                    _ => Ok(RuntimeValue::Null),
                }
            }
            BinaryOperator::Divide => {
                match (left, right) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                        if b != 0 {
                            Ok(RuntimeValue::Integer(a / b))
                        } else {
                            Err(BuluError::RuntimeError {
                                message: "Division by zero".to_string(),
                                file: self.current_file.clone(),
                            })
                        }
                    }
                    (RuntimeValue::Float64(a), RuntimeValue::Float64(b)) => {
                        if b != 0.0 {
                            Ok(RuntimeValue::Float64(a / b))
                        } else {
                            Err(BuluError::RuntimeError {
                                message: "Division by zero".to_string(),
                                file: self.current_file.clone(),
                            })
                        }
                    }
                    _ => Ok(RuntimeValue::Null),
                }
            }
            BinaryOperator::Equal => {
                let result = match (left, right) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a == b,
                    (RuntimeValue::Float64(a), RuntimeValue::Float64(b)) => a == b,
                    (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
                    (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
                    (RuntimeValue::Null, RuntimeValue::Null) => true,
                    _ => false,
                };
                Ok(RuntimeValue::Bool(result))
            }
            BinaryOperator::NotEqual => {
                let result = match (left, right) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a != b,
                    (RuntimeValue::Float64(a), RuntimeValue::Float64(b)) => a != b,
                    (RuntimeValue::String(a), RuntimeValue::String(b)) => a != b,
                    (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a != b,
                    (RuntimeValue::Null, RuntimeValue::Null) => false,
                    _ => true,
                };
                Ok(RuntimeValue::Bool(result))
            }
            _ => {
                // Other operators not implemented yet
                Ok(RuntimeValue::Null)
            }
        }
    }

    fn execute_unary_expr(&mut self, _expr: &UnaryExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_call_expr(&mut self, expr: &CallExpr) -> Result<RuntimeValue> {
        // Check if this is a built-in function call
        if let Expression::Identifier(ident) = expr.callee.as_ref() {
            match ident.name.as_str() {
                "make" => return self.execute_make_call(expr),
                "println" => return self.execute_println_call(expr),
                "print" => return self.execute_print_call(expr),
                "len" => return self.execute_len_call(expr),
                "append" => return self.execute_append_call(expr),
                "close" => return self.execute_close_call(expr),
                _ => {}
            }
        }
        
        // Get the function to call
        let function = self.execute_expression(&expr.callee)?;
        
        // Evaluate arguments
        let mut args = Vec::new();
        for arg in &expr.args {
            args.push(self.execute_expression(arg)?);
        }
        
        // Handle different types of function calls
        match function {
            RuntimeValue::String(func_name) => {
                if func_name.starts_with("function:") {
                    let name = func_name.strip_prefix("function:").unwrap();
                    // For now, return a placeholder result
                    Ok(RuntimeValue::String(format!("result_of_{}", name)))
                } else if func_name.starts_with("struct:") {
                    let name = func_name.strip_prefix("struct:").unwrap();
                    // Constructor call - return a placeholder object
                    Ok(RuntimeValue::String(format!("instance_of_{}", name)))
                } else {
                    // Regular function call
                    Ok(RuntimeValue::String(format!("called_{}", func_name)))
                }
            }
            _ => {
                // Unknown function type
                Ok(RuntimeValue::Null)
            }
        }
    }

    fn execute_member_access_expr(&mut self, expr: &MemberAccessExpr) -> Result<RuntimeValue> {
        let object = self.execute_expression(&expr.object)?;
        
        match object {
            RuntimeValue::String(obj_name) => {
                if obj_name.starts_with("instance_of_") {
                    // Method call on an instance
                    Ok(RuntimeValue::String(format!("method_{}_{}", obj_name, expr.member)))
                } else if obj_name.starts_with("struct:") {
                    // Static method call on a struct
                    let struct_name = obj_name.strip_prefix("struct:").unwrap();
                    Ok(RuntimeValue::String(format!("function:{}_{}", struct_name, expr.member)))
                } else {
                    // Regular member access
                    Ok(RuntimeValue::String(format!("{}_{}", obj_name, expr.member)))
                }
            }
            RuntimeValue::Map(map) => {
                // Access member from a map (module object)
                if let Some(value) = map.get(&expr.member) {
                    Ok(value.clone())
                } else {
                    Err(BuluError::RuntimeError {
                        message: format!("Member '{}' not found", expr.member),
                        file: self.current_file.clone(),
                    })
                }
            }
            _ => {
                // Unknown object type
                Ok(RuntimeValue::Null)
            }
        }
    }

    fn execute_index_expr(&mut self, _expr: &IndexExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_assignment_expr(&mut self, _expr: &AssignmentExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_if_expr(&mut self, _expr: &IfExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_match_expr(&mut self, _expr: &MatchExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_array_expr(&mut self, _expr: &ArrayExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_map_expr(&mut self, _expr: &MapExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_lambda_expr(&mut self, _expr: &LambdaExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_async_expr(&mut self, _expr: &AsyncExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_await_expr(&mut self, _expr: &AwaitExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_run_expr(&mut self, _expr: &RunExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_channel_expr(&mut self, _expr: &ChannelExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_select_expr(&mut self, _expr: &SelectExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_cast_expr(&mut self, _expr: &CastExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_typeof_expr(&mut self, _expr: &TypeOfExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_range_expr(&mut self, expr: &RangeExpr) -> Result<RuntimeValue> {
        let start_val = self.execute_expression(&expr.start)?;
        let end_val = self.execute_expression(&expr.end)?;
        
        // Convert to integers for range creation
        let start = match start_val {
            RuntimeValue::Int32(i) => i as i64,
            RuntimeValue::Int64(i) => i,
            RuntimeValue::Float32(f) => f as i64,
            RuntimeValue::Float64(f) => f as i64,
            _ => return Err(BuluError::RuntimeError {
                message: "Range start must be a number".to_string(),
                file: self.current_file.clone(),
            }),
        };
        
        let end = match end_val {
            RuntimeValue::Int32(i) => i as i64,
            RuntimeValue::Int64(i) => i,
            RuntimeValue::Float32(f) => f as i64,
            RuntimeValue::Float64(f) => f as i64,
            _ => return Err(BuluError::RuntimeError {
                message: "Range end must be a number".to_string(),
                file: self.current_file.clone(),
            }),
        };
        
        // Create array from range
        let mut values = Vec::new();
        
        if expr.inclusive {
            // Inclusive range: 1...5 includes 5
            for i in start..=end {
                values.push(RuntimeValue::Int64(i));
            }
        } else {
            // Exclusive range: 1..<5 or 1..5 excludes 5
            for i in start..end {
                values.push(RuntimeValue::Int64(i));
            }
        }
        
        Ok(RuntimeValue::Array(values))
    }

    fn execute_yield_expr(&mut self, _expr: &YieldExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_block_expr(&mut self, _expr: &BlockExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_tuple_expr(&mut self, _expr: &TupleExpr) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_struct_literal_expr(&mut self, expr: &StructLiteralExpr) -> Result<RuntimeValue> {
        // Debug: print available struct definitions
        println!("Available struct definitions: {:?}", self.struct_definitions.keys().collect::<Vec<_>>());
        println!("Looking for struct: {}", expr.type_name);
        
        // Get the struct definition
        let struct_def = self.struct_definitions.get(&expr.type_name)
            .ok_or_else(|| BuluError::RuntimeError {
                message: format!("Unknown struct type '{}'", expr.type_name),
                file: None,
            })?;

        let mut fields = HashMap::new();

        // First, set default values for all fields
        for field in &struct_def.fields {
            let default_value = self.get_default_value_for_type(&field.field_type);
            fields.insert(field.name.clone(), default_value);
        }

        // Then, override with provided values
        for field_init in &expr.fields {
            let field_value = self.execute_expression(&field_init.value)?;
            fields.insert(field_init.name.clone(), field_value);
        }

        Ok(RuntimeValue::Struct {
            name: expr.type_name.clone(),
            fields,
        })
    }

    /// Get default value for a given type
    fn get_default_value_for_type(&self, field_type: &Type) -> RuntimeValue {
        match field_type {
            Type::Int8 => RuntimeValue::Int8(0),
            Type::Int16 => RuntimeValue::Int16(0),
            Type::Int32 => RuntimeValue::Int32(0),
            Type::Int64 => RuntimeValue::Int64(0),
            Type::UInt8 => RuntimeValue::UInt8(0),
            Type::UInt16 => RuntimeValue::UInt16(0),
            Type::UInt32 => RuntimeValue::UInt32(0),
            Type::UInt64 => RuntimeValue::UInt64(0),
            Type::Float32 => RuntimeValue::Float32(0.0),
            Type::Float64 => RuntimeValue::Float64(0.0),
            Type::Bool => RuntimeValue::Bool(false),
            Type::Char => RuntimeValue::Char('\0'),
            Type::String => RuntimeValue::String(String::new()),
            Type::Any => RuntimeValue::Null,
            Type::Void => RuntimeValue::Null,
            Type::Array(_) => RuntimeValue::Array(Vec::new()),
            Type::Slice(_) => RuntimeValue::Slice(Vec::new()),
            Type::Map(_) => RuntimeValue::Map(HashMap::new()),
            _ => RuntimeValue::Null, // For complex types, default to null
        }
    }

    // Stub implementations for other statements
    fn execute_if_stmt(&mut self, _stmt: &IfStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_while_stmt(&mut self, stmt: &WhileStmt) -> Result<RuntimeValue> {
        loop {
            // Evaluate the condition
            let condition_value = self.execute_expression(&stmt.condition)?;
            
            // Check if condition is truthy
            let should_continue = match condition_value {
                RuntimeValue::Bool(b) => b,
                RuntimeValue::Null => false,
                RuntimeValue::Int32(i) => i != 0,
                RuntimeValue::Int64(i) => i != 0,
                RuntimeValue::Float32(f) => f != 0.0,
                RuntimeValue::Float64(f) => f != 0.0,
                RuntimeValue::String(s) => !s.is_empty(),
                _ => true, // Other values are considered truthy
            };
            
            if !should_continue {
                break;
            }
            
            // Execute the body
            match self.execute_block_stmt(&stmt.body) {
                Ok(_) => continue,
                Err(BuluError::Break) => break,
                Err(BuluError::Continue) => continue,
                Err(e) => return Err(e),
            }
        }
        
        Ok(RuntimeValue::Null)
    }

    fn execute_for_stmt(&mut self, stmt: &ForStmt) -> Result<RuntimeValue> {
        // Evaluate the iterable expression
        let iterable_value = self.execute_expression(&stmt.iterable)?;
        
        match iterable_value {
            RuntimeValue::Array(ref values) => {
                if let Some(ref index_var) = stmt.index_variable {
                    // For loop with index and value: for i, val in array
                    for (index, value) in values.iter().enumerate() {
                        // Create new scope for each iteration
                        let parent_env = self.environment.clone();
                        self.environment = Environment::with_parent(parent_env.clone());
                        
                        // Set the index variable
                        self.environment.define(index_var.clone(), RuntimeValue::Int32(index as i32));
                        // Set the value variable
                        self.environment.define(stmt.variable.clone(), value.clone());
                        
                        // Execute the body
                        let result = self.execute_block_stmt(&stmt.body);
                        
                        // Restore environment
                        self.environment = parent_env;
                        
                        match result {
                            Ok(_) => continue,
                            Err(BuluError::Break) => break,
                            Err(BuluError::Continue) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    // For loop with just value: for val in array
                    for value in values {
                        // Create new scope for each iteration
                        let parent_env = self.environment.clone();
                        self.environment = Environment::with_parent(parent_env.clone());
                        
                        // Set the loop variable
                        self.environment.define(stmt.variable.clone(), value.clone());
                        
                        // Execute the body
                        let result = self.execute_block_stmt(&stmt.body);
                        
                        // Restore environment
                        self.environment = parent_env;
                        
                        match result {
                            Ok(_) => continue,
                            Err(BuluError::Break) => break,
                            Err(BuluError::Continue) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(RuntimeValue::Null)
            }
            RuntimeValue::String(ref s) => {
                // Iterate over characters in string
                if let Some(ref index_var) = stmt.index_variable {
                    // For loop with index and character: for i, char in string
                    for (index, ch) in s.chars().enumerate() {
                        // Create new scope for each iteration
                        let parent_env = self.environment.clone();
                        self.environment = Environment::with_parent(parent_env.clone());
                        
                        // Set the index variable
                        self.environment.define(index_var.clone(), RuntimeValue::Int32(index as i32));
                        // Set the character variable
                        self.environment.define(stmt.variable.clone(), RuntimeValue::String(ch.to_string()));
                        
                        // Execute the body
                        let result = self.execute_block_stmt(&stmt.body);
                        
                        // Restore environment
                        self.environment = parent_env;
                        
                        match result {
                            Ok(_) => continue,
                            Err(BuluError::Break) => break,
                            Err(BuluError::Continue) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    // For loop with just character: for char in string
                    for ch in s.chars() {
                        // Create new scope for each iteration
                        let parent_env = self.environment.clone();
                        self.environment = Environment::with_parent(parent_env.clone());
                        
                        // Set the loop variable
                        self.environment.define(stmt.variable.clone(), RuntimeValue::String(ch.to_string()));
                        
                        // Execute the body
                        let result = self.execute_block_stmt(&stmt.body);
                        
                        // Restore environment
                        self.environment = parent_env;
                        
                        match result {
                            Ok(_) => continue,
                            Err(BuluError::Break) => break,
                            Err(BuluError::Continue) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(RuntimeValue::Null)
            }
            _ => {
                Err(BuluError::RuntimeError {
                    message: format!("Cannot iterate over value of type: {:?}", iterable_value),
                    file: self.current_file.clone(),
                })
            }
        }
    }

    fn execute_match_stmt(&mut self, _stmt: &MatchStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_select_stmt(&mut self, _stmt: &SelectStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_return_stmt(&mut self, _stmt: &ReturnStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_break_stmt(&mut self, _stmt: &BreakStmt) -> Result<RuntimeValue> {
        Err(BuluError::Break)
    }

    fn execute_continue_stmt(&mut self, _stmt: &ContinueStmt) -> Result<RuntimeValue> {
        Err(BuluError::Continue)
    }

    fn execute_defer_stmt(&mut self, _stmt: &DeferStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_try_stmt(&mut self, _stmt: &TryStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    fn execute_fail_stmt(&mut self, _stmt: &FailStmt) -> Result<RuntimeValue> {
        Ok(RuntimeValue::Null)
    }

    /// Get the current environment (for testing)
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    // Built-in function implementations
    
    fn execute_make_call(&mut self, expr: &CallExpr) -> Result<RuntimeValue> {
        if expr.args.is_empty() {
            return Err(BuluError::RuntimeError {
                message: "make() requires at least one argument".to_string(),
                file: self.current_file.clone(),
            });
        }

        // Check the first argument to determine what to create
        match &expr.args[0] {
            Expression::Identifier(ident) => {
                // Handle channel type identifiers like "chan_int32", "chan_string", etc.
                if ident.name.starts_with("chan_") {
                    let capacity = if expr.args.len() > 1 {
                        let cap_val = self.execute_expression(&expr.args[1])?;
                        match cap_val {
                            RuntimeValue::Int32(cap) => Some(cap as usize),
                            RuntimeValue::Int64(cap) => Some(cap as usize),
                            _ => return Err(BuluError::RuntimeError {
                                message: "Channel capacity must be an integer".to_string(),
                                file: self.current_file.clone(),
                            }),
                        }
                    } else {
                        None
                    };
                    return self.create_channel(capacity);
                }
                // Handle legacy "chan" identifier
                else if ident.name == "chan" {
                    // make(chan) - unbuffered channel with any type
                    let capacity = if expr.args.len() > 1 {
                        let cap_val = self.execute_expression(&expr.args[1])?;
                        match cap_val {
                            RuntimeValue::Int32(cap) => Some(cap as usize),
                            RuntimeValue::Int64(cap) => Some(cap as usize),
                            _ => return Err(BuluError::RuntimeError {
                                message: "Channel capacity must be an integer".to_string(),
                                file: self.current_file.clone(),
                            }),
                        }
                    } else {
                        None
                    };
                    
                    self.create_channel(capacity)
                }
                // Handle primitive types
                else {
                    match ident.name.as_str() {
                        // Integer types - return zero value
                        "int8" => Ok(RuntimeValue::Int32(0)),
                        "int16" => Ok(RuntimeValue::Int32(0)),
                        "int32" => Ok(RuntimeValue::Int32(0)),
                        "int64" => Ok(RuntimeValue::Int64(0)),
                        "uint8" => Ok(RuntimeValue::Int32(0)),
                        "uint16" => Ok(RuntimeValue::Int32(0)),
                        "uint32" => Ok(RuntimeValue::Int32(0)),
                        "uint64" => Ok(RuntimeValue::Int64(0)),
                        
                        // Float types - return zero value
                        "float32" => Ok(RuntimeValue::Float64(0.0)),
                        "float64" => Ok(RuntimeValue::Float64(0.0)),
                        
                        // Boolean type - return false
                        "bool" => Ok(RuntimeValue::Bool(false)),
                        
                        // String type - return empty string
                        "string" => Ok(RuntimeValue::String(String::new())),
                        
                        // Character types
                        "char" => Ok(RuntimeValue::String("\0".to_string())),
                        "byte" => Ok(RuntimeValue::Int32(0)),
                        "rune" => Ok(RuntimeValue::Int32(0)),
                        
                        // Any type - return null
                        "any" => Ok(RuntimeValue::Null),
                        
                        // Slice types
                        name if name.starts_with("[]") => {
                            // Extract element type from slice notation
                            let element_type = &name[2..];
                            
                            // Get size if provided
                            let size = if expr.args.len() > 1 {
                                let size_val = self.execute_expression(&expr.args[1])?;
                                match size_val {
                                    RuntimeValue::Int32(s) => s as usize,
                                    RuntimeValue::Int64(s) => s as usize,
                                    _ => return Err(BuluError::RuntimeError {
                                        message: "Slice size must be an integer".to_string(),
                                        file: self.current_file.clone(),
                                    }),
                                }
                            } else {
                                0
                            };
                            
                            // Create slice with zero values
                            let zero_value = self.get_zero_value_for_type(element_type)?;
                            let mut elements = Vec::new();
                            for _ in 0..size {
                                elements.push(zero_value.clone());
                            }
                            
                            Ok(RuntimeValue::Array(elements))
                        }
                        
                        _ => Err(BuluError::RuntimeError {
                            message: format!("Unknown make() type: {}", ident.name),
                            file: self.current_file.clone(),
                        })
                    }
                }
            }
            Expression::Call(call_expr) => {
                // make(chan T) or make(chan T, capacity)
                if let Expression::Identifier(ident) = call_expr.callee.as_ref() {
                    if ident.name == "chan" {
                        // Extract type from chan(T) call
                        let _element_type = if !call_expr.args.is_empty() {
                            // For now, ignore the type and create a generic channel
                            &call_expr.args[0]
                        } else {
                            // Default to any type
                            return self.create_channel(None);
                        };
                        
                        let capacity = if expr.args.len() > 1 {
                            let cap_val = self.execute_expression(&expr.args[1])?;
                            match cap_val {
                                RuntimeValue::Int32(cap) => Some(cap as usize),
                                RuntimeValue::Int64(cap) => Some(cap as usize),
                                _ => return Err(BuluError::RuntimeError {
                                    message: "Channel capacity must be an integer".to_string(),
                                    file: self.current_file.clone(),
                                }),
                            }
                        } else {
                            None
                        };
                        
                        self.create_channel(capacity)
                    } else {
                        Err(BuluError::RuntimeError {
                            message: format!("Unknown make() type: {}", ident.name),
                            file: self.current_file.clone(),
                        })
                    }
                } else {
                    Err(BuluError::RuntimeError {
                        message: "Invalid make() syntax".to_string(),
                        file: self.current_file.clone(),
                    })
                }
            }
            _ => {
                Err(BuluError::RuntimeError {
                    message: "Invalid make() syntax".to_string(),
                    file: self.current_file.clone(),
                })
            }
        }
    }

    fn create_channel(&mut self, capacity: Option<usize>) -> Result<RuntimeValue> {
        // Generate a unique channel ID
        static CHANNEL_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let channel_id = CHANNEL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // For now, just return the channel ID
        // In a full implementation, this would create an actual channel and store it in a registry
        Ok(RuntimeValue::Channel(channel_id))
    }

    fn get_zero_value_for_type(&self, type_name: &str) -> Result<RuntimeValue> {
        match type_name {
            "int8" | "int16" | "int32" | "uint8" | "uint16" | "uint32" | "byte" | "rune" => {
                Ok(RuntimeValue::Int32(0))
            }
            "int64" | "uint64" => Ok(RuntimeValue::Int64(0)),
            "float32" | "float64" => Ok(RuntimeValue::Float64(0.0)),
            "bool" => Ok(RuntimeValue::Bool(false)),
            "string" | "char" => Ok(RuntimeValue::String(String::new())),
            "any" => Ok(RuntimeValue::Null),
            _ => Ok(RuntimeValue::Null), // Default for unknown types
        }
    }

    fn execute_println_call(&mut self, expr: &CallExpr) -> Result<RuntimeValue> {
        let mut output = String::new();
        for (i, arg) in expr.args.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            let value = self.execute_expression(arg)?;
            output.push_str(&self.value_to_string(&value));
        }
        println!("{}", output);
        Ok(RuntimeValue::Null)
    }

    fn execute_print_call(&mut self, expr: &CallExpr) -> Result<RuntimeValue> {
        let mut output = String::new();
        for (i, arg) in expr.args.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            let value = self.execute_expression(arg)?;
            output.push_str(&self.value_to_string(&value));
        }
        print!("{}", output);
        Ok(RuntimeValue::Null)
    }

    fn execute_len_call(&mut self, expr: &CallExpr) -> Result<RuntimeValue> {
        if expr.args.len() != 1 {
            return Err(BuluError::RuntimeError {
                message: "len() requires exactly one argument".to_string(),
                file: self.current_file.clone(),
            });
        }

        let value = self.execute_expression(&expr.args[0])?;
        match value {
            RuntimeValue::String(s) => Ok(RuntimeValue::Int32(s.len() as i32)),
            RuntimeValue::Array(arr) => Ok(RuntimeValue::Int32(arr.len() as i32)),
            _ => Err(BuluError::RuntimeError {
                message: "len() can only be called on strings and arrays".to_string(),
                file: self.current_file.clone(),
            }),
        }
    }

    fn execute_append_call(&mut self, _expr: &CallExpr) -> Result<RuntimeValue> {
        // TODO: Implement append
        Ok(RuntimeValue::Null)
    }

    fn execute_close_call(&mut self, _expr: &CallExpr) -> Result<RuntimeValue> {
        // TODO: Implement close
        Ok(RuntimeValue::Null)
    }

    fn value_to_string(&self, value: &RuntimeValue) -> String {
        match value {
            RuntimeValue::Int32(i) => i.to_string(),
            RuntimeValue::Int64(i) => i.to_string(),
            RuntimeValue::Float32(f) => f.to_string(),
            RuntimeValue::Float64(f) => f.to_string(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Char(c) => c.to_string(),
            RuntimeValue::Null => "null".to_string(),
            RuntimeValue::Channel(id) => format!("channel({})", id),
            RuntimeValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", elements.join(", "))
            }
            RuntimeValue::Map(map) => {
                let entries: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
            _ => format!("{:?}", value),
        }
    }

    /// Get the global environment (for testing)
    pub fn globals(&self) -> &Environment {
        &self.globals
    }

    /// Check if a symbol is accessible (not exported means not accessible from outside)
    pub fn is_symbol_accessible(&self, symbol: &str) -> bool {
        self.globals.contains(symbol)
    }
}

impl Default for AstInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Position;

    #[test]
    fn test_variable_declaration() {
        let mut interpreter = AstInterpreter::new();
        
        let decl = VariableDecl {
            is_const: false,
            name: "x".to_string(),
            type_annotation: None,
            initializer: Some(Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(42),
                position: Position::new(1, 1, 0),
            })),
            doc_comment: None,
            is_exported: false,
            position: Position::new(1, 1, 0),
        };

        let result = interpreter.execute_variable_decl(&decl).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        let value = interpreter.environment.get("x").unwrap();
        assert_eq!(*value, RuntimeValue::Integer(42));
    }

    #[test]
    fn test_exported_variable() {
        let mut interpreter = AstInterpreter::new();
        
        let decl = VariableDecl {
            is_const: false,
            name: "exported_var".to_string(),
            type_annotation: None,
            initializer: Some(Expression::Literal(LiteralExpr {
                value: LiteralValue::String("hello".to_string()),
                position: Position::new(1, 1, 0),
            })),
            doc_comment: None,
            is_exported: true,
            position: Position::new(1, 1, 0),
        };

        interpreter.execute_variable_decl(&decl).unwrap();
        
        // Should be in both local and global environments
        assert!(interpreter.environment.contains("exported_var"));
        assert!(interpreter.globals.contains("exported_var"));
        assert!(interpreter.is_symbol_accessible("exported_var"));
    }

    #[test]
    fn test_non_exported_variable() {
        let mut interpreter = AstInterpreter::new();
        
        let decl = VariableDecl {
            is_const: false,
            name: "private_var".to_string(),
            type_annotation: None,
            initializer: Some(Expression::Literal(LiteralExpr {
                value: LiteralValue::String("private".to_string()),
                position: Position::new(1, 1, 0),
            })),
            doc_comment: None,
            is_exported: false,
            position: Position::new(1, 1, 0),
        };

        interpreter.execute_variable_decl(&decl).unwrap();
        
        // Should only be in local environment
        assert!(interpreter.environment.contains("private_var"));
        assert!(!interpreter.globals.contains("private_var"));
        assert!(!interpreter.is_symbol_accessible("private_var"));
    }

    #[test]
    fn test_import_std_module() {
        let mut interpreter = AstInterpreter::new();
        
        let import = ImportStmt {
            path: "std.io".to_string(),
            alias: Some("io".to_string()),
            items: None,
            position: Position::new(1, 1, 0),
        };

        let result = interpreter.execute_import_stmt(&import).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Should have imported the module as 'io'
        assert!(interpreter.environment.contains("io"));
    }
}