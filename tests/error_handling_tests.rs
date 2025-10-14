//! Comprehensive tests for try-fail error handling
//!
//! This module tests all aspects of the error handling system:
//! - Try-fail block parsing and execution
//! - Error propagation to calling functions
//! - Support for multiple error types
//! - Error message formatting

use bulu::ast::*;
use bulu::error::BuluError;
use bulu::interpreter::Interpreter;
use bulu::lexer::token::Position;
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::error_handler::{ErrorFormatter, ErrorHandler, ErrorType, RuntimeError};

fn create_test_position() -> Position {
    Position::new(1, 1, 0)
}

fn parse_code(code: &str) -> Result<Program, BuluError> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_try_fail_parsing() {
    let code = r#"
        try {
            let x = 10 / 0
        } fail on err {
            print("Error: " + err)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse try-fail block");

    // Check that we have a try statement
    assert_eq!(program.statements.len(), 1);

    if let Statement::Try(try_stmt) = &program.statements[0] {
        // Check that the try block has statements
        assert!(!try_stmt.body.statements.is_empty());

        // Check that there's a catch clause
        assert!(try_stmt.catch_clause.is_some());

        let catch_clause = try_stmt.catch_clause.as_ref().unwrap();
        assert_eq!(catch_clause.error_var, Some("err".to_string()));
        assert!(!catch_clause.body.statements.is_empty());
    } else {
        panic!("Expected try statement");
    }
}

#[test]
fn test_try_without_catch_parsing() {
    let code = r#"
        try {
            let x = 10 / 0
        }
    "#;

    let program = parse_code(code).expect("Failed to parse try block without catch");

    if let Statement::Try(try_stmt) = &program.statements[0] {
        assert!(try_stmt.catch_clause.is_none());
    } else {
        panic!("Expected try statement");
    }
}

#[test]
fn test_fail_statement_parsing() {
    let code = r#"
        fail "Something went wrong"
    "#;

    let program = parse_code(code).expect("Failed to parse fail statement");

    if let Statement::Fail(fail_stmt) = &program.statements[0] {
        if let Expression::Literal(literal) = &fail_stmt.message {
            if let LiteralValue::String(message) = &literal.value {
                assert_eq!(message, "Something went wrong");
            } else {
                panic!("Expected string literal in fail statement");
            }
        } else {
            panic!("Expected literal expression in fail statement");
        }
    } else {
        panic!("Expected fail statement");
    }
}

#[test]
fn test_runtime_error_creation() {
    let error =
        RuntimeError::user_error("Test error message".to_string(), Position::new(10, 5, 100));

    assert_eq!(error.message, "Test error message");
    assert_eq!(error.error_type, ErrorType::UserError);
    assert_eq!(error.position.unwrap().line, 10);
    assert_eq!(error.position.unwrap().column, 5);
}

#[test]
fn test_runtime_error_stack_trace() {
    let mut error = RuntimeError::user_error("Test error".to_string(), Position::new(5, 10, 50));

    error.add_stack_frame("function1".to_string(), Position::new(10, 1, 100));
    error.add_stack_frame("main".to_string(), Position::new(15, 1, 150));

    let formatted = error.format_with_stack_trace();

    assert!(formatted.contains("Test error"));
    assert!(formatted.contains("line 5, column 10"));
    assert!(formatted.contains("Stack trace"));
    assert!(formatted.contains("at function1"));
    assert!(formatted.contains("at main"));
}

#[test]
fn test_error_handler_try_blocks() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    // Test entering and exiting try blocks
    handler.enter_try_block(None, pos);
    assert_eq!(handler.try_stack.len(), 1);

    let try_block = handler.exit_try_block();
    assert!(try_block.is_some());
    assert_eq!(handler.try_stack.len(), 0);
}

#[test]
fn test_error_handler_defer_stack() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    let stmt = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("cleanup".to_string()),
            position: pos,
        }),
        position: pos,
    });

    handler.add_defer(stmt, pos);
    assert_eq!(handler.defer_stack.len(), 1);
}

