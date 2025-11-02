//! Type checking implementation for the Bulu language

use crate::ast::*;
use crate::error::{BuluError, Result};
use crate::lexer::token::Position;
use crate::types::composite::TypeRegistry;
use crate::types::primitive::{PrimitiveType, TypeId};
use std::collections::HashMap;

/// Symbol table entry for type checking
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub type_id: TypeId,
    pub is_mutable: bool,
    pub position: Position,
    pub function_info: Option<FunctionInfo>,
}

/// Function signature information
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub param_types: Vec<TypeId>,
    pub return_type: Option<TypeId>,
}

/// Type checking context
#[derive(Debug)]
pub struct TypeChecker {
    /// Symbol table stack for nested scopes
    pub scopes: Vec<HashMap<String, Symbol>>,
    /// Function return type stack
    return_types: Vec<Option<TypeId>>,
    /// Current function being checked
    current_function: Option<String>,
    /// Error accumulator
    errors: Vec<BuluError>,
    /// Type registry for composite types
    type_registry: TypeRegistry,
    /// Interface declarations
    interfaces: HashMap<String, InterfaceDecl>,
    /// Struct declarations
    structs: HashMap<String, StructDecl>,
    /// Map from type names to TypeIds
    type_name_to_id: HashMap<String, TypeId>,
    /// Map from TypeIds to type names
    type_id_to_name: HashMap<TypeId, String>,
    /// Next available type ID
    next_type_id: u32,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        let mut checker = Self {
            scopes: vec![HashMap::new()], // Global scope
            return_types: Vec::new(),
            current_function: None,
            errors: Vec::new(),
            type_registry: TypeRegistry::new(),
            interfaces: HashMap::new(),
            structs: HashMap::new(),
            type_name_to_id: HashMap::new(),
            type_id_to_name: HashMap::new(),
            next_type_id: 1000, // Start from 1000 to avoid conflicts with primitive types
        };

