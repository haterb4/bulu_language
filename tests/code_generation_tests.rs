//! Unit tests for code generation backend

use bulu::compiler::codegen::CodeGenerator;
use bulu::compiler::ir::{
    IrProgram, IrFunction, IrBasicBlock, IrInstruction, IrOpcode, IrValue, IrConstant,
    IrType, IrRegister, IrTerminator, IrParam, IrLocal,
};
use bulu::lexer::token::Position;

/// Helper function to create a test position
fn test_position() -> Position {
    Position { line: 1, column: 1, offset: 0 }
}

/// Helper function to create a simple IR function for testing
fn create_test_function() -> IrFunction {
    IrFunction {
        name: "test_function".to_string(),
        params: vec![
            IrParam {
                name: "param1".to_string(),
                param_type: IrType::I32,
                register: IrRegister { id: 0 },
            },
            IrParam {
                name: "param2".to_string(),
                param_type: IrType::I32,
                register: IrRegister { id: 1 },
            },
        ],
        return_type: Some(IrType::I32),
        locals: vec![
            IrLocal {
                name: "local1".to_string(),
                local_type: IrType::I32,
                register: IrRegister { id: 2 },
                is_mutable: true,
            },
        ],
        basic_blocks: vec![
            IrBasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    IrInstruction {
                        opcode: IrOpcode::Add,
                        result: Some(IrRegister { id: 2 }),
                        operands: vec![
                            IrValue::Register(IrRegister { id: 0 }),
                            IrValue::Register(IrRegister { id: 1 }),
                        ],
                        position: test_position(),
                    },
                ],
                terminator: IrTerminator::Return(Some(IrValue::Register(IrRegister { id: 2 }))),
            },
        ],
        is_async: false,
        position: test_position(),
    }
}

/// Helper function to create a test IR program
fn create_test_program() -> IrProgram {
    IrProgram {
        functions: vec![create_test_function()],
        globals: vec![],
        structs: vec![],
        interfaces: vec![],
    }
}

#[test]
fn test_code_generator_creation() {
    let _generator = CodeGenerator::new();
    // Just verify it can be created without panicking
    assert!(true);
}

#[test]
fn test_target_setting() {
    let mut generator = CodeGenerator::new();
    
    // Test setting different targets
    generator.set_target("linux-amd64");
    generator.set_target("windows-amd64");
    generator.set_target("darwin-amd64");
    generator.set_target("wasm");
    generator.set_target("native");
    
    // Just verify it doesn't panic
    assert!(true);
}

#[test]
fn test_debug_setting() {
    let mut generator = CodeGenerator::new();
    
    generator.set_debug(true);
    generator.set_debug(false);
    
    // Just verify it doesn't panic
    assert!(true);
}

#[test]
fn test_assembly_generation() {
    let mut generator = CodeGenerator::new();
    let program = create_test_program();
    
    let result = generator.generate_assembly(&program);
    assert!(result.is_ok());
    
    let assembly = result.unwrap();
    assert!(!assembly.is_empty());
    assert!(assembly.contains("# Generated assembly"));
    assert!(assembly.contains(".section .text"));
}

#[test]
fn test_executable_generation() {
    let mut generator = CodeGenerator::new();
    let program = create_test_program();
    
    let result = generator.generate_executable(&program);
    assert!(result.is_ok());
    
    let executable = result.unwrap();
    assert!(!executable.is_empty());
    // Check for our simple magic number
    assert_eq!(&executable[0..4], b"BULU");
}

#[test]
fn test_cross_compilation_targets() {
    let targets = vec![
        "linux-amd64",
        "linux-arm64", 
        "windows-amd64",
        "darwin-amd64",
        "wasm",
        "native",
    ];

    for target in targets {
        let mut generator = CodeGenerator::new();
        generator.set_target(target);
        
        let program = create_test_program();
        let result = generator.generate_assembly(&program);
        
        assert!(result.is_ok(), "Cross-compilation failed for target: {}", target);
        
        let assembly = result.unwrap();
        assert!(!assembly.is_empty());
        assert!(assembly.contains(&format!("# Generated assembly for target: {}", target)));
    }
}

#[test]
fn test_debug_information() {
    let mut generator = CodeGenerator::new();
    generator.set_debug(true);
    
    let program = create_test_program();
    let result = generator.generate_assembly(&program);
    
    assert!(result.is_ok());
    let assembly = result.unwrap();
    assert!(!assembly.is_empty());
}

#[test]
fn test_static_linking() {
    let mut generator = CodeGenerator::new();
    generator.set_static_link(true);
    
    let program = create_test_program();
    let result = generator.generate_assembly(&program);
    
    assert!(result.is_ok());
    let assembly = result.unwrap();
    assert!(!assembly.is_empty());
}

