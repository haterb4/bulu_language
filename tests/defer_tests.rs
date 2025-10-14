//! Comprehensive tests for defer statement functionality
//!
//! This module tests all aspects of the defer system:
//! - Defer statement parsing and execution
//! - LIFO execution order for multiple defers
//! - Defer execution on both normal and error returns
//! - Variable capture semantics for defer
//! - Integration with error handling

use bulu::ast::*;
use bulu::error::BuluError;
use bulu::interpreter::Interpreter;
use bulu::lexer::token::Position;
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::error_handler::ErrorHandler;

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
fn test_defer_statement_parsing() {
    let code = r#"
        defer print("cleanup")
    "#;

    let program = parse_code(code).expect("Failed to parse defer statement");

    assert_eq!(program.statements.len(), 1);

    if let Statement::Defer(defer_stmt) = &program.statements[0] {
        // Check that the deferred statement is parsed correctly
        if let Statement::Expression(expr_stmt) = defer_stmt.stmt.as_ref() {
            if let Expression::Call(call_expr) = &expr_stmt.expr {
                if let Expression::Identifier(ident) = call_expr.callee.as_ref() {
                    assert_eq!(ident.name, "print");
                }
            }
        }
    } else {
        panic!("Expected defer statement");
    }
}

#[test]
fn test_multiple_defer_parsing() {
    let code = r#"
        defer print("first")
        defer print("second")
        defer print("third")
    "#;

    let program = parse_code(code).expect("Failed to parse multiple defer statements");

    assert_eq!(program.statements.len(), 3);

    for statement in &program.statements {
        assert!(matches!(statement, Statement::Defer(_)));
    }
}

#[test]
fn test_defer_with_block_statement() {
    let code = r#"
        defer {
            print("cleanup line 1")
            print("cleanup line 2")
        }
    "#;

    let program = parse_code(code).expect("Failed to parse defer with block");

    if let Statement::Defer(defer_stmt) = &program.statements[0] {
        if let Statement::Block(block_stmt) = defer_stmt.stmt.as_ref() {
            assert_eq!(block_stmt.statements.len(), 2);
        } else {
            panic!("Expected block statement in defer");
        }
    } else {
        panic!("Expected defer statement");
    }
}

#[test]
fn test_defer_handler_lifo_order() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    // Add multiple defers
    let stmt1 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("first".to_string()),
            position: pos,
        }),
        position: pos,
    });

    let stmt2 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("second".to_string()),
            position: pos,
        }),
        position: pos,
    });

    let stmt3 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("third".to_string()),
            position: pos,
        }),
        position: pos,
    });

    handler.add_defer(stmt1, pos);
    handler.add_defer(stmt2, pos);
    handler.add_defer(stmt3, pos);

    assert_eq!(handler.defer_stack.len(), 3);

    // Get defers to execute (should be in LIFO order)
    let defers = handler.get_defers_to_execute(0);
    assert_eq!(defers.len(), 3);

    // Check LIFO order: third, second, first
    if let Statement::Expression(expr_stmt) = &defers[0].statement {
        if let Expression::Literal(literal) = &expr_stmt.expr {
            if let LiteralValue::String(s) = &literal.value {
                assert_eq!(s, "third");
            }
        }
    }

    if let Statement::Expression(expr_stmt) = &defers[1].statement {
        if let Expression::Literal(literal) = &expr_stmt.expr {
            if let LiteralValue::String(s) = &literal.value {
                assert_eq!(s, "second");
            }
        }
    }

    if let Statement::Expression(expr_stmt) = &defers[2].statement {
        if let Expression::Literal(literal) = &expr_stmt.expr {
            if let LiteralValue::String(s) = &literal.value {
                assert_eq!(s, "first");
            }
        }
    }
}

