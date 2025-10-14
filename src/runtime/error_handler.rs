//! Runtime error handling for the Bulu language
//!
//! This module implements the try-fail error handling mechanism,
//! error propagation, and error formatting.

use crate::ast::*;
use crate::error::{BuluError, Result};
use crate::lexer::token::Position;

use std::fmt;

/// Runtime error value that can be thrown and caught
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
    pub error_type: ErrorType,
    pub position: Option<Position>,
    pub stack_trace: Vec<StackFrame>,
}

/// Different types of runtime errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    /// User-defined error thrown with 'fail'
    UserError,
    /// Division by zero
    DivisionByZero,
    /// Null pointer dereference
    NullPointer,
    /// Array index out of bounds
    IndexOutOfBounds,
    /// Type cast failure
    TypeCastError,
    /// Channel operation error
    ChannelError,
    /// Panic (unrecoverable error)
    Panic,
}

/// Stack frame for error stack traces
#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    pub function_name: String,
    pub position: Position,
}

/// Error handler manages try-fail blocks and error propagation
pub struct ErrorHandler {
    /// Stack of active try blocks
    pub try_stack: Vec<TryBlock>,
    /// Current error being handled
    current_error: Option<RuntimeError>,
    /// Defer stack for cleanup
    pub defer_stack: Vec<DeferredAction>,
}

/// Active try block information
#[derive(Debug, Clone)]
pub struct TryBlock {
    pub catch_clause: Option<CatchClause>,
    pub position: Position,
    pub defer_count: usize, // Number of defers when this try block started
}

/// Deferred action for cleanup
#[derive(Debug, Clone)]
pub struct DeferredAction {
    pub statement: Statement,
    pub position: Position,
}

impl RuntimeError {
    /// Create a new runtime error
    pub fn new(message: String, error_type: ErrorType, position: Option<Position>) -> Self {
        Self {
            message,
            error_type,
            position,
            stack_trace: Vec::new(),
        }
    }

    /// Create a user error from a fail statement
    pub fn user_error(message: String, position: Position) -> Self {
        Self::new(message, ErrorType::UserError, Some(position))
    }

    /// Add a stack frame to the error
    pub fn add_stack_frame(&mut self, function_name: String, position: Position) {
        self.stack_trace.push(StackFrame {
            function_name,
            position,
        });
    }

    /// Format the error with stack trace
    pub fn format_with_stack_trace(&self) -> String {
        let mut result = format!("Error: {}", self.message);
        
        if let Some(pos) = &self.position {
            result.push_str(&format!(" at line {}, column {}", pos.line, pos.column));
        }

        if !self.stack_trace.is_empty() {
            result.push_str("\nStack trace:");
            for frame in &self.stack_trace {
                result.push_str(&format!(
                    "\n  at {} (line {}, column {})",
                    frame.function_name, frame.position.line, frame.position.column
                ));
            }
        }

        result
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_with_stack_trace())
    }
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        Self {
            try_stack: Vec::new(),
            current_error: None,
            defer_stack: Vec::new(),
        }
    }

    /// Enter a try block
    pub fn enter_try_block(&mut self, catch_clause: Option<CatchClause>, position: Position) {
        let try_block = TryBlock {
            catch_clause,
            position,
            defer_count: self.defer_stack.len(),
        };
        self.try_stack.push(try_block);
    }

    /// Exit a try block
    pub fn exit_try_block(&mut self) -> Option<TryBlock> {
        self.try_stack.pop()
    }

    /// Add a deferred action
    pub fn add_defer(&mut self, statement: Statement, position: Position) {
        self.defer_stack.push(DeferredAction {
            statement,
            position,
        });
    }

    /// Execute deferred actions in LIFO order
    /// Returns the deferred actions to be executed by the caller (interpreter)
    pub fn get_defers_to_execute(&mut self, until_count: usize) -> Vec<DeferredAction> {
        let mut defers_to_execute = Vec::new();
        
        while self.defer_stack.len() > until_count {
            if let Some(deferred) = self.defer_stack.pop() {
                defers_to_execute.push(deferred);
            }
        }
        
        defers_to_execute
    }

    /// Execute deferred actions in LIFO order (legacy method for compatibility)
    pub fn execute_defers(&mut self, until_count: usize) -> Result<()> {
        let _defers = self.get_defers_to_execute(until_count);
        // Note: This method is kept for compatibility but doesn't actually execute
        // The caller should use get_defers_to_execute and execute them properly
        Ok(())
    }

    /// Throw an error (fail statement)
    pub fn throw_error(&mut self, error: RuntimeError) -> Result<()> {
        self.current_error = Some(error.clone());

        // Look for a matching try block
        while let Some(try_block) = self.try_stack.pop() {
            // Get defers to execute up to this try block
            let _defers = self.get_defers_to_execute(try_block.defer_count);

            // If there's a catch clause, handle the error
            if let Some(_catch_clause) = try_block.catch_clause {
                // Error is caught, clear current error
                self.current_error = None;
                return Ok(());
            }
        }

        // No try block caught the error, propagate it
        Err(BuluError::RuntimeError {
            file: None,
            message: error.format_with_stack_trace(),
        })
    }

    /// Check if there's an active error
    pub fn has_error(&self) -> bool {
        self.current_error.is_some()
    }

    /// Get the current error
    pub fn get_error(&self) -> Option<&RuntimeError> {
        self.current_error.as_ref()
    }

    /// Clear the current error
    pub fn clear_error(&mut self) {
        self.current_error = None;
    }

    /// Handle function return (get defers to execute)
    pub fn handle_return(&mut self) -> Vec<DeferredAction> {
        // Return all defers in LIFO order for execution by caller
        self.get_defers_to_execute(0)
    }

    /// Handle panic (unrecoverable error)
    pub fn panic(&mut self, message: String, position: Position) -> ! {
        let error = RuntimeError::new(message, ErrorType::Panic, Some(position));
        eprintln!("PANIC: {}", error.format_with_stack_trace());
        std::process::exit(1);
    }
}

