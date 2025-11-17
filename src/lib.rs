//! Bulu Programming Language
//! 
//! A modern programming language with strong concurrency support, 
//! memory safety, and expressive syntax.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod compiler;
pub mod runtime;
pub mod error;
pub mod error_reporter;
pub mod resolver;
pub mod types;

pub mod std;
pub mod project;
pub mod build;
pub mod testing;
pub mod formatter;
pub mod linter;
pub mod docs;
pub mod package;
pub mod lsp;

pub use error::{BuluError, Result};

// Re-export commonly used types for convenience
pub use runtime::interpreter::Interpreter;
pub use types::primitive::RuntimeValue as Value;
pub use types::primitive::RuntimeValue;

// Re-export interpreter module for backward compatibility
pub mod interpreter {
    pub use crate::runtime::interpreter::*;
    pub use crate::types::primitive::RuntimeValue as Value;
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LANGUAGE_VERSION: &str = "1.0.0";
pub const BULU_EXTENSION: &str = "bu";