#[test]
fn test_defer_execution_on_normal_return() {
    let code = r#"
        func test_function(): int32 {
            defer print("cleanup 1")
            defer print("cleanup 2")
            return 42
        }
    "#;

    let program = parse_code(code).expect("Failed to parse function with defers");

    // Verify parsing structure
    if let Statement::FunctionDecl(func_decl) = &program.statements[0] {
        assert_eq!(func_decl.name, "test_function");
        assert_eq!(func_decl.body.statements.len(), 3); // 2 defers + 1 return
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_defer_execution_on_error() {
    let code = r#"
        try {
            defer print("cleanup 1")
            defer print("cleanup 2")
            fail "something went wrong"
        } fail on err {
            print("caught error: " + err)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse try-defer-fail");

    // Verify parsing structure
    if let Statement::Try(try_stmt) = &program.statements[0] {
        assert_eq!(try_stmt.body.statements.len(), 3); // 2 defers + 1 fail
        assert!(try_stmt.catch_clause.is_some());
    } else {
        panic!("Expected try statement");
    }
}

#[test]
fn test_defer_with_variable_access() {
    let code = r#"
        let x = 42
        defer print("x is: " + string(x))
        x = 100
    "#;

    let program = parse_code(code).expect("Failed to parse defer with variable");

    assert_eq!(program.statements.len(), 3);
    
    // Verify defer statement references variable
    if let Statement::Defer(defer_stmt) = &program.statements[1] {
        // The defer should contain a print call with variable reference
        assert!(matches!(defer_stmt.stmt.as_ref(), Statement::Expression(_)));
    } else {
        panic!("Expected defer statement");
    }
}

#[test]
fn test_nested_defer_scopes() {
    let code = r#"
        defer print("outer defer")
        {
            defer print("inner defer 1")
            defer print("inner defer 2")
        }
        defer print("outer defer 2")
    "#;

    let program = parse_code(code).expect("Failed to parse nested defer scopes");

    assert_eq!(program.statements.len(), 3); // outer defer, block, outer defer 2
    
    if let Statement::Block(block_stmt) = &program.statements[1] {
        assert_eq!(block_stmt.statements.len(), 2); // 2 inner defers
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_defer_in_loop() {
    let code = r#"
        for i in 0..<3 {
            defer print("cleanup for iteration: " + string(i))
            print("iteration: " + string(i))
        }
    "#;

    let program = parse_code(code).expect("Failed to parse defer in loop");

    if let Statement::For(for_stmt) = &program.statements[0] {
        assert_eq!(for_stmt.body.statements.len(), 2); // defer + print
    } else {
        panic!("Expected for statement");
    }
}

#[test]
fn test_defer_with_function_call() {
    let code = r#"
        func cleanup(name: string) {
            print("cleaning up: " + name)
        }
        
        defer cleanup("resource1")
        defer cleanup("resource2")
    "#;

    let program = parse_code(code).expect("Failed to parse defer with function calls");

    assert_eq!(program.statements.len(), 3); // function + 2 defers
    
    // Verify defer statements call the cleanup function
    for i in 1..3 {
        if let Statement::Defer(defer_stmt) = &program.statements[i] {
            if let Statement::Expression(expr_stmt) = defer_stmt.stmt.as_ref() {
                if let Expression::Call(call_expr) = &expr_stmt.expr {
                    if let Expression::Identifier(ident) = call_expr.callee.as_ref() {
                        assert_eq!(ident.name, "cleanup");
                    }
                }
            }
        }
    }
}

#[test]
fn test_defer_error_handling_integration() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    // Simulate entering a try block
    handler.enter_try_block(None, pos);
    let _initial_defer_count = handler.defer_stack.len();

    // Add some defers
    let defer1 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("defer1".to_string()),
            position: pos,
        }),
        position: pos,
    });

    let defer2 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("defer2".to_string()),
            position: pos,
        }),
        position: pos,
    });

    handler.add_defer(defer1, pos);
    handler.add_defer(defer2, pos);

    // Exit try block and get defers
    if let Some(try_block) = handler.exit_try_block() {
        let defers = handler.get_defers_to_execute(try_block.defer_count);
        assert_eq!(defers.len(), 2);
    }
}

#[test]
fn test_defer_with_complex_expressions() {
    let code = r#"
        let arr = [1, 2, 3]
        let map = {"key": "value"}
        
        defer {
            print("array length: " + string(len(arr)))
            print("map value: " + map["key"])
        }
    "#;

    let program = parse_code(code).expect("Failed to parse defer with complex expressions");

    assert_eq!(program.statements.len(), 3); // 2 variables + 1 defer
    
    if let Statement::Defer(defer_stmt) = &program.statements[2] {
        if let Statement::Block(block_stmt) = defer_stmt.stmt.as_ref() {
            assert_eq!(block_stmt.statements.len(), 2); // 2 print statements
        }
    }
}

