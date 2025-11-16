//! IR optimization passes for the Bulu language
//!
//! This module implements various optimization passes that can be applied
//! to IR code to improve performance and reduce code size.

use super::ir::*;
use super::control_flow::{ControlFlowAnalyzer, NaturalLoop};
use super::OptLevel;
use crate::error::Result;
use std::collections::{HashMap, HashSet};

/// IR optimizer that applies various optimization passes
pub struct IrOptimizer {
    /// Whether to enable aggressive optimizations
    aggressive: bool,
    /// Optimization level
    level: OptLevel,
}

impl IrOptimizer {
    pub fn new() -> Self {
        Self { 
            aggressive: false,
            level: OptLevel::O0,
        }
    }

    /// Set the optimization level
    pub fn set_level(&mut self, level: OptLevel) {
        self.level = level;
        self.aggressive = matches!(self.level, OptLevel::O3);
    }

    /// Apply all optimization passes to an IR program
    pub fn optimize(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        // Apply optimization passes in order
        // ONLY run constant folding for now - other passes break the native backend
        program = self.constant_folding(program)?;

        if self.aggressive {
            program = self.function_inlining(program)?;
            program = self.loop_optimization(program)?;
        }

        Ok(program)
    }

    /// Dead code elimination pass
    /// Removes instructions that have no effect on program output
    pub fn dead_code_elimination(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        for function in &mut program.functions {
            self.eliminate_dead_code_in_function(function)?;
        }
        Ok(program)
    }