#[test]
fn test_different_instruction_types() {
    let mut generator = CodeGenerator::new();
    
    // Create a function with various instruction types
    let function = IrFunction {
        name: "test_instructions".to_string(),
        params: vec![],
        return_type: Some(IrType::I32),
        locals: vec![],
        basic_blocks: vec![
            IrBasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // Arithmetic operations
                    IrInstruction {
                        opcode: IrOpcode::Add,
                        result: Some(IrRegister { id: 0 }),
                        operands: vec![
                            IrValue::Constant(IrConstant::Integer(10)),
                            IrValue::Constant(IrConstant::Integer(20)),
                        ],
                        position: test_position(),
                    },
                    IrInstruction {
                        opcode: IrOpcode::Sub,
                        result: Some(IrRegister { id: 1 }),
                        operands: vec![
                            IrValue::Register(IrRegister { id: 0 }),
                            IrValue::Constant(IrConstant::Integer(5)),
                        ],
                        position: test_position(),
                    },
                    IrInstruction {
                        opcode: IrOpcode::Mul,
                        result: Some(IrRegister { id: 2 }),
                        operands: vec![
                            IrValue::Register(IrRegister { id: 1 }),
                            IrValue::Constant(IrConstant::Integer(2)),
                        ],
                        position: test_position(),
                    },
                    // Copy operation
                    IrInstruction {
                        opcode: IrOpcode::Copy,
                        result: Some(IrRegister { id: 3 }),
                        operands: vec![IrValue::Register(IrRegister { id: 2 })],
                        position: test_position(),
                    },
                ],
                terminator: IrTerminator::Return(Some(IrValue::Register(IrRegister { id: 3 }))),
            },
        ],
        is_async: false,
        position: test_position(),
    };

    let program = IrProgram {
        functions: vec![function],
        globals: vec![],
        structs: vec![],
        interfaces: vec![],
    };
    
    let result = generator.generate_assembly(&program);
    assert!(result.is_ok());
    
    let assembly = result.unwrap();
    assert!(!assembly.is_empty());
    assert!(assembly.contains("test_instructions"));
}

#[test]
fn test_control_flow_generation() {
    let mut generator = CodeGenerator::new();
    
    // Create a function with control flow
    let function = IrFunction {
        name: "test_control_flow".to_string(),
        params: vec![
            IrParam {
                name: "condition".to_string(),
                param_type: IrType::Bool,
                register: IrRegister { id: 0 },
            },
        ],
        return_type: Some(IrType::I32),
        locals: vec![],
        basic_blocks: vec![
            IrBasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: IrTerminator::ConditionalBranch {
                    condition: IrValue::Register(IrRegister { id: 0 }),
                    true_label: "then_block".to_string(),
                    false_label: "else_block".to_string(),
                },
            },
            IrBasicBlock {
                label: "then_block".to_string(),
                instructions: vec![
                    IrInstruction {
                        opcode: IrOpcode::Copy,
                        result: Some(IrRegister { id: 1 }),
                        operands: vec![IrValue::Constant(IrConstant::Integer(42))],
                        position: test_position(),
                    },
                ],
                terminator: IrTerminator::Branch("exit".to_string()),
            },
            IrBasicBlock {
                label: "else_block".to_string(),
                instructions: vec![
                    IrInstruction {
                        opcode: IrOpcode::Copy,
                        result: Some(IrRegister { id: 1 }),
                        operands: vec![IrValue::Constant(IrConstant::Integer(0))],
                        position: test_position(),
                    },
                ],
                terminator: IrTerminator::Branch("exit".to_string()),
            },
            IrBasicBlock {
                label: "exit".to_string(),
                instructions: vec![],
                terminator: IrTerminator::Return(Some(IrValue::Register(IrRegister { id: 1 }))),
            },
        ],
        is_async: false,
        position: test_position(),
    };

    let program = IrProgram {
        functions: vec![function],
        globals: vec![],
        structs: vec![],
        interfaces: vec![],
    };
    
    let result = generator.generate_assembly(&program);
    assert!(result.is_ok());
    
    let assembly = result.unwrap();
    assert!(!assembly.is_empty());
    assert!(assembly.contains("then_block"));
    assert!(assembly.contains("else_block"));
}

#[test]
fn test_simple_program_compilation() {
    let mut generator = CodeGenerator::new();
    let program = create_test_program();
    
    // Test both assembly and executable generation
    let assembly_result = generator.generate_assembly(&program);
    assert!(assembly_result.is_ok());
    
    let executable_result = generator.generate_executable(&program);
    assert!(executable_result.is_ok());
    
    let assembly = assembly_result.unwrap();
    let executable = executable_result.unwrap();
    
    assert!(!assembly.is_empty());
    assert!(!executable.is_empty());
    assert!(assembly.contains("test_function"));
}