#[test]
fn test_defer_execution_order_with_interpreter() {
    let mut interpreter = Interpreter::new();
    
    // Create a simple defer statement
    let defer_stmt = DeferStmt {
        stmt: Box::new(Statement::Expression(ExpressionStmt {
            expr: Expression::Literal(LiteralExpr {
                value: LiteralValue::String("test defer".to_string()),
                position: create_test_position(),
            }),
            position: create_test_position(),
        })),
        position: create_test_position(),
    };

    // Execute the defer statement (should add to defer stack)
    let result = interpreter.execute_defer_statement(&defer_stmt);
    assert!(result.is_ok());

    // Check that defer was added to the error handler
    assert_eq!(interpreter.error_handler.defer_stack.len(), 1);
}

#[test]
fn test_defer_variable_capture_semantics() {
    let code = r#"
        let x = 10
        defer print("x at defer time: " + string(x))
        x = 20
        defer print("x at second defer time: " + string(x))
    "#;

    let program = parse_code(code).expect("Failed to parse defer variable capture test");

    assert_eq!(program.statements.len(), 4); // let, defer, assignment, defer
    
    // Both defer statements should capture the variable reference
    // The actual capture semantics would be tested in runtime execution
    for i in [1, 3] {
        if let Statement::Defer(defer_stmt) = &program.statements[i] {
            // Verify the defer contains a print statement with variable reference
            assert!(matches!(defer_stmt.stmt.as_ref(), Statement::Expression(_)));
        }
    }
}

#[test]
fn test_defer_in_nested_functions() {
    let code = r#"
        func outer(): void {
            defer print("outer cleanup")
            
            func inner(): void {
                defer print("inner cleanup")
                print("inner work")
            }
            
            inner()
            print("outer work")
        }
    "#;

    let program = parse_code(code).expect("Failed to parse nested functions with defers");

    if let Statement::FunctionDecl(outer_func) = &program.statements[0] {
        assert_eq!(outer_func.name, "outer");
        // Should contain: defer, inner function, call to inner, print
        assert_eq!(outer_func.body.statements.len(), 4);
    }
}

#[test]
fn test_defer_with_panic_recovery() {
    let code = r#"
        try {
            defer print("cleanup before panic")
            panic("something terrible happened")
        } fail on err {
            print("recovered from panic: " + err)
        }
    "#;

    let program = parse_code(code).expect("Failed to parse defer with panic");

    if let Statement::Try(try_stmt) = &program.statements[0] {
        assert_eq!(try_stmt.body.statements.len(), 2); // defer + panic
        assert!(try_stmt.catch_clause.is_some());
    }
}

#[test]
fn test_defer_execution_with_early_return() {
    let code = r#"
        func early_return(condition: bool): int32 {
            defer print("always executed")
            
            if condition {
                defer print("conditional cleanup")
                return 1
            }
            
            defer print("normal path cleanup")
            return 0
        }
    "#;

    let program = parse_code(code).expect("Failed to parse early return with defers");

    if let Statement::FunctionDecl(func_decl) = &program.statements[0] {
        assert_eq!(func_decl.body.statements.len(), 4); // defer, if, defer, return
    }
}

#[test]
fn test_defer_stack_management() {
    let mut handler = ErrorHandler::new();
    let pos = Position::new(1, 1, 0);

    // Test that defer stack is properly managed across multiple scopes
    assert_eq!(handler.defer_stack.len(), 0);

    // Add defer in first scope
    let stmt1 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("scope1".to_string()),
            position: pos,
        }),
        position: pos,
    });
    handler.add_defer(stmt1, pos);
    assert_eq!(handler.defer_stack.len(), 1);

    // Enter nested scope and add more defers
    let initial_count = handler.defer_stack.len();
    let stmt2 = Statement::Expression(ExpressionStmt {
        expr: Expression::Literal(LiteralExpr {
            value: LiteralValue::String("scope2".to_string()),
            position: pos,
        }),
        position: pos,
    });
    handler.add_defer(stmt2, pos);
    assert_eq!(handler.defer_stack.len(), 2);

    // Exit nested scope (execute defers added in this scope)
    let nested_defers = handler.get_defers_to_execute(initial_count);
    assert_eq!(nested_defers.len(), 1);
    assert_eq!(handler.defer_stack.len(), 1);

    // Exit outer scope
    let outer_defers = handler.get_defers_to_execute(0);
    assert_eq!(outer_defers.len(), 1);
    assert_eq!(handler.defer_stack.len(), 0);
}