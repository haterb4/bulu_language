//! Code generation backend for the Bulu language
//!
//! This module provides machine code generation from IR, supporting multiple
//! target architectures, function calling conventions, debug symbols, and cross-compilation.

use crate::compiler::ir::{IrConstant, IrFunction, IrInstruction, IrProgram, IrType};
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
        assembly.push_str(&format!(
            "# Generated assembly for target: {}\n",
            self.target
        ));
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
                            if i > 0 {
                                assembly.push_str(", ");
                            }
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
        // Check if we should generate bytecode or native executable
        if self.should_generate_bytecode() {
            // Debug mode: generate bytecode for fast compilation
            self.generate_bytecode(ir_program)
        } else {
            // Release mode: generate native executable (Go-style)
            use crate::compiler::native_backend::NativeBackend;
            let backend = NativeBackend::new();
            backend.generate_executable(ir_program)
        }
    }

    /// Check if the output should be pure bytecode (for debug mode)
    pub fn is_bytecode_output(&self) -> bool {
        self.debug
    }
    
    /// Determine if we should generate bytecode (debug mode) or native executable (release mode)
    fn should_generate_bytecode(&self) -> bool {
        // Generate bytecode for debug mode (fast compilation)
        // Generate native code for release mode (optimized)
        // For now, always generate native code to test the backend
        false
    }

    /// Generate bytecode compatible with our interpreter
    fn generate_bytecode(&mut self, ir_program: &IrProgram) -> Result<Vec<u8>> {
        let mut bytecode = Vec::new();

        // Magic number "BULU"
        bytecode.extend_from_slice(b"BULU");

        // Version (1 byte)
        bytecode.push(1);

        // Reserved bytes
        bytecode.extend_from_slice(&[0, 0, 0]);

        // Function count
        bytecode.extend_from_slice(&(ir_program.functions.len() as u32).to_le_bytes());

        // Generate bytecode for each function
        for function in &ir_program.functions {
            self.generate_function_bytecode(function, &mut bytecode)?;
        }

        Ok(bytecode)
    }

    /// Create a standalone executable that embeds the interpreter and bytecode
    fn create_standalone_executable(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        // Create a real executable by embedding the bytecode into a compiled binary
        // This approach is similar to Go's compilation model

        #[cfg(unix)]
        {
            self.create_elf_executable(bytecode)
        }

        #[cfg(windows)]
        {
            self.create_pe_executable(bytecode)
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback: create a simple wrapper
            self.create_simple_wrapper(bytecode)
        }
    }

    #[cfg(unix)]
    fn create_elf_executable(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        // Create a minimal ELF executable that contains:
        // 1. A small runtime stub that can execute the bytecode
        // 2. The bytecode embedded in a data section

        // For now, we'll create a more efficient approach:
        // Compile a minimal C program that includes the bytecode as data
        // and links with a minimal Bulu runtime

        self.create_native_executable_with_embedded_runtime(bytecode)
    }

    #[cfg(windows)]
    fn create_pe_executable(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        // Similar approach for Windows PE format
        self.create_native_executable_with_embedded_runtime(bytecode)
    }

    fn create_native_executable_with_embedded_runtime(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        use std::env;
        use std::fs;
        use std::process::Command;

        // Create a temporary directory for compilation
        let temp_dir = env::temp_dir().join(format!("bulu_compile_{}", std::process::id()));
        fs::create_dir_all(&temp_dir)?;

        // Generate a C source file that embeds the bytecode and minimal runtime
        let c_source = self.generate_c_runtime_with_bytecode(bytecode)?;
        let c_file = temp_dir.join("main.c");
        fs::write(&c_file, c_source)?;

        // Compile with gcc/clang to create a native executable
        let output_file = temp_dir.join("executable");
        let mut cmd = Command::new("gcc");
        cmd.arg("-O2")
            .arg("-static")
            .arg("-o")
            .arg(&output_file)
            .arg(&c_file);

        let output = cmd
            .output()
            .map_err(|e| crate::error::BuluError::Other(format!("Failed to run gcc: {}", e)))?;

        if !output.status.success() {
            return Err(crate::error::BuluError::Other(format!(
                "Compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Read the compiled executable
        let executable_data = fs::read(&output_file)?;

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(executable_data)
    }

    fn generate_c_runtime_with_bytecode(&self, bytecode: &[u8]) -> Result<String> {
        // Generate a minimal C program that executes our Bulu program
        // This approach creates a simple interpreter that executes the bytecode

        let mut c_code = String::new();

        // Headers
        c_code.push_str("#include <stdio.h>\n");
        c_code.push_str("#include <stdlib.h>\n");
        c_code.push_str("#include <string.h>\n");
        c_code.push_str("#include <stdint.h>\n\n");

        // Embed bytecode as static data
        c_code.push_str("static const unsigned char bytecode[] = {\n    ");
        for (i, byte) in bytecode.iter().enumerate() {
            if i > 0 && i % 16 == 0 {
                c_code.push_str(",\n    ");
            } else if i > 0 {
                c_code.push_str(", ");
            }
            c_code.push_str(&format!("0x{:02x}", byte));
        }
        c_code.push_str("\n};\n\n");

        c_code.push_str(&format!(
            "static const size_t bytecode_size = {};\n\n",
            bytecode.len()
        ));

        // Generate a simple bytecode interpreter
        c_code.push_str(
            r#"
// Simple bytecode interpreter
typedef struct {
    const unsigned char* code;
    size_t code_size;
    size_t pc;  // program counter
    double stack[1000];  // simple stack for values
    int stack_top;
} Interpreter;

void interpreter_init(Interpreter* interp, const unsigned char* code, size_t size) {
    interp->code = code;
    interp->code_size = size;
    interp->pc = 8;  // Skip header (BULU + version + reserved + function count)
    interp->stack_top = 0;
}

void interpreter_push(Interpreter* interp, double value) {
    if (interp->stack_top < 1000) {
        interp->stack[interp->stack_top++] = value;
    }
}

double interpreter_pop(Interpreter* interp) {
    if (interp->stack_top > 0) {
        return interp->stack[--interp->stack_top];
    }
    return 0.0;
}

int interpreter_run(Interpreter* interp) {
    while (interp->pc < interp->code_size) {
        unsigned char opcode = interp->code[interp->pc++];
        
        switch (opcode) {
            case 0x06: {  // LOAD_STRING
                uint32_t len = *(uint32_t*)&interp->code[interp->pc];
                interp->pc += 4;
                char* str = (char*)&interp->code[interp->pc];
                interp->pc += len;
                printf("%.*s", (int)len, str);
                break;
            }
            case 0x40: {  // PRINTLN
                printf("\n");
                break;
            }
            case 0x30: {  // RETURN
                return 0;
            }
            default:
                // Skip unknown opcodes
                break;
        }
    }
    return 0;
}

int main() {
    // Validate bytecode
    if (bytecode_size < 4 || memcmp(bytecode, "BULU", 4) != 0) {
        fprintf(stderr, "Invalid bytecode format\n");
        return 1;
    }
    
    // Create and run interpreter
    Interpreter interp;
    interpreter_init(&interp, bytecode, bytecode_size);
    return interpreter_run(&interp);
}
"#,
        );

        Ok(c_code)
    }

    fn create_simple_wrapper(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        // Fallback: create a shell script wrapper
        #[cfg(unix)]
        {
            let script = self.create_unix_wrapper(bytecode)?;
            Ok(script.into_bytes())
        }

        #[cfg(not(unix))]
        {
            // Just return the bytecode
            Ok(bytecode.to_vec())
        }
    }

    #[cfg(unix)]
    fn create_unix_wrapper(&self, bytecode: &[u8]) -> Result<String> {
        use std::process::Command;

        // Try to find the bulu_vm executable
        let bulu_vm_path = self.find_bulu_vm_executable()?;

        // Create a self-extracting script
        use base64::{engine::general_purpose, Engine as _};
        let bytecode_base64 = general_purpose::STANDARD.encode(bytecode);

        let script = format!(
            r#"#!/bin/bash
# Bulu compiled executable
# This is a self-contained executable generated by langc

# Create temporary file for bytecode
TEMP_BYTECODE=$(mktemp)
trap "rm -f $TEMP_BYTECODE" EXIT

# Decode embedded bytecode
echo "{}" | base64 -d > "$TEMP_BYTECODE"

# Execute with bulu_vm
exec "{}" "$TEMP_BYTECODE"
"#,
            bytecode_base64, bulu_vm_path
        );

        Ok(script)
    }

    #[cfg(windows)]
    fn create_windows_wrapper(&self, bytecode: &[u8]) -> Result<String> {
        let bulu_vm_path = self.find_bulu_vm_executable()?;
        use base64::{engine::general_purpose, Engine as _};
        let bytecode_base64 = general_purpose::STANDARD.encode(bytecode);

        let script = format!(
            r#"@echo off
REM Bulu compiled executable
REM This is a self-contained executable generated by langc

REM Create temporary file for bytecode
set TEMP_BYTECODE=%TEMP%\bulu_bytecode_%RANDOM%.tmp

REM Decode embedded bytecode (requires certutil on Windows)
echo {}| certutil -decode -f - "%TEMP_BYTECODE%" >nul

REM Execute with bulu_vm
"{}" "%TEMP_BYTECODE%"

REM Cleanup
del "%TEMP_BYTECODE%"
"#,
            bytecode_base64, bulu_vm_path
        );

        Ok(script)
    }

    fn find_bulu_vm_executable(&self) -> Result<String> {
        use std::env;
        use std::path::Path;

        // Try to find bulu (our VM) in the same directory as the current executable
        if let Ok(current_exe) = env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let bulu_path = parent.join("bulu");
                if bulu_path.exists() {
                    return Ok(bulu_path.to_string_lossy().to_string());
                }

                // Try with .exe extension on Windows
                #[cfg(windows)]
                {
                    let bulu_exe = parent.join("bulu.exe");
                    if bulu_exe.exists() {
                        return Ok(bulu_exe.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Try to find in PATH
        if let Ok(path_var) = env::var("PATH") {
            for path_dir in env::split_paths(&path_var) {
                let bulu_path = path_dir.join("bulu");
                if bulu_path.exists() {
                    return Ok(bulu_path.to_string_lossy().to_string());
                }

                #[cfg(windows)]
                {
                    let bulu_exe = path_dir.join("bulu.exe");
                    if bulu_exe.exists() {
                        return Ok(bulu_exe.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Fallback: assume bulu is in PATH
        #[cfg(windows)]
        return Ok("bulu.exe".to_string());

        #[cfg(not(windows))]
        return Ok("bulu".to_string());
    }

    /// Generate bytecode for a single function
    fn generate_function_bytecode(
        &self,
        function: &IrFunction,
        bytecode: &mut Vec<u8>,
    ) -> Result<()> {
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
    fn generate_instruction_bytecode(
        &self,
        instruction: &IrInstruction,
        bytecode: &mut Vec<u8>,
    ) -> Result<()> {
        use crate::compiler::ir::IrOpcode;

        match instruction.opcode {
            IrOpcode::Load => {
                // Check if we're loading a string constant
                if let Some(operand) = instruction.operands.first() {
                    if let crate::compiler::ir::IrValue::Constant(constant) = operand {
                        if let crate::compiler::ir::IrConstant::String(s) = constant {
                            bytecode.push(0x06); // LOAD_STRING opcode
                            bytecode.extend_from_slice(&(s.len() as u32).to_le_bytes());
                            bytecode.extend_from_slice(s.as_bytes());
                        }
                    }
                }
            }
            IrOpcode::Call => {
                // Check if this is a println call
                if let Some(operand) = instruction.operands.first() {
                    if let crate::compiler::ir::IrValue::Function(name) = operand {
                        if name == "println" {
                            bytecode.push(0x40); // PRINTLN opcode
                        }
                    }
                }
            }
            _ => {
                // Skip other opcodes for now
            }
        }

        Ok(())
    }

    /// Generate bytecode for a terminator instruction
    fn generate_terminator_bytecode(
        &self,
        terminator: &crate::compiler::ir::IrTerminator,
        bytecode: &mut Vec<u8>,
    ) -> Result<()> {
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
    fn generate_terminator_assembly(
        &self,
        terminator: &crate::compiler::ir::IrTerminator,
    ) -> Result<String> {
        let mut assembly = String::new();

        match terminator {
            crate::compiler::ir::IrTerminator::Return(_) => {
                assembly.push_str("    ret\n");
            }
            crate::compiler::ir::IrTerminator::Branch(label) => {
                assembly.push_str(&format!("    jmp {}\n", label));
            }
            crate::compiler::ir::IrTerminator::ConditionalBranch {
                condition: _,
                true_label,
                false_label,
            } => {
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
            IrType::Tuple(types) => types.iter().map(|t| self.calculate_type_size(t)).sum(),
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
