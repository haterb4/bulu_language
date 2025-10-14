//! Parser module for the Bulu language
//! 
//! This module provides parsing functionality, converting
//! a stream of tokens into an Abstract Syntax Tree (AST).

pub mod parser;
pub mod precedence;

pub use parser::Parser;