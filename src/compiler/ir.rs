//! Intermediate Representation (IR) for the Bulu language
//!
//! This module defines the IR instruction set and provides functionality
//! for translating AST to IR, optimizing IR, and analyzing control flow.

use crate::ast::*;
use crate::error::{BuluError, Result};
use crate::lexer::token::Position;
use std::collections::HashMap;
use std::fmt;

/// A complete IR program
#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub structs: Vec<IrStruct>,
    pub interfaces: Vec<IrInterface>,
}

/// IR function representation
#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParam>,
    pub return_type: Option<IrType>,
    pub locals: Vec<IrLocal>,
    pub basic_blocks: Vec<IrBasicBlock>,
    pub is_async: bool,
    pub position: Position,
}

/// IR function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct IrParam {
    pub name: String,
    pub param_type: IrType,
    pub register: IrRegister,
}

/// IR local variable
#[derive(Debug, Clone, PartialEq)]
pub struct IrLocal {
    pub name: String,
    pub local_type: IrType,
    pub register: IrRegister,
    pub is_mutable: bool,
}

/// IR global variable
#[derive(Debug, Clone, PartialEq)]
pub struct IrGlobal {
    pub name: String,
    pub global_type: IrType,
    pub initializer: Option<IrValue>,
    pub is_const: bool,
    pub position: Position,
}

/// IR struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct IrStruct {
    pub name: String,
    pub fields: Vec<IrStructField>,
    pub methods: Vec<String>, // Function names
    pub position: Position,
}

/// IR struct field
#[derive(Debug, Clone, PartialEq)]
pub struct IrStructField {
    pub name: String,
    pub field_type: IrType,
    pub offset: usize,
}

/// IR interface definition
#[derive(Debug, Clone, PartialEq)]
pub struct IrInterface {
    pub name: String,
    pub methods: Vec<IrInterfaceMethod>,
    pub position: Position,
}

/// IR interface method
#[derive(Debug, Clone, PartialEq)]
pub struct IrInterfaceMethod {
    pub name: String,
    pub params: Vec<IrType>,
    pub return_type: Option<IrType>,
}

/// Basic block in IR
#[derive(Debug, Clone, PartialEq)]
pub struct IrBasicBlock {
    pub label: String,
    pub instructions: Vec<IrInstruction>,
    pub terminator: IrTerminator,
}

/// IR instruction
#[derive(Debug, Clone, PartialEq)]
pub struct IrInstruction {
    pub opcode: IrOpcode,
    pub result: Option<IrRegister>,
    pub operands: Vec<IrValue>,
    pub position: Position,
}

/// IR terminator instruction (ends a basic block)
#[derive(Debug, Clone, PartialEq)]
pub enum IrTerminator {
    Return(Option<IrValue>),
    Branch(String), // Unconditional branch to label
    ConditionalBranch {
        condition: IrValue,
        true_label: String,
        false_label: String,
    },
    Switch {
        value: IrValue,
        cases: Vec<(IrValue, String)>,
        default_label: Option<String>,
    },
    Unreachable,
}

/// IR opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrOpcode {
    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Neg,

    // Bitwise operations
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,

    // Comparison operations
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical operations
    LogicalAnd,
    LogicalOr,
    LogicalNot,

    // Memory operations
    Load,
    Store,
    Alloca,

    // Type operations
    Cast,
    TypeOf,
    IsNull,

    // Function operations
    Call,
    CallIndirect,

    // Array/slice operations
    ArrayAccess,
    ArrayLength,
    SliceAccess,
    SliceLength,

    // Map operations
    MapAccess,
    MapInsert,
    MapDelete,
    MapLength,

    // Channel operations
    ChannelSend,
    ChannelReceive,
    ChannelClose,
    ChannelSelect,

    // Concurrency operations
    Spawn,
    Await,

    // Control flow
    Phi, // SSA phi node

    // Struct operations
    StructAccess,
    StructConstruct,
    RegisterStruct,

    // Tuple operations
    TupleAccess,
    TupleConstruct,

    // String operations
    StringConcat,
    StringLength,

    // Utility operations
    Copy,
    Move,
    Clone,

    // Generator operations
    Yield,
    GeneratorNext,

    // Error handling
    Throw,
    Catch,
}

/// IR register (virtual register in SSA form)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IrRegister {
    pub id: u32,
}

/// IR value (operand)
#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    Register(IrRegister),
    Constant(IrConstant),
    Global(String),
    Function(String),
}

/// IR constant value
#[derive(Debug, Clone, PartialEq)]
pub enum IrConstant {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    Null,
    Array(Vec<IrConstant>),
    Struct(Vec<IrConstant>),
    Tuple(Vec<IrConstant>),
}

/// IR type system
#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    // Primitive types
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    String,
    Any,
    Void,

    // Composite types
    Array(Box<IrType>, Option<usize>), // element type, size
    Slice(Box<IrType>),
    Map(Box<IrType>, Box<IrType>), // key type, value type
    Tuple(Vec<IrType>),
    Function(Vec<IrType>, Option<Box<IrType>>), // params, return type

    // User-defined types
    Struct(String),
    Interface(String),

    // Channel types
    Channel(Box<IrType>),

    // Async types
    Promise(Box<IrType>),

    // Pointer type (for FFI and unsafe operations)
    Pointer(Box<IrType>),
}

/// Control flow graph for analysis
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub nodes: Vec<CfgNode>,
    pub edges: Vec<CfgEdge>,
}

/// Control flow graph node
#[derive(Debug, Clone)]
pub struct CfgNode {
    pub id: usize,
    pub block_label: String,
    pub predecessors: Vec<usize>,
    pub successors: Vec<usize>,
}

/// Control flow graph edge
#[derive(Debug, Clone)]
pub struct CfgEdge {
    pub from: usize,
    pub to: usize,
    pub condition: Option<IrValue>, // For conditional branches
}

/// AST to IR translator
pub struct IrGenerator {
    next_register_id: u32,
    next_block_id: u32,
    current_function: Option<String>,
    pub register_map: HashMap<String, IrRegister>, // Variable name to register mapping
    block_stack: Vec<String>,                      // Stack of block labels for break/continue

    // Current function being generated
    current_function_blocks: Vec<IrBasicBlock>,
    current_block_instructions: Vec<IrInstruction>,
    current_block_label: Option<String>,