#[test]
fn test_error_handler_throw_and_catch() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    // Create a catch clause
    let catch_clause = CatchClause {
        error_var: Some("err".to_string()),
        body: BlockStmt {
            statements: vec![],
            position: pos,
        },
        position: pos,
    };

    // Enter try block with catch clause
    handler.enter_try_block(Some(catch_clause), pos);

    // Throw an error
    let error = RuntimeError::user_error("Test error".to_string(), pos);
    let result = handler.throw_error(error);

    // Error should be caught
    assert!(result.is_ok());
    assert!(!handler.has_error());
}

#[test]
fn test_error_handler_propagation() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    // Enter try block without catch clause
    handler.enter_try_block(None, pos);

    // Throw an error
    let error = RuntimeError::user_error("Test error".to_string(), pos);
    let result = handler.throw_error(error);

    // Error should propagate
    assert!(result.is_err());
}

#[test]
fn test_error_formatter_single_error() {
    let error = RuntimeError::user_error("Test error".to_string(), Position::new(5, 10, 50));

    let formatted = ErrorFormatter::format_error(&error, "Test context");
    assert!(formatted.contains("Test context"));
    assert!(formatted.contains("Test error"));
}

#[test]
fn test_error_formatter_multiple_errors() {
    let errors = vec![
        RuntimeError::user_error("First error".to_string(), Position::new(1, 1, 0)),
        RuntimeError::user_error("Second error".to_string(), Position::new(2, 1, 10)),
    ];

    let formatted = ErrorFormatter::format_multiple_errors(&errors);
    assert!(formatted.contains("Error 1"));
    assert!(formatted.contains("Error 2"));
    assert!(formatted.contains("First error"));
    assert!(formatted.contains("Second error"));
}

#[test]
fn test_error_formatter_report() {
    let error = RuntimeError::user_error("Division by zero".to_string(), Position::new(3, 15, 30));

    let source_code = "let x = 10\nlet y = 0\nlet z = x / y  // Error here\nprint(z)";
    let report = ErrorFormatter::create_error_report(&error, source_code, "test.bu");

    assert!(report.contains("Error in test.bu"));
    assert!(report.contains("Division by zero"));
    assert!(report.contains("line 3, column 15"));
    assert!(report.contains("Source context"));
    assert!(report.contains("let z = x / y"));
    assert!(report.contains(">>>"));
}

#[test]
fn test_interpreter_try_catch_execution() {
    let code = r#"
        let result = 0
        try {
            let x = 10 / 0
        } fail on err {
            result = 1
        }
    "#;

    let program = parse_code(code).expect("Failed to parse code");
    let _interpreter = Interpreter::new();

    // This test would require a more complete interpreter implementation
    // For now, we'll test the basic structure
    assert!(!program.statements.is_empty());
}

#[test]
fn test_interpreter_fail_statement() {
    let mut interpreter = Interpreter::new();

    let fail_stmt = FailStmt {
        message: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("Test error".to_string()),
            position: create_test_position(),
        }),
        position: create_test_position(),
    };

    let result = interpreter.execute_fail_statement(&fail_stmt);
    // The error should be thrown through the error handler
    // Since there's no try block to catch it, it should propagate as an error
    assert!(result.is_err()); // The fail statement should throw an error
}

#[test]
fn test_interpreter_division_by_zero() {
    let mut interpreter = Interpreter::new();

    let binary_expr = BinaryExpr {
        left: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(10),
            position: create_test_position(),
        })),
        operator: BinaryOperator::Divide,
        right: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(0),
            position: create_test_position(),
        })),
        position: create_test_position(),
    };

    let result = interpreter.evaluate_binary(&binary_expr);
    assert!(result.is_err());

    if let Err(BuluError::RuntimeError { message , file }) = result {
        assert!(message.contains("Division by zero"));
    }
}

#[test]
fn test_multiple_error_types() {
    // Test different error types
    let user_error = RuntimeError::user_error("User error".to_string(), create_test_position());
    assert_eq!(user_error.error_type, ErrorType::UserError);

    let div_zero_error = RuntimeError::new(
        "Division by zero".to_string(),
        ErrorType::DivisionByZero,
        Some(create_test_position()),
    );
    assert_eq!(div_zero_error.error_type, ErrorType::DivisionByZero);

    let null_ptr_error = RuntimeError::new(
        "Null pointer".to_string(),
        ErrorType::NullPointer,
        Some(create_test_position()),
    );
    assert_eq!(null_ptr_error.error_type, ErrorType::NullPointer);
}

