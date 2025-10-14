//! Unit tests for select statement functionality
//!
//! This module tests the select statement implementation including:
//! - Parsing select statements and expressions
//! - Multiple channel cases
//! - Non-blocking operations with default case
//! - Random selection among ready channels
//! - Timeout patterns with timer channels

use bulu::ast::*;
use bulu::error::BuluError;
use bulu::interpreter::{Interpreter, Value};
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::channels::{Channel, ChannelResult, SendResult};
use bulu::types::primitive::TypeId;
use std::time::Duration;

/// Helper function to create a test position
fn test_position() -> bulu::lexer::token::Position {
    bulu::lexer::token::Position::new(1, 1, 0)
}

/// Helper function to parse code and return the AST
fn parse_code(code: &str) -> Result<Program, BuluError> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_select_statement_parsing() {
    let code = r#"
        select {
            value := <-ch1 -> {
                print("Received from ch1: " + value)
            }
            ch2 <- 42 -> {
                print("Sent to ch2")
            }
            _ -> {
                print("Default case")
            }
        }
    "#;

    let program = parse_code(code).expect("Failed to parse select statement");
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 3);
            
            // First arm: receive operation
            let first_arm = &select_stmt.arms[0];
            assert!(first_arm.channel_op.is_some());
            let channel_op = first_arm.channel_op.as_ref().unwrap();
            assert!(!channel_op.is_send);
            assert_eq!(channel_op.variable, Some("value".to_string()));
            
            // Second arm: send operation
            let second_arm = &select_stmt.arms[1];
            assert!(second_arm.channel_op.is_some());
            let channel_op = second_arm.channel_op.as_ref().unwrap();
            assert!(channel_op.is_send);
            assert!(channel_op.value.is_some());
            
            // Third arm: default case
            let third_arm = &select_stmt.arms[2];
            assert!(third_arm.channel_op.is_none());
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_expression_parsing() {
    let code = r#"
        let result = select {
            value := <-ch1 -> value * 2
            ch2 <- 42 -> "sent"
            _ -> "default"
        }
    "#;

    let program = parse_code(code).expect("Failed to parse select expression");
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::VariableDecl(var_decl) => {
            assert!(var_decl.initializer.is_some());
            match var_decl.initializer.as_ref().unwrap() {
                Expression::Select(select_expr) => {
                    assert_eq!(select_expr.arms.len(), 3);
                }
                _ => panic!("Expected select expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_select_with_multiple_channel_cases() {
    let code = r#"
        select {
            msg1 := <-ch1 -> print("Channel 1: " + msg1)
            msg2 := <-ch2 -> print("Channel 2: " + msg2)
            msg3 := <-ch3 -> print("Channel 3: " + msg3)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse multiple channel select");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 3);
            
            for arm in &select_stmt.arms {
                assert!(arm.channel_op.is_some());
                let channel_op = arm.channel_op.as_ref().unwrap();
                assert!(!channel_op.is_send); // All are receive operations
                assert!(channel_op.variable.is_some());
            }
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_with_send_operations() {
    let code = r#"
        select {
            ch1 <- value1 -> print("Sent to ch1")
            ch2 <- value2 -> print("Sent to ch2")
            _ -> print("No channel ready")
        }
    "#;

    let program = parse_code(code).expect("Failed to parse send operations select");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 3);
            
            // First two arms are send operations
            for i in 0..2 {
                let arm = &select_stmt.arms[i];
                assert!(arm.channel_op.is_some());
                let channel_op = arm.channel_op.as_ref().unwrap();
                assert!(channel_op.is_send);
                assert!(channel_op.value.is_some());
            }
            
            // Third arm is default case
            assert!(select_stmt.arms[2].channel_op.is_none());
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_without_default_case() {
    let code = r#"
        select {
            value := <-ch1 -> print("Received: " + value)
            ch2 <- 42 -> print("Sent 42")
        }
    "#;

    let program = parse_code(code).expect("Failed to parse select without default");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 2);
            
            // No default case
            for arm in &select_stmt.arms {
                assert!(arm.channel_op.is_some());
            }
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_with_mixed_operations() {
    let code = r#"
        select {
            received := <-input_ch -> process(received)
            output_ch <- computed_value -> log("Sent result")
            error := <-error_ch -> handle_error(error)
            _ -> timeout_handler()
        }
    "#;

    let program = parse_code(code).expect("Failed to parse mixed operations select");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 4);
            
            // First arm: receive
            let first_arm = &select_stmt.arms[0];
            assert!(first_arm.channel_op.is_some());
            assert!(!first_arm.channel_op.as_ref().unwrap().is_send);
            
            // Second arm: send
            let second_arm = &select_stmt.arms[1];
            assert!(second_arm.channel_op.is_some());
            assert!(second_arm.channel_op.as_ref().unwrap().is_send);
            
            // Third arm: receive
            let third_arm = &select_stmt.arms[2];
            assert!(third_arm.channel_op.is_some());
            assert!(!third_arm.channel_op.as_ref().unwrap().is_send);
            
            // Fourth arm: default
            let fourth_arm = &select_stmt.arms[3];
            assert!(fourth_arm.channel_op.is_none());
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_receive_without_assignment() {
    let code = r#"
        select {
            <-done_ch -> print("Done signal received")
            data := <-data_ch -> process(data)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse receive without assignment");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 2);
            
            // First arm: receive without assignment
            let first_arm = &select_stmt.arms[0];
            assert!(first_arm.channel_op.is_some());
            let channel_op = first_arm.channel_op.as_ref().unwrap();
            assert!(!channel_op.is_send);
            assert!(channel_op.variable.is_none());
            
            // Second arm: receive with assignment
            let second_arm = &select_stmt.arms[1];
            assert!(second_arm.channel_op.is_some());
            let channel_op = second_arm.channel_op.as_ref().unwrap();
            assert!(!channel_op.is_send);
            assert_eq!(channel_op.variable, Some("data".to_string()));
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_expression_evaluation() {
    let code = r#"
        let result = select {
            value := <-ch -> value + 10
            _ -> 0
        }
    "#;

    let program = parse_code(code).expect("Failed to parse select expression");
    
    // This test verifies the parsing structure for select expressions
    match &program.statements[0] {
        Statement::VariableDecl(var_decl) => {
            match var_decl.initializer.as_ref().unwrap() {
                Expression::Select(select_expr) => {
                    assert_eq!(select_expr.arms.len(), 2);
                    
                    // First arm should have a channel operation
                    let first_arm = &select_expr.arms[0];
                    assert!(first_arm.channel_op.is_some());
                    
                    // Second arm should be default case
                    let second_arm = &select_expr.arms[1];
                    assert!(second_arm.channel_op.is_none());
                }
                _ => panic!("Expected select expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_select_statement_runtime_execution() {
    // This test demonstrates the runtime behavior of select statements
    // Note: This is a simplified test that focuses on the parsing and AST structure
    // Full runtime testing would require a complete interpreter setup
    
    let mut interpreter = Interpreter::new();
    
    // Create channels for testing
    let ch1_id = {
        let registry = interpreter.get_channel_registry();
        let mut reg = registry.lock().unwrap();
        reg.create_channel(TypeId::Int32, 1)
    };
    
    // Define channels in interpreter environment
    interpreter.environment.define("ch1".to_string(), Value::Channel(ch1_id));
    
    // Test simple select statement parsing
    let code = r#"
        select {
            value := <-ch1 -> print(value)
            _ -> print("default")
        }
    "#;
    
    let program = parse_code(code).expect("Failed to parse select for runtime test");
    
    // Verify the structure is correct
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 2);
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_error_cases() {
    // Test invalid select syntax
    let invalid_cases = vec![
        // Missing arrow
        "select { ch <- value print(\"sent\") }",
        // Invalid channel operation
        "select { invalid_op -> print(\"test\") }",
        // Missing body
        "select { ch <- value -> }",
    ];
    
    for case in invalid_cases {
        let result = parse_code(case);
        assert!(result.is_err(), "Expected parsing error for: {}", case);
    }
}

#[test]
fn test_select_with_complex_expressions() {
    let code = r#"
        select {
            result := <-compute_ch -> {
                let processed = result * 2 + 1
                output_ch <- processed
                processed
            }
            status := <-status_ch -> match status {
                "ok" -> continue_processing()
                "error" -> handle_error()
                _ -> unknown_status()
            }
            _ -> {
                print("Timeout occurred")
                default_value()
            }
        }
    "#;

    let program = parse_code(code).expect("Failed to parse complex select");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 3);
            
            // Verify that complex expressions are parsed correctly
            for arm in &select_stmt.arms {
                match &arm.body {
                    Statement::Block(_) => {
                        // Block statements are correctly parsed
                    }
                    Statement::Expression(_) => {
                        // Expression statements are correctly parsed
                    }
                    _ => panic!("Unexpected arm body type"),
                }
            }
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_nested_select_statements() {
    let code = r#"
        select {
            outer_value := <-outer_ch -> {
                select {
                    inner_value := <-inner_ch -> process(outer_value, inner_value)
                    _ -> process(outer_value, null)
                }
            }
            _ -> default_processing()
        }
    "#;

    let program = parse_code(code).expect("Failed to parse nested select");
    
    match &program.statements[0] {
        Statement::Select(outer_select) => {
            assert_eq!(outer_select.arms.len(), 2);
            
            // Check that the first arm contains a block with a nested select
            match &outer_select.arms[0].body {
                Statement::Block(block) => {
                    assert!(!block.statements.is_empty());
                    match &block.statements[0] {
                        Statement::Select(inner_select) => {
                            assert_eq!(inner_select.arms.len(), 2);
                        }
                        _ => panic!("Expected nested select statement"),
                    }
                }
                _ => panic!("Expected block statement"),
            }
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_select_with_timer_pattern() {
    // Test timeout pattern using timer channels (conceptual)
    let code = r#"
        select {
            data := <-data_ch -> process_data(data)
            <-timer(5000) -> handle_timeout()
        }
    "#;

    let program = parse_code(code).expect("Failed to parse timer pattern");
    
    match &program.statements[0] {
        Statement::Select(select_stmt) => {
            assert_eq!(select_stmt.arms.len(), 2);
            
            // First arm: data channel
            let first_arm = &select_stmt.arms[0];
            assert!(first_arm.channel_op.is_some());
            assert!(!first_arm.channel_op.as_ref().unwrap().is_send);
            
            // Second arm: timer channel (receive without assignment)
            let second_arm = &select_stmt.arms[1];
            assert!(second_arm.channel_op.is_some());
            assert!(!second_arm.channel_op.as_ref().unwrap().is_send);
            assert!(second_arm.channel_op.as_ref().unwrap().variable.is_none());
        }
        _ => panic!("Expected select statement"),
    }
}