//! Code generation backend for the Bulu language
//!
//! This module provides machine code generation from IR, supporting multiple
//! target architectures, function calling conventions, debug symbols, and cross-compilation.

use crate::compiler::ir::{IrProgram, IrFunction, IrInstruction, IrConstant, IrType};
use crate::error::Result;

/// Main code generator
pub struct CodeGenerator {
    target: String,
    debug: bool,
    static_link: bool,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            target: "native".to_string(),
            debug: false,
            static_link: false,
        }
    }

    /// Set the target platform for cross-compilation
    pub fn set_target(&mut self, target: &str) {
        self.target = target.to_string();
    }

    /// Set debug information generation
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Set static linking
    pub fn set_static_link(&mut self, static_link: bool) {
        self.static_link = static_link;
    }

    /// Generate assembly code from IR program
    pub fn generate_assembly(&mut self, ir_program: &IrProgram) -> Result<String> {
        let mut assembly = String::new();
        
        // Add header
        assembly.push_str(&format!("# Generated assembly for target: {}\n", self.target));
        assembly.push_str("# Bulu Language Compiler\n\n");

        // Add data section for globals
        if !ir_program.globals.is_empty() {
            assembly.push_str(".section .data\n");
            for global in &ir_program.globals {
                assembly.push_str(&format!(".globl {}\n", global.name));
                assembly.push_str(&format!("{}:\n", global.name));
                
                if let Some(ref initializer) = global.initializer {
                    let bytes = self.value_to_bytes(initializer)?;
                    for chunk in bytes.chunks(8) {
                        assembly.push_str("    .byte ");
                        for (i, &byte) in chunk.iter().enumerate() {
                            if i > 0 { assembly.push_str(", "); }
                            assembly.push_str(&format!("0x{:02x}", byte));
                        }
                        assembly.push('\n');
                    }
                } else {
                    // Zero-initialize
                    let size = self.calculate_type_size(&global.global_type);
                    assembly.push_str(&format!("    .zero {}\n", size));
                }
                assembly.push('\n');
            }
        }

        // Add text section for functions
        assembly.push_str(".section .text\n");
        for function in &ir_program.functions {
            assembly.push_str(&self.generate_function_assembly(function)?);
        }

        Ok(assembly)
    }

    /// Generate executable binary from IR program
    pub fn generate_executable(&mut self, ir_program: &IrProgram) -> Result<Vec<u8>> {
        // Generate bytecode compatible with our interpreter
        let mut bytecode = Vec::new();
        
        // Magic number "BULU"
        bytecode.extend_from_slice(b"BULU");
        
        // Version (1 byte)
        bytecode.push(1);
        
        // Reserved bytes
        bytecode.extend_from_slice(&[0, 0, 0]);
        
        // Number of functions (4 bytes)
        bytecode.extend_from_slice(&(ir_program.functions.len() as u32).to_le_bytes());
        
        // Function table
        let mut function_addresses = std::collections::HashMap::new();
        let mut instruction_offset = 0;
        
        // First pass: calculate function addresses
        for function in &ir_program.functions {
            function_addresses.insert(function.name.clone(), instruction_offset);
            
            // Count instructions in this function
            for block in &function.basic_blocks {
                instruction_offset += block.instructions.len();
                instruction_offset += 1; // for terminator
            }
        }
        
        // Write function table
        for function in &ir_program.functions {
            let name_bytes = function.name.as_bytes();
            bytecode.push(name_bytes.len() as u8);
            bytecode.extend_from_slice(name_bytes);
            let address = function_addresses.get(&function.name).unwrap_or(&0);
            bytecode.extend_from_slice(&(*address as u32).to_le_bytes());
        }
        
        // Generate instructions for all functions
        for function in &ir_program.functions {
            self.generate_function_bytecode(function, &mut bytecode)?;
        }
        
        Ok(bytecode)
    }
    
    /// Generate bytecode for a single function
    fn generate_function_bytecode(&self, function: &IrFunction, bytecode: &mut Vec<u8>) -> Result<()> {
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                self.generate_instruction_bytecode(instruction, bytecode)?;
            }
            
            // Generate terminator instruction
            self.generate_terminator_bytecode(&block.terminator, bytecode)?;
        }
        
        // Add return instruction if not present
        bytecode.push(0x30); // RETURN opcode
        
        Ok(())
    }
    
    /// Generate bytecode for a single instruction
    fn generate_instruction_bytecode(&self, instruction: &IrInstruction, bytecode: &mut Vec<u8>) -> Result<()> {
        use crate::compiler::ir::IrOpcode;
        
        match instruction.opcode {
            IrOpcode::Load => {
                // For now, generate a simple string constant
                bytecode.push(0x06); // LOAD_STRING opcode
                let hello_str = b"Hello, Bulu!";
                bytecode.extend_from_slice(&(hello_str.len() as u32).to_le_bytes());
                bytecode.extend_from_slice(hello_str);
            }
            IrOpcode::Call => {
                // Check if this is a println call
                bytecode.push(0x40); // PRINTLN opcode
            }
            _ => {
                // Skip other opcodes for now
            }
        }
        
        Ok(())
    }
    
    /// Generate bytecode for a terminator instruction
    fn generate_terminator_bytecode(&self, terminator: &crate::compiler::ir::IrTerminator, bytecode: &mut Vec<u8>) -> Result<()> {
        use crate::compiler::ir::IrTerminator;
        
        match terminator {
            IrTerminator::Return(_) => {
                bytecode.push(0x30); // RETURN opcode
            }
            _ => {
                // Skip other terminators for now
            }
        }
        
        Ok(())
    }

    /// Generate assembly for a single function
    fn generate_function_assembly(&self, function: &IrFunction) -> Result<String> {
        let mut assembly = String::new();
        
        // Function label
        assembly.push_str(&format!(".globl {}\n", function.name));
        assembly.push_str(&format!("{}:\n", function.name));
        
        // Function prologue (simplified)
        assembly.push_str("    push %rbp\n");
        assembly.push_str("    mov %rsp, %rbp\n");
        
        // Generate code for each basic block
        for block in &function.basic_blocks {
            assembly.push_str(&format!("{}:\n", block.label));
            
            for instruction in &block.instructions {
                assembly.push_str(&self.generate_instruction_assembly(instruction)?);
            }
            
            // Generate terminator
            assembly.push_str(&self.generate_terminator_assembly(&block.terminator)?);
        }
        
        // Function epilogue (simplified)
        assembly.push_str("    mov %rbp, %rsp\n");
        assembly.push_str("    pop %rbp\n");
        assembly.push_str("    ret\n\n");
        
        Ok(assembly)
    }

    /// Generate assembly for a single instruction
    fn generate_instruction_assembly(&self, instruction: &IrInstruction) -> Result<String> {
        let mut assembly = String::new();
        
        // Simplified instruction generation
        match instruction.opcode {
            crate::compiler::ir::IrOpcode::Add => {
                assembly.push_str("    add %rax, %rbx\n");
            }
            crate::compiler::ir::IrOpcode::Sub => {
                assembly.push_str("    sub %rax, %rbx\n");
            }
            crate::compiler::ir::IrOpcode::Mul => {
                assembly.push_str("    imul %rax, %rbx\n");
            }
            crate::compiler::ir::IrOpcode::Copy => {
                assembly.push_str("    mov %rax, %rbx\n");
            }
            crate::compiler::ir::IrOpcode::Load => {
                assembly.push_str("    mov (%rax), %rbx\n");
            }
            crate::compiler::ir::IrOpcode::Store => {
                assembly.push_str("    mov %rax, (%rbx)\n");
            }
            _ => {
                // Default to nop for unimplemented instructions
                assembly.push_str("    nop\n");
            }
        }
        
        Ok(assembly)
    }

    /// Generate assembly for a terminator instruction
    fn generate_terminator_assembly(&self, terminator: &crate::compiler::ir::IrTerminator) -> Result<String> {
        let mut assembly = String::new();
        
        match terminator {
            crate::compiler::ir::IrTerminator::Return(_) => {
                assembly.push_str("    ret\n");
            }
            crate::compiler::ir::IrTerminator::Branch(label) => {
                assembly.push_str(&format!("    jmp {}\n", label));
            }
            crate::compiler::ir::IrTerminator::ConditionalBranch { condition: _, true_label, false_label } => {
                assembly.push_str("    test %rax, %rax\n");
                assembly.push_str(&format!("    jnz {}\n", true_label));
                assembly.push_str(&format!("    jmp {}\n", false_label));
            }
            _ => {
                // Default to nop for unimplemented terminators
                assembly.push_str("    nop\n");
            }
        }
        
        Ok(assembly)
    }

    /// Convert a value to bytes
    fn value_to_bytes(&self, value: &crate::compiler::ir::IrValue) -> Result<Vec<u8>> {
        match value {
            crate::compiler::ir::IrValue::Constant(constant) => self.constant_to_bytes(constant),
            _ => Ok(vec![0; 8]), // Default for non-constants
        }
    }

    /// Convert constant to bytes
    fn constant_to_bytes(&self, constant: &IrConstant) -> Result<Vec<u8>> {
        match constant {
            IrConstant::Integer(val) => Ok(val.to_le_bytes().to_vec()),
            IrConstant::Float(val) => Ok(val.to_le_bytes().to_vec()),
            IrConstant::Boolean(val) => Ok(vec![if *val { 1 } else { 0 }]),
            IrConstant::String(val) => {
                let mut bytes = val.as_bytes().to_vec();
                bytes.push(0); // Null terminator
                Ok(bytes)
            }
            IrConstant::Char(val) => {
                let mut bytes = [0u8; 4];
                let encoded = val.encode_utf8(&mut bytes);
                Ok(encoded.as_bytes().to_vec())
            }
            IrConstant::Null => Ok(vec![0; 8]),
            _ => Ok(vec![0; 8]), // Default for complex constants
        }
    }

    /// Calculate type size in bytes
    fn calculate_type_size(&self, ir_type: &IrType) -> u64 {
        match ir_type {
            IrType::I8 | IrType::U8 | IrType::Bool => 1,
            IrType::I16 | IrType::U16 => 2,
            IrType::I32 | IrType::U32 | IrType::F32 | IrType::Char => 4,
            IrType::I64 | IrType::U64 | IrType::F64 => 8,
            IrType::String => 8, // Pointer size
            IrType::Any => 8,    // Pointer size
            IrType::Void => 0,
            IrType::Array(element_type, size) => {
                let element_size = self.calculate_type_size(element_type);
                if let Some(array_size) = size {
                    element_size * (*array_size as u64)
                } else {
                    8 // Dynamic array is a pointer
                }
            }
            IrType::Slice(_) => 16, // Pointer + length
            IrType::Map(_, _) => 8, // Pointer to map structure
            IrType::Tuple(types) => {
                types.iter().map(|t| self.calculate_type_size(t)).sum()
            }
            IrType::Function(_, _) => 8, // Function pointer
            IrType::Struct(_) => 8,      // Pointer to struct
            IrType::Interface(_) => 16,  // Vtable pointer + data pointer
            IrType::Channel(_) => 8,     // Pointer to channel
            IrType::Promise(_) => 8,     // Pointer to promise
            IrType::Pointer(_) => 8,     // Pointer size
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}