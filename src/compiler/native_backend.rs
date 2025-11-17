//! Native code generation backend for Bulu (Go-style)

use crate::compiler::ir::{
    IrConstant, IrFunction, IrInstruction, IrOpcode, IrProgram, IrTerminator, IrValue,
};
use crate::error::{BuluError, Result};
use std::collections::HashMap;

pub struct NativeBackend {
    target_arch: String,
}

impl NativeBackend {
    pub fn new() -> Self {
        Self {
            target_arch: "x86_64".to_string(),
        }
    }

    /// Generate a native executable from IR (Go-style)
    pub fn generate_executable(&self, ir_program: &IrProgram) -> Result<Vec<u8>> {
        self.generate_executable_with_name(ir_program, "program")
    }

    /// Generate a native executable with a specific name
    pub fn generate_executable_with_name(
        &self,
        ir_program: &IrProgram,
        name: &str,
    ) -> Result<Vec<u8>> {
        let asm_code = self.generate_assembly(ir_program)?;
        self.assemble_and_link(&asm_code, name)
    }

    /// Generate assembly code from IR program
    fn generate_assembly(&self, ir_program: &IrProgram) -> Result<String> {
        let mut asm = String::new();

        // Data section for strings and globals
        asm.push_str(".section .data\n");
        
        // Global variable to store the length of concatenated strings
        asm.push_str("__concat_length: .quad 0\n");
        
        // Heap management globals
        asm.push_str("__heap_start: .quad 0\n");
        asm.push_str("__heap_current: .quad 0\n");
        asm.push_str("__heap_size: .quad 1048576\n");  // 1MB heap

        // Collect all string constants
        let mut strings = HashMap::new();
        let mut string_counter = 0;

        for func in &ir_program.functions {
            for bb in &func.basic_blocks {
                for inst in &bb.instructions {
                    // Collect strings from all instructions
                    for operand in &inst.operands {
                        if let IrValue::Constant(IrConstant::String(s)) = operand {
                            if !strings.contains_key(s) {
                                strings.insert(s.clone(), string_counter);
                                string_counter += 1;
                            }
                        }
                    }
                }
            }
        }

        // Emit string constants
        // Note: strings used in println get a newline, strings used in concatenation don't
        for (string, id) in &strings {
            // Check if this string is used in concatenation (doesn't end with typical sentence markers)
            // For now, emit without newline - println will add it
            asm.push_str(&format!("str_{}: .ascii \"{}\"\n", id, string));
            asm.push_str(&format!("str_{}_len = . - str_{}\n", id, id));
        }

        // Add newline constant
        asm.push_str("newline: .byte 10\n\n");

        // Text section
        asm.push_str(".section .text\n");
        asm.push_str(".global _start\n\n");

        // Generate runtime functions
        self.generate_runtime(&mut asm)?;

        // Generate code for each function
        for func in &ir_program.functions {
            self.generate_function(&mut asm, func, &strings)?;
        }

        // Entry point
        asm.push_str("_start:\n");
        asm.push_str("    call __init_heap\n");
        asm.push_str("    call main\n");
        asm.push_str("    mov $60, %rax    # sys_exit\n");
        asm.push_str("    xor %rdi, %rdi   # exit code 0\n");
        asm.push_str("    syscall\n");

        Ok(asm)
    }