        // Add built-in functions to global scope
        checker.add_builtin_functions();
        checker
    }

    /// Add built-in functions to the global scope (public method for re-adding after imports)
    pub fn add_builtin_functions_after_import(&mut self) {
        self.add_builtin_functions();
    }

    /// Add built-in functions to the global scope
    fn add_builtin_functions(&mut self) {
        let builtin_functions = vec![
            // I/O functions
            ("print", vec![], None),
            ("println", vec![], None),
            ("printf", vec![TypeId::String], None),
            ("input", vec![], Some(TypeId::String)),
            // Type conversion functions
            ("int8", vec![TypeId::Any], Some(TypeId::Int8)),
            ("int16", vec![TypeId::Any], Some(TypeId::Int16)),
            ("int32", vec![TypeId::Any], Some(TypeId::Int32)),
            ("int64", vec![TypeId::Any], Some(TypeId::Int64)),
            ("uint8", vec![TypeId::Any], Some(TypeId::UInt8)),
            ("uint16", vec![TypeId::Any], Some(TypeId::UInt16)),
            ("uint32", vec![TypeId::Any], Some(TypeId::UInt32)),
            ("uint64", vec![TypeId::Any], Some(TypeId::UInt64)),
            ("float32", vec![TypeId::Any], Some(TypeId::Float32)),
            ("float64", vec![TypeId::Any], Some(TypeId::Float64)),
            ("bool", vec![TypeId::Any], Some(TypeId::Bool)),
            ("char", vec![TypeId::Any], Some(TypeId::Char)),
            ("string", vec![TypeId::Any], Some(TypeId::String)),
            // Memory functions
            ("len", vec![TypeId::Any], Some(TypeId::Int32)),
            ("cap", vec![TypeId::Any], Some(TypeId::Int32)),
            ("clone", vec![TypeId::Any], Some(TypeId::Any)),
            ("sizeof", vec![TypeId::Any], Some(TypeId::Int32)),
            // Collection functions
            ("make", vec![TypeId::Any], Some(TypeId::Any)),
            ("append", vec![TypeId::Any, TypeId::Any], Some(TypeId::Any)),
            ("copy", vec![TypeId::Any, TypeId::Any], Some(TypeId::Int32)),
            ("delete", vec![TypeId::Any, TypeId::Any], None),
            // Utility functions
            ("typeof", vec![TypeId::Any], Some(TypeId::String)),
            (
                "instanceof",
                vec![TypeId::Any, TypeId::String],
                Some(TypeId::Bool),
            ),
            ("panic", vec![TypeId::Any], None),
            ("assert", vec![TypeId::Bool], None),
            ("recover", vec![], Some(TypeId::Any)),
            // Channel functions
            ("close", vec![TypeId::Any], None),
            // Synchronization functions
            ("lock", vec![], Some(TypeId::Any)),
            ("sleep", vec![TypeId::Int32], None),
            ("yield", vec![], None),
            ("timer", vec![TypeId::Int32], Some(TypeId::Any)),
            ("atomic_load", vec![TypeId::Any], Some(TypeId::Any)),
            ("atomic_store", vec![TypeId::Any, TypeId::Any], None),
            (
                "atomic_add",
                vec![TypeId::Any, TypeId::Any],
                Some(TypeId::Any),
            ),
            (
                "atomic_sub",
                vec![TypeId::Any, TypeId::Any],
                Some(TypeId::Any),
            ),
            (
                "atomic_cas",
                vec![TypeId::Any, TypeId::Any, TypeId::Any],
                Some(TypeId::Bool),
            ),
        ];

        // Add primitive type identifiers for make() calls
        let primitive_type_identifiers = vec![
            ("int8", TypeId::Int8),
            ("int16", TypeId::Int16),
            ("int32", TypeId::Int32),
            ("int64", TypeId::Int64),
            ("uint8", TypeId::UInt8),
            ("uint16", TypeId::UInt16),
            ("uint32", TypeId::UInt32),
            ("uint64", TypeId::UInt64),
            ("float32", TypeId::Float32),
            ("float64", TypeId::Float64),
            ("bool", TypeId::Bool),
            ("char", TypeId::Char),
            ("string", TypeId::String),
            ("byte", TypeId::UInt8),  // alias for uint8
            ("rune", TypeId::Int32),  // alias for int32
            ("any", TypeId::Any),
            ("chan", TypeId::Any),    // channel type identifier
        ];

        if let Some(global_scope) = self.scopes.first_mut() {
            // Add primitive type identifiers
            for (name, type_id) in primitive_type_identifiers {
                let symbol = Symbol {
                    name: name.to_string(),
                    type_id,
                    is_mutable: false,
                    position: Position::new(0, 0, 0),
                    function_info: None,
                };
                global_scope.insert(name.to_string(), symbol);
            }
            
            // Add builtin functions (force insert to overwrite any conflicting imports)
            for (name, param_types, return_type) in builtin_functions {
                let symbol = Symbol {
                    name: name.to_string(),
                    type_id: TypeId::Function(0), // Placeholder function type
                    is_mutable: false,
                    position: Position::new(0, 0, 0),
                    function_info: Some(FunctionInfo {
                        param_types,
                        return_type,
                    }),
                };
                // Force insert to ensure builtin functions are always available
                global_scope.insert(name.to_string(), symbol);
            }
            
            // Add channel type identifiers (generated by make() parser)
            let channel_types = vec![
                "chan_int8", "chan_int16", "chan_int32", "chan_int64",
                "chan_uint8", "chan_uint16", "chan_uint32", "chan_uint64",
                "chan_float32", "chan_float64", "chan_bool", "chan_char",
                "chan_string", "chan_any", "chan_unknown", "chan"
            ];
            
            for chan_type in channel_types {
                let symbol = Symbol {
                    name: chan_type.to_string(),
                    type_id: TypeId::String, // Channel type identifiers are treated as strings
                    is_mutable: false,
                    position: Position::new(0, 0, 0),
                    function_info: None,
                };
                global_scope.insert(chan_type.to_string(), symbol);
            }

            // Add slice type identifiers (generated by make() parser)
            let slice_types = vec![
                "slice_int8", "slice_int16", "slice_int32", "slice_int64",
                "slice_uint8", "slice_uint16", "slice_uint32", "slice_uint64",
                "slice_float32", "slice_float64", "slice_bool", "slice_char",
                "slice_string", "slice_any", "slice_unknown"
            ];
            
            for slice_type in slice_types {
                let symbol = Symbol {
                    name: slice_type.to_string(),
                    type_id: TypeId::String, // Slice type identifiers are treated as strings
                    is_mutable: false,
                    position: Position::new(0, 0, 0),
                    function_info: None,
                };
                global_scope.insert(slice_type.to_string(), symbol);
            }
        }
    }

    /// Type check a complete program (alias for check_program)
    pub fn check(&mut self, program: &Program) -> Result<()> {
        self.check_program(program)
    }

    /// Convert AST type to TypeId using the type registry
    fn ast_type_to_type_id(&mut self, ast_type: &Type) -> TypeId {
        match ast_type {
            Type::Int8 => TypeId::Int8,
            Type::Int16 => TypeId::Int16,
            Type::Int32 => TypeId::Int32,
            Type::Int64 => TypeId::Int64,
            Type::UInt8 => TypeId::UInt8,
            Type::UInt16 => TypeId::UInt16,
            Type::UInt32 => TypeId::UInt32,
            Type::UInt64 => TypeId::UInt64,
            Type::Float32 => TypeId::Float32,
            Type::Float64 => TypeId::Float64,
            Type::Bool => TypeId::Bool,
            Type::Char => TypeId::Char,
            Type::String => TypeId::String,
            Type::Any => TypeId::Any,
            Type::Void => TypeId::Void,
            Type::Array(array_type) => {
                let element_type = self.ast_type_to_type_id(&array_type.element_type);
                let array_id = self.type_registry.register_array_type(element_type);
                TypeId::Array(array_id)
            }
            Type::Slice(slice_type) => {
                let element_type = self.ast_type_to_type_id(&slice_type.element_type);
                let slice_id = self.type_registry.register_slice_type(element_type);
                TypeId::Slice(slice_id)
            }
            Type::Map(map_type) => {
                let key_type = self.ast_type_to_type_id(&map_type.key_type);
                let value_type = self.ast_type_to_type_id(&map_type.value_type);
                let map_id = self.type_registry.register_map_type(key_type, value_type);
                TypeId::Map(map_id)
            }
            Type::Promise(promise_type) => {
                let result_type = self.ast_type_to_type_id(&promise_type.result_type);
                let promise_id = self.type_registry.register_promise_type(result_type);
                TypeId::Promise(promise_id)
            }
            Type::Function(_) => TypeId::Function(0), // Placeholder
            Type::Named(name) => {
                // Check if it's an interface or struct and create/get proper TypeId
                if self.interfaces.contains_key(name) {
                    self.get_or_create_named_type_id(name, true)
                } else if self.structs.contains_key(name) {
                    self.get_or_create_named_type_id(name, false)
                } else {
                    TypeId::Unknown
                }
            }
            _ => TypeId::Unknown,
        }
    }

    /// Type check a complete program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        for statement in &program.statements {
            self.check_statement(statement)?;
        }

        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(())
    }

    /// Type check a statement
    pub fn check_statement(&mut self, statement: &Statement) -> Result<TypeId> {
        match statement {
            Statement::VariableDecl(decl) => self.check_variable_declaration(decl),
            Statement::MultipleVariableDecl(decl) => self.check_multiple_variable_declaration(decl),
            Statement::MultipleAssignment(stmt) => self.check_multiple_assignment_statement(stmt),
            Statement::DestructuringDecl(decl) => self.check_destructuring_declaration(decl),
            Statement::FunctionDecl(decl) => self.check_function_declaration(decl),
            Statement::StructDecl(decl) => self.check_struct_declaration(decl),
            Statement::InterfaceDecl(decl) => self.check_interface_declaration(decl),

            Statement::If(stmt) => self.check_if_statement(stmt),
            Statement::While(stmt) => self.check_while_statement(stmt),
            Statement::For(stmt) => self.check_for_statement(stmt),
            Statement::Return(stmt) => self.check_return_statement(stmt),
            Statement::Break(_) | Statement::Continue(_) => Ok(TypeId::Any), // No type for control flow
            Statement::Expression(stmt) => self.check_expression(&stmt.expr),
            Statement::Block(stmt) => self.check_block_statement(stmt),
            _ => {
                // For now, return Any for unimplemented statement types
                Ok(TypeId::Any)
            }
        }
    }

    /// Type check a variable declaration
    fn check_variable_declaration(&mut self, decl: &VariableDecl) -> Result<TypeId> {
        let mut inferred_type = None;

        // Check initializer if present
        if let Some(ref initializer) = decl.initializer {
            let init_type = self.check_expression(initializer)?;
            inferred_type = Some(init_type);
        }

        // Determine the final type
        let final_type = match (&decl.type_annotation, inferred_type) {
            // Explicit type annotation
            (Some(ref type_ann), None) => self.ast_type_to_type_id(type_ann),
            // Type inference from initializer
            (None, Some(inferred)) => inferred,
            // Both explicit type and initializer - check compatibility
            (Some(ref type_ann), Some(inferred)) => {
                let explicit_type = self.ast_type_to_type_id(type_ann);

                // Check compatibility with special cases
                let is_compatible = if PrimitiveType::is_assignable(inferred, explicit_type) {
                    // Standard assignability check passes
                    true
                } else if let Some(ref initializer) = decl.initializer {
                    if let Expression::Literal(lit_expr) = initializer {
                        if let crate::ast::LiteralValue::Integer(_) = lit_expr.value {
                            // Allow integer literals to be assigned to any integer type
                            PrimitiveType::is_integer_type_id(explicit_type)
                                && inferred == TypeId::Int32
                        } else {
                            false
                        }
                    } else if let Expression::Array(_) = initializer {
                        // Allow array literals to be assigned to slice types if element types match
                        match (inferred, explicit_type) {
                            (TypeId::Array(_), TypeId::Slice(_)) => {
                                // Check if element types are compatible
                                if let (Some(array_elem), Some(slice_elem)) = (
                                    self.type_registry.get_element_type(inferred),
                                    self.type_registry.get_element_type(explicit_type),
                                ) {
                                    PrimitiveType::is_assignable(array_elem, slice_elem)
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !is_compatible {
                    return Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Cannot assign {} to variable of type {}",
                            PrimitiveType::type_name(inferred),
                            PrimitiveType::type_name(explicit_type)
                        ),
                        line: decl.position.line,
                        column: decl.position.column,
                    });
                }
                explicit_type
            }
            // Neither type annotation nor initializer
            (None, None) => {
                return Err(BuluError::TypeError {
                    file: None,
                    message: "Variable declaration must have either type annotation or initializer"
                        .to_string(),
                    line: decl.position.line,
                    column: decl.position.column,
                });
            }
        };

        // Add to symbol table
        let symbol = Symbol {
            name: decl.name.clone(),
            type_id: final_type,
            is_mutable: !decl.is_const,
            position: decl.position,
            function_info: None,
        };

        self.add_symbol(symbol)?;
        Ok(final_type)
    }

    /// Type check a multiple variable declaration
    fn check_multiple_variable_declaration(&mut self, decl: &MultipleVariableDecl) -> Result<TypeId> {
        for var_decl in &decl.declarations {
            // For each variable in the multiple declaration, check it like a single variable
            let mut inferred_type = None;

            // Check initializer if present
            if let Some(ref initializer) = var_decl.initializer {
                let init_type = self.check_expression(initializer)?;
                inferred_type = Some(init_type);
            }

            // Determine the final type
            let final_type = match (&var_decl.type_annotation, inferred_type) {
                // Explicit type annotation
                (Some(ref type_ann), None) => self.ast_type_to_type_id(type_ann),
                // Type inference from initializer
                (None, Some(inferred)) => inferred,
                // Both explicit type and initializer - check compatibility
                (Some(ref type_ann), Some(inferred)) => {
                    let explicit_type = self.ast_type_to_type_id(type_ann);
                    if !PrimitiveType::is_assignable(inferred, explicit_type) {
                        return Err(BuluError::TypeError {
                            file: None,
                            message: format!(
                                "Cannot assign {} to variable of type {}",
                                PrimitiveType::type_name(inferred),
                                PrimitiveType::type_name(explicit_type)
                            ),
                            line: decl.position.line,
                            column: decl.position.column,
                        });
                    }
                    explicit_type
                }
                // Neither type annotation nor initializer
                (None, None) => {
                    return Err(BuluError::TypeError {
                        file: None,
                        message: "Variable declaration must have either type annotation or initializer"
                            .to_string(),
                        line: decl.position.line,
                        column: decl.position.column,
                    });
                }
            };

            // Add to symbol table
            let symbol = Symbol {
                name: var_decl.name.clone(),
                type_id: final_type,
                is_mutable: !decl.is_const,
                position: decl.position,
                function_info: None,
            };

            self.add_symbol(symbol)?;
        }
        Ok(TypeId::Any)
    }

    /// Type check a destructuring declaration
    fn check_destructuring_declaration(&mut self, decl: &DestructuringDecl) -> Result<TypeId> {
        // First, check the type of the initializer
        let initializer_type = self.check_expression(&decl.initializer)?;
        
        // Extract variables from the pattern and add them to the current scope
        self.check_pattern_and_add_variables(&decl.pattern, initializer_type)?;
        
        Ok(TypeId::Void)
    }

    /// Type check a multiple assignment statement
    fn check_multiple_assignment_statement(&mut self, stmt: &MultipleAssignmentStmt) -> Result<TypeId> {
        // Check that all targets are valid lvalues (identifiers for now)
        for target in &stmt.targets {
            match target {
                Expression::Identifier(ident) => {
                    // Check that the identifier exists
                    if self.lookup_symbol(&ident.name).is_none() {
                        return Err(BuluError::TypeError {
                            message: format!("Undefined variable '{}'", ident.name),
                            line: ident.position.line,
                            column: ident.position.column,
                            file: None,
                        });
                    }
                }
                _ => {
                    return Err(BuluError::TypeError {
                        message: "Only simple identifiers are supported in multiple assignment".to_string(),
                        line: 0,
                        column: 0,
                        file: None,
                    });
                }
            }
        }

        // Check all value expressions
        for value in &stmt.values {
            self.check_expression(value)?;
        }

        // For now, we don't enforce type compatibility between targets and values
        // This could be enhanced later
        Ok(TypeId::Void)
    }

    /// Type check a function declaration
    fn check_function_declaration(&mut self, decl: &FunctionDecl) -> Result<TypeId> {
        // Collect parameter types
        let param_types: Vec<TypeId> = decl
            .params
            .iter()
            .map(|p| self.ast_type_to_type_id(&p.param_type))
            .collect();

        let declared_return_type = decl
            .return_type
            .as_ref()
            .map(|t| self.ast_type_to_type_id(t));

        // For async functions, wrap the return type in a Promise
        let actual_return_type = if decl.is_async {
            match declared_return_type {
                Some(ret_type) => {
                    // Wrap the declared return type in a Promise
                    let promise_id = self.type_registry.register_promise_type(ret_type);
                    Some(TypeId::Promise(promise_id))
                }
                None => {
                    // Async function with no explicit return type returns Promise<void>
                    let promise_id = self.type_registry.register_promise_type(TypeId::Void);
                    Some(TypeId::Promise(promise_id))
                }
            }
        } else {
            declared_return_type
        };

        // Add function to current scope first (for recursive calls)
        let func_symbol = Symbol {
            name: decl.name.clone(),
            type_id: TypeId::Function(0), // Placeholder function type
            is_mutable: false,
            position: decl.position,
            function_info: Some(FunctionInfo {
                param_types: param_types.clone(),
                return_type: actual_return_type,
            }),
        };
        self.add_symbol(func_symbol)?;

        // Enter new scope for function
        self.enter_scope();
        self.current_function = Some(decl.name.clone());

        // Set return type for checking return statements
        // For async functions, we check against the unwrapped return type
        let check_return_type = if decl.is_async {
            declared_return_type.or(Some(TypeId::Void))
        } else {
            actual_return_type
        };
        self.return_types.push(check_return_type);

        // Add parameters to scope
        for param in &decl.params {
            let param_type = self.ast_type_to_type_id(&param.param_type);
            let symbol = Symbol {
                name: param.name.clone(),
                type_id: param_type,
                is_mutable: true, // Parameters are mutable by default
                position: param.position,
                function_info: None,
            };
            self.add_symbol(symbol)?;
        }

        // Check function body
        self.check_block_statement(&decl.body)?;

        // Exit function scope
        self.return_types.pop();
        self.current_function = None;
        self.exit_scope();

        Ok(TypeId::Function(0)) // Placeholder function type
    }

    /// Type check an interface declaration
    fn check_interface_declaration(&mut self, decl: &InterfaceDecl) -> Result<TypeId> {
        // Create a unique TypeId for this interface
        let interface_type_id = self.get_or_create_named_type_id(&decl.name, true);

        // Store the interface declaration
        self.interfaces.insert(decl.name.clone(), decl.clone());

        // Register the interface name in the symbol table
        let interface_symbol = Symbol {
            name: decl.name.clone(),
            type_id: interface_type_id,
            is_mutable: false,
            position: decl.position,
            function_info: None,
        };

        self.add_symbol(interface_symbol)?;
        Ok(interface_type_id)
    }

    /// Type check a struct declaration
    fn check_struct_declaration(&mut self, decl: &StructDecl) -> Result<TypeId> {
        // Create a unique TypeId for this struct
        let struct_type_id = self.get_or_create_named_type_id(&decl.name, false);

        // Store the struct declaration
        self.structs.insert(decl.name.clone(), decl.clone());

        // Register the struct name in the symbol table
        let struct_symbol = Symbol {
            name: decl.name.clone(),
            type_id: struct_type_id,
            is_mutable: false,
            position: decl.position,
            function_info: None,
        };

        self.add_symbol(struct_symbol)?;

        // Type check all methods in the struct
        for method in &decl.methods {
            self.check_struct_method_declaration(method, &decl.name)?;
        }

        Ok(struct_type_id)
    }

    /// Type check a method declaration within a struct context
    fn check_struct_method_declaration(
        &mut self,
        decl: &FunctionDecl,
        struct_name: &str,
    ) -> Result<TypeId> {
        // Collect parameter types
        let param_types: Vec<TypeId> = decl
            .params
            .iter()
            .map(|p| self.ast_type_to_type_id(&p.param_type))
            .collect();

        let declared_return_type = decl
            .return_type
            .as_ref()
            .map(|t| self.ast_type_to_type_id(t));

        // For async functions, wrap the return type in a Promise
        let actual_return_type = if decl.is_async {
            match declared_return_type {
                Some(ret_type) => {
                    // Wrap the declared return type in a Promise
                    let promise_id = self.type_registry.register_promise_type(ret_type);
                    Some(TypeId::Promise(promise_id))
                }
                None => {
                    // Async function with no explicit return type returns Promise<void>
                    let promise_id = self.type_registry.register_promise_type(TypeId::Void);
                    Some(TypeId::Promise(promise_id))
                }
            }
        } else {
            declared_return_type
        };

        // Enter new scope for method
        self.enter_scope();
        self.current_function = Some(decl.name.clone());

        // Set return type for checking return statements
        let check_return_type = if decl.is_async {
            declared_return_type.or(Some(TypeId::Void))
        } else {
            actual_return_type
        };
        self.return_types.push(check_return_type);

        // Add 'this' parameter to scope (refers to the struct instance)
        let struct_type_id = self.get_or_create_named_type_id(struct_name, false);
        let this_symbol = Symbol {
            name: "this".to_string(),
            type_id: struct_type_id, // Reference to the actual struct type
            is_mutable: true,        // 'this' is mutable by default
            position: decl.position,
            function_info: None,
        };
        self.add_symbol(this_symbol)?;

        // Add parameters to scope
        for param in &decl.params {
            let param_type = self.ast_type_to_type_id(&param.param_type);
            let symbol = Symbol {
                name: param.name.clone(),
                type_id: param_type,
                is_mutable: true, // Parameters are mutable by default
                position: param.position,
                function_info: None,
            };
            self.add_symbol(symbol)?;
        }

        // Check method body
        self.check_block_statement(&decl.body)?;

        // Exit method scope
        self.return_types.pop();
        self.current_function = None;
        self.exit_scope();

        Ok(TypeId::Function(0)) // Placeholder function type
    }

    /// Type check an if statement
    fn check_if_statement(&mut self, stmt: &IfStmt) -> Result<TypeId> {
        // Check condition
        let condition_type = self.check_expression(&stmt.condition)?;
        if condition_type != TypeId::Bool {
            return Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "If condition must be bool, got {}",
                    PrimitiveType::type_name(condition_type)
                ),
                line: stmt.position.line,
                column: stmt.position.column,
            });
        }

        // Check then branch
        self.check_block_statement(&stmt.then_branch)?;

        // Check else branch if present
        if let Some(ref else_branch) = stmt.else_branch {
            self.check_statement(else_branch)?;
        }

        Ok(TypeId::Any) // If statements don't have a type
    }

    /// Type check a while statement
    fn check_while_statement(&mut self, stmt: &WhileStmt) -> Result<TypeId> {
        // Check condition
        let condition_type = self.check_expression(&stmt.condition)?;
        if condition_type != TypeId::Bool {
            return Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "While condition must be bool, got {}",
                    PrimitiveType::type_name(condition_type)
                ),
                line: stmt.position.line,
                column: stmt.position.column,
            });
        }

        // Check body
        self.check_block_statement(&stmt.body)?;

        Ok(TypeId::Any) // While statements don't have a type
    }

    /// Type check a for statement
    fn check_for_statement(&mut self, stmt: &ForStmt) -> Result<TypeId> {
        // Enter new scope for loop variable
        self.enter_scope();

        // Check iterable expression
        let iterable_type = self.check_expression(&stmt.iterable)?;

        // Determine element type based on iterable type
        let element_type = match iterable_type {
            TypeId::String => TypeId::Char,
            TypeId::Array(_) | TypeId::Slice(_) => TypeId::Any, // Placeholder
            TypeId::Any => {
                // This could be a range (0..5) which returns Any for now
                // For ranges, the element type is the same as the range bounds
                // We'll assume Int32 for now since most ranges are integer ranges
                TypeId::Int32
            }
            _ => {
                return Err(BuluError::TypeError {
                    file: None,
                    message: format!(
                        "Cannot iterate over {}",
                        PrimitiveType::type_name(iterable_type)
                    ),
                    line: stmt.position.line,
                    column: stmt.position.column,
                });
            }
        };

        // Add index variable to scope if present
        if let Some(ref index_var) = stmt.index_variable {
            let index_symbol = Symbol {
                name: index_var.clone(),
                type_id: TypeId::Int32, // Index is always int32
                is_mutable: false,      // Loop variables are immutable
                position: stmt.position,
                function_info: None,
            };
            self.add_symbol(index_symbol)?;
        }

        // Add loop variable to scope
        let symbol = Symbol {
            name: stmt.variable.clone(),
            type_id: element_type,
            is_mutable: false, // Loop variables are immutable
            position: stmt.position,
            function_info: None,
        };
        self.add_symbol(symbol)?;

        // Check body
        self.check_block_statement(&stmt.body)?;

        // Exit loop scope
        self.exit_scope();

        Ok(TypeId::Any) // For statements don't have a type
    }

    /// Type check a return statement
    fn check_return_statement(&mut self, stmt: &ReturnStmt) -> Result<TypeId> {
        let expected_return_type = self.return_types.last().copied().flatten();

        match (&stmt.value, expected_return_type) {
            // Return with value
            (Some(ref expr), Some(expected)) => {
                let actual_type = self.check_expression(expr)?;
                if !PrimitiveType::is_assignable(actual_type, expected) {
                    return Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Cannot return {} from function expecting {}",
                            PrimitiveType::type_name(actual_type),
                            PrimitiveType::type_name(expected)
                        ),
                        line: stmt.position.line,
                        column: stmt.position.column,
                    });
                }
                Ok(actual_type)
            }
            // Return without value from void function or function without explicit return type
            (None, None) => Ok(TypeId::Any), // Void return
            // Return with value from function without explicit return type (infer return type)
            (Some(ref expr), None) => {
                let actual_type = self.check_expression(expr)?;
                Ok(actual_type)
            }
            // Return without value but function expects a value
            (None, Some(expected)) => Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "Function expects return value of type {}",
                    PrimitiveType::type_name(expected)
                ),
                line: stmt.position.line,
                column: stmt.position.column,
            }),
        }
    }

    /// Type check a block statement
    fn check_block_statement(&mut self, stmt: &BlockStmt) -> Result<TypeId> {
        self.enter_scope();

        let mut last_type = TypeId::Any;
        for statement in &stmt.statements {
            last_type = self.check_statement(statement)?;
        }

        self.exit_scope();
        Ok(last_type)
    }

    /// Type check an expression
    pub fn check_expression(&mut self, expr: &Expression) -> Result<TypeId> {
        match expr {
            Expression::Literal(lit) => Ok(self.check_literal_expression(lit)),
            Expression::Identifier(ident) => self.check_identifier_expression(ident),
            Expression::Binary(bin) => self.check_binary_expression(bin),
            Expression::Unary(unary) => self.check_unary_expression(unary),
            Expression::Call(call) => self.check_call_expression(call),
            Expression::MemberAccess(access) => self.check_member_access_expression(access),
            Expression::Index(index) => self.check_index_expression(index),
            Expression::Assignment(assign) => self.check_assignment_expression(assign),
            Expression::Array(array) => self.check_array_expression(array),
            Expression::Map(map) => self.check_map_expression(map),
            Expression::StructLiteral(struct_lit) => {
                self.check_struct_literal_expression(struct_lit)
            }
            Expression::Cast(cast) => self.check_cast_expression(cast),
            Expression::TypeOf(typeof_expr) => self.check_typeof_expression(typeof_expr),
            Expression::Async(async_expr) => self.check_async_expression(async_expr),
            Expression::Await(await_expr) => self.check_await_expression(await_expr),
            Expression::Range(range) => self.check_range_expression(range),
            Expression::Parenthesized(paren) => self.check_expression(&paren.expr),
            _ => {
                // For now, return Any for unimplemented expression types
                Ok(TypeId::Any)
            }
        }
    }

    /// Type check a literal expression
    fn check_literal_expression(&self, lit: &LiteralExpr) -> TypeId {
        PrimitiveType::infer_from_literal(&lit.value)
    }

    /// Type check an identifier expression
    fn check_identifier_expression(&self, ident: &IdentifierExpr) -> Result<TypeId> {
        match self.lookup_symbol(&ident.name) {
            Some(symbol) => {
                Ok(symbol.type_id)
            },
            None => {
                // Check if it's a generated type identifier for make()
                if let Some(base_type) = self.extract_base_type_from_generated(&ident.name) {
                    // Verify that the base type actually exists
                    if self.is_valid_type(&base_type) {
                        // This is a valid generated type identifier, treat it as string
                        return Ok(TypeId::String);
                    }
                }
                
                Err(BuluError::TypeError {
                    file: None,
                    message: format!("Undefined identifier '{}'", ident.name),
                    line: ident.position.line,
                    column: ident.position.column,
                })
            }
        }
    }

    /// Extract base type from generated type identifiers like "slice_Person" -> "Person"
    fn extract_base_type_from_generated(&self, name: &str) -> Option<String> {
        if let Some(base) = name.strip_prefix("chan_") {
            Some(base.to_string())
        } else if let Some(base) = name.strip_prefix("slice_") {
            Some(base.to_string())
        } else if let Some(base) = name.strip_prefix("array_") {
            Some(base.to_string())
        } else if name.starts_with("map_") {
            // For maps, we'd need more complex parsing, but for now just accept
            Some("any".to_string())
        } else {
            None
        }
    }

    /// Check if a type name is valid (either primitive or user-defined)
    fn is_valid_type(&self, type_name: &str) -> bool {
        
        // Check primitive types
        match type_name {
            "int8" | "int16" | "int32" | "int64" |
            "uint8" | "uint16" | "uint32" | "uint64" |
            "float32" | "float64" | "bool" | "char" |
            "string" | "any" | "unknown" => {
                true
            }
            _ => {
                // Check if it's a user-defined struct
                if self.structs.contains_key(type_name) {
                    return true;
                }
                
                // Check if it's a user-defined interface
                if self.interfaces.contains_key(type_name) {
                    return true;
                }
                
                // Check if it's in the symbol table (could be imported)
                self.lookup_symbol(type_name).is_some()
            }
        }
    }

    /// Type check a binary expression
    fn check_binary_expression(&mut self, bin: &BinaryExpr) -> Result<TypeId> {
        let left_type = self.check_expression(&bin.left)?;
        let right_type = self.check_expression(&bin.right)?;

        let op_str = match bin.operator {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Power => "**",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::Greater => ">",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            _ => "unknown",
        };

        PrimitiveType::binary_operation_result_type(left_type, right_type, op_str).map_err(
            |mut e| {
                if let BuluError::TypeError {
                    file: None,
                    ref mut line,
                    ref mut column,
                    ..
                } = e
                {
                    *line = bin.position.line;
                    *column = bin.position.column;
                }
                e
            },
        )
    }

    /// Type check a unary expression
    fn check_unary_expression(&mut self, unary: &UnaryExpr) -> Result<TypeId> {
        let operand_type = self.check_expression(&unary.operand)?;

        match unary.operator {
            UnaryOperator::Plus | UnaryOperator::Minus => {
                if PrimitiveType::is_numeric_type_id(operand_type) {
                    Ok(operand_type)
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Unary {} operator requires numeric operand, got {}",
                            match unary.operator {
                                UnaryOperator::Plus => "+",
                                UnaryOperator::Minus => "-",
                                _ => "unknown",
                            },
                            PrimitiveType::type_name(operand_type)
                        ),
                        line: unary.position.line,
                        column: unary.position.column,
                    })
                }
            }
            UnaryOperator::Not => {
                if operand_type == TypeId::Bool {
                    Ok(TypeId::Bool)
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Unary not operator requires bool operand, got {}",
                            PrimitiveType::type_name(operand_type)
                        ),
                        line: unary.position.line,
                        column: unary.position.column,
                    })
                }
            }
            _ => Ok(operand_type), // For other operators, assume same type
        }
    }

    /// Type check a function call expression
    fn check_call_expression(&mut self, call: &CallExpr) -> Result<TypeId> {
        match &*call.callee {
            // Handle direct function calls (e.g., func())
            Expression::Identifier(ident) => {
                // Handle make built-in function FIRST (before symbol lookup)

                if ident.name == "make" {

                    // make() takes 1-3 arguments depending on type
                    if call.args.is_empty() || call.args.len() > 3 {
                        return Err(BuluError::TypeError {
                            file: None,
                            message: format!(
                                "make() expects 1-3 arguments, got {}",
                                call.args.len()
                            ),
                            line: call.position.line,
                            column: call.position.column,
                        });
                    }
                    
                    // Check the first argument (type specifier)
                    // For make(), the first argument can be a type identifier
                    match &call.args[0] {
                        Expression::Identifier(type_ident) => {

                            // Check if it's a valid type for make()
                            let valid_types = vec![
                                "int8", "int16", "int32", "int64",
                                "uint8", "uint16", "uint32", "uint64",
                                "float32", "float64", "bool", "string",
                                "char", "byte", "rune", "any", "chan"
                            ];
                            
                            if valid_types.contains(&type_ident.name.as_str()) {

                                // Check additional arguments if present
                                for arg in &call.args[1..] {
                                    let arg_type = self.check_expression(arg)?;
                                    // Size/capacity arguments should be integers
                                    if !matches!(arg_type, TypeId::Int32 | TypeId::Int64 | TypeId::UInt32 | TypeId::UInt64) {
                                        return Err(BuluError::TypeError {
                                            file: None,
                                            message: "make() size/capacity arguments must be integers".to_string(),
                                            line: call.position.line,
                                            column: call.position.column,
                                        });
                                    }
                                }
                                
                                // Return appropriate type based on what's being made
                                return match type_ident.name.as_str() {
                                    "chan" => Ok(TypeId::Any), // Channel type
                                    "int8" | "int16" | "int32" | "uint8" | "uint16" | "uint32" | "byte" | "rune" => Ok(TypeId::Int32),
                                    "int64" | "uint64" => Ok(TypeId::Int64),
                                    "float32" | "float64" => Ok(TypeId::Float64),
                                    "bool" => Ok(TypeId::Bool),
                                    "string" | "char" => Ok(TypeId::String),
                                    "any" => Ok(TypeId::Any),
                                    _ => Ok(TypeId::Any),
                                };
                            } else {
                                // Not a primitive type, check as normal expression
                                self.check_expression(&call.args[0])?;
                                return Ok(TypeId::Any); // make() returns the appropriate type
                            }
                        }
                        _ => {
                            // Not an identifier, check as normal expression
                            self.check_expression(&call.args[0])?;
                            // Check additional arguments
                            for arg in &call.args[1..] {
                                self.check_expression(arg)?;
                            }
                            return Ok(TypeId::Any); // make() returns the appropriate type
                        }
                    }
                }
                
                // Look up function in symbol table and clone the info to avoid borrow issues
                let symbol_opt = self.lookup_symbol(&ident.name);
                let func_info_opt = symbol_opt.and_then(|s| s.function_info.clone());

                if let Some(func_info) = func_info_opt {
                    // For built-in functions like print, we're more lenient
                    if ident.name == "print" {
                        // Print can take any number of arguments of any type
                        for arg in &call.args {
                            self.check_expression(arg)?;
                        }
                        return Ok(TypeId::Any); // print doesn't return a value
                    }

                    // Handle println built-in function
                    if ident.name == "println" {
                        // println can take any number of arguments of any type
                        for arg in &call.args {
                            self.check_expression(arg)?;
                        }
                        return Ok(TypeId::Any); // println doesn't return a value
                    }

                    // Handle typeof built-in function
                    if ident.name == "typeof" {
                        // typeof takes exactly one argument of any type
                        if call.args.len() != 1 {
                            return Err(BuluError::TypeError {
                                file: None,
                                message: format!(
                                    "typeof() expects exactly 1 argument, got {}",
                                    call.args.len()
                                ),
                                line: call.position.line,
                                column: call.position.column,
                            });
                        }
                        // Check the argument
                        self.check_expression(&call.args[0])?;
                        return Ok(TypeId::String); // typeof returns string
                    }



                    // Check argument count
                    if call.args.len() != func_info.param_types.len() {
                        return Err(BuluError::TypeError {
                            file: None,
                            message: format!(
                                "Function '{}' expects {} arguments, got {}",
                                ident.name,
                                func_info.param_types.len(),
                                call.args.len()
                            ),
                            line: call.position.line,
                            column: call.position.column,
                        });
                    }

                    // Check argument types
                    for (i, (arg, expected_type)) in call
                        .args
                        .iter()
                        .zip(func_info.param_types.iter())
                        .enumerate()
                    {
                        let actual_type = self.check_expression(arg)?;
                        if !self.is_type_compatible(actual_type, *expected_type) {
                            return Err(BuluError::TypeError {
                                file: None,
                                message: format!(
                                    "Argument {} to function '{}': expected {}, got {}",
                                    i + 1,
                                    ident.name,
                                    self.type_name_for_error(*expected_type),
                                    self.type_name_for_error(actual_type)
                                ),
                                line: call.position.line,
                                column: call.position.column,
                            });
                        }
                    }

                    // Return the function's return type
                    Ok(func_info.return_type.unwrap_or(TypeId::Any))
                } else if self.lookup_symbol(&ident.name).is_some() {
                    // Symbol exists but is not a function
                    return Err(BuluError::TypeError {
                        file: None,
                        message: format!("'{}' is not a function", ident.name),
                        line: call.position.line,
                        column: call.position.column,
                    });
                } else {
                    return Err(BuluError::TypeError {
                        file: None,
                        message: format!("Undefined function '{}'", ident.name),
                        line: call.position.line,
                        column: call.position.column,
                    });
                }
            }
            Expression::MemberAccess(member_access) => {
                // Handle method calls: obj.method()
                let object_type = self.check_expression(&member_access.object)?;

                // Check arguments
                for arg in &call.args {
                    self.check_expression(arg)?;
                }

                // Look up the method in the object's type
                let type_name = self.get_type_name_from_expression(&member_access.object)?;
                let type_name_for_error =
                    type_name.clone().unwrap_or_else(|| "unknown".to_string());

                match object_type {
                    TypeId::Struct(_) => {
                        if let Some(struct_name) = type_name.as_ref() {
                            if let Some(struct_decl) = self.structs.get(struct_name).cloned() {
                                // Look for the method in the struct
                                for method in &struct_decl.methods {
                                    if method.name == member_access.member {
                                        return match &method.return_type {
                                            Some(return_type) => {
                                                Ok(self.ast_type_to_type_id(return_type))
                                            }
                                            None => Ok(TypeId::Void),
                                        };
                                    }
                                }

                                // Check if struct implements any interface with this method
                                let interfaces_clone = self.interfaces.clone();
                                for (interface_name, interface_decl) in &interfaces_clone {
                                    if self
                                        .struct_implements_interface(&struct_name, interface_name)
                                    {
                                        for method in &interface_decl.methods {
                                            if method.name == member_access.member {
                                                return match &method.return_type {
                                                    Some(return_type) => {
                                                        Ok(self.ast_type_to_type_id(return_type))
                                                    }
                                                    None => Ok(TypeId::Void),
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    TypeId::Interface(_) => {
                        if let Some(interface_name) = type_name.as_ref() {
                            if let Some(interface_decl) =
                                self.interfaces.get(interface_name).cloned()
                            {
                                for method in &interface_decl.methods {
                                    if method.name == member_access.member {
                                        return match &method.return_type {
                                            Some(return_type) => {
                                                Ok(self.ast_type_to_type_id(return_type))
                                            }
                                            None => Ok(TypeId::Void),
                                        };
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Handle built-in methods for primitive types
                        match member_access.member.as_str() {
                            "toString" => return Ok(TypeId::String),
                            _ => {}
                        }
                    }
                }

                // If method not found, provide a helpful error message
                match object_type {
                    TypeId::Interface(_) => Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Method '{}' not found in interface '{}'",
                            member_access.member, type_name_for_error
                        ),
                        line: call.position.line,
                        column: call.position.column,
                    }),
                    TypeId::Struct(_) => Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Method '{}' not found in struct '{}'",
                            member_access.member, type_name_for_error
                        ),
                        line: call.position.line,
                        column: call.position.column,
                    }),
                    _ => Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Method '{}' not found on type '{}'",
                            member_access.member,
                            self.type_name_for_error(object_type)
                        ),
                        line: call.position.line,
                        column: call.position.column,
                    }),
                }
            }
            _ => {
                // For other non-identifier callees, just check the expression
                let _callee_type = self.check_expression(&call.callee)?;

                // Check arguments
                for arg in &call.args {
                    self.check_expression(arg)?;
                }

                Ok(TypeId::Any)
            }
        }
    }

    /// Type check a member access expression
    fn check_member_access_expression(&mut self, access: &MemberAccessExpr) -> Result<TypeId> {
        let object_type = self.check_expression(&access.object)?;

        // Get the type name from the object
        let type_name = self.get_type_name_from_expression(&access.object)?;

        match object_type {
            TypeId::Interface(_) => {
                // Look up the method in the specific interface
                if let Some(interface_name) = type_name {
                    if let Some(interface_decl) = self.interfaces.get(&interface_name).cloned() {
                        for method in &interface_decl.methods {
                            if method.name == access.member {
                                return match &method.return_type {
                                    Some(return_type) => Ok(self.ast_type_to_type_id(return_type)),
                                    None => Ok(TypeId::Void),
                                };
                            }
                        }
                    }
                }
            }
            TypeId::Struct(_) => {
                // Look up the field or method in the struct
                if let Some(struct_name) = type_name {
                    if let Some(struct_decl) = self.structs.get(&struct_name).cloned() {
                        // First check struct fields
                        for field in &struct_decl.fields {
                            if field.name == access.member {
                                return Ok(self.ast_type_to_type_id(&field.field_type));
                            }
                        }

                        // Then check struct's own methods
                        for method in &struct_decl.methods {
                            if method.name == access.member {
                                return match &method.return_type {
                                    Some(return_type) => Ok(self.ast_type_to_type_id(return_type)),
                                    None => Ok(TypeId::Void),
                                };
                            }
                        }

                        // Finally check if struct implements any interface with this method
                        let interfaces_clone = self.interfaces.clone();
                        for (interface_name, interface_decl) in &interfaces_clone {
                            if self.struct_implements_interface(&struct_name, interface_name) {
                                for method in &interface_decl.methods {
                                    if method.name == access.member {
                                        return match &method.return_type {
                                            Some(return_type) => {
                                                Ok(self.ast_type_to_type_id(return_type))
                                            }
                                            None => Ok(TypeId::Void),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Handle built-in methods for primitive types
                match access.member.as_str() {
                    "toString" => return Ok(TypeId::String),
                    _ => {}
                }
            }
        }

        Err(BuluError::TypeError {
            message: format!("Member '{}' not found", access.member),
            line: access.position.line,
            column: access.position.column,
            file: None,
        })
    }

    /// Type check an index expression
    fn check_index_expression(&mut self, index: &IndexExpr) -> Result<TypeId> {
        let object_type = self.check_expression(&index.object)?;
        let index_type = self.check_expression(&index.index)?;

        // Check if this is a slicing operation (index is a range)
        let is_slicing = matches!(index.index.as_ref(), Expression::Range(_));
        
        // Check index type based on the object type
        match object_type {
            TypeId::String | TypeId::Array(_) | TypeId::Slice(_) => {
                if is_slicing {
                    // For slicing, we need to validate the range bounds are integers
                    if let Expression::Range(range) = index.index.as_ref() {
                        let start_type = self.check_expression(&range.start)?;
                        let end_type = self.check_expression(&range.end)?;
                        
                        if !PrimitiveType::is_integer_type_id(start_type) {
                            return Err(BuluError::TypeError {
                                file: None,
                                message: format!(
                                    "Slice start index must be integer, got {}",
                                    PrimitiveType::type_name(start_type)
                                ),
                                line: index.position.line,
                                column: index.position.column,
                            });
                        }
                        
                        if !PrimitiveType::is_integer_type_id(end_type) {
                            return Err(BuluError::TypeError {
                                file: None,
                                message: format!(
                                    "Slice end index must be integer, got {}",
                                    PrimitiveType::type_name(end_type)
                                ),
                                line: index.position.line,
                                column: index.position.column,
                            });
                        }
                    }
                } else {
                    // Arrays, slices, and strings require integer indices for simple indexing
                    if !PrimitiveType::is_integer_type_id(index_type) {
                        return Err(BuluError::TypeError {
                            file: None,
                            message: format!(
                                "Array/slice/string index must be integer, got {}",
                                PrimitiveType::type_name(index_type)
                            ),
                            line: index.position.line,
                            column: index.position.column,
                        });
                    }
                }
            }
            TypeId::Map(_type_id) => {
                // For maps, check that the index type matches the key type
                if let Some((key_type, _value_type)) = self.type_registry.get_map_types(object_type)
                {
                    if !PrimitiveType::is_assignable(index_type, key_type) {
                        return Err(BuluError::TypeError {
                            file: None,
                            message: format!(
                                "Map key must be {}, got {}",
                                PrimitiveType::type_name(key_type),
                                PrimitiveType::type_name(index_type)
                            ),
                            line: index.position.line,
                            column: index.position.column,
                        });
                    }
                }
                // If we can't determine the key type, allow any index type for now
            }
            _ => {
                // For other types, we'll check later if indexing is supported
            }
        }

        // Return the appropriate type based on the object type and operation
        match object_type {
            TypeId::String => {
                if is_slicing {
                    // String slicing returns a string
                    Ok(TypeId::String)
                } else {
                    // String indexing returns a char
                    Ok(TypeId::Char)
                }
            }
            TypeId::Array(_type_id) | TypeId::Slice(_type_id) => {
                if is_slicing {
                    // Array/slice slicing returns a slice of the same element type
                    if let Some(element_type) = self.type_registry.get_element_type(object_type) {
                        // Create a slice type with the same element type
                        let slice_id = self.type_registry.register_slice_type(element_type);
                        Ok(TypeId::Slice(slice_id))
                    } else {
                        // Fallback to Any slice if we can't determine the element type
                        let slice_id = self.type_registry.register_slice_type(TypeId::Any);
                        Ok(TypeId::Slice(slice_id))
                    }
                } else {
                    // Array/slice indexing returns the element type
                    if let Some(element_type) = self.type_registry.get_element_type(object_type) {
                        Ok(element_type)
                    } else {
                        // Fallback to Any if we can't determine the element type
                        Ok(TypeId::Any)
                    }
                }
            }
            TypeId::Map(_type_id) => {
                // For maps, indexing returns the value type
                if let Some((_key_type, value_type)) = self.type_registry.get_map_types(object_type)
                {
                    Ok(value_type)
                } else {
                    // Fallback to Any if we can't determine the value type
                    Ok(TypeId::Any)
                }
            }
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "Cannot index into {}",
                    PrimitiveType::type_name(object_type)
                ),
                line: index.position.line,
                column: index.position.column,
            }),
        }
    }

    /// Type check an assignment expression
    fn check_assignment_expression(&mut self, assign: &AssignmentExpr) -> Result<TypeId> {
        let target_type = self.check_expression(&assign.target)?;
        let value_type = self.check_expression(&assign.value)?;

        // Check assignment compatibility
        if !PrimitiveType::is_assignable(value_type, target_type) {
            return Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "Cannot assign {} to {}",
                    PrimitiveType::type_name(value_type),
                    PrimitiveType::type_name(target_type)
                ),
                line: assign.position.line,
                column: assign.position.column,
            });
        }

        // Check if target is mutable (for identifier assignments)
        if let Expression::Identifier(ident) = &*assign.target {
            if let Some(symbol) = self.lookup_symbol(&ident.name) {
                if !symbol.is_mutable {
                    return Err(BuluError::TypeError {
                        file: None,
                        message: format!("Cannot assign to immutable variable '{}'", ident.name),
                        line: assign.position.line,
                        column: assign.position.column,
                    });
                }
            }
        }

        Ok(target_type)
    }

    /// Type check an array expression
    fn check_array_expression(&mut self, array: &ArrayExpr) -> Result<TypeId> {
        if array.elements.is_empty() {
            return Ok(TypeId::Array(0)); // Empty array
        }

        // Check all elements and ensure they have the same type
        let first_type = self.check_expression(&array.elements[0])?;
        for element in &array.elements[1..] {
            let element_type = self.check_expression(element)?;
            if !PrimitiveType::is_assignable(element_type, first_type) {
                return Err(BuluError::TypeError {
                    file: None,
                    message: format!(
                        "Array elements must have the same type, expected {}, got {}",
                        PrimitiveType::type_name(first_type),
                        PrimitiveType::type_name(element_type)
                    ),
                    line: array.position.line,
                    column: array.position.column,
                });
            }
        }

        // Register the array type and return it
        let array_type_id = self.type_registry.register_array_type(first_type);
        Ok(TypeId::Array(array_type_id))
    }

    /// Type check a map expression
    fn check_map_expression(&mut self, map: &MapExpr) -> Result<TypeId> {
        if map.entries.is_empty() {
            return Ok(TypeId::Map(0)); // Empty map
        }

        // Check first entry to determine key and value types
        let first_entry = &map.entries[0];
        let key_type = self.check_expression(&first_entry.key)?;
        let value_type = self.check_expression(&first_entry.value)?;

        // Check all other entries have compatible types
        for entry in &map.entries[1..] {
            let entry_key_type = self.check_expression(&entry.key)?;
            let entry_value_type = self.check_expression(&entry.value)?;

            if !PrimitiveType::is_assignable(entry_key_type, key_type) {
                let key_type_name = self.type_registry.get_type_name(key_type);
                let entry_key_type_name = self.type_registry.get_type_name(entry_key_type);
                return Err(BuluError::TypeError {
                    file: None,
                    message: format!(
                        "Map keys must have the same type, expected {}, got {}",
                        key_type_name, entry_key_type_name
                    ),
                    line: entry.position.line,
                    column: entry.position.column,
                });
            }

            if !PrimitiveType::is_assignable(entry_value_type, value_type) {
                let value_type_name = self.type_registry.get_type_name(value_type);
                let entry_value_type_name = self.type_registry.get_type_name(entry_value_type);
                return Err(BuluError::TypeError {
                    file: None,
                    message: format!(
                        "Map values must have the same type, expected {}, got {}",
                        value_type_name, entry_value_type_name
                    ),
                    line: entry.position.line,
                    column: entry.position.column,
                });
            }
        }

        // Register the map type and return it
        let map_type_id = self.type_registry.register_map_type(key_type, value_type);
        Ok(TypeId::Map(map_type_id))
    }

    /// Type check a struct literal expression
    fn check_struct_literal_expression(
        &mut self,
        struct_lit: &StructLiteralExpr,
    ) -> Result<TypeId> {
        // Check if the struct type exists
        if let Some(struct_decl) = self.structs.get(&struct_lit.type_name).cloned() {
            // Get or create the TypeId for this struct
            let struct_type_id = self.get_or_create_named_type_id(&struct_lit.type_name, false);

            // Verify that all required fields are provided and have correct types
            for field in &struct_decl.fields {
                let mut field_found = false;

                for field_init in &struct_lit.fields {
                    if field_init.name == field.name {
                        field_found = true;

                        // Check that the field value has the correct type
                        let value_type = self.check_expression(&field_init.value)?;
                        let expected_type = self.ast_type_to_type_id(&field.field_type);

                        if !PrimitiveType::is_assignable(value_type, expected_type) {
                            return Err(BuluError::TypeError {
                                message: format!(
                                    "Field '{}' expects type {}, got {}",
                                    field.name,
                                    PrimitiveType::type_name(expected_type),
                                    PrimitiveType::type_name(value_type)
                                ),
                                line: field_init.position.line,
                                column: field_init.position.column,
                                file: None,
                            });
                        }
                        break;
                    }
                }

                // Field not found is OK - it will get a default value
                // We don't need to error here anymore
            }

            // Check for extra fields that don't exist in the struct
            for field_init in &struct_lit.fields {
                let mut field_exists = false;

                for field in &struct_decl.fields {
                    if field.name == field_init.name {
                        field_exists = true;
                        break;
                    }
                }

                if !field_exists {
                    return Err(BuluError::TypeError {
                        message: format!(
                            "Unknown field '{}' in struct '{}'",
                            field_init.name, struct_lit.type_name
                        ),
                        line: field_init.position.line,
                        column: field_init.position.column,
                        file: None,
                    });
                }
            }

            Ok(struct_type_id)
        } else {
            Err(BuluError::TypeError {
                message: format!("Unknown struct type '{}'", struct_lit.type_name),
                line: struct_lit.position.line,
                column: struct_lit.position.column,
                file: None,
            })
        }
    }

    /// Get the default value for a type
    fn get_default_value_for_type(&self, type_id: TypeId) -> crate::types::primitive::RuntimeValue {
        use crate::types::primitive::RuntimeValue;
        
        match type_id {
            TypeId::Int8 => RuntimeValue::Int8(0),
            TypeId::Int16 => RuntimeValue::Int16(0),
            TypeId::Int32 => RuntimeValue::Int32(0),
            TypeId::Int64 => RuntimeValue::Int64(0),
            TypeId::UInt8 => RuntimeValue::UInt8(0),
            TypeId::UInt16 => RuntimeValue::UInt16(0),
            TypeId::UInt32 => RuntimeValue::UInt32(0),
            TypeId::UInt64 => RuntimeValue::UInt64(0),
            TypeId::Float32 => RuntimeValue::Float32(0.0),
            TypeId::Float64 => RuntimeValue::Float64(0.0),
            TypeId::Bool => RuntimeValue::Bool(false),
            TypeId::Char => RuntimeValue::Char('\0'),
            TypeId::String => RuntimeValue::String(String::new()),
            TypeId::Struct(_) => {
                // For structs, create an empty struct (all fields will get default values)
                RuntimeValue::Struct {
                    name: "".to_string(),
                    fields: std::collections::HashMap::new(),
                }
            }
            _ => RuntimeValue::Null, // For any other type, use null as default
        }
    }

    /// Type check a cast expression
    fn check_cast_expression(&mut self, cast: &CastExpr) -> Result<TypeId> {
        let expr_type = self.check_expression(&cast.expr)?;
        let target_type = PrimitiveType::ast_type_to_type_id(&cast.target_type);

        // Check if the cast is valid
        use crate::types::casting::TypeCaster;
        if !TypeCaster::is_cast_valid(expr_type, target_type) {
            return Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "Cannot cast {} to {}",
                    PrimitiveType::type_name(expr_type),
                    PrimitiveType::type_name(target_type)
                ),
                line: cast.position.line,
                column: cast.position.column,
            });
        }

        Ok(target_type)
    }

    /// Type check a typeof expression
    fn check_typeof_expression(&mut self, typeof_expr: &TypeOfExpr) -> Result<TypeId> {
        // Check the inner expression to ensure it's valid
        let _expr_type = self.check_expression(&typeof_expr.expr)?;

        // typeof() always returns a string containing the type name
        Ok(TypeId::String)
    }

    /// Type check a range expression
    fn check_range_expression(&mut self, range: &RangeExpr) -> Result<TypeId> {
        // Check start and end expressions
        let start_type = self.check_expression(&range.start)?;
        let end_type = self.check_expression(&range.end)?;

        // Both start and end should be numeric types
        if !PrimitiveType::is_numeric_type_id(start_type) {
            return Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "Range start must be numeric, got {}",
                    PrimitiveType::type_name(start_type)
                ),
                line: range.position.line,
                column: range.position.column,
            });
        }

        if !PrimitiveType::is_numeric_type_id(end_type) {
            return Err(BuluError::TypeError {
                file: None,
                message: format!(
                    "Range end must be numeric, got {}",
                    PrimitiveType::type_name(end_type)
                ),
                line: range.position.line,
                column: range.position.column,
            });
        }

        // Check step if present
        if let Some(ref step) = range.step {
            let step_type = self.check_expression(step)?;
            if !PrimitiveType::is_numeric_type_id(step_type) {
                return Err(BuluError::TypeError {
                    file: None,
                    message: format!(
                        "Range step must be numeric, got {}",
                        PrimitiveType::type_name(step_type)
                    ),
                    line: range.position.line,
                    column: range.position.column,
                });
            }
        }

        // For now, return a special Range type ID
        // We'll need to add this to the TypeId enum
        Ok(TypeId::Any) // Placeholder for now
    }

    /// Type check an async expression
    fn check_async_expression(&mut self, async_expr: &AsyncExpr) -> Result<TypeId> {
        // Check the inner expression
        let expr_type = self.check_expression(&async_expr.expr)?;

        // Wrap the result type in a Promise
        let promise_id = self.type_registry.register_promise_type(expr_type);
        Ok(TypeId::Promise(promise_id))
    }

    /// Type check an await expression
    fn check_await_expression(&mut self, await_expr: &AwaitExpr) -> Result<TypeId> {
        // Check the inner expression
        let expr_type = self.check_expression(&await_expr.expr)?;

        // The expression must be a Promise type
        match expr_type {
            TypeId::Promise(promise_id) => {
                // Get the result type from the Promise
                if let Some(composite_type) = self.type_registry.get_composite_type(promise_id) {
                    if let crate::types::composite::CompositeTypeId::Promise(result_type) =
                        composite_type
                    {
                        Ok(**result_type)
                    } else {
                        Err(BuluError::TypeError {
            file: None,
                            message: "Internal error: Promise type ID does not map to Promise composite type".to_string(),
                            line: await_expr.position.line,
                            column: await_expr.position.column,
                        })
                    }
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: "Internal error: Promise type ID not found in registry"
                            .to_string(),
                        line: await_expr.position.line,
                        column: await_expr.position.column,
                    })
                }
            }
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot await non-Promise type: {:?}", expr_type),
                line: await_expr.position.line,
                column: await_expr.position.column,
            }),
        }
    }

    /// Enter a new scope
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Add a symbol to the current scope
    fn add_symbol(&mut self, symbol: Symbol) -> Result<()> {
        if let Some(current_scope) = self.scopes.last_mut() {
            if current_scope.contains_key(&symbol.name) {
                return Err(BuluError::TypeError {
                    file: None,
                    message: format!(
                        "Variable '{}' is already defined in this scope",
                        symbol.name
                    ),
                    line: symbol.position.line,
                    column: symbol.position.column,
                });
            }
            current_scope.insert(symbol.name.clone(), symbol);
        }
        Ok(())
    }

    /// Look up a symbol in the scope stack
    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Get all errors accumulated during type checking
    pub fn get_errors(&self) -> &[BuluError] {
        &self.errors
    }

    /// Import symbols from a SymbolResolver into the global scope
    pub fn import_symbols_from_resolver(
        &mut self,
        symbol_resolver: &crate::compiler::symbol_resolver::SymbolResolver,
    ) {
        let symbol_table = symbol_resolver.symbol_table();

        // First, import struct declarations from loaded modules
        for module in symbol_resolver.get_loaded_modules() {
            for statement in &module.ast.statements {
                match statement {
                    Statement::StructDecl(struct_decl) if struct_decl.is_exported => {
                        // Add the struct declaration to our structs collection
                        self.structs.insert(struct_decl.name.clone(), struct_decl.clone());
                    }
                    Statement::Export(export_stmt) => {
                        // Check if this is an exported struct
                        if let Statement::StructDecl(struct_decl) = export_stmt.item.as_ref() {
                            self.structs.insert(struct_decl.name.clone(), struct_decl.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Only import imported symbols (not local symbols, as they are handled by the TypeChecker itself)
        for (name, imported_symbol) in &symbol_table.imported_symbols {
            let symbol = match imported_symbol.symbol_type {
                crate::compiler::symbol_resolver::SymbolType::Function => {
                    let function_info =
                        if let Some(ref signature) = imported_symbol.function_signature {
                            // Extract parameter types from function signature
                            let param_types: Vec<TypeId> = signature
                                .parameters
                                .iter()
                                .map(|param| self.convert_ast_type_to_type_id(&param.param_type))
                                .collect();

                            // Extract return type from function signature
                            let return_type = signature
                                .return_type
                                .as_ref()
                                .map(|rt| self.convert_ast_type_to_type_id(rt));

                            Some(FunctionInfo {
                                param_types,
                                return_type,
                            })
                        } else {
                            // Fallback for functions without signature info
                            Some(FunctionInfo {
                                param_types: Vec::new(),
                                return_type: Some(TypeId::Any),
                            })
                        };

                    Symbol {
                        name: name.clone(),
                        type_id: TypeId::Function(0), // Use placeholder ID
                        is_mutable: false,
                        position: imported_symbol.position,
                        function_info,
                    }
                }
                crate::compiler::symbol_resolver::SymbolType::Variable => {
                    let type_id = if let Some(ref type_info) = imported_symbol.type_info {
                        self.ast_type_to_type_id(type_info)
                    } else {
                        TypeId::Any // Fallback if no type info available
                    };
                    
                    Symbol {
                        name: name.clone(),
                        type_id,
                        is_mutable: imported_symbol.is_mutable,
                        position: imported_symbol.position,
                        function_info: None,
                    }
                }
                crate::compiler::symbol_resolver::SymbolType::Constant => {
                    let type_id = if let Some(ref type_info) = imported_symbol.type_info {
                        self.ast_type_to_type_id(type_info)
                    } else {
                        // Fallback to the old method if no type info available
                        self.infer_constant_type_from_modules(name, symbol_resolver)
                            .unwrap_or(TypeId::Any)
                    };
                    
                    Symbol {
                        name: name.clone(),
                        type_id,
                        is_mutable: false,
                        position: imported_symbol.position,
                        function_info: None,
                    }
                }
                crate::compiler::symbol_resolver::SymbolType::Struct => {
                    // Create a proper TypeId for the imported struct
                    let struct_type_id = self.get_or_create_named_type_id(name, false);
                    Symbol {
                        name: name.clone(),
                        type_id: struct_type_id,
                        is_mutable: false,
                        position: imported_symbol.position,
                        function_info: None,
                    }
                }
                crate::compiler::symbol_resolver::SymbolType::Interface => {
                    Symbol {
                        name: name.clone(),
                        type_id: TypeId::Interface(0), // Use placeholder ID
                        is_mutable: false,
                        position: imported_symbol.position,
                        function_info: None,
                    }
                }
                crate::compiler::symbol_resolver::SymbolType::TypeAlias => {
                    let type_id = if let Some(ref type_info) = imported_symbol.type_info {
                        self.ast_type_to_type_id(type_info)
                    } else {
                        TypeId::Any
                    };
                    
                    Symbol {
                        name: name.clone(),
                        type_id,
                        is_mutable: false,
                        position: imported_symbol.position,
                        function_info: None,
                    }
                }
                crate::compiler::symbol_resolver::SymbolType::Module => Symbol {
                    name: name.clone(),
                    type_id: TypeId::Any,
                    is_mutable: false,
                    position: imported_symbol.position,
                    function_info: None,
                },
            };

            // Add to global scope (first scope in the stack)
            if let Some(global_scope) = self.scopes.first_mut() {
                global_scope.insert(name.clone(), symbol);
            }
        }
    }

    /// Infer the type of a constant from loaded modules
    fn infer_constant_type_from_modules(
        &mut self,
        constant_name: &str,
        symbol_resolver: &crate::compiler::symbol_resolver::SymbolResolver,
    ) -> Option<TypeId> {
        // Search through loaded modules to find the constant declaration
        for module in symbol_resolver.get_loaded_modules() {
            for statement in &module.ast.statements {
                match statement {
                    Statement::VariableDecl(var_decl) if var_decl.is_const && var_decl.name == constant_name => {
                        // Found the constant declaration, extract its type
                        if let Some(ref type_annotation) = var_decl.type_annotation {
                            return Some(self.ast_type_to_type_id(type_annotation));
                        } else if let Some(ref initializer) = var_decl.initializer {
                            // Infer type from initializer
                            return Some(self.infer_type_from_expression(initializer));
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }
    
    /// Infer type from an expression (simplified version)
    fn infer_type_from_expression(&self, expr: &Expression) -> TypeId {
        match expr {
            Expression::Literal(lit) => {
                match &lit.value {
                    crate::ast::LiteralValue::Integer(_) => TypeId::Int32,
                    crate::ast::LiteralValue::Float(_) => TypeId::Float64,
                    crate::ast::LiteralValue::String(_) => TypeId::String,
                    crate::ast::LiteralValue::Boolean(_) => TypeId::Bool,
                    crate::ast::LiteralValue::Char(_) => TypeId::Char,
                    crate::ast::LiteralValue::Null => TypeId::Any,
                }
            }
            _ => TypeId::Any, // For complex expressions, default to Any
        }
    }

    /// Convert AST Type to TypeId
    fn convert_ast_type_to_type_id(&self, ast_type: &Type) -> TypeId {
        match ast_type {
            Type::Int8 => TypeId::Int8,
            Type::Int16 => TypeId::Int16,
            Type::Int32 => TypeId::Int32,
            Type::Int64 => TypeId::Int64,
            Type::UInt8 => TypeId::UInt8,
            Type::UInt16 => TypeId::UInt16,
            Type::UInt32 => TypeId::UInt32,
            Type::UInt64 => TypeId::UInt64,
            Type::Float32 => TypeId::Float32,
            Type::Float64 => TypeId::Float64,
            Type::Bool => TypeId::Bool,
            Type::Char => TypeId::Char,
            Type::String => TypeId::String,
            Type::Any => TypeId::Any,
            Type::Void => TypeId::Void,
            Type::Array(_) => TypeId::Array(0), // Use placeholder ID for now
            Type::Slice(_) => TypeId::Slice(0), // Use placeholder ID for now
            Type::Map(_) => TypeId::Map(0),     // Use placeholder ID for now
            Type::Function(_) => TypeId::Function(0), // Use placeholder ID for now
            Type::Named(_) => TypeId::Any,      // Custom types - simplified for now
            Type::Generic(_) => TypeId::Any,    // Generic types - simplified for now
            _ => TypeId::Any,                   // Fallback for other types
        }
    }

    /// Find an interface declaration by name
    fn find_interface_declaration(&self, interface_name: &str) -> Option<&InterfaceDecl> {
        self.interfaces.get(interface_name)
    }

    /// Find a struct declaration by name
    fn find_struct_declaration(&self, struct_name: &str) -> Option<&StructDecl> {
        self.structs.get(struct_name)
    }

    /// Check if a struct implements an interface
    fn struct_implements_interface(&self, struct_name: &str, interface_name: &str) -> bool {
        let struct_decl = match self.find_struct_declaration(struct_name) {
            Some(decl) => decl,
            None => return false,
        };

        let interface_decl = match self.find_interface_declaration(interface_name) {
            Some(decl) => decl,
            None => return false,
        };

        // Check if the struct has all the methods required by the interface
        for interface_method in &interface_decl.methods {
            let mut found_method = false;

            for struct_method in &struct_decl.methods {
                if struct_method.name == interface_method.name {
                    // Check if the method signatures match
                    if self.method_signatures_match(struct_method, interface_method) {
                        found_method = true;
                        break;
                    }
                }
            }

            if !found_method {
                return false;
            }
        }

        true
    }

    /// Check if two method signatures match
    fn method_signatures_match(
        &self,
        struct_method: &FunctionDecl,
        interface_method: &InterfaceMethod,
    ) -> bool {
        // Check parameter count
        if struct_method.params.len() != interface_method.params.len() {
            return false;
        }

        // Check parameter types
        for (struct_param, interface_param) in struct_method
            .params
            .iter()
            .zip(interface_method.params.iter())
        {
            if !self.types_match(&struct_param.param_type, &interface_param.param_type) {
                return false;
            }
        }

        // Check return types
        match (&struct_method.return_type, &interface_method.return_type) {
            (Some(struct_ret), Some(interface_ret)) => {
                if !self.types_match(struct_ret, interface_ret) {
                    return false;
                }
            }
            (None, None) => {} // Both void
            _ => return false, // One void, one not
        }

        true
    }

    /// Check if two types match
    fn types_match(&self, type1: &Type, type2: &Type) -> bool {
        // For now, do a simple comparison
        // In a full implementation, this would handle type equivalence more thoroughly
        type1 == type2
    }

    /// Create or get a TypeId for a named type
    fn get_or_create_named_type_id(&mut self, name: &str, is_interface: bool) -> TypeId {
        if let Some(&type_id) = self.type_name_to_id.get(name) {
            return type_id;
        }

        let type_id = if is_interface {
            TypeId::Interface(self.next_type_id)
        } else {
            TypeId::Struct(self.next_type_id)
        };

        self.next_type_id += 1;
        self.type_name_to_id.insert(name.to_string(), type_id);
        self.type_id_to_name.insert(type_id, name.to_string());
        type_id
    }

    /// Get the type name from a TypeId
    fn get_type_name_from_id(&self, type_id: TypeId) -> Option<&String> {
        self.type_id_to_name.get(&type_id)
    }

    /// Check if actual_type is compatible with expected_type (including interface implementation)
    fn is_type_compatible(&self, actual_type: TypeId, expected_type: TypeId) -> bool {
        // Direct type match
        if actual_type == expected_type {
            return true;
        }

        // Check primitive type compatibility
        if PrimitiveType::is_assignable(actual_type, expected_type) {
            return true;
        }

        // Check if a struct implements an interface
        match (actual_type, expected_type) {
            (TypeId::Struct(_), TypeId::Interface(_)) => {
                // Get the struct and interface names
                if let (Some(struct_name), Some(interface_name)) = (
                    self.get_type_name_from_id(actual_type),
                    self.get_type_name_from_id(expected_type),
                ) {
                    return self.struct_implements_interface(struct_name, interface_name);
                }
            }
            _ => {}
        }

        false
    }

    /// Get a user-friendly type name for error messages
    fn type_name_for_error(&self, type_id: TypeId) -> String {
        if let Some(name) = self.get_type_name_from_id(type_id) {
            match type_id {
                TypeId::Interface(_) => format!("interface {}", name),
                TypeId::Struct(_) => format!("struct {}", name),
                _ => name.clone(),
            }
        } else {
            PrimitiveType::type_name(type_id).to_string()
        }
    }

    /// Get the type name from an expression (for named types like interfaces/structs)
    fn get_type_name_from_expression(&self, expr: &Expression) -> Result<Option<String>> {
        match expr {
            Expression::Identifier(ident) => {
                if let Some(symbol) = self.lookup_symbol(&ident.name) {
                    Ok(self.get_type_name_from_id(symbol.type_id).cloned())
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Check a pattern and add variables to the current scope
    fn check_pattern_and_add_variables(&mut self, pattern: &Pattern, value_type: TypeId) -> Result<()> {
        match pattern {
            Pattern::Identifier(name, position) => {
                // Add the variable to the current scope
                let symbol = Symbol {
                    name: name.clone(),
                    type_id: value_type,
                    is_mutable: true, // TODO: get from declaration
                    position: *position,
                    function_info: None,
                };
                self.add_symbol(symbol)?;
            }
            Pattern::Struct(struct_pattern) => {
                // For struct destructuring, we need to check that the value type is a struct
                // and extract field types
                match value_type {
                    TypeId::Struct(_struct_id) => {
                        // Get the struct definition to know field types
                        // For now, we'll look up by struct name since we don't have a direct ID->struct mapping
                        let struct_def = self.structs.get(&struct_pattern.name).cloned();
                        if let Some(struct_def) = struct_def {
                            for field_pattern in &struct_pattern.fields {
                                // Find the field type in the struct definition
                                if let Some(field_def) = struct_def.fields.iter().find(|f| f.name == field_pattern.name) {
                                    let field_type = self.ast_type_to_type_id(&field_def.field_type);
                                    self.check_pattern_and_add_variables(&field_pattern.pattern, field_type)?;
                                } else {
                                    return Err(BuluError::TypeError {
                                        message: format!("Field '{}' not found in struct", field_pattern.name),
                                        line: field_pattern.position.line,
                                        column: field_pattern.position.column,
                                        file: None,
                                    });
                                }
                            }
                        } else {
                            return Err(BuluError::TypeError {
                                message: "Unknown struct type in destructuring".to_string(),
                                line: 0,
                                column: 0,
                                file: None,
                            });
                        }
                    }
                    TypeId::Any => {
                        // For Any type (like object literals), assume all fields exist and are Any
                        for field_pattern in &struct_pattern.fields {
                            self.check_pattern_and_add_variables(&field_pattern.pattern, TypeId::Any)?;
                        }
                    }
                    _ => {
                        return Err(BuluError::TypeError {
                            message: format!("Cannot destructure non-struct type"),
                            line: 0,
                            column: 0,
                            file: None,
                        });
                    }
                }
            }
            Pattern::Array(array_pattern) => {
                // For array destructuring, extract element type
                let element_type = match value_type {
                    TypeId::Array(_element_type_id) => {
                        // For now, assume Any type for array elements
                        // TODO: Implement proper type registry lookup
                        TypeId::Any
                    }
                    TypeId::Any => TypeId::Any,
                    _ => {
                        return Err(BuluError::TypeError {
                            message: "Cannot destructure non-array type".to_string(),
                            line: 0,
                            column: 0,
                            file: None,
                        });
                    }
                };
                
                for element_pattern in &array_pattern.elements {
                    self.check_pattern_and_add_variables(element_pattern, element_type)?;
                }
            }
            Pattern::Or(or_pattern) => {
                // For OR patterns, all alternatives should bind the same variables with the same types
                for alternative in &or_pattern.patterns {
                    self.check_pattern_and_add_variables(alternative, value_type)?;
                }
            }
            Pattern::Wildcard(_) | Pattern::Literal(_, _) | Pattern::Range(_) => {
                // These patterns don't bind variables
            }
        }
        Ok(())
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