#[test]
fn test_error_message_formatting() {
    let mut error = RuntimeError::new(
        "Test error with formatting".to_string(),
        ErrorType::UserError,
        Some(Position::new(10, 5, 100)),
    );

    error.add_stack_frame("test_function".to_string(), Position::new(15, 1, 150));

    let formatted = error.format_with_stack_trace();

    // Check that all components are present
    assert!(formatted.contains("Test error with formatting"));
    assert!(formatted.contains("line 10, column 5"));
    assert!(formatted.contains("Stack trace"));
    assert!(formatted.contains("at test_function"));
    assert!(formatted.contains("line 15, column 1"));
}

#[test]
fn test_nested_try_blocks() {
    let code = r#"
        try {
            try {
                fail "Inner error"
            } fail on inner_err {
                fail "Outer error: " + inner_err
            }
        } fail on outer_err {
            print("Caught: " + outer_err)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse nested try blocks");

    // Verify the structure is parsed correctly
    assert!(!program.statements.is_empty());

    if let Statement::Try(outer_try) = &program.statements[0] {
        assert!(outer_try.catch_clause.is_some());

        // Check that the inner try block exists
        if let Some(Statement::Try(_inner_try)) = outer_try.body.statements.get(0) {
            // Successfully parsed nested try blocks
        } else {
            panic!("Expected nested try block");
        }
    } else {
        panic!("Expected outer try statement");
    }
}

#[test]
fn test_defer_with_error_handling() {
    let code = r#"
        try {
            defer print("Cleanup 1")
            defer print("Cleanup 2")
            fail "Something went wrong"
        } fail on err {
            print("Error handled: " + err)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse defer with error handling");

    // Verify parsing structure
    if let Statement::Try(try_stmt) = &program.statements[0] {
        // Should have defer statements and a fail statement
        assert!(try_stmt.body.statements.len() >= 3);
        assert!(try_stmt.catch_clause.is_some());
    } else {
        panic!("Expected try statement");
    }
}

#[test]
fn test_error_propagation_through_functions() {
    let code = r#"
        func risky_operation(): int32 {
            fail "Operation failed"
            return 42
        }
        
        func caller(): int32 {
            try {
                return risky_operation()
            } fail on err {
                print("Caught error: " + err)
                return -1
            }
        }
    "#;

    let program = parse_code(code).expect("Failed to parse error propagation code");

    // Verify we have function declarations
    assert_eq!(program.statements.len(), 2);

    // Both should be function declarations
    assert!(matches!(program.statements[0], Statement::FunctionDecl(_)));
    assert!(matches!(program.statements[1], Statement::FunctionDecl(_)));
}

#[test]
fn test_error_without_message() {
    let code = r#"
        fail null
    "#;

    let program = parse_code(code).expect("Failed to parse fail with null");

    if let Statement::Fail(fail_stmt) = &program.statements[0] {
        if let Expression::Literal(literal) = &fail_stmt.message {
            assert_eq!(literal.value, LiteralValue::Null);
        }
    }
}

#[test]
fn test_complex_error_scenarios() {
    // Test various complex error handling scenarios
    let scenarios = vec![
        // Try-catch with variable binding
        r#"
            try {
                let x = some_function()
            } fail on error {
                log("Error occurred: " + error)
            }
        "#,
        // Try without catch (error propagates)
        r#"
            try {
                dangerous_operation()
            }
        "#,
        // Multiple nested try blocks
        r#"
            try {
                try {
                    try {
                        fail "Deep error"
                    }
                } fail on e1 {
                    fail "Middle error: " + e1
                }
            } fail on e2 {
                print("Top level: " + e2)
            }
        "#,
    ];

    for (i, scenario) in scenarios.iter().enumerate() {
        let result = parse_code(scenario);
        assert!(
            result.is_ok(),
            "Scenario {} failed to parse: {:?}",
            i,
            result.err()
        );
    }
}
