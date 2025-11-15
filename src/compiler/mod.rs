//! Compiler module for the Bulu language
//! 
//! This module provides compilation functionality, including
//! semantic analysis, type checking, IR generation, optimization, and code generation.

pub mod semantic;
pub mod codegen;
pub mod optimizer;
pub mod ir;
pub mod ir_optimizer;
pub mod control_flow;
pub mod symbol_resolver;
pub mod native_backend;

pub use semantic::SemanticAnalyzer;
pub use codegen::CodeGenerator;
pub use ir::{IrGenerator, IrProgram};
pub use ir_optimizer::IrOptimizer;
pub use control_flow::ControlFlowAnalyzer;
pub use symbol_resolver::SymbolResolver;

/// Optimization levels
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    O0, // No optimization
    O1, // Basic optimization
    O2, // Standard optimization
    O3, // Aggressive optimization
    Os, // Optimize for size
}