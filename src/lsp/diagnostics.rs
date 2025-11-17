use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::error::BuluError;

use super::backend::DocumentState;

/// Provides real-time diagnostics for Bulu code
pub struct DiagnosticsProvider {
    documents: Arc<DashMap<String, DocumentState>>,
}

impl DiagnosticsProvider {
    pub fn new(documents: Arc<DashMap<String, DocumentState>>) -> Self {
        Self { documents }
    }

    /// Analyze document and return diagnostics
    pub async fn analyze(&self, _uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Lexical analysis
        let mut lexer = Lexer::new(text);
        match lexer.tokenize() {
            Ok(tokens) => {
                // Syntax analysis
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(_ast) => {
                        // Successfully parsed - no errors
                        // In a full implementation, we would run type checking and linting here
                    }
                    Err(parse_error) => {
                        diagnostics.push(self.error_to_diagnostic(&parse_error, DiagnosticSeverity::ERROR));
                    }
                }
            }
            Err(lex_error) => {
                diagnostics.push(self.error_to_diagnostic(&lex_error, DiagnosticSeverity::ERROR));
            }
        }

        diagnostics
    }

    /// Convert BuluError to LSP Diagnostic
    fn error_to_diagnostic(&self, error: &BuluError, severity: DiagnosticSeverity) -> Diagnostic {
        let (line, column, message) = match error {
            BuluError::LexError { line, column, message, .. } => (*line, *column, message.clone()),
            BuluError::ParseError { line, column, message, .. } => (*line, *column, message.clone()),
            BuluError::TypeError { line, column, message, .. } => (*line, *column, message.clone()),
            BuluError::RuntimeError { message, .. } => (0, 0, message.clone()),
            _ => (0, 0, error.to_string()),
        };

        let start_line = if line > 0 { line - 1 } else { 0 };
        let start_char = if column > 0 { column - 1 } else { 0 };

        Diagnostic {
            range: Range {
                start: Position {
                    line: start_line as u32,
                    character: start_char as u32,
                },
                end: Position {
                    line: start_line as u32,
                    character: (start_char + 1) as u32,
                },
            },
            severity: Some(severity),
            code: None,
            code_description: None,
            source: Some("bulu".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        }
    }
}