    // Control flow management
    break_labels: Vec<String>,    // Stack of break target labels
    continue_labels: Vec<String>, // Stack of continue target labels
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {
            next_register_id: 0,
            next_block_id: 0,
            current_function: None,
            register_map: HashMap::new(),
            block_stack: Vec::new(),
            current_function_blocks: Vec::new(),
            current_block_instructions: Vec::new(),
            current_block_label: None,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
        }
    }

    /// Helper function to create an error with position information
    fn error(&self, message: String, position: Position) -> BuluError {
        BuluError::Other(format!(
            "{} at line {}, column {}",
            message, position.line, position.column
        ))
    }

    /// Generate IR from AST program
    pub fn generate(&mut self, program: &Program) -> Result<IrProgram> {
        let mut ir_program = IrProgram {
            functions: Vec::new(),
            globals: Vec::new(),
            structs: Vec::new(),
            interfaces: Vec::new(),
        };

        // Process all top-level declarations
        for statement in &program.statements {
            match statement {
                Statement::FunctionDecl(func_decl) => {
                    let ir_function = self.generate_function(func_decl)?;
                    ir_program.functions.push(ir_function);
                }

                Statement::VariableDecl(var_decl) => {
                    let ir_global = self.generate_global(var_decl)?;
                    ir_program.globals.push(ir_global);
                }

                Statement::DestructuringDecl(_) => {
                    // Destructuring declarations at global scope are not supported yet
                    return Err(BuluError::RuntimeError {
                        message: "Global destructuring declarations are not supported".to_string(),
                        file: None,
                    });
                }

                Statement::MultipleVariableDecl(decl) => {
                    // Generate globals for each variable in the multiple declaration
                    for var_decl in &decl.declarations {
                        // Create a temporary VariableDecl for generate_global
                        let temp_var_decl = VariableDecl {
                            is_const: decl.is_const,
                            name: var_decl.name.clone(),
                            type_annotation: var_decl.type_annotation.clone(),
                            initializer: var_decl.initializer.clone(),
                            doc_comment: None,
                            is_exported: decl.is_exported,
                            position: decl.position,
                        };
                        let ir_global = self.generate_global(&temp_var_decl)?;
                        ir_program.globals.push(ir_global);
                    }
                }

                Statement::StructDecl(struct_decl) => {
                    let ir_struct = self.generate_struct(struct_decl)?;
                    ir_program.structs.push(ir_struct);

                    // Generate method functions
                    for method in &struct_decl.methods {
                        let method_function = self.generate_method_function(struct_decl, method)?;
                        ir_program.functions.push(method_function);
                    }
                }

                Statement::InterfaceDecl(interface_decl) => {
                    let ir_interface = self.generate_interface(interface_decl)?;
                    ir_program.interfaces.push(ir_interface);
                }

                Statement::Import(_) => {
                    // Import statements are handled by the symbol resolver
                    // and don't generate IR code
                }

                Statement::Export(export_stmt) => {
                    // Export statements wrap other declarations
                    // Process the wrapped statement
                    match export_stmt.item.as_ref() {
                        Statement::FunctionDecl(func_decl) => {
                            let ir_function = self.generate_function(func_decl)?;
                            ir_program.functions.push(ir_function);
                        }
                        Statement::VariableDecl(var_decl) => {
                            let ir_global = self.generate_global(var_decl)?;
                            ir_program.globals.push(ir_global);
                        }
                        Statement::DestructuringDecl(_) => {
                            // Destructuring declarations at global scope are not supported yet
                            return Err(BuluError::RuntimeError {
                                message: "Global destructuring declarations are not supported"
                                    .to_string(),
                                file: None,
                            });
                        }
                        Statement::MultipleVariableDecl(decl) => {
                            // Generate globals for each variable in the multiple declaration
                            for var_decl in &decl.declarations {
                                // Create a temporary VariableDecl for generate_global
                                let temp_var_decl = VariableDecl {
                                    is_const: decl.is_const,
                                    name: var_decl.name.clone(),
                                    type_annotation: var_decl.type_annotation.clone(),
                                    initializer: var_decl.initializer.clone(),
                                    doc_comment: None,
                                    is_exported: decl.is_exported,
                                    position: decl.position,
                                };
                                let ir_global = self.generate_global(&temp_var_decl)?;
                                ir_program.globals.push(ir_global);
                            }
                        }
                        Statement::StructDecl(struct_decl) => {
                            let ir_struct = self.generate_struct(struct_decl)?;
                            ir_program.structs.push(ir_struct);
                        }
                        Statement::InterfaceDecl(interface_decl) => {
                            let ir_interface = self.generate_interface(interface_decl)?;
                            ir_program.interfaces.push(ir_interface);
                        }
                        _ => {
                            return Err(self.error(
                                format!("Unsupported export statement: {:?}", export_stmt.item),
                                export_stmt.position,
                            ));
                        }
                    }
                }

                _ => {
                    return Err(self.error(
                        format!("Unsupported top-level statement: {:?}", statement),
                        statement.position(),
                    ));
                }
            }
        }

        Ok(ir_program)
    }

    /// Generate IR function from AST function declaration
    pub fn generate_function(&mut self, func_decl: &FunctionDecl) -> Result<IrFunction> {
        // Reset function-level state
        self.current_function = Some(func_decl.name.clone());
        self.register_map.clear();
        self.next_register_id = 0;
        self.next_block_id = 0;
        self.current_function_blocks.clear();
        self.current_block_instructions.clear();
        self.current_block_label = None;
        self.break_labels.clear();
        self.continue_labels.clear();

        // Generate parameters
        let mut params = Vec::new();
        for param in &func_decl.params {
            let register = self.new_register();
            let ir_type = self.convert_type(&param.param_type)?;
            params.push(IrParam {
                name: param.name.clone(),
                param_type: ir_type,
                register,
            });
            self.register_map.insert(param.name.clone(), register);
        }

        // Start entry block
        let entry_label = self.next_block_label();
        self.start_block(entry_label);

        // Generate function body
        self.generate_block_statement(&func_decl.body)?;

        // Ensure function ends with a return
        if self.current_block_label.is_some() {
            // Add implicit return if no explicit return
            self.emit_return(None);
        }

        let return_type = func_decl
            .return_type
            .as_ref()
            .map(|t| self.convert_type(t))
            .transpose()?;

        Ok(IrFunction {
            name: func_decl.name.clone(),
            params,
            return_type,
            locals: self.collect_locals(&func_decl.body),
            basic_blocks: std::mem::take(&mut self.current_function_blocks),
            is_async: func_decl.is_async,
            position: func_decl.position,
        })
    }

    /// Generate IR global from AST variable declaration
    pub fn generate_global(&mut self, var_decl: &VariableDecl) -> Result<IrGlobal> {
        let global_type = if let Some(ref type_annotation) = var_decl.type_annotation {
            self.convert_type(type_annotation)?
        } else if let Some(ref init_expr) = var_decl.initializer {
            // Infer type from initializer
            self.infer_type_from_expression(init_expr)?
        } else {
            IrType::Any
        };

        let initializer = if let Some(ref init_expr) = var_decl.initializer {
            Some(self.evaluate_constant_expression(init_expr)?)
        } else {
            None
        };

        Ok(IrGlobal {
            name: var_decl.name.clone(),
            global_type,
            initializer,
            is_const: var_decl.is_const,
            position: var_decl.position,
        })
    }

    /// Generate IR struct from AST struct declaration
    pub fn generate_struct(&mut self, struct_decl: &StructDecl) -> Result<IrStruct> {
        let mut fields = Vec::new();
        let mut offset = 0;

        for field in &struct_decl.fields {
            let field_type = self.convert_type(&field.field_type)?;
            let type_size = self.calculate_type_size(&field_type);
            fields.push(IrStructField {
                name: field.name.clone(),
                field_type,
                offset,
            });
            offset += type_size;
        }

        let methods = struct_decl
            .methods
            .iter()
            .map(|method| method.name.clone())
            .collect();

        Ok(IrStruct {
            name: struct_decl.name.clone(),
            fields,
            methods,
            position: struct_decl.position,
        })
    }

    /// Generate IR function for a struct method
    fn generate_method_function(
        &mut self,
        struct_decl: &StructDecl,
        method: &FunctionDecl,
    ) -> Result<IrFunction> {
        // Create method function name: StructName.methodName
        let function_name = format!("{}.{}", struct_decl.name, method.name);

        // Create parameters: 'this' parameter + method parameters
        let mut params = Vec::new();

        // Add 'this' parameter
        params.push(IrParam {
            name: "this".to_string(),
            param_type: IrType::Struct(struct_decl.name.clone()),
            register: IrRegister { id: 0 },
        });

        // Add method parameters
        for (i, param) in method.params.iter().enumerate() {
            params.push(IrParam {
                name: param.name.clone(),
                param_type: self.convert_type(&param.param_type)?,
                register: IrRegister { id: (i + 1) as u32 },
            });
        }

        // Generate method body
        let old_next_register_id = self.next_register_id;
        let old_next_block_id = self.next_block_id; // Save block ID state
        self.next_register_id = params.len() as u32; // Start after parameters
        self.next_block_id = 0; // Reset block ID for this method

        let mut basic_blocks = Vec::new();

        // Save current state
        let old_function_blocks = std::mem::take(&mut self.current_function_blocks);
        let old_block_label = self.current_block_label.take();
        let old_block_instructions = std::mem::take(&mut self.current_block_instructions);
        let old_register_map = self.register_map.clone();

        // Set up register map with method parameters
        self.register_map.clear();
        for param in &params {
            self.register_map.insert(param.name.clone(), param.register);
        }

        // Start a new basic block for the method using proper label generation
        let entry_label = self.next_block_label();
        self.start_block(entry_label);

        // Generate instructions for method body
        self.generate_block_statement(&method.body)?;

        // Ensure method ends with a return
        if self.current_block_label.is_some() {
            // Add implicit return if no explicit return
            self.emit_return(None);
        }

        // Collect the generated basic blocks
        basic_blocks = std::mem::take(&mut self.current_function_blocks);

        // Restore previous state
        self.current_function_blocks = old_function_blocks;
        self.current_block_label = old_block_label;
        self.current_block_instructions = old_block_instructions;
        self.register_map = old_register_map;

        self.next_register_id = old_next_register_id;
        self.next_block_id = old_next_block_id; // Restore block ID state

        Ok(IrFunction {
            name: function_name,
            params,
            return_type: method
                .return_type
                .as_ref()
                .map(|t| self.convert_type(t))
                .transpose()?,
            locals: Vec::new(),
            basic_blocks,
            is_async: method.is_async,
            position: method.position,
        })
    }

    /// Generate IR interface from AST interface declaration
    fn generate_interface(&mut self, interface_decl: &InterfaceDecl) -> Result<IrInterface> {
        let mut methods = Vec::new();

        for method in &interface_decl.methods {
            let params = method
                .params
                .iter()
                .map(|p| self.convert_type(&p.param_type))
                .collect::<Result<Vec<_>>>()?;

            let return_type = method
                .return_type
                .as_ref()
                .map(|t| self.convert_type(t))
                .transpose()?;

            methods.push(IrInterfaceMethod {
                name: method.name.clone(),
                params,
                return_type,
            });
        }

        Ok(IrInterface {
            name: interface_decl.name.clone(),
            methods,
            position: interface_decl.position,
        })
    }

    /// Generate instructions for a block statement
    fn generate_block_statement(&mut self, block: &BlockStmt) -> Result<()> {
        // Create new scope for variables
        let saved_register_map = self.register_map.clone();

        for statement in &block.statements {
            self.generate_statement(statement)?;
        }

        // Restore previous scope
        self.register_map = saved_register_map;
        Ok(())
    }

    /// Generate instructions for a statement
    fn generate_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::VariableDecl(var_decl) => {
                self.generate_variable_declaration(var_decl)?;
            }

            Statement::DestructuringDecl(decl) => {
                self.generate_destructuring_declaration(decl)?;
            }

            Statement::MultipleVariableDecl(decl) => {
                self.generate_multiple_variable_declaration(decl)?;
            }

            Statement::MultipleAssignment(stmt) => {
                self.generate_multiple_assignment_statement(stmt)?;
            }

            Statement::Expression(expr_stmt) => {
                self.generate_expression(&expr_stmt.expr)?;
            }

            Statement::Return(return_stmt) => {
                let return_value = if let Some(ref value_expr) = return_stmt.value {
                    Some(self.generate_expression(value_expr)?)
                } else {
                    None
                };
                self.emit_return(return_value);
            }

            Statement::If(if_stmt) => {
                self.generate_if_statement(if_stmt)?;
            }

            Statement::While(while_stmt) => {
                self.generate_while_statement(while_stmt)?;
            }

            Statement::For(for_stmt) => {
                self.generate_for_statement(for_stmt)?;
            }

            Statement::Block(block_stmt) => {
                self.generate_block_statement(block_stmt)?;
            }

            Statement::Break(_) => {
                if let Some(break_label) = self.break_labels.last() {
                    self.emit_branch(break_label.clone());
                }
            }

            Statement::Continue(_) => {
                if let Some(continue_label) = self.continue_labels.last() {
                    self.emit_branch(continue_label.clone());
                }
            }

            Statement::Match(match_stmt) => {
                self.generate_match_statement(match_stmt)?;
            }

            Statement::Try(try_stmt) => {
                self.generate_try_statement(try_stmt)?;
            }

            Statement::Defer(defer_stmt) => {
                // Defer statements need special handling - for now, just generate the deferred statement
                // In a full implementation, this would be added to a defer stack
                self.generate_statement(&defer_stmt.stmt)?;
            }

            Statement::Fail(fail_stmt) => {
                let message = self.generate_expression(&fail_stmt.message)?;
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Throw,
                    result: None,
                    operands: vec![message],
                    position: fail_stmt.position,
                });
            }

            Statement::Import(_) | Statement::Export(_) => {
                // Import/export statements are handled at the module level
                // No IR generation needed at runtime
            }

            Statement::TypeAlias(_) => {
                // Type aliases are compile-time constructs
                // No IR generation needed
            }

            _ => {
                // For any remaining statement types, do nothing
            }
        }
        Ok(())
    }

    /// Generate instructions for variable declaration
    pub fn generate_variable_declaration(&mut self, var_decl: &VariableDecl) -> Result<()> {
        // Allocate register for the variable
        let register = self.new_register();
        self.register_map.insert(var_decl.name.clone(), register);

        // Generate initializer if present, otherwise use default value
        let init_value = if let Some(ref initializer) = var_decl.initializer {
            self.generate_expression(initializer)?
        } else {
            // Use default value based on type annotation
            if let Some(ref type_annotation) = var_decl.type_annotation {
                self.get_default_value_for_type(type_annotation)
            } else {
                IrValue::Constant(IrConstant::Null)
            }
        };

        // Generate copy instruction to assign the value
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Copy,
            result: Some(register),
            operands: vec![init_value],
            position: var_decl.position,
        });

        Ok(())
    }

    /// Generate instructions for multiple assignment statement
    pub fn generate_multiple_assignment_statement(
        &mut self,
        stmt: &MultipleAssignmentStmt,
    ) -> Result<()> {
        // First, evaluate all the values
        let mut value_registers = Vec::new();
        for value_expr in &stmt.values {
            let value_reg = self.generate_expression(value_expr)?;
            value_registers.push(value_reg);
        }

        // Create temporary registers to hold all values to avoid overwriting
        let mut temp_registers = Vec::new();
        for (i, _) in stmt.targets.iter().enumerate() {
            let value_reg = if i < value_registers.len() {
                value_registers[i].clone()
            } else {
                // If there are more targets than values, assign null
                let null_reg = self.new_register();
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(null_reg.clone()),
                    operands: vec![IrValue::Constant(IrConstant::Null)],
                    position: stmt.position,
                });
                IrValue::Register(null_reg)
            };

            // Copy to a temporary register
            let temp_reg = self.new_register();
            self.emit_instruction(IrInstruction {
                opcode: IrOpcode::Copy,
                result: Some(temp_reg.clone()),
                operands: vec![value_reg],
                position: stmt.position,
            });
            temp_registers.push(IrValue::Register(temp_reg));
        }

        // Then assign from temporary registers to the targets
        for (i, target_expr) in stmt.targets.iter().enumerate() {
            let temp_reg = &temp_registers[i];

            // For now, only support identifier targets
            match target_expr {
                Expression::Identifier(ident) => {
                    if let Some(&register) = self.register_map.get(&ident.name) {
                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::Move,
                            result: Some(register),
                            operands: vec![temp_reg.clone()],
                            position: stmt.position,
                        });
                    }
                }
                _ => {
                    return Err(BuluError::RuntimeError {
                        message: "Complex assignment targets not yet supported in IR generation"
                            .to_string(),
                        file: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Generate instructions for destructuring declaration
    pub fn generate_destructuring_declaration(&mut self, decl: &DestructuringDecl) -> Result<()> {
        // Generate the initializer expression
        let init_value = self.generate_expression(&decl.initializer)?;

        // Generate pattern assignment
        self.generate_pattern_assignment(&decl.pattern, init_value)?;

        Ok(())
    }

    /// Generate instructions for multiple variable declaration
    pub fn generate_multiple_variable_declaration(
        &mut self,
        decl: &MultipleVariableDecl,
    ) -> Result<()> {
        for var_decl in &decl.declarations {
            // Allocate register for each variable
            let register = self.new_register();
            self.register_map.insert(var_decl.name.clone(), register);

            // Generate initializer if present, otherwise use default value
            let init_value = if let Some(ref initializer) = var_decl.initializer {
                self.generate_expression(initializer)?
            } else {
                // Use default value based on type annotation
                if let Some(ref type_annotation) = var_decl.type_annotation {
                    self.get_default_value_for_type(type_annotation)
                } else {
                    IrValue::Constant(IrConstant::Null)
                }
            };

            // Generate copy instruction to assign the value
            self.emit_instruction(IrInstruction {
                opcode: IrOpcode::Copy,
                result: Some(register),
                operands: vec![init_value],
                position: decl.position,
            });
        }

        Ok(())
    }

    /// Get default value for a type
    fn get_default_value_for_type(&self, type_annotation: &Type) -> IrValue {
        match type_annotation {
            Type::Int32 => IrValue::Constant(IrConstant::Integer(0)),
            Type::Int64 => IrValue::Constant(IrConstant::Integer(0)),
            Type::Float32 => IrValue::Constant(IrConstant::Float(0.0)),
            Type::Float64 => IrValue::Constant(IrConstant::Float(0.0)),
            Type::Bool => IrValue::Constant(IrConstant::Boolean(false)),
            Type::String => IrValue::Constant(IrConstant::String("".to_string())),
            Type::Char => IrValue::Constant(IrConstant::Char('\0')),
            _ => IrValue::Constant(IrConstant::Null),
        }
    }

    /// Generate pattern assignment for destructuring
    pub fn generate_pattern_assignment(&mut self, pattern: &Pattern, value: IrValue) -> Result<()> {
        match pattern {
            Pattern::Identifier(name, _) => {
                let register = self.new_register();
                self.register_map.insert(name.clone(), register);

                // Generate copy instruction
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Copy,
                    result: Some(register),
                    operands: vec![value],
                    position: Position::new(0, 0, 0),
                });
            }

            Pattern::Struct(struct_pattern) => {
                // For struct destructuring, we need to extract fields
                for field_pattern in &struct_pattern.fields {
                    // Generate field access
                    let field_value =
                        self.generate_field_access(value.clone(), &field_pattern.name)?;

                    // Recursively assign to the field pattern
                    self.generate_pattern_assignment(&field_pattern.pattern, field_value)?;
                }
            }

            Pattern::Array(array_pattern) => {
                // For array destructuring, we need to extract elements by index
                for (index, element_pattern) in array_pattern.elements.iter().enumerate() {
                    // Generate index access
                    let index_value = self.generate_index_access(value.clone(), index)?;

                    // Recursively assign to the element pattern
                    self.generate_pattern_assignment(element_pattern, index_value)?;
                }
            }

            Pattern::Tuple(tuple_pattern) => {
                // For tuple destructuring, we need to extract elements by index
                for (index, element_pattern) in tuple_pattern.elements.iter().enumerate() {
                    // Generate tuple element access
                    let element_value = self.generate_tuple_access(value.clone(), index)?;

                    // Recursively assign to the element pattern
                    self.generate_pattern_assignment(element_pattern, element_value)?;
                }
            }

            // Handle other pattern types
            Pattern::Wildcard(_) => {
                // Wildcard patterns don't bind to variables, so we do nothing
            }

            Pattern::Literal(_, _) => {
                // Literal patterns are used for matching, not assignment
                // In destructuring context, we might want to validate the value
            }

            Pattern::Range(_) => {
                // Range patterns are used for matching, not assignment
            }

            Pattern::Or(_) => {
                // Or patterns are complex and would need special handling
            }
        }

        Ok(())
    }

    /// Generate field access for struct destructuring
    fn generate_field_access(&mut self, object: IrValue, field_name: &str) -> Result<IrValue> {
        let result_register = self.new_register();

        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::StructAccess,
            result: Some(result_register),
            operands: vec![
                object,
                IrValue::Constant(IrConstant::String(field_name.to_string())),
            ],
            position: Position::new(0, 0, 0),
        });

        Ok(IrValue::Register(result_register))
    }

    /// Generate index access for array destructuring
    fn generate_index_access(&mut self, array: IrValue, index: usize) -> Result<IrValue> {
        let result_register = self.new_register();

        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::ArrayAccess,
            result: Some(result_register),
            operands: vec![array, IrValue::Constant(IrConstant::Integer(index as i64))],
            position: Position::new(0, 0, 0),
        });

        Ok(IrValue::Register(result_register))
    }

    /// Generate tuple access for tuple destructuring
    fn generate_tuple_access(&mut self, tuple: IrValue, index: usize) -> Result<IrValue> {
        let result_register = self.new_register();

        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::TupleAccess,
            result: Some(result_register),
            operands: vec![tuple, IrValue::Constant(IrConstant::Integer(index as i64))],
            position: Position::new(0, 0, 0),
        });

        Ok(IrValue::Register(result_register))
    }

    /// Generate IR value for an expression
    pub fn generate_expression(&mut self, expression: &Expression) -> Result<IrValue> {
        match expression {
            Expression::Literal(literal) => {
                Ok(IrValue::Constant(self.convert_literal(&literal.value)?))
            }

            Expression::Identifier(ident) => {
                if let Some(&register) = self.register_map.get(&ident.name) {
                    Ok(IrValue::Register(register))
                } else {
                    // Assume it's a global
                    Ok(IrValue::Global(ident.name.clone()))
                }
            }

            Expression::Binary(binary) => {
                let left = self.generate_expression(&binary.left)?;
                let right = self.generate_expression(&binary.right)?;
                let result_register = self.new_register();

                let opcode = match binary.operator {
                    BinaryOperator::Add => IrOpcode::Add,
                    BinaryOperator::Subtract => IrOpcode::Sub,
                    BinaryOperator::Multiply => IrOpcode::Mul,
                    BinaryOperator::Divide => IrOpcode::Div,
                    BinaryOperator::Modulo => IrOpcode::Mod,
                    BinaryOperator::Power => IrOpcode::Pow,
                    BinaryOperator::Equal => IrOpcode::Eq,
                    BinaryOperator::NotEqual => IrOpcode::Ne,
                    BinaryOperator::Less => IrOpcode::Lt,
                    BinaryOperator::Greater => IrOpcode::Gt,
                    BinaryOperator::LessEqual => IrOpcode::Le,
                    BinaryOperator::GreaterEqual => IrOpcode::Ge,
                    BinaryOperator::And => IrOpcode::LogicalAnd,
                    BinaryOperator::Or => IrOpcode::LogicalOr,
                    BinaryOperator::BitwiseAnd => IrOpcode::And,
                    BinaryOperator::BitwiseOr => IrOpcode::Or,
                    BinaryOperator::BitwiseXor => IrOpcode::Xor,
                    BinaryOperator::LeftShift => IrOpcode::Shl,
                    BinaryOperator::RightShift => IrOpcode::Shr,
                };

                // Emit the binary operation instruction
                self.emit_instruction(IrInstruction {
                    opcode,
                    result: Some(result_register),
                    operands: vec![left, right],
                    position: binary.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Call(call) => {
                let callee = self.generate_expression(&call.callee)?;
                let mut args = Vec::new();
                for arg in &call.args {
                    args.push(self.generate_expression(arg)?);
                }

                let result_register = self.new_register();

                // Create the call instruction
                let mut operands = vec![callee];
                operands.extend(args);

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Call,
                    result: Some(result_register),
                    operands,
                    position: call.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Unary(unary) => {
                let operand = self.generate_expression(&unary.operand)?;
                let result_register = self.new_register();

                let opcode = match unary.operator {
                    UnaryOperator::Plus => IrOpcode::Copy, // Unary plus is just a copy
                    UnaryOperator::Minus => IrOpcode::Neg,
                    UnaryOperator::Not => IrOpcode::LogicalNot,
                    UnaryOperator::BitwiseNot => IrOpcode::Not,
                };

                self.emit_instruction(IrInstruction {
                    opcode,
                    result: Some(result_register),
                    operands: vec![operand],
                    position: unary.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Assignment(assignment) => {
                let value = self.generate_expression(&assignment.value)?;

                // Handle different assignment targets
                match assignment.target.as_ref() {
                    Expression::Identifier(ident) => {
                        // Direct variable assignment
                        if let Some(&register) = self.register_map.get(&ident.name) {
                            self.emit_instruction(IrInstruction {
                                opcode: IrOpcode::Copy,
                                result: Some(register),
                                operands: vec![value.clone()],
                                position: assignment.position,
                            });
                        }
                    }
                    Expression::MemberAccess(member_access) => {
                        let object = self.generate_expression(&member_access.object)?;
                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::Store,
                            result: None,
                            operands: vec![
                                object,
                                IrValue::Global(member_access.member.clone()),
                                value.clone(),
                            ],
                            position: assignment.position,
                        });
                    }
                    Expression::Index(index) => {
                        let object = self.generate_expression(&index.object)?;
                        let index_val = self.generate_expression(&index.index)?;
                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::Store,
                            result: None,
                            operands: vec![object, index_val, value.clone()],
                            position: assignment.position,
                        });
                    }
                    _ => {
                        // For other assignment targets, generate a generic store
                        let target = self.generate_expression(&assignment.target)?;
                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::Store,
                            result: None,
                            operands: vec![target, value.clone()],
                            position: assignment.position,
                        });
                    }
                }

                // Return the assigned value
                Ok(value)
            }

            Expression::Array(array) => {
                let mut elements = Vec::new();
                for element in &array.elements {
                    elements.push(self.generate_expression(element)?);
                }

                let result_register = self.new_register();

                // Create array construction instruction
                // First allocate the array
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Alloca,
                    result: Some(result_register),
                    operands: vec![IrValue::Constant(
                        IrConstant::Integer(elements.len() as i64),
                    )],
                    position: array.position,
                });

                // Then store each element
                for (i, element) in elements.iter().enumerate() {
                    self.emit_instruction(IrInstruction {
                        opcode: IrOpcode::Store,
                        result: None,
                        operands: vec![
                            IrValue::Register(result_register),
                            IrValue::Constant(IrConstant::Integer(i as i64)),
                            element.clone(),
                        ],
                        position: array.position,
                    });
                }

                Ok(IrValue::Register(result_register))
            }

            Expression::MemberAccess(member_access) => {
                let object = self.generate_expression(&member_access.object)?;
                let result_register = self.new_register();

                // Create member access instruction
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::StructAccess,
                    result: Some(result_register),
                    operands: vec![object, IrValue::Global(member_access.member.clone())],
                    position: member_access.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Index(index) => {
                let object = self.generate_expression(&index.object)?;
                let index_val = self.generate_expression(&index.index)?;
                let result_register = self.new_register();

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::ArrayAccess,
                    result: Some(result_register),
                    operands: vec![object, index_val],
                    position: index.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::If(if_expr) => {
                // For if expressions, we need to generate conditional logic with phi nodes
                let condition = self.generate_expression(&if_expr.condition)?;

                let then_label = self.next_block_label();
                let else_label = self.next_block_label();
                let merge_label = self.next_block_label();
                let result_register = self.new_register();

                // Conditional branch
                self.emit_conditional_branch(condition, then_label.clone(), else_label.clone());

                // Then block
                self.start_block(then_label);
                let then_val = self.generate_expression(&if_expr.then_expr)?;
                let then_block_label = self
                    .current_block_label
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                self.emit_branch(merge_label.clone());

                // Else block
                self.start_block(else_label);
                let else_val = self.generate_expression(&if_expr.else_expr)?;
                let else_block_label = self
                    .current_block_label
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                self.emit_branch(merge_label.clone());

                // Merge block with phi node
                self.start_block(merge_label);
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Phi,
                    result: Some(result_register),
                    operands: vec![
                        then_val,
                        IrValue::Global(then_block_label),
                        else_val,
                        IrValue::Global(else_block_label),
                    ],
                    position: if_expr.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Map(map) => {
                let result_register = self.new_register();

                // Allocate map
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Alloca,
                    result: Some(result_register),
                    operands: vec![IrValue::Constant(IrConstant::Integer(0))], // Empty map initially
                    position: map.position,
                });

                // Insert each key-value pair
                for entry in &map.entries {
                    let key = self.generate_expression(&entry.key)?;
                    let value = self.generate_expression(&entry.value)?;

                    self.emit_instruction(IrInstruction {
                        opcode: IrOpcode::MapInsert,
                        result: None,
                        operands: vec![IrValue::Register(result_register), key, value],
                        position: entry.position,
                    });
                }

                Ok(IrValue::Register(result_register))
            }

            Expression::Lambda(lambda) => {
                // For lambda expressions, we need to create a closure
                let result_register = self.new_register();

                // Generate a unique function name for the lambda
                let lambda_name = format!("lambda_{}", self.next_register_id);

                // For now, just return a function reference
                // In a full implementation, we'd generate a separate function and handle captures
                Ok(IrValue::Function(lambda_name))
            }

            Expression::Tuple(tuple) => {
                let mut elements = Vec::new();
                for element in &tuple.elements {
                    elements.push(self.generate_expression(element)?);
                }

                let result_register = self.new_register();

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::TupleConstruct,
                    result: Some(result_register),
                    operands: elements,
                    position: tuple.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Cast(cast) => {
                let expr_val = self.generate_expression(&cast.expr)?;
                let result_register = self.new_register();
                let target_type = self.convert_type(&cast.target_type)?;

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Cast,
                    result: Some(result_register),
                    operands: vec![expr_val, IrValue::Global(format!("{:?}", target_type))],
                    position: cast.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::TypeOf(typeof_expr) => {
                let expr_val = self.generate_expression(&typeof_expr.expr)?;
                let result_register = self.new_register();

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::TypeOf,
                    result: Some(result_register),
                    operands: vec![expr_val],
                    position: typeof_expr.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Range(range) => {
                let start = self.generate_expression(&range.start)?;
                let end = self.generate_expression(&range.end)?;
                let result_register = self.new_register();

                // Use a special function call for range construction
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Call,
                    result: Some(result_register),
                    operands: vec![
                        IrValue::Global("__create_range".to_string()),
                        start,
                        end,
                        IrValue::Constant(IrConstant::Boolean(range.inclusive)),
                    ],
                    position: range.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Async(async_expr) => {
                let expr_val = self.generate_expression(&async_expr.expr)?;
                let result_register = self.new_register();

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Spawn,
                    result: Some(result_register),
                    operands: vec![expr_val],
                    position: async_expr.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Await(await_expr) => {
                let expr_val = self.generate_expression(&await_expr.expr)?;
                let result_register = self.new_register();

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Await,
                    result: Some(result_register),
                    operands: vec![expr_val],
                    position: await_expr.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Yield(yield_expr) => {
                let result_register = self.new_register();
                let operands = if let Some(ref value) = yield_expr.value {
                    vec![self.generate_expression(value)?]
                } else {
                    vec![IrValue::Constant(IrConstant::Null)]
                };

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Yield,
                    result: Some(result_register),
                    operands,
                    position: yield_expr.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Parenthesized(paren) => {
                // Parenthesized expressions are just their inner expression
                self.generate_expression(&paren.expr)
            }

            Expression::Block(block) => {
                // Block expressions evaluate to the last expression in the block
                let mut last_value = IrValue::Constant(IrConstant::Null);

                for statement in &block.statements {
                    match statement {
                        Statement::Expression(expr_stmt) => {
                            last_value = self.generate_expression(&expr_stmt.expr)?;
                        }
                        _ => {
                            self.generate_statement(statement)?;
                        }
                    }
                }

                Ok(last_value)
            }

            Expression::Match(match_expr) => {
                // Match expressions require complex control flow
                let expr_val = self.generate_expression(&match_expr.expr)?;
                let result_register = self.new_register();

                // For now, use a switch-like instruction
                // In a full implementation, this would generate pattern matching logic
                let mut cases = Vec::new();
                let mut arm_labels = Vec::new();

                for (i, _arm) in match_expr.arms.iter().enumerate() {
                    let arm_label = self.next_block_label();
                    arm_labels.push(arm_label.clone());
                    cases.push((IrValue::Constant(IrConstant::Integer(i as i64)), arm_label));
                }

                let merge_label = self.next_block_label();

                self.finish_current_block(IrTerminator::Switch {
                    value: expr_val,
                    cases,
                    default_label: Some(merge_label.clone()),
                });

                // Generate code for each arm (simplified)
                for (i, arm) in match_expr.arms.iter().enumerate() {
                    self.start_block(arm_labels[i].clone());
                    let arm_val = self.generate_expression(&arm.expr)?;

                    // Copy result to the result register
                    self.emit_instruction(IrInstruction {
                        opcode: IrOpcode::Copy,
                        result: Some(result_register),
                        operands: vec![arm_val],
                        position: arm.position,
                    });

                    self.emit_branch(merge_label.clone());
                }

                self.start_block(merge_label);
                Ok(IrValue::Register(result_register))
            }

            Expression::StructLiteral(struct_lit) => {
                // Generate struct construction
                let result_register = self.new_register();

                // Collect field names and values alternately
                let mut operands = vec![IrValue::Global(struct_lit.type_name.clone())];
                for field in &struct_lit.fields {
                    // Add field name as global
                    operands.push(IrValue::Global(field.name.clone()));
                    // Add field value
                    let field_value = self.generate_expression(&field.value)?;
                    operands.push(field_value);
                }

                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::StructConstruct,
                    result: Some(result_register),
                    operands,
                    position: struct_lit.position,
                });

                Ok(IrValue::Register(result_register))
            }

            Expression::Channel(channel_expr) => {
                use crate::ast::ChannelDirection;

                match channel_expr.direction {
                    ChannelDirection::Send => {
                        // ch <- value
                        let channel_val = self.generate_expression(&channel_expr.channel)?;
                        let value_val = if let Some(ref value_expr) = channel_expr.value {
                            self.generate_expression(value_expr)?
                        } else {
                            return Err(BuluError::Other(
                                "Send operation requires a value".to_string(),
                            ));
                        };

                        let result_register = self.new_register();

                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::ChannelSend,
                            result: Some(result_register),
                            operands: vec![channel_val, value_val],
                            position: channel_expr.position,
                        });

                        Ok(IrValue::Register(result_register))
                    }
                    ChannelDirection::Receive => {
                        // <-ch
                        let channel_val = self.generate_expression(&channel_expr.channel)?;
                        let result_register = self.new_register();

                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::ChannelReceive,
                            result: Some(result_register),
                            operands: vec![channel_val],
                            position: channel_expr.position,
                        });

                        Ok(IrValue::Register(result_register))
                    }
                    ChannelDirection::Bidirectional => {
                        // This shouldn't happen in expressions
                        Err(BuluError::Other(
                            "Bidirectional channel direction not supported in expressions"
                                .to_string(),
                        ))
                    }
                }
            }

            Expression::Run(run_expr) => {
                // println!("🔧 IR_GENERATOR: Processing Expression::Run");

                // For goroutine spawn, we need to handle the expression specially
                // We DON'T want to execute it now, but defer it for the goroutine
                let result_register = self.new_register();

                // Check if it's a function call
                match &*run_expr.expr {
                    Expression::Call(call_expr) => {
                        // Extract function name from callee
                        let function_name = match &*call_expr.callee {
                            Expression::Identifier(ident) => ident.name.clone(),
                            _ => {
                                return Err(BuluError::Other(
                                    "Run expression must call a named function".to_string(),
                                ));
                            }
                        };

                        // println!("🔧 IR_GENERATOR: Run expression is a function call: {}", function_name);

                        // Generate arguments but don't execute the function yet
                        let mut operands = vec![IrValue::Global(function_name.clone())];

                        for arg in &call_expr.args {
                            let arg_val = self.generate_expression(arg)?;
                            operands.push(arg_val);
                        }

                        // println!("🔧 IR_GENERATOR: Emitting IrOpcode::Spawn with function '{}' and {} args", function_name, call_expr.args.len());

                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::Spawn,
                            result: Some(result_register),
                            operands,
                            position: run_expr.position,
                        });
                    }
                    _ => {
                        // For other expressions, generate them normally
                        // println!("🔧 IR_GENERATOR: Run expression is not a function call, generating normally");
                        let expr_val = self.generate_expression(&run_expr.expr)?;

                        self.emit_instruction(IrInstruction {
                            opcode: IrOpcode::Spawn,
                            result: Some(result_register),
                            operands: vec![expr_val],
                            position: run_expr.position,
                        });
                    }
                }

                // println!("🔧 IR_GENERATOR: IrOpcode::Spawn instruction emitted");

                Ok(IrValue::Register(result_register))
            }

            _ => {
                // For any remaining expression types, return null
                Ok(IrValue::Constant(IrConstant::Null))
            }
        }
    }

    /// Convert AST type to IR type
    pub fn convert_type(&self, ast_type: &Type) -> Result<IrType> {
        match ast_type {
            Type::Int8 => Ok(IrType::I8),
            Type::Int16 => Ok(IrType::I16),
            Type::Int32 => Ok(IrType::I32),
            Type::Int64 => Ok(IrType::I64),
            Type::UInt8 => Ok(IrType::U8),
            Type::UInt16 => Ok(IrType::U16),
            Type::UInt32 => Ok(IrType::U32),
            Type::UInt64 => Ok(IrType::U64),
            Type::Float32 => Ok(IrType::F32),
            Type::Float64 => Ok(IrType::F64),
            Type::Bool => Ok(IrType::Bool),
            Type::Char => Ok(IrType::Char),
            Type::String => Ok(IrType::String),
            Type::Any => Ok(IrType::Any),
            Type::Void => Ok(IrType::Void),

            Type::Array(array_type) => {
                let element_type = Box::new(self.convert_type(&array_type.element_type)?);
                Ok(IrType::Array(element_type, array_type.size))
            }

            Type::Slice(slice_type) => {
                let element_type = Box::new(self.convert_type(&slice_type.element_type)?);
                Ok(IrType::Slice(element_type))
            }

            Type::Map(map_type) => {
                let key_type = Box::new(self.convert_type(&map_type.key_type)?);
                let value_type = Box::new(self.convert_type(&map_type.value_type)?);
                Ok(IrType::Map(key_type, value_type))
            }

            Type::Function(func_type) => {
                let param_types = func_type
                    .param_types
                    .iter()
                    .map(|t| self.convert_type(t))
                    .collect::<Result<Vec<_>>>()?;

                let return_type = func_type
                    .return_type
                    .as_ref()
                    .map(|t| self.convert_type(t).map(Box::new))
                    .transpose()?;

                Ok(IrType::Function(param_types, return_type))
            }

            Type::Named(name) => {
                // Assume it's a struct for now
                Ok(IrType::Struct(name.clone()))
            }

            Type::Tuple(tuple_type) => {
                let element_types = tuple_type
                    .element_types
                    .iter()
                    .map(|t| self.convert_type(t))
                    .collect::<Result<Vec<_>>>()?;
                Ok(IrType::Tuple(element_types))
            }

            Type::Channel(channel_type) => {
                let element_type = Box::new(self.convert_type(&channel_type.element_type)?);
                Ok(IrType::Channel(element_type))
            }

            Type::Struct(struct_type) => Ok(IrType::Struct(struct_type.name.clone())),

            Type::Interface(interface_type) => Ok(IrType::Interface(interface_type.name.clone())),

            Type::Generic(generic_type) => {
                // For generics, we'll use the name as a placeholder
                // In a full implementation, this would be resolved during monomorphization
                Ok(IrType::Struct(generic_type.name.clone()))
            }

            _ => {
                // For other type conversions not yet implemented
                Ok(IrType::Any)
            }
        }
    }

    /// Convert AST literal to IR constant
    pub fn convert_literal(&self, literal: &LiteralValue) -> Result<IrConstant> {
        match literal {
            LiteralValue::Integer(i) => Ok(IrConstant::Integer(*i)),
            LiteralValue::Float(f) => Ok(IrConstant::Float(*f)),
            LiteralValue::String(s) => Ok(IrConstant::String(s.clone())),
            LiteralValue::Char(c) => Ok(IrConstant::Char(*c)),
            LiteralValue::Boolean(b) => Ok(IrConstant::Boolean(*b)),
            LiteralValue::Null => Ok(IrConstant::Null),
        }
    }

    /// Evaluate constant expression at compile time
    pub fn evaluate_constant_expression(&self, expression: &Expression) -> Result<IrValue> {
        match expression {
            Expression::Literal(literal) => {
                Ok(IrValue::Constant(self.convert_literal(&literal.value)?))
            }

            Expression::Binary(binary) => {
                let left = self.evaluate_constant_expression(&binary.left)?;
                let right = self.evaluate_constant_expression(&binary.right)?;

                if let (IrValue::Constant(left_const), IrValue::Constant(right_const)) =
                    (left, right)
                {
                    self.evaluate_constant_binary_op(&binary.operator, &left_const, &right_const)
                } else {
                    Ok(IrValue::Constant(IrConstant::Null))
                }
            }

            Expression::Unary(unary) => {
                let operand = self.evaluate_constant_expression(&unary.operand)?;

                if let IrValue::Constant(operand_const) = operand {
                    self.evaluate_constant_unary_op(&unary.operator, &operand_const)
                } else {
                    Ok(IrValue::Constant(IrConstant::Null))
                }
            }

            Expression::Identifier(_ident) => {
                // For identifiers, we can't evaluate them as constants unless they're const globals
                // For now, return null
                Ok(IrValue::Constant(IrConstant::Null))
            }

            _ => {
                // For other expressions, we can't evaluate them as constants
                Ok(IrValue::Constant(IrConstant::Null))
            }
        }
    }

    /// Generate next register ID
    fn next_register(&mut self) -> IrRegister {
        let id = self.next_register_id;
        self.next_register_id += 1;
        IrRegister { id }
    }

    /// Generate next basic block label
    fn next_block_label(&mut self) -> String {
        let id = self.next_block_id;
        self.next_block_id += 1;
        format!("bb{}", id)
    }

    /// Start a new basic block
    fn start_block(&mut self, label: String) {
        // Finish current block if it exists
        if self.current_block_label.is_some() {
            self.finish_current_block(IrTerminator::Unreachable);
        }

        self.current_block_label = Some(label);
        self.current_block_instructions.clear();
    }

    /// Finish the current basic block with a terminator
    fn finish_current_block(&mut self, terminator: IrTerminator) {
        if let Some(label) = self.current_block_label.take() {
            let block = IrBasicBlock {
                label,
                instructions: std::mem::take(&mut self.current_block_instructions),
                terminator,
            };
            self.current_function_blocks.push(block);
        }
    }

    /// Add an instruction to the current basic block
    fn emit_instruction(&mut self, instruction: IrInstruction) {
        self.current_block_instructions.push(instruction);
    }

    /// Create a new register and return it
    fn new_register(&mut self) -> IrRegister {
        let reg = IrRegister {
            id: self.next_register_id,
        };
        self.next_register_id += 1;
        reg
    }

    /// Branch to a target block
    fn emit_branch(&mut self, target: String) {
        self.finish_current_block(IrTerminator::Branch(target));
    }

    /// Conditional branch
    fn emit_conditional_branch(
        &mut self,
        condition: IrValue,
        true_label: String,
        false_label: String,
    ) {
        self.finish_current_block(IrTerminator::ConditionalBranch {
            condition,
            true_label,
            false_label,
        });
    }

    /// Return from function
    fn emit_return(&mut self, value: Option<IrValue>) {
        self.finish_current_block(IrTerminator::Return(value));
    }

    /// Collect local variables from a block statement
    fn collect_locals(&self, block: &BlockStmt) -> Vec<IrLocal> {
        let mut locals = Vec::new();
        self.collect_locals_from_statements(&block.statements, &mut locals);
        locals
    }

    /// Recursively collect locals from statements
    fn collect_locals_from_statements(&self, statements: &[Statement], locals: &mut Vec<IrLocal>) {
        for statement in statements {
            match statement {
                Statement::VariableDecl(var_decl) => {
                    if let Ok(ir_type) =
                        self.convert_type(&var_decl.type_annotation.as_ref().unwrap_or(&Type::Any))
                    {
                        locals.push(IrLocal {
                            name: var_decl.name.clone(),
                            local_type: ir_type,
                            register: IrRegister {
                                id: locals.len() as u32,
                            },
                            is_mutable: !var_decl.is_const,
                        });
                    }
                }
                Statement::DestructuringDecl(_) => {
                    // For destructuring, we would need to analyze the pattern to extract variable names
                    // For now, we skip this as it's complex
                }
                Statement::MultipleVariableDecl(decl) => {
                    for var_decl in &decl.declarations {
                        if let Ok(ir_type) = self
                            .convert_type(&var_decl.type_annotation.as_ref().unwrap_or(&Type::Any))
                        {
                            locals.push(IrLocal {
                                name: var_decl.name.clone(),
                                local_type: ir_type,
                                register: IrRegister {
                                    id: locals.len() as u32,
                                },
                                is_mutable: !decl.is_const,
                            });
                        }
                    }
                }
                Statement::Block(block) => {
                    self.collect_locals_from_statements(&block.statements, locals);
                }
                Statement::If(if_stmt) => {
                    self.collect_locals_from_statements(&if_stmt.then_branch.statements, locals);
                    if let Some(else_stmt) = &if_stmt.else_branch {
                        if let Statement::Block(else_block) = else_stmt.as_ref() {
                            self.collect_locals_from_statements(&else_block.statements, locals);
                        }
                    }
                }
                Statement::While(while_stmt) => {
                    self.collect_locals_from_statements(&while_stmt.body.statements, locals);
                }
                Statement::For(for_stmt) => {
                    // Add the loop variable
                    locals.push(IrLocal {
                        name: for_stmt.variable.clone(),
                        local_type: IrType::Any, // Would need better type inference
                        register: IrRegister {
                            id: locals.len() as u32,
                        },
                        is_mutable: true,
                    });
                    if let Some(ref index_var) = for_stmt.index_variable {
                        locals.push(IrLocal {
                            name: index_var.clone(),
                            local_type: IrType::I32,
                            register: IrRegister {
                                id: locals.len() as u32,
                            },
                            is_mutable: true,
                        });
                    }
                    self.collect_locals_from_statements(&for_stmt.body.statements, locals);
                }
                _ => {}
            }
        }
    }

    /// Infer type from an expression
    pub fn infer_type_from_expression(&self, expression: &Expression) -> Result<IrType> {
        match expression {
            Expression::Literal(literal) => match &literal.value {
                LiteralValue::Integer(_) => Ok(IrType::I64),
                LiteralValue::Float(_) => Ok(IrType::F64),
                LiteralValue::String(_) => Ok(IrType::String),
                LiteralValue::Char(_) => Ok(IrType::Char),
                LiteralValue::Boolean(_) => Ok(IrType::Bool),
                LiteralValue::Null => Ok(IrType::Any),
            },
            Expression::Binary(binary) => {
                // For binary operations, infer based on operands and operator
                let left_type = self.infer_type_from_expression(&binary.left)?;
                let right_type = self.infer_type_from_expression(&binary.right)?;

                match binary.operator {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide
                    | BinaryOperator::Modulo
                    | BinaryOperator::Power => {
                        // Arithmetic operations preserve numeric types
                        if matches!(left_type, IrType::F32 | IrType::F64)
                            || matches!(right_type, IrType::F32 | IrType::F64)
                        {
                            Ok(IrType::F64)
                        } else {
                            Ok(IrType::I64)
                        }
                    }
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Less
                    | BinaryOperator::Greater
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::And
                    | BinaryOperator::Or => Ok(IrType::Bool),
                    _ => Ok(IrType::I64),
                }
            }
            Expression::Call(_) => {
                // Would need function signature lookup for proper inference
                Ok(IrType::Any)
            }
            _ => Ok(IrType::Any),
        }
    }

    /// Calculate the size of a type in bytes
    pub fn calculate_type_size(&self, ir_type: &IrType) -> usize {
        match ir_type {
            IrType::I8 | IrType::U8 => 1,
            IrType::I16 | IrType::U16 => 2,
            IrType::I32 | IrType::U32 | IrType::F32 => 4,
            IrType::I64 | IrType::U64 | IrType::F64 => 8,
            IrType::Bool | IrType::Char => 1,
            IrType::String => 8, // Pointer size
            IrType::Array(_, Some(size)) => {
                // Array size * element size
                size * 8 // Simplified - would need element type size
            }
            IrType::Slice(_) => 16, // Pointer + length
            IrType::Map(_, _) => 8, // Pointer to map structure
            IrType::Tuple(elements) => elements.iter().map(|t| self.calculate_type_size(t)).sum(),
            IrType::Pointer(_) => 8,
            _ => 8, // Default pointer size for complex types
        }
    }

    /// Generate IR for if statement
    fn generate_if_statement(&mut self, if_stmt: &IfStmt) -> Result<()> {
        let condition = self.generate_expression(&if_stmt.condition)?;

        let then_label = self.next_block_label();
        let else_label = self.next_block_label();
        let merge_label = self.next_block_label();

        // Conditional branch
        self.emit_conditional_branch(condition, then_label.clone(), else_label.clone());

        // Then block
        self.start_block(then_label);
        self.generate_block_statement(&if_stmt.then_branch)?;

        // Only branch to merge if we haven't already terminated (e.g., with return)
        if self.current_block_label.is_some() {
            self.emit_branch(merge_label.clone());
        }

        // Else block
        self.start_block(else_label);
        if let Some(else_stmt) = &if_stmt.else_branch {
            self.generate_statement(else_stmt)?;
        }

        // Only branch to merge if we haven't already terminated
        if self.current_block_label.is_some() {
            self.emit_branch(merge_label.clone());
        }

        // Merge block
        self.start_block(merge_label);

        Ok(())
    }

    /// Generate IR for while statement
    fn generate_while_statement(&mut self, while_stmt: &WhileStmt) -> Result<()> {
        let loop_header = self.next_block_label();
        let loop_body = self.next_block_label();
        let loop_exit = self.next_block_label();

        // Push loop labels for break/continue
        self.break_labels.push(loop_exit.clone());
        self.continue_labels.push(loop_header.clone());

        // Branch to loop header
        self.emit_branch(loop_header.clone());

        // Loop header - evaluate condition
        self.start_block(loop_header.clone());
        let condition = self.generate_expression(&while_stmt.condition)?;
        self.emit_conditional_branch(condition, loop_body.clone(), loop_exit.clone());

        // Loop body
        self.start_block(loop_body);
        self.generate_block_statement(&while_stmt.body)?;

        // Branch back to header (if not terminated by break/return)
        if self.current_block_label.is_some() {
            self.emit_branch(loop_header);
        }

        // Loop exit
        self.start_block(loop_exit);

        // Pop loop labels
        self.break_labels.pop();
        self.continue_labels.pop();

        Ok(())
    }

    /// Generate IR for for statement
    fn generate_for_statement(&mut self, for_stmt: &ForStmt) -> Result<()> {
        let loop_init = self.next_block_label();
        let loop_header = self.next_block_label();
        let loop_body = self.next_block_label();
        let loop_exit = self.next_block_label();

        // Push loop labels for break/continue
        self.break_labels.push(loop_exit.clone());
        self.continue_labels.push(loop_header.clone());

        // Branch to loop initialization
        self.emit_branch(loop_init.clone());

        // Loop initialization - set up iterator
        self.start_block(loop_init);
        let iterable = self.generate_expression(&for_stmt.iterable)?;

        // Create index register (always needed for iteration)
        let index_reg = self.new_register();

        // Initialize index to 0
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Copy,
            result: Some(index_reg),
            operands: vec![IrValue::Constant(IrConstant::Integer(0))],
            position: for_stmt.position,
        });

        // Store the array in a register for later access
        let array_reg = self.new_register();
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Copy,
            result: Some(array_reg),
            operands: vec![iterable],
            position: for_stmt.position,
        });

        // If there's an explicit index variable, map it to our index register
        if let Some(ref index_var) = for_stmt.index_variable {
            self.register_map.insert(index_var.clone(), index_reg);
        }

        self.emit_branch(loop_header.clone());

        // Loop header - check if index < array.length
        self.start_block(loop_header.clone());

        // Get array length
        let array_length = self.new_register();
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::ArrayLength,
            result: Some(array_length),
            operands: vec![IrValue::Register(array_reg)],
            position: for_stmt.position,
        });

        // Compare index < array_length
        let condition = self.new_register();
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Lt,
            result: Some(condition),
            operands: vec![
                IrValue::Register(index_reg),
                IrValue::Register(array_length),
            ],
            position: for_stmt.position,
        });

        self.emit_conditional_branch(
            IrValue::Register(condition),
            loop_body.clone(),
            loop_exit.clone(),
        );

        // Loop body
        self.start_block(loop_body);

        // Extract current element from array[index] and assign to loop variable
        let current_element = self.new_register();
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::ArrayAccess,
            result: Some(current_element),
            operands: vec![IrValue::Register(array_reg), IrValue::Register(index_reg)],
            position: for_stmt.position,
        });

        // Check if current_element is null (for channel iteration)
        // If null, exit the loop
        let is_null = self.new_register();
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::IsNull,
            result: Some(is_null),
            operands: vec![IrValue::Register(current_element)],
            position: for_stmt.position,
        });

        let continue_label = self.next_block_label();
        self.emit_conditional_branch(
            IrValue::Register(is_null),
            loop_exit.clone(),
            continue_label.clone(),
        );

        // Continue with loop body if not null
        self.start_block(continue_label);

        // Assign current element to the loop variable
        let loop_var_reg = self.new_register();
        self.register_map
            .insert(for_stmt.variable.clone(), loop_var_reg);
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Copy,
            result: Some(loop_var_reg),
            operands: vec![IrValue::Register(current_element)],
            position: for_stmt.position,
        });

        // Execute the loop body
        self.generate_block_statement(&for_stmt.body)?;

        // Increment index (always needed)
        let incremented = self.new_register();
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Add,
            result: Some(incremented),
            operands: vec![
                IrValue::Register(index_reg),
                IrValue::Constant(IrConstant::Integer(1)),
            ],
            position: for_stmt.position,
        });
        self.emit_instruction(IrInstruction {
            opcode: IrOpcode::Copy,
            result: Some(index_reg),
            operands: vec![IrValue::Register(incremented)],
            position: for_stmt.position,
        });

        // Branch back to header (if not terminated by break/return)
        if self.current_block_label.is_some() {
            self.emit_branch(loop_header);
        }

        // Loop exit
        self.start_block(loop_exit);

        // Pop loop labels
        self.break_labels.pop();
        self.continue_labels.pop();

        Ok(())
    }

    /// Generate IR for match statement
    fn generate_match_statement(&mut self, match_stmt: &MatchStmt) -> Result<()> {
        let expr_val = self.generate_expression(&match_stmt.expr)?;

        let merge_label = self.next_block_label();
        let mut next_arm_label = None;

        // Generate code for each arm in sequence
        for (i, arm) in match_stmt.arms.iter().enumerate() {
            // Start with the current block or the "next arm" label from previous iteration
            if let Some(label) = next_arm_label.take() {
                self.start_block(label);
            }

            // Generate pattern matching condition
            let pattern_matches = self.generate_pattern_match(&arm.pattern, &expr_val)?;

            let arm_body_label = self.next_block_label();
            let next_check_label = if i + 1 < match_stmt.arms.len() {
                let label = self.next_block_label();
                next_arm_label = Some(label.clone());
                Some(label)
            } else {
                None
            };

            // Branch based on pattern match
            if let Some(next_label) = next_check_label {
                self.emit_conditional_branch(pattern_matches, arm_body_label.clone(), next_label);
            } else {
                // Last arm - if pattern doesn't match, go to merge (shouldn't happen with wildcard)
                self.emit_conditional_branch(
                    pattern_matches,
                    arm_body_label.clone(),
                    merge_label.clone(),
                );
            }

            // Generate arm body
            self.start_block(arm_body_label);

            // Generate guard condition if present
            if let Some(ref guard) = arm.guard {
                let guard_val = self.generate_expression(guard)?;
                let guard_true = self.next_block_label();
                let guard_false = if let Some(ref next_label) = next_arm_label {
                    next_label.clone()
                } else {
                    merge_label.clone()
                };

                self.emit_conditional_branch(guard_val, guard_true.clone(), guard_false);

                // Guard true - execute arm body
                self.start_block(guard_true);
            }

            // Execute arm body
            self.generate_statement(&arm.body)?;
            if self.current_block_label.is_some() {
                self.emit_branch(merge_label.clone());
            }
        }

        // Merge block
        self.start_block(merge_label);

        Ok(())
    }

    /// Generate pattern matching logic
    fn generate_pattern_match(&mut self, pattern: &Pattern, expr_val: &IrValue) -> Result<IrValue> {
        match pattern {
            Pattern::Wildcard(_) => {
                // Wildcard always matches
                Ok(IrValue::Constant(IrConstant::Boolean(true)))
            }
            Pattern::Literal(literal, _) => {
                // Generate comparison with literal
                let literal_val = match literal {
                    LiteralValue::Integer(i) => IrValue::Constant(IrConstant::Integer(*i)),
                    LiteralValue::Float(f) => IrValue::Constant(IrConstant::Float(*f)),
                    LiteralValue::String(s) => IrValue::Constant(IrConstant::String(s.clone())),
                    LiteralValue::Boolean(b) => IrValue::Constant(IrConstant::Boolean(*b)),
                    LiteralValue::Char(c) => IrValue::Constant(IrConstant::Integer(*c as i64)),
                    LiteralValue::Null => IrValue::Constant(IrConstant::Null),
                };

                let result_reg = self.new_register();
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Eq,
                    result: Some(result_reg.clone()),
                    operands: vec![expr_val.clone(), literal_val],
                    position: Position::new(0, 0, 0),
                });

                Ok(IrValue::Register(result_reg))
            }
            Pattern::Identifier(name, _) => {
                // For now, treat identifier patterns as wildcards and bind the value
                // TODO: Implement proper variable binding in patterns
                let _ = name; // Suppress unused warning
                Ok(IrValue::Constant(IrConstant::Boolean(true)))
            }
            Pattern::Range(range_pattern) => {
                // Generate range check: expr_val >= start && expr_val <= end (if inclusive)
                let start_val = match &range_pattern.start {
                    LiteralValue::Integer(i) => IrValue::Constant(IrConstant::Integer(*i)),
                    LiteralValue::Float(f) => IrValue::Constant(IrConstant::Float(*f)),
                    _ => {
                        return Err(BuluError::Other(
                            "Range patterns only support numeric literals".to_string(),
                        ))
                    }
                };
                let end_val = match &range_pattern.end {
                    LiteralValue::Integer(i) => IrValue::Constant(IrConstant::Integer(*i)),
                    LiteralValue::Float(f) => IrValue::Constant(IrConstant::Float(*f)),
                    _ => {
                        return Err(BuluError::Other(
                            "Range patterns only support numeric literals".to_string(),
                        ))
                    }
                };

                // expr_val >= start
                let ge_reg = self.new_register();
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Ge,
                    result: Some(ge_reg.clone()),
                    operands: vec![expr_val.clone(), start_val],
                    position: Position::new(0, 0, 0),
                });

                // expr_val <= end (inclusive) or expr_val < end (exclusive)
                let end_op = if range_pattern.inclusive {
                    IrOpcode::Le
                } else {
                    IrOpcode::Lt
                };
                let le_reg = self.new_register();
                self.emit_instruction(IrInstruction {
                    opcode: end_op,
                    result: Some(le_reg.clone()),
                    operands: vec![expr_val.clone(), end_val],
                    position: Position::new(0, 0, 0),
                });

                // Combine with AND
                let result_reg = self.new_register();
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::LogicalAnd,
                    result: Some(result_reg.clone()),
                    operands: vec![IrValue::Register(ge_reg), IrValue::Register(le_reg)],
                    position: Position::new(0, 0, 0),
                });

                Ok(IrValue::Register(result_reg))
            }
            _ => {
                // TODO: Implement other pattern types (Struct, Array, Or)
                Ok(IrValue::Constant(IrConstant::Boolean(false)))
            }
        }
    }

    /// Generate IR for try statement
    fn generate_try_statement(&mut self, try_stmt: &TryStmt) -> Result<()> {
        let try_label = self.next_block_label();
        let catch_label = self.next_block_label();
        let merge_label = self.next_block_label();

        // Branch to try block
        self.emit_branch(try_label.clone());

        // Try block
        self.start_block(try_label);
        self.generate_block_statement(&try_stmt.body)?;

        // If no exception, branch to merge
        if self.current_block_label.is_some() {
            self.emit_branch(merge_label.clone());
        }

        // Catch block
        self.start_block(catch_label);
        if let Some(ref catch_clause) = try_stmt.catch_clause {
            // If there's an error variable, bind it
            if let Some(ref error_var) = catch_clause.error_var {
                let error_reg = self.new_register();
                self.register_map.insert(error_var.clone(), error_reg);

                // Get the current exception (simplified)
                self.emit_instruction(IrInstruction {
                    opcode: IrOpcode::Catch,
                    result: Some(error_reg),
                    operands: vec![],
                    position: catch_clause.position,
                });
            }

            self.generate_block_statement(&catch_clause.body)?;
        }

        if self.current_block_label.is_some() {
            self.emit_branch(merge_label.clone());
        }

        // Merge block
        self.start_block(merge_label);

        Ok(())
    }

    /// Evaluate constant binary operation
    fn evaluate_constant_binary_op(
        &self,
        operator: &BinaryOperator,
        left: &IrConstant,
        right: &IrConstant,
    ) -> Result<IrValue> {
        match (left, right) {
            (IrConstant::Integer(a), IrConstant::Integer(b)) => {
                let result = match operator {
                    BinaryOperator::Add => a + b,
                    BinaryOperator::Subtract => a - b,
                    BinaryOperator::Multiply => a * b,
                    BinaryOperator::Divide => {
                        if *b != 0 {
                            a / b
                        } else {
                            return Ok(IrValue::Constant(IrConstant::Null));
                        }
                    }
                    BinaryOperator::Modulo => {
                        if *b != 0 {
                            a % b
                        } else {
                            return Ok(IrValue::Constant(IrConstant::Null));
                        }
                    }
                    BinaryOperator::Equal => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a == b)))
                    }
                    BinaryOperator::NotEqual => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a != b)))
                    }
                    BinaryOperator::Less => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a < b)))
                    }
                    BinaryOperator::Greater => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a > b)))
                    }
                    BinaryOperator::LessEqual => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a <= b)))
                    }
                    BinaryOperator::GreaterEqual => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a >= b)))
                    }
                    BinaryOperator::Power => {
                        // For integer power, we need to handle it carefully
                        if *b >= 0 {
                            a.pow(*b as u32)
                        } else {
                            // Negative exponents for integers should return float
                            return Ok(IrValue::Constant(IrConstant::Float(
                                (*a as f64).powf(*b as f64),
                            )));
                        }
                    }
                    _ => return Ok(IrValue::Constant(IrConstant::Null)),
                };
                Ok(IrValue::Constant(IrConstant::Integer(result)))
            }
            (IrConstant::Float(a), IrConstant::Float(b)) => {
                let result = match operator {
                    BinaryOperator::Add => a + b,
                    BinaryOperator::Subtract => a - b,
                    BinaryOperator::Multiply => a * b,
                    BinaryOperator::Divide => {
                        if *b != 0.0 {
                            a / b
                        } else {
                            return Ok(IrValue::Constant(IrConstant::Null));
                        }
                    }
                    BinaryOperator::Equal => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a == b)))
                    }
                    BinaryOperator::NotEqual => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a != b)))
                    }
                    BinaryOperator::Less => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a < b)))
                    }
                    BinaryOperator::Greater => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a > b)))
                    }
                    BinaryOperator::LessEqual => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a <= b)))
                    }
                    BinaryOperator::GreaterEqual => {
                        return Ok(IrValue::Constant(IrConstant::Boolean(a >= b)))
                    }
                    BinaryOperator::Power => a.powf(*b),
                    _ => return Ok(IrValue::Constant(IrConstant::Null)),
                };
                Ok(IrValue::Constant(IrConstant::Float(result)))
            }
            (IrConstant::Boolean(a), IrConstant::Boolean(b)) => {
                let result = match operator {
                    BinaryOperator::And => *a && *b,
                    BinaryOperator::Or => *a || *b,
                    BinaryOperator::Equal => a == b,
                    BinaryOperator::NotEqual => a != b,
                    _ => return Ok(IrValue::Constant(IrConstant::Null)),
                };
                Ok(IrValue::Constant(IrConstant::Boolean(result)))
            }
            _ => Ok(IrValue::Constant(IrConstant::Null)),
        }
    }

    /// Evaluate constant unary operation
    fn evaluate_constant_unary_op(
        &self,
        operator: &UnaryOperator,
        operand: &IrConstant,
    ) -> Result<IrValue> {
        match operand {
            IrConstant::Integer(n) => {
                let result = match operator {
                    UnaryOperator::Plus => *n,
                    UnaryOperator::Minus => -*n,
                    _ => return Ok(IrValue::Constant(IrConstant::Null)),
                };
                Ok(IrValue::Constant(IrConstant::Integer(result)))
            }
            IrConstant::Float(f) => {
                let result = match operator {
                    UnaryOperator::Plus => *f,
                    UnaryOperator::Minus => -*f,
                    _ => return Ok(IrValue::Constant(IrConstant::Null)),
                };
                Ok(IrValue::Constant(IrConstant::Float(result)))
            }
            IrConstant::Boolean(b) => {
                let result = match operator {
                    UnaryOperator::Not => !*b,
                    _ => return Ok(IrValue::Constant(IrConstant::Null)),
                };
                Ok(IrValue::Constant(IrConstant::Boolean(result)))
            }
            _ => Ok(IrValue::Constant(IrConstant::Null)),
        }
    }
}