/// Error formatting utilities
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format an error message with context
    pub fn format_error(error: &RuntimeError, context: &str) -> String {
        format!("{}: {}", context, error.format_with_stack_trace())
    }

    /// Format multiple errors
    pub fn format_multiple_errors(errors: &[RuntimeError]) -> String {
        let mut result = String::new();
        for (i, error) in errors.iter().enumerate() {
            if i > 0 {
                result.push_str("\n\n");
            }
            result.push_str(&format!("Error {}: {}", i + 1, error.format_with_stack_trace()));
        }
        result
    }

    /// Create a formatted error report
    pub fn create_error_report(
        error: &RuntimeError,
        source_code: &str,
        file_name: &str,
    ) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Error in {}\n", file_name));
        report.push_str(&format!("Type: {:?}\n", error.error_type));
        report.push_str(&format!("Message: {}\n", error.message));
        
        if let Some(pos) = &error.position {
            report.push_str(&format!("Location: line {}, column {}\n", pos.line, pos.column));
            
            // Show source code context
            let lines: Vec<&str> = source_code.lines().collect();
            if pos.line > 0 && pos.line <= lines.len() {
                let line_idx = pos.line - 1;
                
                // Show surrounding lines for context
                let start = if line_idx >= 2 { line_idx - 2 } else { 0 };
                let end = std::cmp::min(line_idx + 3, lines.len());
                
                report.push_str("\nSource context:\n");
                for i in start..end {
                    let marker = if i == line_idx { ">>>" } else { "   " };
                    report.push_str(&format!("{} {}: {}\n", marker, i + 1, lines[i]));
                }
                
                // Show column indicator
                if pos.column > 0 {
                    let spaces = " ".repeat(pos.column - 1 + 7); // 7 = ">>> N: ".len()
                    report.push_str(&format!("{}^\n", spaces));
                }
            }
        }
        
        if !error.stack_trace.is_empty() {
            report.push_str("\nStack trace:\n");
            for frame in &error.stack_trace {
                report.push_str(&format!(
                    "  at {} (line {}, column {})\n",
                    frame.function_name, frame.position.line, frame.position.column
                ));
            }
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_error_creation() {
        let error = RuntimeError::user_error(
            "Test error".to_string(),
            Position::new(10, 5, 100),
        );
        
        assert_eq!(error.message, "Test error");
        assert_eq!(error.error_type, ErrorType::UserError);
        assert_eq!(error.position.unwrap().line, 10);
    }

    #[test]
    fn test_error_handler_try_block() {
        let mut handler = ErrorHandler::new();
        let pos = Position::new(1, 1, 0);
        
        // Enter try block
        handler.enter_try_block(None, pos);
        assert_eq!(handler.try_stack.len(), 1);
        
        // Exit try block
        let try_block = handler.exit_try_block();
        assert!(try_block.is_some());
        assert_eq!(handler.try_stack.len(), 0);
    }

    #[test]
    fn test_defer_stack() {
        let mut handler = ErrorHandler::new();
        let pos = Position::new(1, 1, 0);
        let stmt = Statement::Expression(ExpressionStmt {
            expr: Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(42),
                position: pos,
            }),
            position: pos,
        });
        
        handler.add_defer(stmt, pos);
        assert_eq!(handler.defer_stack.len(), 1);
    }

    #[test]
    fn test_error_formatting() {
        let mut error = RuntimeError::user_error(
            "Division by zero".to_string(),
            Position::new(15, 8, 200),
        );
        
        error.add_stack_frame("main".to_string(), Position::new(20, 1, 250));
        
        let formatted = error.format_with_stack_trace();
        assert!(formatted.contains("Division by zero"));
        assert!(formatted.contains("line 15, column 8"));
        assert!(formatted.contains("Stack trace"));
        assert!(formatted.contains("at main"));
    }

    #[test]
    fn test_error_report() {
        let error = RuntimeError::user_error(
            "Invalid operation".to_string(),
            Position::new(3, 10, 50),
        );
        
        let source_code = "let x = 5\nlet y = 10\nlet z = x / 0  // Error here\nprint(z)";
        let report = ErrorFormatter::create_error_report(&error, source_code, "test.bu");
        
        assert!(report.contains("Error in test.bu"));
        assert!(report.contains("Invalid operation"));
        assert!(report.contains("line 3, column 10"));
        assert!(report.contains("Source context"));
        assert!(report.contains("let z = x / 0"));
    }
}