    /// Generate runtime helper functions
    fn generate_runtime(&self, asm: &mut String) -> Result<()> {
        // Compiler intrinsic: print integer function
        asm.push_str("__bulu_print_int:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    sub $32, %rsp\n");
        asm.push_str("    mov %rdi, %rax\n");
        asm.push_str("    lea -32(%rbp), %rsi\n");
        asm.push_str("    mov $0, %rcx\n");
        asm.push_str("    mov $10, %rbx\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jns .print_int_positive\n");
        asm.push_str("    neg %rax\n");
        asm.push_str(".print_int_positive:\n");
        asm.push_str(".print_int_digit_loop:\n");
        asm.push_str("    xor %rdx, %rdx\n");
        asm.push_str("    div %rbx\n");
        asm.push_str("    add $'0', %rdx\n");
        asm.push_str("    push %rdx\n");
        asm.push_str("    inc %rcx\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jnz .print_int_digit_loop\n");
        asm.push_str("    mov %rcx, %rdx\n");
        asm.push_str(".print_int_write_loop:\n");
        asm.push_str("    pop %rax\n");
        asm.push_str("    mov %al, (%rsi)\n");
        asm.push_str("    inc %rsi\n");
        asm.push_str("    loop .print_int_write_loop\n");
        asm.push_str("    # Write to stdout\n");
        asm.push_str("    mov $1, %rax\n");
        asm.push_str("    mov $1, %rdi\n");
        asm.push_str("    lea -32(%rbp), %rsi\n");
        asm.push_str("    syscall\n");
        asm.push_str("    # Write newline\n");
        asm.push_str("    mov $1, %rax\n");
        asm.push_str("    mov $1, %rdi\n");
        asm.push_str("    lea newline(%rip), %rsi\n");
        asm.push_str("    mov $1, %rdx\n");
        asm.push_str("    syscall\n");
        asm.push_str("    add $32, %rsp\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // Compiler intrinsic: convert integer to string
        // Input: %rdi = integer to convert, %rsi = buffer pointer
        // Output: %rax = length of string written
        asm.push_str("__bulu_int_to_string:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    push %rbx\n");
        asm.push_str("    push %r12\n");
        asm.push_str("    push %r13\n");
        asm.push_str("    mov %rdi, %rax\n");
        asm.push_str("    mov %rsi, %r12      # Save buffer pointer\n");
        asm.push_str("    mov $0, %r13        # Digit count\n");
        asm.push_str("    mov $10, %rbx\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jns .i2s_positive\n");
        asm.push_str("    neg %rax\n");
        asm.push_str(".i2s_positive:\n");
        asm.push_str(".i2s_digit_loop:\n");
        asm.push_str("    xor %rdx, %rdx\n");
        asm.push_str("    div %rbx\n");
        asm.push_str("    add $'0', %rdx\n");
        asm.push_str("    push %rdx\n");
        asm.push_str("    inc %r13\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jnz .i2s_digit_loop\n");
        asm.push_str("    mov %r13, %rcx      # Digit count for loop\n");
        asm.push_str("    mov %r12, %rsi      # Restore buffer pointer\n");
        asm.push_str(".i2s_write_loop:\n");
        asm.push_str("    pop %rax\n");
        asm.push_str("    mov %al, (%rsi)\n");
        asm.push_str("    inc %rsi\n");
        asm.push_str("    loop .i2s_write_loop\n");
        asm.push_str("    mov %r13, %rax      # Return length\n");
        asm.push_str("    pop %r13\n");
        asm.push_str("    pop %r12\n");
        asm.push_str("    pop %rbx\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // Initialize heap on program start
        asm.push_str("__init_heap:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    # Allocate heap using brk syscall\n");
        asm.push_str("    mov $12, %rax       # sys_brk\n");
        asm.push_str("    mov $0, %rdi        # Get current break\n");
        asm.push_str("    syscall\n");
        asm.push_str("    movq %rax, __heap_start(%rip)\n");
        asm.push_str("    movq %rax, __heap_current(%rip)\n");
        asm.push_str("    # Extend heap\n");
        asm.push_str("    mov $12, %rax       # sys_brk\n");
        asm.push_str("    movq __heap_start(%rip), %rdi\n");
        asm.push_str("    addq __heap_size(%rip), %rdi\n");
        asm.push_str("    syscall\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // Simple malloc implementation
        // Input: %rdi = size to allocate
        // Output: %rax = pointer to allocated memory (or 0 if failed)
        asm.push_str("__malloc:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    # Align size to 8 bytes\n");
        asm.push_str("    add $7, %rdi\n");
        asm.push_str("    and $-8, %rdi\n");
        asm.push_str("    # Check if we have enough space\n");
        asm.push_str("    movq __heap_current(%rip), %rax\n");
        asm.push_str("    movq %rax, %rcx\n");
        asm.push_str("    addq %rdi, %rcx\n");
        asm.push_str("    movq __heap_start(%rip), %rdx\n");
        asm.push_str("    addq __heap_size(%rip), %rdx\n");
        asm.push_str("    cmpq %rdx, %rcx\n");
        asm.push_str("    ja .malloc_fail\n");
        asm.push_str("    # Update heap pointer\n");
        asm.push_str("    movq %rcx, __heap_current(%rip)\n");
        asm.push_str("    # Return old pointer\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n");
        asm.push_str(".malloc_fail:\n");
        asm.push_str("    mov $0, %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // String structure: [length:8][data:length]
        // Create string from C-style string
        // Input: %rdi = C-string pointer, %rsi = length
        // Output: %rax = string structure pointer
        asm.push_str("__string_create:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    push %rdi\n");
        asm.push_str("    push %rsi\n");
        asm.push_str("    # Allocate memory for length + data\n");
        asm.push_str("    mov %rsi, %rdi\n");
        asm.push_str("    add $8, %rdi        # 8 bytes for length\n");
        asm.push_str("    call __malloc\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jz .string_create_fail\n");
        asm.push_str("    pop %rsi            # length\n");
        asm.push_str("    pop %rdi            # source\n");
        asm.push_str("    # Store length\n");
        asm.push_str("    movq %rsi, (%rax)\n");
        asm.push_str("    # Copy data\n");
        asm.push_str("    lea 8(%rax), %rdx   # destination\n");
        asm.push_str("    mov %rsi, %rcx      # count\n");
        asm.push_str("    push %rax\n");
        asm.push_str("    mov %rdi, %rsi      # source\n");
        asm.push_str("    mov %rdx, %rdi      # destination\n");
        asm.push_str("    rep movsb\n");
        asm.push_str("    pop %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n");
        asm.push_str(".string_create_fail:\n");
        asm.push_str("    pop %rsi\n");
        asm.push_str("    pop %rdi\n");
        asm.push_str("    mov $0, %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // String concatenation: string1 + string2
        // Input: %rdi = string1 pointer, %rsi = string2 pointer
        // Output: %rax = new concatenated string pointer
        asm.push_str("__string_concat:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    push %r12\n");
        asm.push_str("    push %r13\n");
        asm.push_str("    mov %rdi, %r12      # Save string1\n");
        asm.push_str("    mov %rsi, %r13      # Save string2\n");
        asm.push_str("    # Get lengths\n");
        asm.push_str("    movq (%r12), %rcx   # len1\n");
        asm.push_str("    movq (%r13), %rdx   # len2\n");
        asm.push_str("    # Calculate total length\n");
        asm.push_str("    mov %rcx, %rdi\n");
        asm.push_str("    add %rdx, %rdi\n");
        asm.push_str("    add $8, %rdi        # + 8 for length field\n");
        asm.push_str("    push %rcx\n");
        asm.push_str("    push %rdx\n");
        asm.push_str("    call __malloc\n");
        asm.push_str("    pop %rdx\n");
        asm.push_str("    pop %rcx\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jz .concat_fail\n");
        asm.push_str("    # Store total length\n");
        asm.push_str("    mov %rcx, %r8\n");
        asm.push_str("    add %rdx, %r8\n");
        asm.push_str("    movq %r8, (%rax)\n");
        asm.push_str("    # Copy first string\n");
        asm.push_str("    push %rax           # Save result\n");
        asm.push_str("    lea 8(%r12), %rsi   # source1\n");
        asm.push_str("    lea 8(%rax), %rdi   # dest\n");
        asm.push_str("    rep movsb\n");
        asm.push_str("    # Copy second string\n");
        asm.push_str("    lea 8(%r13), %rsi   # source2\n");
        asm.push_str("    mov %rdx, %rcx      # len2\n");
        asm.push_str("    rep movsb\n");
        asm.push_str("    pop %rax            # Restore result\n");
        asm.push_str("    pop %r13\n");
        asm.push_str("    pop %r12\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n");
        asm.push_str(".concat_fail:\n");
        asm.push_str("    mov $0, %rax\n");
        asm.push_str("    pop %r13\n");
        asm.push_str("    pop %r12\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // Print string structure
        // Input: %rdi = string structure pointer
        asm.push_str("__string_print:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    test %rdi, %rdi\n");
        asm.push_str("    jz .string_print_null\n");
        asm.push_str("    # Get length and data\n");
        asm.push_str("    movq (%rdi), %rdx   # length\n");
        asm.push_str("    lea 8(%rdi), %rsi   # data\n");
        asm.push_str("    # Write syscall\n");
        asm.push_str("    mov $1, %rax\n");
        asm.push_str("    mov $1, %rdi\n");
        asm.push_str("    syscall\n");
        asm.push_str("    # Print newline\n");
        asm.push_str("    mov $1, %rax\n");
        asm.push_str("    mov $1, %rdi\n");
        asm.push_str("    lea newline(%rip), %rsi\n");
        asm.push_str("    mov $1, %rdx\n");
        asm.push_str("    syscall\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n");
        asm.push_str(".string_print_null:\n");
        asm.push_str("    # Print \"(null)\"\n");
        asm.push_str("    mov $1, %rax\n");
        asm.push_str("    mov $1, %rdi\n");
        asm.push_str("    lea .null_str(%rip), %rsi\n");
        asm.push_str("    mov $7, %rdx\n");
        asm.push_str("    syscall\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // String uppercase
        // Input: %rdi = string structure pointer
        // Output: %rax = new uppercase string pointer
        asm.push_str("__string_uppercase:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    test %rdi, %rdi\n");
        asm.push_str("    jz .uppercase_null\n");
        asm.push_str("    push %rdi\n");
        asm.push_str("    # Get length\n");
        asm.push_str("    movq (%rdi), %rsi   # length\n");
        asm.push_str("    lea 8(%rdi), %rdi   # source data\n");
        asm.push_str("    call __string_create\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jz .uppercase_fail\n");
        asm.push_str("    pop %rdi            # original string\n");
        asm.push_str("    # Convert to uppercase\n");
        asm.push_str("    movq (%rdi), %rcx   # length\n");
        asm.push_str("    lea 8(%rdi), %rsi   # source\n");
        asm.push_str("    lea 8(%rax), %rdi   # dest\n");
        asm.push_str("    push %rax\n");
        asm.push_str(".uppercase_loop:\n");
        asm.push_str("    test %rcx, %rcx\n");
        asm.push_str("    jz .uppercase_done\n");
        asm.push_str("    movb (%rsi), %al\n");
        asm.push_str("    # Check if lowercase letter\n");
        asm.push_str("    cmpb $'a', %al\n");
        asm.push_str("    jb .uppercase_copy\n");
        asm.push_str("    cmpb $'z', %al\n");
        asm.push_str("    ja .uppercase_copy\n");
        asm.push_str("    # Convert to uppercase\n");
        asm.push_str("    subb $32, %al\n");
        asm.push_str(".uppercase_copy:\n");
        asm.push_str("    movb %al, (%rdi)\n");
        asm.push_str("    inc %rsi\n");
        asm.push_str("    inc %rdi\n");
        asm.push_str("    dec %rcx\n");
        asm.push_str("    jmp .uppercase_loop\n");
        asm.push_str(".uppercase_done:\n");
        asm.push_str("    pop %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n");
        asm.push_str(".uppercase_null:\n");
        asm.push_str(".uppercase_fail:\n");
        asm.push_str("    mov $0, %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // String repeat
        // Input: %rdi = source string, %rsi = source length, %rdx = count
        // Output: %rax = new repeated string pointer
        asm.push_str("__string_repeat:\n");
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");
        asm.push_str("    push %rdi\n");
        asm.push_str("    push %rsi\n");
        asm.push_str("    push %rdx\n");
        asm.push_str("    # Calculate total length\n");
        asm.push_str("    mov %rsi, %rax\n");
        asm.push_str("    mul %rdx\n");
        asm.push_str("    mov %rax, %rdi\n");
        asm.push_str("    add $8, %rdi        # + 8 for length field\n");
        asm.push_str("    call __malloc\n");
        asm.push_str("    test %rax, %rax\n");
        asm.push_str("    jz .repeat_fail\n");
        asm.push_str("    pop %rdx            # count\n");
        asm.push_str("    pop %rsi            # length\n");
        asm.push_str("    pop %rdi            # source\n");
        asm.push_str("    # Store total length\n");
        asm.push_str("    push %rax           # save result pointer\n");
        asm.push_str("    mov %rsi, %rax\n");
        asm.push_str("    mul %rdx\n");
        asm.push_str("    mov %rax, %r8       # total length\n");
        asm.push_str("    pop %rax            # restore result pointer\n");
        asm.push_str("    movq %r8, (%rax)\n");
        asm.push_str("    # Copy string multiple times\n");
        asm.push_str("    lea 8(%rax), %r9    # dest pointer\n");
        asm.push_str("    push %rax\n");
        asm.push_str(".repeat_loop:\n");
        asm.push_str("    test %rdx, %rdx\n");
        asm.push_str("    jz .repeat_done\n");
        asm.push_str("    # Copy one instance\n");
        asm.push_str("    mov %rdi, %rsi      # source\n");
        asm.push_str("    mov %r9, %rdi       # dest\n");
        asm.push_str("    mov -16(%rbp), %rcx # original length\n");
        asm.push_str("    rep movsb\n");
        asm.push_str("    mov %rdi, %r9       # update dest\n");
        asm.push_str("    mov -24(%rbp), %rdi # restore source\n");
        asm.push_str("    dec %rdx\n");
        asm.push_str("    jmp .repeat_loop\n");
        asm.push_str(".repeat_done:\n");
        asm.push_str("    pop %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n");
        asm.push_str(".repeat_fail:\n");
        asm.push_str("    pop %rdx\n");
        asm.push_str("    pop %rsi\n");
        asm.push_str("    pop %rdi\n");
        asm.push_str("    mov $0, %rax\n");
        asm.push_str("    pop %rbp\n");
        asm.push_str("    ret\n\n");
        
        // Add null string constant
        asm.push_str(".section .rodata\n");
        asm.push_str(".null_str: .ascii \"(null)\\n\"\n");
        asm.push_str(".section .text\n\n");
        
        Ok(())
    }

    /// Generate assembly for a single function
    fn generate_function(
        &self,
        asm: &mut String,
        func: &IrFunction,
        strings: &HashMap<String, usize>,
    ) -> Result<()> {
        asm.push_str(&format!("{}:\n", func.name));
        asm.push_str("    push %rbp\n");
        asm.push_str("    mov %rsp, %rbp\n");

        // Collect all registers used in the function
        let mut all_registers = std::collections::HashSet::new();
        for local in &func.locals {
            all_registers.insert(local.register.id);
        }
        for bb in &func.basic_blocks {
            for inst in &bb.instructions {
                if let Some(result) = inst.result {
                    all_registers.insert(result.id);
                }
                for operand in &inst.operands {
                    if let IrValue::Register(reg) = operand {
                        all_registers.insert(reg.id);
                    }
                }
            }
        }

        // Allocate stack space for all registers
        let num_registers = all_registers.len();
        if num_registers > 0 {
            let stack_size = num_registers * 8;
            asm.push_str(&format!("    sub ${}, %rsp\n", stack_size));
        }

        // Map registers to stack offsets
        let mut reg_map: HashMap<u32, i32> = HashMap::new();
        let mut sorted_regs: Vec<_> = all_registers.into_iter().collect();
        sorted_regs.sort();
        for (i, reg_id) in sorted_regs.iter().enumerate() {
            reg_map.insert(*reg_id, -((i + 1) as i32 * 8));
        }

        // Copy function parameters from registers to stack
        // System V AMD64 ABI: rdi, rsi, rdx, rcx, r8, r9
        let param_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
        for (i, param) in func.params.iter().enumerate() {
            if i >= 6 {
                // TODO: Handle stack parameters
                break;
            }
            if let Some(&offset) = reg_map.get(&param.register.id) {
                asm.push_str(&format!("    movq {}, {}(%rbp)\n", param_regs[i], offset));
            }
        }

        // Generate code for each basic block
        let mut label_counter = 0;
        for (i, bb) in func.basic_blocks.iter().enumerate() {
            // Only add label for non-first blocks
            // Make labels unique by prefixing with function name
            if i > 0 {
                asm.push_str(&format!(".{}_{}:\n", func.name, bb.label));
            }

            for inst in &bb.instructions {
                self.generate_instruction(asm, inst, &reg_map, strings, &func.name, &mut label_counter)?;
            }

            self.generate_terminator(asm, &bb.terminator, &reg_map, &func.name)?;
        }

        asm.push_str("\n");
        Ok(())
    }

    /// Helper to generate comparison instruction
    fn generate_comparison(
        &self,
        asm: &mut String,
        inst: &IrInstruction,
        reg_map: &HashMap<u32, i32>,
        set_instruction: &str, // e.g., "setg", "setl", "sete"
    ) {
        if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
            if let Some(result) = inst.result {
                if let Some(&res_off) = reg_map.get(&result.id) {
                    match (op1, op2) {
                        (IrValue::Register(r1), IrValue::Register(r2)) => {
                            if let (Some(&off1), Some(&off2)) = (reg_map.get(&r1.id), reg_map.get(&r2.id)) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str(&format!("    cmpq {}(%rbp), %rax\n", off2));
                                asm.push_str(&format!("    {} %al\n", set_instruction));
                                asm.push_str("    movzbq %al, %rax\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                        (IrValue::Register(r1), IrValue::Constant(IrConstant::Integer(val))) => {
                            if let Some(&off1) = reg_map.get(&r1.id) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str(&format!("    cmpq ${}, %rax\n", val));
                                asm.push_str(&format!("    {} %al\n", set_instruction));
                                asm.push_str("    movzbq %al, %rax\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                        (IrValue::Constant(IrConstant::Integer(val)), IrValue::Register(r2)) => {
                            if let Some(&off2) = reg_map.get(&r2.id) {
                                asm.push_str(&format!("    mov ${}, %rax\n", val));
                                asm.push_str(&format!("    cmpq {}(%rbp), %rax\n", off2));
                                asm.push_str(&format!("    {} %al\n", set_instruction));
                                asm.push_str("    movzbq %al, %rax\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Generate assembly for a single instruction
    fn generate_instruction(
        &self,
        asm: &mut String,
        inst: &IrInstruction,
        reg_map: &HashMap<u32, i32>,
        strings: &HashMap<String, usize>,
        func_name: &str,
        label_counter: &mut usize,
    ) -> Result<()> {
        match inst.opcode {
            IrOpcode::Copy => {
                if let Some(operand) = inst.operands.first() {
                    if let Some(result) = inst.result {
                        if let Some(&res_offset) = reg_map.get(&result.id) {
                            match operand {
                                IrValue::Constant(IrConstant::Integer(val)) => {
                                    asm.push_str(&format!(
                                        "    movq ${}, {}(%rbp)\n",
                                        val, res_offset
                                    ));
                                }
                                IrValue::Constant(IrConstant::Boolean(val)) => {
                                    let int_val = if *val { 1 } else { 0 };
                                    asm.push_str(&format!(
                                        "    movq ${}, {}(%rbp)\n",
                                        int_val, res_offset
                                    ));
                                }
                                IrValue::Constant(IrConstant::String(s)) => {
                                    // Create string structure for string constant
                                    if let Some(&id) = strings.get(s) {
                                        asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                                        asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                                        asm.push_str("    call __string_create\n");
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                    }
                                }
                                IrValue::Register(src_reg) => {
                                    if let Some(&src_offset) = reg_map.get(&src_reg.id) {
                                        asm.push_str(&format!(
                                            "    movq {}(%rbp), %rax\n",
                                            src_offset
                                        ));
                                        asm.push_str(&format!(
                                            "    movq %rax, {}(%rbp)\n",
                                            res_offset
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            IrOpcode::Add => {
                // Integer addition only (string concatenation uses StringConcat)
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let (IrValue::Register(r1), IrValue::Register(r2)) = (op1, op2) {
                        // Register + Register addition
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&off2), Some(&res_off)) = (
                                reg_map.get(&r1.id),
                                reg_map.get(&r2.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str(&format!("    addq {}(%rbp), %rax\n", off2));
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                    } else if let (IrValue::Register(r1), IrValue::Constant(IrConstant::Integer(val))) = (op1, op2) {
                        // Register + Constant addition
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&res_off)) = (
                                reg_map.get(&r1.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str(&format!("    addq ${}, %rax\n", val));
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                    } else if let (IrValue::Constant(IrConstant::Integer(val)), IrValue::Register(r2)) = (op1, op2) {
                        // Constant + Register addition
                        if let Some(result) = inst.result {
                            if let (Some(&off2), Some(&res_off)) = (
                                reg_map.get(&r2.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off2));
                                asm.push_str(&format!("    addq ${}, %rax\n", val));
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                    }
                }
            }
            IrOpcode::StringConcat => {
                // String concatenation - always treat operands as strings
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let (IrValue::Constant(IrConstant::String(s1)), IrValue::Constant(IrConstant::String(s2))) = (op1, op2) {
                        // String constant + String constant
                        if let Some(result) = inst.result {
                            if let (Some(&id1), Some(&id2), Some(&res_offset)) = (strings.get(s1), strings.get(s2), reg_map.get(&result.id)) {
                                // Create first string
                                asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id1));
                                asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id1));
                                asm.push_str("    call __string_create\n");
                                asm.push_str("    push %rax\n");
                                // Create second string
                                asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id2));
                                asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id2));
                                asm.push_str("    call __string_create\n");
                                asm.push_str("    mov %rax, %rsi\n");
                                asm.push_str("    pop %rdi\n");
                                asm.push_str("    call __string_concat\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                            }
                        }
                    } else if let (IrValue::Register(r1), IrValue::Register(r2)) = (op1, op2) {
                        // Register (string) + Register (string)
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&off2), Some(&res_off)) = (
                                reg_map.get(&r1.id),
                                reg_map.get(&r2.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rdi\n", off1));
                                asm.push_str(&format!("    movq {}(%rbp), %rsi\n", off2));
                                asm.push_str("    call __string_concat\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                    } else if let (IrValue::Register(r1), IrValue::Constant(IrConstant::String(s))) = (op1, op2) {
                        // Register (string) + String constant
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&id), Some(&res_offset)) = (reg_map.get(&r1.id), strings.get(s), reg_map.get(&result.id)) {
                                // Load first string from register
                                asm.push_str(&format!("    movq {}(%rbp), %rdi\n", off1));
                                asm.push_str("    push %rdi\n");
                                // Create second string
                                asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                                asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                                asm.push_str("    call __string_create\n");
                                asm.push_str("    mov %rax, %rsi\n");
                                asm.push_str("    pop %rdi\n");
                                asm.push_str("    call __string_concat\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                            }
                        }
                    } else if let (IrValue::Constant(IrConstant::String(s)), IrValue::Register(r2)) = (op1, op2) {
                        // String constant + Register (string)
                        if let Some(result) = inst.result {
                            if let (Some(&id), Some(&off2), Some(&res_offset)) = (strings.get(s), reg_map.get(&r2.id), reg_map.get(&result.id)) {
                                // Create first string
                                asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                                asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                                asm.push_str("    call __string_create\n");
                                asm.push_str("    mov %rax, %rdi\n");
                                // Load second string from register
                                asm.push_str(&format!("    movq {}(%rbp), %rsi\n", off2));
                                asm.push_str("    call __string_concat\n");
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                            }
                        }
                    }
                }
            }
            IrOpcode::Sub => {
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let Some(result) = inst.result {
                        if let Some(&res_off) = reg_map.get(&result.id) {
                            match (op1, op2) {
                                (IrValue::Register(r1), IrValue::Register(r2)) => {
                                    if let (Some(&off1), Some(&off2)) = (reg_map.get(&r1.id), reg_map.get(&r2.id)) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                        asm.push_str(&format!("    subq {}(%rbp), %rax\n", off2));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                (IrValue::Register(r1), IrValue::Constant(IrConstant::Integer(val))) => {
                                    if let Some(&off1) = reg_map.get(&r1.id) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                        asm.push_str(&format!("    subq ${}, %rax\n", val));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                (IrValue::Constant(IrConstant::Integer(val)), IrValue::Register(r2)) => {
                                    if let Some(&off2) = reg_map.get(&r2.id) {
                                        asm.push_str(&format!("    mov ${}, %rax\n", val));
                                        asm.push_str(&format!("    subq {}(%rbp), %rax\n", off2));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            IrOpcode::Mul => {
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let Some(result) = inst.result {
                        if let Some(&res_off) = reg_map.get(&result.id) {
                            match (op1, op2) {
                                (IrValue::Register(r1), IrValue::Register(r2)) => {
                                    if let (Some(&off1), Some(&off2)) = (reg_map.get(&r1.id), reg_map.get(&r2.id)) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                        asm.push_str(&format!("    imulq {}(%rbp), %rax\n", off2));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                (IrValue::Register(r1), IrValue::Constant(IrConstant::Integer(val))) => {
                                    if let Some(&off1) = reg_map.get(&r1.id) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                        asm.push_str(&format!("    imulq ${}, %rax\n", val));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                (IrValue::Constant(IrConstant::Integer(val)), IrValue::Register(r2)) => {
                                    if let Some(&off2) = reg_map.get(&r2.id) {
                                        asm.push_str(&format!("    mov ${}, %rax\n", val));
                                        asm.push_str(&format!("    imulq {}(%rbp), %rax\n", off2));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            IrOpcode::Call => {
                if let Some(called_func) = inst.operands.first() {
                    let name = match called_func {
                        IrValue::Function(n) => n.as_str(),
                        IrValue::Global(n) => n.as_str(),
                        _ => return Ok(()),
                    };

                    if name == "println" {
                        // Compiler intrinsic: inline syscalls for println
                        // If there are multiple arguments, concatenate them with spaces
                        if inst.operands.len() > 2 {
                            // Multiple arguments - concatenate them with spaces
                            let mut concat_reg = None;
                            for (i, arg) in inst.operands.iter().skip(1).enumerate() {
                                // Add space before each argument except the first
                                if i > 0 && concat_reg.is_some() {
                                    // Create a space string
                                    asm.push_str("    mov $9, %rdi\n");  // 1 byte + 8 for length
                                    asm.push_str("    call __malloc\n");
                                    asm.push_str("    movq $1, (%rax)\n");  // length = 1
                                    asm.push_str("    movb $32, 8(%rax)\n");  // space character
                                    asm.push_str("    mov %rax, %rsi\n");
                                    asm.push_str("    pop %rdi\n");
                                    asm.push_str("    call __string_concat\n");
                                    asm.push_str("    push %rax\n");
                                }
                                
                                // Get the current argument and convert to string if needed
                                match arg {
                                    IrValue::Register(reg) => {
                                        if let Some(&offset) = reg_map.get(&reg.id) {
                                            // Check if it's a string or needs conversion
                                            let label_id = *label_counter;
                                            *label_counter += 1;
                                            
                                            asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                            // Check if it's a valid string pointer (> 0x1000)
                                            asm.push_str("    cmp $0x1000, %rdi\n");
                                            asm.push_str(&format!("    jb .{}_println_convert_int_{}\n", func_name, label_id));
                                            // Check if it looks like a string (length < 1MB)
                                            asm.push_str("    movq (%rdi), %rax\n");
                                            asm.push_str("    cmp $1048576, %rax\n");
                                            asm.push_str(&format!("    ja .{}_println_convert_int_{}\n", func_name, label_id));
                                            
                                            // It's a string - use it directly
                                            asm.push_str(&format!("    jmp .{}_println_string_ready_{}\n", func_name, label_id));
                                            
                                            // Convert integer to string
                                            asm.push_str(&format!(".{}_println_convert_int_{}:\n", func_name, label_id));
                                            asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                            asm.push_str("    mov $32, %rdi\n");
                                            asm.push_str("    call __malloc\n");
                                            asm.push_str("    push %rax\n");
                                            asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                            asm.push_str("    lea 8(%rax), %rsi\n");
                                            asm.push_str("    call __bulu_int_to_string\n");
                                            asm.push_str("    pop %rbx\n");
                                            asm.push_str("    movq %rax, (%rbx)\n");
                                            asm.push_str("    mov %rbx, %rdi\n");
                                            
                                            asm.push_str(&format!(".{}_println_string_ready_{}:\n", func_name, label_id));
                                            
                                            // Now concatenate
                                            if concat_reg.is_some() {
                                                asm.push_str("    mov %rdi, %rsi\n");
                                                asm.push_str("    pop %rdi\n");
                                                asm.push_str("    call __string_concat\n");
                                                asm.push_str("    push %rax\n");
                                            } else {
                                                asm.push_str("    push %rdi\n");
                                                concat_reg = Some(());
                                            }
                                        }
                                    }
                                    IrValue::Constant(IrConstant::String(s)) => {
                                        if let Some(&id) = strings.get(s) {
                                            // Create string from constant
                                            if concat_reg.is_some() {
                                                asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                                                asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                                                asm.push_str("    call __string_create\n");
                                                asm.push_str("    mov %rax, %rsi\n");
                                                asm.push_str("    pop %rdi\n");
                                                asm.push_str("    call __string_concat\n");
                                                asm.push_str("    push %rax\n");
                                            } else {
                                                asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                                                asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                                                asm.push_str("    call __string_create\n");
                                                asm.push_str("    push %rax\n");
                                                concat_reg = Some(());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            
                            // Print the concatenated result
                            if concat_reg.is_some() {
                                asm.push_str("    pop %rdi\n");
                                // Print without newline (we'll add it after)
                                asm.push_str("    movq (%rdi), %rdx\n");  // length
                                asm.push_str("    lea 8(%rdi), %rsi\n");  // data
                                asm.push_str("    mov $1, %rax\n");
                                asm.push_str("    mov $1, %rdi\n");
                                asm.push_str("    syscall\n");
                                // Print newline
                                asm.push_str("    mov $1, %rax\n");
                                asm.push_str("    mov $1, %rdi\n");
                                asm.push_str("    lea newline(%rip), %rsi\n");
                                asm.push_str("    mov $1, %rdx\n");
                                asm.push_str("    syscall\n");
                            }
                        } else if let Some(arg) = inst.operands.get(1) {
                            match arg {
                                IrValue::Register(reg) => {
                                    if let Some(&offset) = reg_map.get(&reg.id) {
                                        let label_id = *label_counter;
                                        *label_counter += 1;
                                        
                                        // Check if it's a valid string pointer or an integer
                                        asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                        // Check if it's a valid string pointer (> 0x1000)
                                        asm.push_str("    cmp $0x1000, %rdi\n");
                                        asm.push_str(&format!("    jb .{}_print_as_int_{}\n", func_name, label_id));
                                        // Check if it looks like a string (length < 1MB)
                                        asm.push_str("    movq (%rdi), %rax\n");
                                        asm.push_str("    cmp $1048576, %rax\n");
                                        asm.push_str(&format!("    ja .{}_print_as_int_{}\n", func_name, label_id));
                                        
                                        // Print as string (__string_print already adds newline)
                                        asm.push_str("    call __string_print\n");
                                        asm.push_str(&format!("    jmp .{}_print_done_{}\n", func_name, label_id));
                                        
                                        // Print as integer (__bulu_print_int already adds newline)
                                        asm.push_str(&format!(".{}_print_as_int_{}:\n", func_name, label_id));
                                        asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                        asm.push_str("    call __bulu_print_int\n");
                                        
                                        asm.push_str(&format!(".{}_print_done_{}:\n", func_name, label_id));
                                    }
                                }
                                IrValue::Constant(IrConstant::String(s)) => {
                                    // Print string constant
                                    if let Some(&id) = strings.get(s) {
                                        asm.push_str("    # write syscall for string\n");
                                        asm.push_str("    mov $1, %rax\n");
                                        asm.push_str("    mov $1, %rdi\n");
                                        asm.push_str(&format!("    lea str_{}(%rip), %rsi\n", id));
                                        asm.push_str(&format!("    mov $str_{}_len, %rdx\n", id));
                                        asm.push_str("    syscall\n");
                                        
                                        // Print newline
                                        asm.push_str("    mov $1, %rax\n");
                                        asm.push_str("    mov $1, %rdi\n");
                                        asm.push_str("    lea newline(%rip), %rsi\n");
                                        asm.push_str("    mov $1, %rdx\n");
                                        asm.push_str("    syscall\n");
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else if name == "ord" {
                        // Built-in: ord(c) - convert character to ASCII code
                        if let Some(arg) = inst.operands.get(1) {
                            match arg {
                                IrValue::Register(reg) => {
                                    if let Some(&offset) = reg_map.get(&reg.id) {
                                        // Load the string pointer
                                        asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                        // Check if it's a valid string
                                        asm.push_str("    cmp $0x1000, %rdi\n");
                                        let label_id = *label_counter;
                                        *label_counter += 1;
                                        asm.push_str(&format!("    jb .{}_ord_error_{}\n", func_name, label_id));
                                        // Get first character (at offset 8)
                                        asm.push_str("    movzbq 8(%rdi), %rax\n");
                                        asm.push_str(&format!("    jmp .{}_ord_done_{}\n", func_name, label_id));
                                        asm.push_str(&format!(".{}_ord_error_{}:\n", func_name, label_id));
                                        asm.push_str("    mov $0, %rax\n");
                                        asm.push_str(&format!(".{}_ord_done_{}:\n", func_name, label_id));
                                        // Store result
                                        if let Some(result) = inst.result {
                                            if let Some(&res_offset) = reg_map.get(&result.id) {
                                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                            }
                                        }
                                    }
                                }
                                IrValue::Constant(IrConstant::String(s)) => {
                                    // String constant - return ASCII of first character
                                    if !s.is_empty() {
                                        let ascii = s.chars().next().unwrap() as i64;
                                        asm.push_str(&format!("    mov ${}, %rax\n", ascii));
                                        if let Some(result) = inst.result {
                                            if let Some(&res_offset) = reg_map.get(&result.id) {
                                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else if name == "chr" {
                        // Built-in: chr(code) - convert ASCII code to character
                        if let Some(arg) = inst.operands.get(1) {
                            // Get the ASCII code
                            match arg {
                                IrValue::Register(reg) => {
                                    if let Some(&offset) = reg_map.get(&reg.id) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                    }
                                }
                                IrValue::Constant(IrConstant::Integer(code)) => {
                                    asm.push_str(&format!("    mov ${}, %rdi\n", code));
                                }
                                _ => {}
                            }
                            
                            // Create a single-character string
                            asm.push_str("    push %rdi\n");
                            asm.push_str("    mov $9, %rdi\n");
                            asm.push_str("    call __malloc\n");
                            asm.push_str("    pop %rdi\n");
                            asm.push_str("    test %rax, %rax\n");
                            let label_id = *label_counter;
                            *label_counter += 1;
                            asm.push_str(&format!("    jz .{}_chr_error_{}\n", func_name, label_id));
                            
                            // Set length to 1
                            asm.push_str("    movq $1, (%rax)\n");
                            // Store the character
                            asm.push_str("    movb %dil, 8(%rax)\n");
                            asm.push_str(&format!("    jmp .{}_chr_done_{}\n", func_name, label_id));
                            
                            asm.push_str(&format!(".{}_chr_error_{}:\n", func_name, label_id));
                            asm.push_str("    mov $0, %rax\n");
                            
                            asm.push_str(&format!(".{}_chr_done_{}:\n", func_name, label_id));
                            // Store result
                            if let Some(result) = inst.result {
                                if let Some(&res_offset) = reg_map.get(&result.id) {
                                    asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                }
                            }
                        }
                    } else if name == "len" {
                        // Built-in: len() for strings, arrays, etc.
                        if let Some(arg) = inst.operands.get(1) {
                            match arg {
                                IrValue::Register(reg) => {
                                    if let Some(&offset) = reg_map.get(&reg.id) {
                                        // Load the value
                                        asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                                        // Check if it's a string structure (pointer > 0x1000)
                                        asm.push_str("    cmp $0x1000, %rdi\n");
                                        let label_id = *label_counter;
                                        *label_counter += 1;
                                        asm.push_str(&format!("    jb .{}_len_not_string_{}\n", func_name, label_id));
                                        // It's a string structure, get length from first field
                                        asm.push_str("    movq (%rdi), %rax\n");
                                        asm.push_str(&format!("    jmp .{}_len_done_{}\n", func_name, label_id));
                                        asm.push_str(&format!(".{}_len_not_string_{}:\n", func_name, label_id));
                                        // Not a string, return 0 for now (TODO: support arrays)
                                        asm.push_str("    mov $0, %rax\n");
                                        asm.push_str(&format!(".{}_len_done_{}:\n", func_name, label_id));
                                        // Store result
                                        if let Some(result) = inst.result {
                                            if let Some(&res_offset) = reg_map.get(&result.id) {
                                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                            }
                                        }
                                    }
                                }
                                IrValue::Constant(IrConstant::String(s)) => {
                                    // String constant - return its length
                                    if let Some(&id) = strings.get(s) {
                                        asm.push_str(&format!("    mov $str_{}_len, %rax\n", id));
                                        if let Some(result) = inst.result {
                                            if let Some(&res_offset) = reg_map.get(&result.id) {
                                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else {
                        // Normal function call - treat all user functions the same
                        self.generate_normal_function_call(asm, inst, reg_map, strings, name, func_name, label_counter)?;
                    }
                }
            }
            IrOpcode::Div => {
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let Some(result) = inst.result {
                        if let Some(&res_off) = reg_map.get(&result.id) {
                            match (op1, op2) {
                                (IrValue::Register(r1), IrValue::Register(r2)) => {
                                    if let (Some(&off1), Some(&off2)) = (reg_map.get(&r1.id), reg_map.get(&r2.id)) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                        asm.push_str("    cqo\n");
                                        asm.push_str(&format!("    idivq {}(%rbp)\n", off2));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                (IrValue::Register(r1), IrValue::Constant(IrConstant::Integer(val))) => {
                                    if let Some(&off1) = reg_map.get(&r1.id) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                        asm.push_str("    cqo\n");
                                        asm.push_str(&format!("    mov ${}, %rcx\n", val));
                                        asm.push_str("    idivq %rcx\n");
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            IrOpcode::Mod => {
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let (IrValue::Register(r1), IrValue::Register(r2)) = (op1, op2) {
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&off2), Some(&res_off)) = (
                                reg_map.get(&r1.id),
                                reg_map.get(&r2.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str("    cqo\n");
                                asm.push_str(&format!("    idivq {}(%rbp)\n", off2));
                                asm.push_str(&format!("    movq %rdx, {}(%rbp)\n", res_off)); // Remainder in rdx
                            }
                        }
                    }
                }
            }
            IrOpcode::Neg => {
                if let Some(IrValue::Register(r1)) = inst.operands.first() {
                    if let Some(result) = inst.result {
                        if let (Some(&off1), Some(&res_off)) = 
                            (reg_map.get(&r1.id), reg_map.get(&result.id)) {
                            asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                            asm.push_str("    negq %rax\n");
                            asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                        }
                    }
                }
            }
            IrOpcode::Eq => {
                self.generate_comparison(asm, inst, reg_map, "sete");
            }
            IrOpcode::Ne => {
                self.generate_comparison(asm, inst, reg_map, "setne");
            }
            IrOpcode::Lt => {
                self.generate_comparison(asm, inst, reg_map, "setl");
            }
            IrOpcode::Le => {
                self.generate_comparison(asm, inst, reg_map, "setle");
            }
            IrOpcode::Gt => {
                self.generate_comparison(asm, inst, reg_map, "setg");
            }
            IrOpcode::Ge => {
                self.generate_comparison(asm, inst, reg_map, "setge");
            }
            IrOpcode::LogicalAnd => {
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let (IrValue::Register(r1), IrValue::Register(r2)) = (op1, op2) {
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&off2), Some(&res_off)) = (
                                reg_map.get(&r1.id),
                                reg_map.get(&r2.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str(&format!("    andq {}(%rbp), %rax\n", off2));
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                    }
                }
            }
            IrOpcode::LogicalOr => {
                if let (Some(op1), Some(op2)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let (IrValue::Register(r1), IrValue::Register(r2)) = (op1, op2) {
                        if let Some(result) = inst.result {
                            if let (Some(&off1), Some(&off2), Some(&res_off)) = (
                                reg_map.get(&r1.id),
                                reg_map.get(&r2.id),
                                reg_map.get(&result.id),
                            ) {
                                asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                                asm.push_str(&format!("    orq {}(%rbp), %rax\n", off2));
                                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                            }
                        }
                    }
                }
            }
            IrOpcode::LogicalNot => {
                if let Some(IrValue::Register(r1)) = inst.operands.first() {
                    if let Some(result) = inst.result {
                        if let (Some(&off1), Some(&res_off)) = 
                            (reg_map.get(&r1.id), reg_map.get(&result.id)) {
                            asm.push_str(&format!("    movq {}(%rbp), %rax\n", off1));
                            asm.push_str("    test %rax, %rax\n");
                            asm.push_str("    setz %al\n");
                            asm.push_str("    movzbq %al, %rax\n");
                            asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_off));
                        }
                    }
                }
            }
            IrOpcode::ArrayAccess => {
                // String/Array indexing: s[i] returns a single-character string
                if let (Some(array_val), Some(index_val)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let Some(result) = inst.result {
                        if let Some(&res_offset) = reg_map.get(&result.id) {
                            // Get array/string pointer
                            match array_val {
                                IrValue::Register(array_reg) => {
                                    if let Some(&array_offset) = reg_map.get(&array_reg.id) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rdi\n", array_offset));
                                    }
                                }
                                _ => {}
                            }
                            
                            // Get index
                            match index_val {
                                IrValue::Register(index_reg) => {
                                    if let Some(&index_offset) = reg_map.get(&index_reg.id) {
                                        asm.push_str(&format!("    movq {}(%rbp), %rsi\n", index_offset));
                                    }
                                }
                                IrValue::Constant(IrConstant::Integer(idx)) => {
                                    asm.push_str(&format!("    mov ${}, %rsi\n", idx));
                                }
                                _ => {}
                            }
                            
                            // Check if it's a string structure (pointer > 0x1000)
                            asm.push_str("    cmp $0x1000, %rdi\n");
                            let label_id = *label_counter;
                            *label_counter += 1;
                            asm.push_str(&format!("    jb .{}_index_error_{}\n", func_name, label_id));
                            
                            // Get string length for bounds checking
                            asm.push_str("    movq (%rdi), %rcx   # length\n");
                            asm.push_str("    cmp %rcx, %rsi\n");
                            asm.push_str(&format!("    jae .{}_index_error_{}\n", func_name, label_id));
                            
                            // Create a single-character string
                            // Allocate 9 bytes (8 for length + 1 for character)
                            asm.push_str("    push %rdi\n");
                            asm.push_str("    push %rsi\n");
                            asm.push_str("    mov $9, %rdi\n");
                            asm.push_str("    call __malloc\n");
                            asm.push_str("    pop %rsi\n");
                            asm.push_str("    pop %rdi\n");
                            asm.push_str("    test %rax, %rax\n");
                            asm.push_str(&format!("    jz .{}_index_error_{}\n", func_name, label_id));
                            
                            // Set length to 1
                            asm.push_str("    movq $1, (%rax)\n");
                            
                            // Copy the character
                            asm.push_str("    lea 8(%rdi), %rcx   # source data pointer\n");
                            asm.push_str("    movb (%rcx,%rsi,1), %cl  # load character\n");
                            asm.push_str("    movb %cl, 8(%rax)   # store in new string\n");
                            
                            asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                            asm.push_str(&format!("    jmp .{}_index_done_{}\n", func_name, label_id));
                            
                            // Index error - return empty string
                            asm.push_str(&format!(".{}_index_error_{}:\n", func_name, label_id));
                            asm.push_str("    mov $0, %rax\n");
                            asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                            
                            asm.push_str(&format!(".{}_index_done_{}:\n", func_name, label_id));
                        }
                    }
                }
            }
            IrOpcode::StructConstruct => {
                // Construct a struct: operands are [struct_name, field1_name, field1_value, field2_name, field2_value, ...]
                // We allocate space on the heap and store field values there
                if let Some(result) = inst.result {
                    if let Some(&res_offset) = reg_map.get(&result.id) {
                        // Count the number of fields
                        let num_fields = (inst.operands.len() - 1) / 2;
                        let struct_size = num_fields * 8;
                        
                        // Allocate memory for the struct
                        asm.push_str(&format!("    mov ${}, %rdi\n", struct_size));
                        asm.push_str("    call __malloc\n");
                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                        
                        // Store field values in the allocated memory
                        let mut field_index = 0;
                        let mut i = 1;
                        while i < inst.operands.len() {
                            // Skip field name (operands[i])
                            if i + 1 < inst.operands.len() {
                                let field_value = &inst.operands[i + 1];
                                let field_offset = field_index * 8;
                                
                                match field_value {
                                    IrValue::Constant(IrConstant::Integer(val)) => {
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", res_offset));
                                        asm.push_str(&format!("    movq ${}, {}(%rax)\n", val, field_offset));
                                    }
                                    IrValue::Register(reg) => {
                                        if let Some(&src_offset) = reg_map.get(&reg.id) {
                                            asm.push_str(&format!("    movq {}(%rbp), %rax\n", res_offset));
                                            asm.push_str(&format!("    movq {}(%rbp), %rbx\n", src_offset));
                                            asm.push_str(&format!("    movq %rbx, {}(%rax)\n", field_offset));
                                        }
                                    }
                                    IrValue::Constant(IrConstant::String(s)) => {
                                        if let Some(&id) = strings.get(s) {
                                            asm.push_str(&format!("    push {}(%rbp)\n", res_offset));
                                            asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                                            asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                                            asm.push_str("    call __string_create\n");
                                            asm.push_str("    pop %rbx\n");
                                            asm.push_str(&format!("    movq %rax, {}(%rbx)\n", field_offset));
                                        }
                                    }
                                    _ => {}
                                }
                                
                                field_index += 1;
                            }
                            i += 2;
                        }
                    }
                }
            }
            IrOpcode::StructAccess => {
                // Access a struct field: operands are [struct_value, field_index]
                if let (Some(struct_val), Some(field_idx)) = (inst.operands.get(0), inst.operands.get(1)) {
                    if let Some(result) = inst.result {
                        if let Some(&res_offset) = reg_map.get(&result.id) {
                            match (struct_val, field_idx) {
                                (IrValue::Register(struct_reg), IrValue::Constant(IrConstant::Integer(idx))) => {
                                    if let Some(&struct_offset) = reg_map.get(&struct_reg.id) {
                                        // Load struct base address
                                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", struct_offset));
                                        // Access field at offset idx * 8
                                        let field_offset = idx * 8;
                                        asm.push_str(&format!("    movq {}(%rax), %rax\n", field_offset));
                                        asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {
                // Skip unsupported opcodes for now
            }
        }

        Ok(())
    }
    
    /// Generate normal function call
    fn generate_normal_function_call(
        &self,
        asm: &mut String,
        inst: &IrInstruction,
        reg_map: &HashMap<u32, i32>,
        strings: &HashMap<String, usize>,
        name: &str,
        func_name: &str,
        label_counter: &mut usize,
    ) -> Result<()> {
        // Check if this is a native method call (e.g., Int64.toString, String.toString)
        // But first check if it's a user-defined method (like Complex.toString)
        if name.ends_with(".toString") {
            // Check if this is a primitive type (native toString)
            let is_primitive = name.starts_with("Int64") || name.starts_with("Int32") 
                || name.starts_with("Float64") || name.starts_with("Bool") 
                || name.starts_with("String");
            
            if !is_primitive {
                // User-defined toString method - call it normally
                // Fall through to normal function call
            } else if name.starts_with("Int64") || name.starts_with("Int32") {
                // Int64.toString() or Int32.toString()
                // First argument is the integer value
                if let Some(arg) = inst.operands.get(1) {
                    if let IrValue::Register(reg) = arg {
                        if let Some(&offset) = reg_map.get(&reg.id) {
                            // Load the integer value
                            asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                            // Allocate buffer for string (max 20 chars for int64)
                            asm.push_str("    mov $32, %rdi\n");
                            asm.push_str("    call __malloc\n");
                            asm.push_str("    push %rax\n");  // Save buffer pointer
                            
                            // Convert int to string
                            if let Some(&offset) = reg_map.get(&reg.id) {
                                asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                            }
                            asm.push_str("    lea 8(%rax), %rsi\n");  // Buffer data area
                            asm.push_str("    call __bulu_int_to_string\n");
                            
                            // Store length in the string structure
                            asm.push_str("    pop %rbx\n");  // Restore buffer pointer
                            asm.push_str("    movq %rax, (%rbx)\n");  // Store length
                            asm.push_str("    mov %rbx, %rax\n");  // Return string pointer
                            
                            // Store result
                            if let Some(result) = inst.result {
                                if let Some(&res_offset) = reg_map.get(&result.id) {
                                    asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                }
                            }
                        }
                    }
                }
                return Ok(());
            } else if name.starts_with("String") {
                // String.toString() - just return the string itself
                if let Some(arg) = inst.operands.get(1) {
                    if let IrValue::Register(reg) = arg {
                        if let Some(&offset) = reg_map.get(&reg.id) {
                            asm.push_str(&format!("    movq {}(%rbp), %rax\n", offset));
                            if let Some(result) = inst.result {
                                if let Some(&res_offset) = reg_map.get(&result.id) {
                                    asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                }
                            }
                        }
                    }
                }
                return Ok(());
            } else if name.starts_with("Bool") {
                // Bool.toString() - convert boolean to "true" or "false"
                if let Some(arg) = inst.operands.get(1) {
                    if let IrValue::Register(reg) = arg {
                        if let Some(&offset) = reg_map.get(&reg.id) {
                            let label_id = *label_counter;
                            *label_counter += 1;
                            
                            // Load the boolean value
                            asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                            
                            // Allocate buffer for string (max 6 chars: "false\0")
                            asm.push_str("    mov $16, %rdi\n");
                            asm.push_str("    call __malloc\n");
                            asm.push_str("    push %rax\n");  // Save buffer pointer
                            
                            // Check if boolean is true or false
                            if let Some(&offset) = reg_map.get(&reg.id) {
                                asm.push_str(&format!("    movq {}(%rbp), %rdi\n", offset));
                            }
                            asm.push_str("    test %rdi, %rdi\n");
                            asm.push_str(&format!("    jz .{}_bool_false_{}\n", func_name, label_id));
                            
                            // True case
                            asm.push_str("    pop %rbx\n");  // Restore buffer pointer
                            asm.push_str("    movq $4, (%rbx)\n");  // Length = 4
                            asm.push_str("    movl $0x65757274, 8(%rbx)\n");  // "true" (little-endian)
                            asm.push_str("    movb $0, 12(%rbx)\n");  // Null terminator
                            asm.push_str("    mov %rbx, %rax\n");
                            asm.push_str(&format!("    jmp .{}_bool_done_{}\n", func_name, label_id));
                            
                            // False case
                            asm.push_str(&format!(".{}_bool_false_{}:\n", func_name, label_id));
                            asm.push_str("    pop %rbx\n");  // Restore buffer pointer
                            asm.push_str("    movq $5, (%rbx)\n");  // Length = 5
                            asm.push_str("    movl $0x736c6166, 8(%rbx)\n");  // "fals" (little-endian)
                            asm.push_str("    movb $0x65, 12(%rbx)\n");  // "e"
                            asm.push_str("    movb $0, 13(%rbx)\n");  // Null terminator
                            asm.push_str("    mov %rbx, %rax\n");
                            
                            asm.push_str(&format!(".{}_bool_done_{}:\n", func_name, label_id));
                            
                            // Store result
                            if let Some(result) = inst.result {
                                if let Some(&res_offset) = reg_map.get(&result.id) {
                                    asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
                                }
                            }
                        }
                    }
                }
                return Ok(());
            }
        }
        
        // Set up arguments in registers (System V AMD64 ABI)
        // rdi, rsi, rdx, rcx, r8, r9 for first 6 args
        let arg_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];

        for (i, arg) in inst.operands.iter().skip(1).enumerate() {
            if i >= 6 {
                // TODO: Handle stack arguments
                break;
            }

            match arg {
                IrValue::Register(reg) => {
                    if let Some(&offset) = reg_map.get(&reg.id) {
                        asm.push_str(&format!(
                            "    movq {}(%rbp), {}\n",
                            offset, arg_regs[i]
                        ));
                    }
                }
                IrValue::Constant(IrConstant::Integer(val)) => {
                    asm.push_str(&format!("    movq ${}, {}\n", val, arg_regs[i]));
                }
                IrValue::Constant(IrConstant::String(s)) => {
                    // Create string structure and pass pointer
                    if let Some(&id) = strings.get(s) {
                        // Save registers that might be clobbered by __string_create
                        for j in 0..i {
                            asm.push_str(&format!("    push {}\n", arg_regs[j]));
                        }
                        
                        asm.push_str(&format!("    lea str_{}(%rip), %rdi\n", id));
                        asm.push_str(&format!("    mov $str_{}_len, %rsi\n", id));
                        asm.push_str("    call __string_create\n");
                        asm.push_str(&format!("    mov %rax, {}\n", arg_regs[i]));
                        
                        // Restore registers
                        for j in (0..i).rev() {
                            asm.push_str(&format!("    pop {}\n", arg_regs[j]));
                        }
                    }
                }
                IrValue::Constant(IrConstant::Float(val)) => {
                    // For floats, we'd use xmm registers, but for simplicity, convert to int
                    asm.push_str(&format!("    # TODO: Float argument: {}\n", val));
                    asm.push_str(&format!("    movq $0, {}\n", arg_regs[i]));
                }
                IrValue::Constant(IrConstant::Boolean(val)) => {
                    let int_val = if *val { 1 } else { 0 };
                    asm.push_str(&format!("    movq ${}, {}\n", int_val, arg_regs[i]));
                }
                _ => {}
            }
        }

        // Call the function
        asm.push_str(&format!("    call {}\n", name));

        // Store result if needed
        if let Some(result) = inst.result {
            if let Some(&res_offset) = reg_map.get(&result.id) {
                asm.push_str(&format!("    movq %rax, {}(%rbp)\n", res_offset));
            }
        }
        
        Ok(())
    }

    /// Generate assembly for a terminator instruction
    fn generate_terminator(
        &self,
        asm: &mut String,
        terminator: &IrTerminator,
        reg_map: &HashMap<u32, i32>,
        func_name: &str,
    ) -> Result<()> {
        match terminator {
            IrTerminator::Return(Some(ret_val)) => {
                match ret_val {
                    IrValue::Register(reg) => {
                        if let Some(&offset) = reg_map.get(&reg.id) {
                            asm.push_str(&format!("    movq {}(%rbp), %rax\n", offset));
                        }
                    }
                    IrValue::Constant(IrConstant::Integer(val)) => {
                        asm.push_str(&format!("    movq ${}, %rax\n", val));
                    }
                    _ => {}
                }
                asm.push_str("    mov %rbp, %rsp\n");
                asm.push_str("    pop %rbp\n");
                asm.push_str("    ret\n");
            }
            IrTerminator::Return(None) => {
                asm.push_str("    mov %rbp, %rsp\n");
                asm.push_str("    pop %rbp\n");
                asm.push_str("    ret\n");
            }
            IrTerminator::Branch(label) => {
                asm.push_str(&format!("    jmp .{}_{}\n", func_name, label));
            }
            IrTerminator::ConditionalBranch {
                condition,
                true_label,
                false_label,
            } => match condition {
                IrValue::Register(reg) => {
                    if let Some(&offset) = reg_map.get(&reg.id) {
                        asm.push_str(&format!("    movq {}(%rbp), %rax\n", offset));
                        asm.push_str("    test %rax, %rax\n");
                        asm.push_str(&format!("    jnz .{}_{}\n", func_name, true_label));
                        asm.push_str(&format!("    jmp .{}_{}\n", func_name, false_label));
                    }
                }
                _ => {}
            },
            _ => {
                // Default: just return
                asm.push_str("    mov %rbp, %rsp\n");
                asm.push_str("    pop %rbp\n");
                asm.push_str("    ret\n");
            }
        }

        Ok(())
    }

    fn generate_minimal_hello_world(&self) -> String {
        // Minimal x86-64 assembly that prints "Hello from Bulu!" and exits
        ".section .data\n\
msg: .ascii \"Hello from Bulu!\\n\"\n\
msg_len = . - msg\n\
\n\
.section .text\n\
.global _start\n\
\n\
_start:\n\
    # write syscall\n\
    mov $1, %rax        # sys_write\n\
    mov $1, %rdi        # stdout\n\
    lea msg(%rip), %rsi # message\n\
    mov $msg_len, %rdx  # length\n\
    syscall\n\
    \n\
    # exit syscall\n\
    mov $60, %rax       # sys_exit\n\
    mov $0, %rdi        # exit code\n\
    syscall\n"
            .to_string()
    }

    fn assemble_and_link(&self, asm_code: &str, name: &str) -> Result<Vec<u8>> {
        use std::env;
        use std::fs;
        use std::process::Command;

        // Create build directory in target (like Rust/Go)
        let current_dir = env::current_dir()
            .map_err(|e| BuluError::Other(format!("Cannot get current directory: {}", e)))?;

        let build_dir = current_dir.join("target").join("build");
        fs::create_dir_all(&build_dir)
            .map_err(|e| BuluError::Other(format!("Cannot create build directory: {}", e)))?;

        // Use project name for files (overwrite previous builds)
        let asm_file = build_dir.join(format!("{}.s", name));
        let obj_file = build_dir.join(format!("{}.o", name));

        // Write assembly file
        fs::write(&asm_file, asm_code)
            .map_err(|e| BuluError::Other(format!("Failed to write assembly: {}", e)))?;

        eprintln!("📝 Assembly: {}", asm_file.display());

        // Assemble
        let output = Command::new("as")
            .args(&[
                "--64",
                "-o",
                obj_file.to_str().unwrap(),
                asm_file.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| BuluError::Other(format!("Failed to run assembler: {}", e)))?;

        if !output.status.success() {
            return Err(BuluError::Other(format!(
                "Assembly failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        eprintln!("🔧 Object file: {}", obj_file.display());

        // Create a temporary executable for reading
        let temp_exe = build_dir.join(format!("{}.tmp", name));

        // Link
        let output = Command::new("ld")
            .args(&["-o", temp_exe.to_str().unwrap(), obj_file.to_str().unwrap()])
            .output()
            .map_err(|e| BuluError::Other(format!("Failed to run linker: {}", e)))?;

        if !output.status.success() {
            return Err(BuluError::Other(format!(
                "Linking failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Read executable
        let exe_bytes = fs::read(&temp_exe)
            .map_err(|e| BuluError::Other(format!("Failed to read executable: {}", e)))?;

        // Clean up temporary executable (keep .s and .o files for debugging)
        let _ = fs::remove_file(&temp_exe);

        Ok(exe_bytes)
    }
}