    /// Eliminate dead code in a single function
    fn eliminate_dead_code_in_function(&mut self, function: &mut IrFunction) -> Result<()> {
        // Track which registers are used
        let mut used_registers = HashSet::new();
        let mut worklist = Vec::new();

        // Mark all registers used in terminators as live
        for block in &function.basic_blocks {
            match &block.terminator {
                IrTerminator::Return(Some(value)) => {
                    self.mark_value_as_used(value, &mut used_registers, &mut worklist);
                }
                IrTerminator::ConditionalBranch { condition, .. } => {
                    self.mark_value_as_used(condition, &mut used_registers, &mut worklist);
                }
                IrTerminator::Switch { value, cases, .. } => {
                    self.mark_value_as_used(value, &mut used_registers, &mut worklist);
                    for (case_value, _) in cases {
                        self.mark_value_as_used(case_value, &mut used_registers, &mut worklist);
                    }
                }
                _ => {}
            }
        }

        // Mark registers used in side-effecting instructions
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                if self.has_side_effects(&instruction.opcode) {
                    for operand in &instruction.operands {
                        self.mark_value_as_used(operand, &mut used_registers, &mut worklist);
                    }
                }
            }
        }

        // Propagate liveness backwards
        while let Some(register) = worklist.pop() {
            // Find the instruction that defines this register
            for block in &function.basic_blocks {
                for instruction in &block.instructions {
                    if let Some(result_reg) = instruction.result {
                        if result_reg == register {
                            // Mark all operands of this instruction as used
                            for operand in &instruction.operands {
                                self.mark_value_as_used(
                                    operand,
                                    &mut used_registers,
                                    &mut worklist,
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
        


        // Remove dead instructions
        for block in &mut function.basic_blocks {
            block.instructions.retain(|instruction| {
                if let Some(result_reg) = instruction.result {
                    used_registers.contains(&result_reg)
                        || self.has_side_effects(&instruction.opcode)
                } else {
                    self.has_side_effects(&instruction.opcode)
                }
            });
        }

        Ok(())
    }

    /// Mark a value as used and add registers to worklist
    fn mark_value_as_used(
        &self,
        value: &IrValue,
        used_registers: &mut HashSet<IrRegister>,
        worklist: &mut Vec<IrRegister>,
    ) {
        if let IrValue::Register(reg) = value {
            if used_registers.insert(*reg) {
                worklist.push(*reg);
            }
        }
    }

    /// Check if an instruction has side effects
    fn has_side_effects(&self, opcode: &IrOpcode) -> bool {
        match opcode {
            // Memory operations have side effects
            IrOpcode::Store | IrOpcode::Alloca => true,

            // Function calls may have side effects
            IrOpcode::Call | IrOpcode::CallIndirect => true,

            // Channel operations have side effects
            IrOpcode::ChannelSend | IrOpcode::ChannelReceive | IrOpcode::ChannelClose => true,

            // Concurrency operations have side effects
            IrOpcode::Spawn => true,

            // Map mutations have side effects
            IrOpcode::MapInsert | IrOpcode::MapDelete => true,

            // Error handling has side effects
            IrOpcode::Throw => true,

            // Generator operations have side effects
            IrOpcode::Yield => true,

            // IMPORTANT: Copy operations that initialize variables should not be eliminated
            // They are needed for the native backend even if the result seems unused
            // The liveness analysis will handle truly dead copies
            // IrOpcode::Copy => false, // Keep as pure for now, liveness analysis handles it

            // Pure operations don't have side effects
            _ => false,
        }
    }

    /// Constant folding pass
    /// Evaluates constant expressions at compile time
    pub fn constant_folding(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        for function in &mut program.functions {
            self.fold_constants_in_function(function)?;
        }
        Ok(program)
    }

    /// Fold constants in a single function
    fn fold_constants_in_function(&mut self, function: &mut IrFunction) -> Result<()> {
        for block in &mut function.basic_blocks {
            for instruction in &mut block.instructions {
                if let Some(folded_value) = self.try_fold_instruction(instruction)? {
                    // Replace instruction with a simple copy of the constant
                    instruction.opcode = IrOpcode::Copy;
                    instruction.operands = vec![folded_value];
                }
            }
        }
        Ok(())
    }

    /// Try to fold an instruction into a constant
    fn try_fold_instruction(&self, instruction: &IrInstruction) -> Result<Option<IrValue>> {
        match instruction.opcode {
            IrOpcode::Add | IrOpcode::Sub | IrOpcode::Mul | IrOpcode::Div | IrOpcode::Mod => {
                if instruction.operands.len() == 2 {
                    if let (IrValue::Constant(left), IrValue::Constant(right)) =
                        (&instruction.operands[0], &instruction.operands[1])
                    {
                        return self.fold_arithmetic_operation(&instruction.opcode, left, right);
                    }
                }
            }

            IrOpcode::Eq
            | IrOpcode::Ne
            | IrOpcode::Lt
            | IrOpcode::Le
            | IrOpcode::Gt
            | IrOpcode::Ge => {
                if instruction.operands.len() == 2 {
                    if let (IrValue::Constant(left), IrValue::Constant(right)) =
                        (&instruction.operands[0], &instruction.operands[1])
                    {
                        return self.fold_comparison_operation(&instruction.opcode, left, right);
                    }
                }
            }

            IrOpcode::LogicalAnd | IrOpcode::LogicalOr => {
                if instruction.operands.len() == 2 {
                    if let (IrValue::Constant(left), IrValue::Constant(right)) =
                        (&instruction.operands[0], &instruction.operands[1])
                    {
                        return self.fold_logical_operation(&instruction.opcode, left, right);
                    }
                }
            }

            IrOpcode::LogicalNot => {
                if instruction.operands.len() == 1 {
                    if let IrValue::Constant(operand) = &instruction.operands[0] {
                        return self.fold_unary_operation(&instruction.opcode, operand);
                    }
                }
            }

            _ => {}
        }

        Ok(None)
    }

    /// Fold arithmetic operations
    fn fold_arithmetic_operation(
        &self,
        opcode: &IrOpcode,
        left: &IrConstant,
        right: &IrConstant,
    ) -> Result<Option<IrValue>> {
        match (left, right) {
            (IrConstant::Integer(a), IrConstant::Integer(b)) => {
                let result = match opcode {
                    IrOpcode::Add => a + b,
                    IrOpcode::Sub => a - b,
                    IrOpcode::Mul => a * b,
                    IrOpcode::Div => {
                        if *b != 0 {
                            a / b
                        } else {
                            return Ok(None);
                        }
                    }
                    IrOpcode::Mod => {
                        if *b != 0 {
                            a % b
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => return Ok(None),
                };
                Ok(Some(IrValue::Constant(IrConstant::Integer(result))))
            }

            (IrConstant::Float(a), IrConstant::Float(b)) => {
                let result = match opcode {
                    IrOpcode::Add => a + b,
                    IrOpcode::Sub => a - b,
                    IrOpcode::Mul => a * b,
                    IrOpcode::Div => {
                        if *b != 0.0 {
                            a / b
                        } else {
                            return Ok(None);
                        }
                    }
                    IrOpcode::Mod => {
                        if *b != 0.0 {
                            a % b
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => return Ok(None),
                };
                Ok(Some(IrValue::Constant(IrConstant::Float(result))))
            }

            // Mixed integer/float operations
            (IrConstant::Integer(a), IrConstant::Float(b)) => {
                let a_float = *a as f64;
                let result = match opcode {
                    IrOpcode::Add => a_float + b,
                    IrOpcode::Sub => a_float - b,
                    IrOpcode::Mul => a_float * b,
                    IrOpcode::Div => {
                        if *b != 0.0 {
                            a_float / b
                        } else {
                            return Ok(None);
                        }
                    }
                    IrOpcode::Mod => {
                        if *b != 0.0 {
                            a_float % b
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => return Ok(None),
                };
                Ok(Some(IrValue::Constant(IrConstant::Float(result))))
            }

            (IrConstant::Float(a), IrConstant::Integer(b)) => {
                let b_float = *b as f64;
                let result = match opcode {
                    IrOpcode::Add => a + b_float,
                    IrOpcode::Sub => a - b_float,
                    IrOpcode::Mul => a * b_float,
                    IrOpcode::Div => {
                        if *b != 0 {
                            a / b_float
                        } else {
                            return Ok(None);
                        }
                    }
                    IrOpcode::Mod => {
                        if *b != 0 {
                            a % b_float
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => return Ok(None),
                };
                Ok(Some(IrValue::Constant(IrConstant::Float(result))))
            }

            _ => Ok(None),
        }
    }

    /// Fold comparison operations
    fn fold_comparison_operation(
        &self,
        opcode: &IrOpcode,
        left: &IrConstant,
        right: &IrConstant,
    ) -> Result<Option<IrValue>> {
        let result = match (left, right) {
            (IrConstant::Integer(a), IrConstant::Integer(b)) => match opcode {
                IrOpcode::Eq => *a == *b,
                IrOpcode::Ne => *a != *b,
                IrOpcode::Lt => *a < *b,
                IrOpcode::Le => *a <= *b,
                IrOpcode::Gt => *a > *b,
                IrOpcode::Ge => *a >= *b,
                _ => return Ok(None),
            },

            (IrConstant::Float(a), IrConstant::Float(b)) => match opcode {
                IrOpcode::Eq => *a == *b,
                IrOpcode::Ne => *a != *b,
                IrOpcode::Lt => *a < *b,
                IrOpcode::Le => *a <= *b,
                IrOpcode::Gt => *a > *b,
                IrOpcode::Ge => *a >= *b,
                _ => return Ok(None),
            },

            (IrConstant::Boolean(a), IrConstant::Boolean(b)) => match opcode {
                IrOpcode::Eq => *a == *b,
                IrOpcode::Ne => *a != *b,
                _ => return Ok(None),
            },

            (IrConstant::String(a), IrConstant::String(b)) => match opcode {
                IrOpcode::Eq => *a == *b,
                IrOpcode::Ne => *a != *b,
                IrOpcode::Lt => *a < *b,
                IrOpcode::Le => *a <= *b,
                IrOpcode::Gt => *a > *b,
                IrOpcode::Ge => *a >= *b,
                _ => return Ok(None),
            },

            _ => return Ok(None),
        };

        Ok(Some(IrValue::Constant(IrConstant::Boolean(result))))
    }

    /// Fold logical operations
    fn fold_logical_operation(
        &self,
        opcode: &IrOpcode,
        left: &IrConstant,
        right: &IrConstant,
    ) -> Result<Option<IrValue>> {
        if let (IrConstant::Boolean(a), IrConstant::Boolean(b)) = (left, right) {
            let result = match opcode {
                IrOpcode::LogicalAnd => *a && *b,
                IrOpcode::LogicalOr => *a || *b,
                _ => return Ok(None),
            };
            Ok(Some(IrValue::Constant(IrConstant::Boolean(result))))
        } else {
            Ok(None)
        }
    }

    /// Fold unary operations
    fn fold_unary_operation(
        &self,
        opcode: &IrOpcode,
        operand: &IrConstant,
    ) -> Result<Option<IrValue>> {
        match opcode {
            IrOpcode::LogicalNot => {
                if let IrConstant::Boolean(b) = operand {
                    Ok(Some(IrValue::Constant(IrConstant::Boolean(!b))))
                } else {
                    Ok(None)
                }
            }

            IrOpcode::Neg => match operand {
                IrConstant::Integer(i) => Ok(Some(IrValue::Constant(IrConstant::Integer(-i)))),
                IrConstant::Float(f) => Ok(Some(IrValue::Constant(IrConstant::Float(-f)))),
                _ => Ok(None),
            },

            _ => Ok(None),
        }
    }

    /// Constant propagation pass
    /// Replaces uses of variables with their constant values
    pub fn constant_propagation(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        for function in &mut program.functions {
            self.propagate_constants_in_function(function)?;
        }
        Ok(program)
    }

    /// Propagate constants in a single function
    fn propagate_constants_in_function(&mut self, function: &mut IrFunction) -> Result<()> {
        let mut constant_map: HashMap<IrRegister, IrConstant> = HashMap::new();

        // Find constant assignments
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                if let Some(result_reg) = instruction.result {
                    match instruction.opcode {
                        IrOpcode::Copy => {
                            if let Some(IrValue::Constant(const_val)) = instruction.operands.first()
                            {
                                constant_map.insert(result_reg, const_val.clone());
                            }
                        }
                        _ => {
                            // Non-constant assignment, remove from map if present
                            constant_map.remove(&result_reg);
                        }
                    }
                }
            }
        }

        // Replace uses of constant registers
        for block in &mut function.basic_blocks {
            for instruction in &mut block.instructions {
                for operand in &mut instruction.operands {
                    if let IrValue::Register(reg) = operand {
                        if let Some(const_val) = constant_map.get(reg) {
                            *operand = IrValue::Constant(const_val.clone());
                        }
                    }
                }
            }

            // Also update terminator operands
            match &mut block.terminator {
                IrTerminator::Return(Some(value)) => {
                    if let IrValue::Register(reg) = value {
                        if let Some(const_val) = constant_map.get(reg) {
                            *value = IrValue::Constant(const_val.clone());
                        }
                    }
                }
                IrTerminator::ConditionalBranch { condition, .. } => {
                    if let IrValue::Register(reg) = condition {
                        if let Some(const_val) = constant_map.get(reg) {
                            *condition = IrValue::Constant(const_val.clone());
                        }
                    }
                }
                IrTerminator::Switch { value, cases, .. } => {
                    if let IrValue::Register(reg) = value {
                        if let Some(const_val) = constant_map.get(reg) {
                            *value = IrValue::Constant(const_val.clone());
                        }
                    }
                    for (case_value, _) in cases {
                        if let IrValue::Register(reg) = case_value {
                            if let Some(const_val) = constant_map.get(reg) {
                                *case_value = IrValue::Constant(const_val.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Copy propagation pass
    /// Replaces uses of copied variables with their original values
    fn copy_propagation(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        for function in &mut program.functions {
            self.propagate_copies_in_function(function)?;
        }
        Ok(program)
    }

    /// Propagate copies in a single function
    fn propagate_copies_in_function(&mut self, function: &mut IrFunction) -> Result<()> {
        let mut copy_map: HashMap<IrRegister, IrValue> = HashMap::new();

        // Find copy instructions
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                if let Some(result_reg) = instruction.result {
                    match instruction.opcode {
                        IrOpcode::Copy => {
                            if let Some(operand) = instruction.operands.first() {
                                copy_map.insert(result_reg, operand.clone());
                            }
                        }
                        _ => {
                            // Non-copy assignment, remove from map if present
                            copy_map.remove(&result_reg);
                        }
                    }
                }
            }
        }

        // Replace uses of copied registers
        for block in &mut function.basic_blocks {
            for instruction in &mut block.instructions {
                for operand in &mut instruction.operands {
                    if let IrValue::Register(reg) = operand {
                        if let Some(original_value) = copy_map.get(reg) {
                            *operand = original_value.clone();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Function inlining pass (aggressive optimization)
    /// Inlines small functions at their call sites
    fn function_inlining(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        // Identify functions suitable for inlining
        let inlinable_functions = self.identify_inlinable_functions(&program);
        
        // Clone the functions for reference during inlining
        let function_map: HashMap<String, IrFunction> = program.functions.iter()
            .map(|f| (f.name.clone(), f.clone()))
            .collect();
        
        // Inline functions in each function
        for function in &mut program.functions {
            self.inline_calls_in_function(function, &inlinable_functions, &function_map)?;
        }
        
        Ok(program)
    }
    
    /// Identify functions that are suitable for inlining
    fn identify_inlinable_functions(&self, program: &IrProgram) -> HashSet<String> {
        let mut inlinable = HashSet::new();
        
        for function in &program.functions {
            if self.is_function_inlinable(function) {
                inlinable.insert(function.name.clone());
            }
        }
        
        inlinable
    }
    
    /// Check if a function is suitable for inlining
    fn is_function_inlinable(&self, function: &IrFunction) -> bool {
        // Don't inline recursive functions
        if self.is_recursive_function(function) {
            return false;
        }
        
        // Don't inline async functions (complex control flow)
        if function.is_async {
            return false;
        }
        
        // Count total instructions
        let total_instructions: usize = function.basic_blocks.iter()
            .map(|block| block.instructions.len())
            .sum();
        
        // Only inline small functions (threshold: 10 instructions)
        if total_instructions > 10 {
            return false;
        }
        
        // Don't inline functions with complex control flow (multiple blocks)
        if function.basic_blocks.len() > 3 {
            return false;
        }
        
        // Don't inline functions that contain calls to other functions
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                if matches!(instruction.opcode, IrOpcode::Call | IrOpcode::CallIndirect) {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Check if a function is recursive
    fn is_recursive_function(&self, function: &IrFunction) -> bool {
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                if let IrOpcode::Call = instruction.opcode {
                    if let Some(IrValue::Function(called_func)) = instruction.operands.first() {
                        if called_func == &function.name {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    /// Inline function calls in a function
    fn inline_calls_in_function(
        &mut self,
        function: &mut IrFunction,
        inlinable_functions: &HashSet<String>,
        function_map: &HashMap<String, IrFunction>,
    ) -> Result<()> {
        let mut changed = true;
        
        // Keep inlining until no more changes
        while changed {
            changed = false;
            
            for block in &mut function.basic_blocks {
                let mut new_instructions = Vec::new();
                
                for instruction in &block.instructions {
                    if let IrOpcode::Call = instruction.opcode {
                        if let Some(IrValue::Function(called_func)) = instruction.operands.first() {
                            if inlinable_functions.contains(called_func) {
                                // Find the function to inline
                                if let Some(target_func) = function_map.get(called_func) {
                                    
                                    // Inline the function
                                    let inlined_instructions = self.inline_function_call(
                                        instruction,
                                        target_func,
                                    )?;
                                    
                                    new_instructions.extend(inlined_instructions);
                                    changed = true;
                                    continue;
                                }
                            }
                        }
                    }
                    
                    // Keep the original instruction
                    new_instructions.push(instruction.clone());
                }
                
                block.instructions = new_instructions;
            }
        }
        
        Ok(())
    }
    
    /// Inline a single function call
    fn inline_function_call(
        &mut self,
        call_instruction: &IrInstruction,
        target_function: &IrFunction,
    ) -> Result<Vec<IrInstruction>> {
        let mut inlined_instructions = Vec::new();
        let mut register_map = HashMap::new();
        
        // Map parameters to arguments
        for (i, param) in target_function.params.iter().enumerate() {
            if let Some(arg_value) = call_instruction.operands.get(i + 1) { // Skip function name
                register_map.insert(param.register, arg_value.clone());
            }
        }
        
        // Generate new registers for the inlined function's locals and results
        // Start with a high register ID to avoid conflicts
        let mut next_reg_id = 1000;
        
        // Inline instructions from the target function
        for block in &target_function.basic_blocks {
            for instruction in &block.instructions {
                let mut inlined_inst = instruction.clone();
                
                // Rename registers
                if let Some(result_reg) = inlined_inst.result {
                    let new_reg = IrRegister { id: next_reg_id };
                    next_reg_id += 1;
                    register_map.insert(result_reg, IrValue::Register(new_reg));
                    inlined_inst.result = Some(new_reg);
                }
                
                // Update operands
                for operand in &mut inlined_inst.operands {
                    if let IrValue::Register(reg) = operand {
                        if let Some(mapped_value) = register_map.get(reg) {
                            *operand = mapped_value.clone();
                        }
                    }
                }
                
                inlined_instructions.push(inlined_inst);
            }
        }
        
        // Handle return value
        if let Some(result_reg) = call_instruction.result {
            // Find the return value from the inlined function
            for block in &target_function.basic_blocks {
                if let IrTerminator::Return(Some(return_value)) = &block.terminator {
                    let mapped_return = if let IrValue::Register(reg) = return_value {
                        register_map.get(reg).cloned().unwrap_or(return_value.clone())
                    } else {
                        return_value.clone()
                    };
                    
                    // Add a copy instruction to assign the return value
                    inlined_instructions.push(IrInstruction {
                        opcode: IrOpcode::Copy,
                        result: Some(result_reg),
                        result_type: None,
                        operands: vec![mapped_return],
                        position: call_instruction.position,
                    });
                    break;
                }
            }
        }
        
        Ok(inlined_instructions)
    }

    /// Loop optimization pass (aggressive optimization)
    /// Applies various loop optimizations like loop unrolling
    fn loop_optimization(&mut self, mut program: IrProgram) -> Result<IrProgram> {
        let analyzer = ControlFlowAnalyzer::new();
        
        for function in &mut program.functions {
            // Build control flow graph
            let cfg = analyzer.build_cfg(function)?;
            
            // Find natural loops
            let loops = analyzer.find_natural_loops(&cfg);
            
            // Apply loop optimizations
            for loop_info in &loops {
                self.optimize_loop(function, loop_info, &cfg)?;
            }
        }
        
        Ok(program)
    }
    
    /// Optimize a single loop
    fn optimize_loop(
        &mut self,
        function: &mut IrFunction,
        loop_info: &NaturalLoop,
        cfg: &ControlFlowGraph,
    ) -> Result<()> {
        // Apply loop invariant code motion
        self.loop_invariant_code_motion(function, loop_info, cfg)?;
        
        // Apply loop unrolling for small loops
        if self.should_unroll_loop(function, loop_info) {
            self.unroll_loop(function, loop_info)?;
        }
        
        // Apply strength reduction
        self.strength_reduction(function, loop_info)?;
        
        Ok(())
    }
    
    /// Move loop-invariant code out of loops
    fn loop_invariant_code_motion(
        &mut self,
        function: &mut IrFunction,
        loop_info: &NaturalLoop,
        _cfg: &ControlFlowGraph,
    ) -> Result<()> {
        let mut invariant_instructions = Vec::new();
        
        // Find loop-invariant instructions
        for &block_id in &loop_info.nodes {
            if block_id >= function.basic_blocks.len() {
                continue;
            }
            
            let block = &function.basic_blocks[block_id];
            for (inst_idx, instruction) in block.instructions.iter().enumerate() {
                if self.is_loop_invariant(instruction, loop_info, function) {
                    invariant_instructions.push((block_id, inst_idx, instruction.clone()));
                }
            }
        }
        
        // Move invariant instructions to the loop preheader
        if !invariant_instructions.is_empty() {
            // Find or create a preheader block
            let preheader_id = self.find_or_create_preheader(function, loop_info)?;
            
            // Move instructions to preheader
            for (block_id, inst_idx, instruction) in invariant_instructions.into_iter().rev() {
                // Remove from original location
                function.basic_blocks[block_id].instructions.remove(inst_idx);
                
                // Add to preheader
                function.basic_blocks[preheader_id].instructions.push(instruction);
            }
        }
        
        Ok(())
    }
    
    /// Check if an instruction is loop-invariant
    fn is_loop_invariant(
        &self,
        instruction: &IrInstruction,
        loop_info: &NaturalLoop,
        function: &IrFunction,
    ) -> bool {
        // Don't move instructions with side effects
        if self.has_side_effects(&instruction.opcode) {
            return false;
        }
        
        // Check if all operands are loop-invariant
        for operand in &instruction.operands {
            if let IrValue::Register(reg) = operand {
                if !self.is_register_loop_invariant(*reg, loop_info, function) {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Check if a register is loop-invariant (defined outside the loop)
    fn is_register_loop_invariant(
        &self,
        register: IrRegister,
        loop_info: &NaturalLoop,
        function: &IrFunction,
    ) -> bool {
        // Find where the register is defined
        for (block_idx, block) in function.basic_blocks.iter().enumerate() {
            for instruction in &block.instructions {
                if let Some(result_reg) = instruction.result {
                    if result_reg == register {
                        // Check if the definition is outside the loop
                        return !loop_info.nodes.contains(&block_idx);
                    }
                }
            }
        }
        
        // If not found, assume it's a parameter (loop-invariant)
        true
    }
    
    /// Find or create a preheader block for the loop
    fn find_or_create_preheader(
        &mut self,
        function: &mut IrFunction,
        loop_info: &NaturalLoop,
    ) -> Result<usize> {
        // For simplicity, we'll use the header block's index
        // In a full implementation, we'd create a proper preheader
        Ok(loop_info.header)
    }
    
    /// Check if a loop should be unrolled
    fn should_unroll_loop(&self, function: &IrFunction, loop_info: &NaturalLoop) -> bool {
        // Only unroll small loops
        if loop_info.nodes.len() > 3 {
            return false;
        }
        
        // Count instructions in the loop
        let total_instructions: usize = loop_info.nodes.iter()
            .filter_map(|&block_id| function.basic_blocks.get(block_id))
            .map(|block| block.instructions.len())
            .sum();
        
        // Only unroll loops with few instructions (threshold: 5)
        total_instructions <= 5
    }
    
    /// Unroll a small loop
    fn unroll_loop(&mut self, function: &mut IrFunction, loop_info: &NaturalLoop) -> Result<()> {
        // For simplicity, we'll do a 2x unroll
        let unroll_factor = 2;
        
        // Clone loop body instructions
        let mut unrolled_instructions = Vec::new();
        
        for &block_id in &loop_info.nodes {
            if block_id >= function.basic_blocks.len() || block_id == loop_info.header {
                continue;
            }
            
            let block = &function.basic_blocks[block_id];
            for _ in 0..unroll_factor {
                for instruction in &block.instructions {
                    unrolled_instructions.push(instruction.clone());
                }
            }
        }
        
        // Add unrolled instructions to the loop body
        if let Some(body_block) = loop_info.nodes.iter()
            .find(|&&id| id != loop_info.header && id < function.basic_blocks.len()) {
            function.basic_blocks[*body_block].instructions.extend(unrolled_instructions);
        }
        
        Ok(())
    }
    
    /// Apply strength reduction optimizations
    fn strength_reduction(&mut self, function: &mut IrFunction, loop_info: &NaturalLoop) -> Result<()> {
        // Look for multiplication by constants that can be replaced with shifts/adds
        for &block_id in &loop_info.nodes {
            if block_id >= function.basic_blocks.len() {
                continue;
            }
            
            let block = &mut function.basic_blocks[block_id];
            for instruction in &mut block.instructions {
                if instruction.opcode == IrOpcode::Mul {
                    if let (Some(IrValue::Register(_)), Some(IrValue::Constant(IrConstant::Integer(n)))) = 
                        (instruction.operands.get(0), instruction.operands.get(1)) {
                        
                        // Replace multiplication by power of 2 with left shift
                        if *n > 0 && (*n as u64).is_power_of_two() {
                            let shift_amount = (*n as u64).trailing_zeros();
                            instruction.opcode = IrOpcode::Shl;
                            instruction.operands[1] = IrValue::Constant(IrConstant::Integer(shift_amount as i64));
                        }
                        // Replace multiplication by 3 with add + shift
                        else if *n == 3 {
                            // This would require more complex transformation
                            // For now, we'll leave it as is
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Utility functions for optimization analysis
impl IrOptimizer {
    /// Check if two values are equivalent
    fn values_equivalent(&self, a: &IrValue, b: &IrValue) -> bool {
        match (a, b) {
            (IrValue::Register(reg_a), IrValue::Register(reg_b)) => reg_a == reg_b,
            (IrValue::Constant(const_a), IrValue::Constant(const_b)) => const_a == const_b,
            (IrValue::Global(name_a), IrValue::Global(name_b)) => name_a == name_b,
            (IrValue::Function(name_a), IrValue::Function(name_b)) => name_a == name_b,
            _ => false,
        }
    }

    /// Check if an instruction is pure (has no side effects)
    fn is_pure_instruction(&self, opcode: &IrOpcode) -> bool {
        !self.has_side_effects(opcode)
    }
    
    /// Estimate the cost of inlining a function (in terms of code size increase)
    fn estimate_inline_cost(&self, function: &IrFunction) -> usize {
        function.basic_blocks.iter()
            .map(|block| block.instructions.len())
            .sum()
    }
    
    /// Check if a loop is suitable for vectorization
    fn is_vectorizable_loop(&self, _function: &IrFunction, _loop_info: &NaturalLoop) -> bool {
        // Placeholder for vectorization analysis
        // Would check for data dependencies, memory access patterns, etc.
        false
    }
    
    /// Perform dead store elimination
    pub fn dead_store_elimination(&mut self, function: &mut IrFunction) -> Result<()> {
        let mut live_stores = HashSet::new();
        
        // Mark stores that are used
        for block in &function.basic_blocks {
            for instruction in &block.instructions {
                if instruction.opcode == IrOpcode::Load {
                    // This load makes the corresponding store live
                    if let Some(IrValue::Register(reg)) = instruction.operands.first() {
                        live_stores.insert(*reg);
                    }
                }
            }
        }
        
        // Remove dead stores
        for block in &mut function.basic_blocks {
            block.instructions.retain(|instruction| {
                if instruction.opcode == IrOpcode::Store {
                    if let Some(IrValue::Register(reg)) = instruction.operands.first() {
                        return live_stores.contains(reg);
                    }
                }
                true
            });
        }
        
        Ok(())
    }
}