// Display implementations for debugging
impl fmt::Display for IrOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            IrOpcode::Add => "add",
            IrOpcode::Sub => "sub",
            IrOpcode::Mul => "mul",
            IrOpcode::Div => "div",
            IrOpcode::Mod => "mod",
            IrOpcode::Pow => "pow",
            IrOpcode::Neg => "neg",
            IrOpcode::And => "and",
            IrOpcode::Or => "or",
            IrOpcode::Xor => "xor",
            IrOpcode::Not => "not",
            IrOpcode::Shl => "shl",
            IrOpcode::Shr => "shr",
            IrOpcode::Eq => "eq",
            IrOpcode::Ne => "ne",
            IrOpcode::Lt => "lt",
            IrOpcode::Le => "le",
            IrOpcode::Gt => "gt",
            IrOpcode::Ge => "ge",
            IrOpcode::LogicalAnd => "land",
            IrOpcode::LogicalOr => "lor",
            IrOpcode::LogicalNot => "lnot",
            IrOpcode::Load => "load",
            IrOpcode::Store => "store",
            IrOpcode::Alloca => "alloca",
            IrOpcode::Cast => "cast",
            IrOpcode::TypeOf => "typeof",
            IrOpcode::IsNull => "is_null",
            IrOpcode::Call => "call",
            IrOpcode::CallIndirect => "call_indirect",
            IrOpcode::ArrayAccess => "array_access",
            IrOpcode::ArrayLength => "array_length",
            IrOpcode::SliceAccess => "slice_access",
            IrOpcode::SliceLength => "slice_length",
            IrOpcode::MapAccess => "map_access",
            IrOpcode::MapInsert => "map_insert",
            IrOpcode::MapDelete => "map_delete",
            IrOpcode::MapLength => "map_length",
            IrOpcode::ChannelSend => "chan_send",
            IrOpcode::ChannelReceive => "chan_recv",
            IrOpcode::ChannelClose => "chan_close",
            IrOpcode::ChannelSelect => "chan_select",
            IrOpcode::Spawn => "spawn",
            IrOpcode::Await => "await",
            IrOpcode::Phi => "phi",
            IrOpcode::StructAccess => "struct_access",
            IrOpcode::StructConstruct => "struct_construct",
            IrOpcode::RegisterStruct => "register_struct",
            IrOpcode::TupleAccess => "tuple_access",
            IrOpcode::TupleConstruct => "tuple_construct",
            IrOpcode::StringConcat => "string_concat",
            IrOpcode::StringLength => "string_length",
            IrOpcode::Copy => "copy",
            IrOpcode::Move => "move",
            IrOpcode::Clone => "clone",
            IrOpcode::Yield => "yield",
            IrOpcode::GeneratorNext => "generator_next",
            IrOpcode::Throw => "throw",
            IrOpcode::Catch => "catch",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for IrRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.id)
    }
}

impl fmt::Display for IrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrValue::Register(reg) => write!(f, "{}", reg),
            IrValue::Constant(const_val) => write!(f, "{}", const_val),
            IrValue::Global(name) => write!(f, "@{}", name),
            IrValue::Function(name) => write!(f, "@{}", name),
        }
    }
}

impl fmt::Display for IrConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrConstant::Integer(i) => write!(f, "{}", i),
            IrConstant::Float(fl) => write!(f, "{}", fl),
            IrConstant::String(s) => write!(f, "\"{}\"", s),
            IrConstant::Char(c) => write!(f, "'{}'", c),
            IrConstant::Boolean(b) => write!(f, "{}", b),
            IrConstant::Null => write!(f, "null"),
            IrConstant::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            IrConstant::Struct(fields) => {
                write!(f, "{{")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", field)?;
                }
                write!(f, "}}")
            }
            IrConstant::Tuple(elements) => {
                write!(f, "(")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
        }
    }
}
