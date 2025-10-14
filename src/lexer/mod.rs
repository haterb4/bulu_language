//! Lexical analysis module for the Bulu language
//! 
//! This module provides tokenization functionality, converting
//! source code text into a stream of tokens for parsing.

pub mod token;
pub mod lexer;

pub use token::{Token, TokenType, Literal};
pub use lexer::Lexer;