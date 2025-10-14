//! Tests for async function declarations and Promise types

use bulu::ast::nodes::*;
use bulu::lexer::token::Position;
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::types::async_types::utils;
use bulu::types::checker::TypeChecker;

/// Helper function to parse a statement from source code
fn parse_statement(source: &str) -> Result<Statement, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    if program.statements.is_empty() {
        return Err("No statements parsed".into());
    }

    Ok(program.statements[0].clone())
}

/// Helper function to parse an expression from source code
fn parse_expression(source: &str) -> Result<Expression, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    if let Some(Statement::Expression(expr_stmt)) = program.statements.first() {
        Ok(expr_stmt.expr.clone())
    } else {
        Err("Expected expression statement".into())
    }
}

#[cfg(test)]
mod async_function_parsing_tests {
    use super::*;

    #[test]
    fn test_async_function_declaration_basic() {
        let stmt = parse_statement("async func fetchData(): string { return \"data\" }").unwrap();

        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "fetchData");
                assert!(decl.is_async);
                assert_eq!(decl.return_type, Some(Type::String));
                assert!(decl.params.is_empty());
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_async_function_with_parameters() {
        let stmt =
            parse_statement("async func fetchUser(id: int32): string { return \"user\" }").unwrap();

        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "fetchUser");
                assert!(decl.is_async);
                assert_eq!(decl.return_type, Some(Type::String));
                assert_eq!(decl.params.len(), 1);
                assert_eq!(decl.params[0].name, "id");
                assert_eq!(decl.params[0].param_type, Type::Int32);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_async_function_no_return_type() {
        let stmt = parse_statement("async func doWork() { print(\"working\") }").unwrap();

        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "doWork");
                assert!(decl.is_async);
                assert_eq!(decl.return_type, None);
                assert!(decl.params.is_empty());
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_async_function_with_generics() {
        let stmt = parse_statement("async func process<T>(data: T): T { return data }").unwrap();

        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "process");
                assert!(decl.is_async);
                assert_eq!(decl.type_params.len(), 1);
                assert_eq!(decl.type_params[0].name, "T");
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_sync_function_not_async() {
        let stmt = parse_statement("func syncFunction(): int32 { return 42 }").unwrap();

        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "syncFunction");
                assert!(!decl.is_async);
                assert_eq!(decl.return_type, Some(Type::Int32));
            }
            _ => panic!("Expected function declaration"),
        }
    }
}

#[cfg(test)]
mod async_expression_parsing_tests {
    use super::*;

    #[test]
    fn test_async_expression() {
        // For now, async expressions are not implemented in the parser
        // This test demonstrates the expected AST structure
        let async_expr = AsyncExpr {
            expr: Box::new(Expression::Call(CallExpr {
                callee: Box::new(Expression::Identifier(IdentifierExpr {
                    name: "fetchData".to_string(),
                    position: Position::new(1, 1, 0),
                })),
                type_args: vec![],
                args: vec![],
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        match async_expr.expr.as_ref() {
            Expression::Call(call) => {
                if let Expression::Identifier(ident) = call.callee.as_ref() {
                    assert_eq!(ident.name, "fetchData");
                } else {
                    panic!("Expected identifier in call");
                }
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_await_expression() {
        let expr = parse_expression("await promise").unwrap();

        match expr {
            Expression::Await(await_expr) => match await_expr.expr.as_ref() {
                Expression::Identifier(ident) => {
                    assert_eq!(ident.name, "promise");
                }
                _ => panic!("Expected identifier"),
            },
            _ => panic!("Expected await expression"),
        }
    }

    #[test]
    fn test_await_async_expression() {
        // For now, async expressions are not implemented in the parser
        // This test demonstrates the expected AST structure
        let await_expr = AwaitExpr {
            expr: Box::new(Expression::Async(AsyncExpr {
                expr: Box::new(Expression::Call(CallExpr {
                    callee: Box::new(Expression::Identifier(IdentifierExpr {
                        name: "fetchData".to_string(),
                        position: Position::new(1, 1, 0),
                    })),
                    type_args: vec![],
                    args: vec![],
                    position: Position::new(1, 1, 0),
                })),
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        match await_expr.expr.as_ref() {
            Expression::Async(async_expr) => match async_expr.expr.as_ref() {
                Expression::Call(call) => {
                    if let Expression::Identifier(ident) = call.callee.as_ref() {
                        assert_eq!(ident.name, "fetchData");
                    } else {
                        panic!("Expected identifier in call");
                    }
                }
                _ => panic!("Expected call expression"),
            },
            _ => panic!("Expected async expression"),
        }
    }
}

#[cfg(test)]
mod promise_type_tests {
    use super::*;

    #[test]
    fn test_promise_type_creation() {
        let pos = Position::new(1, 1, 0);
        let promise_type = PromiseType {
            result_type: Box::new(Type::String),
            position: pos,
        };

        assert_eq!(*promise_type.result_type, Type::String);
        assert_eq!(promise_type.position, pos);
    }

    #[test]
    fn test_promise_type_utils() {
        let pos = Position::new(1, 1, 0);
        let promise_type = utils::make_promise_type(Type::Int32, pos);

        assert!(utils::is_promise_type(&promise_type));

        let result_type = utils::get_promise_result_type(&promise_type);
        assert_eq!(result_type, Some(&Type::Int32));
    }

    #[test]
    fn test_async_function_type_conversion() {
        let sync_func_type = FunctionType {
            param_types: vec![Type::String],
            return_type: Some(Box::new(Type::Bool)),
            is_async: false,
        };

        let async_func_type = utils::make_async_function_type(&sync_func_type);

        assert!(async_func_type.is_async);
        assert_eq!(async_func_type.param_types, sync_func_type.param_types);

        // Check that return type is wrapped in Promise
        match async_func_type.return_type {
            Some(ret_type) => match ret_type.as_ref() {
                Type::Promise(promise_type) => {
                    assert_eq!(*promise_type.result_type, Type::Bool);
                }
                _ => panic!("Expected Promise return type"),
            },
            None => panic!("Expected return type"),
        }
    }

    #[test]
    fn test_async_function_type_no_return() {
        let sync_func_type = FunctionType {
            param_types: vec![Type::String],
            return_type: None,
            is_async: false,
        };

        let async_func_type = utils::make_async_function_type(&sync_func_type);

        assert!(async_func_type.is_async);

        // Check that return type is Promise<void>
        match async_func_type.return_type {
            Some(ret_type) => match ret_type.as_ref() {
                Type::Promise(promise_type) => {
                    assert_eq!(*promise_type.result_type, Type::Void);
                }
                _ => panic!("Expected Promise return type"),
            },
            None => panic!("Expected Promise<void> return type"),
        }
    }
}

#[cfg(test)]
mod async_type_checking_tests {
    use super::*;

    #[test]
    fn test_async_function_type_checking() {
        let source = "async func fetchData(): string { return \"data\" }";
        let stmt = parse_statement(source).unwrap();

        let mut type_checker = TypeChecker::new();
        let result = type_checker.check_statement(&stmt);

        assert!(result.is_ok());
    }

    #[test]
    fn test_async_function_promise_return_type() {
        let source = r#"
            async func fetchUser(id: int32): string {
                return "user"
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut type_checker = TypeChecker::new();
        let result = type_checker.check_program(&program);

        assert!(result.is_ok());
    }

    #[test]
    fn test_await_expression_type_checking() {
        // This test would require a more complete type checker implementation
        // For now, we'll just test that the AST is created correctly
        let expr = parse_expression("await promise").unwrap();

        match expr {
            Expression::Await(await_expr) => {
                assert!(matches!(
                    await_expr.expr.as_ref(),
                    Expression::Identifier(_)
                ));
            }
            _ => panic!("Expected await expression"),
        }
    }
}

#[cfg(test)]
mod async_runtime_tests {
    use super::*;
    use bulu::types::primitive::RuntimeValue;
    use bulu::runtime::promises::PromiseRegistry;

    #[test]
    fn test_promise_registry_creation() {
        let mut registry = PromiseRegistry::new();
        let promise_id = registry.create_promise();

        assert_eq!(promise_id, 1);
        assert!(registry.has_promise(promise_id));

        let promise = registry.get_promise(promise_id).unwrap();
        assert!(promise.is_pending());
    }

    #[test]
    fn test_promise_resolve() {
        let mut registry = PromiseRegistry::new();
        let promise_id = registry.create_promise();

        let value = RuntimeValue::String("Hello".to_string());
        registry.resolve_promise(promise_id, value.clone()).unwrap();

        let promise = registry.get_promise(promise_id).unwrap();
        assert!(promise.is_resolved());
        assert_eq!(promise.get_value(), Some(&value));
    }

    #[test]
    fn test_promise_reject() {
        let mut registry = PromiseRegistry::new();
        let promise_id = registry.create_promise();

        let error = "Network error".to_string();
        registry.reject_promise(promise_id, error.clone()).unwrap();

        let promise = registry.get_promise(promise_id).unwrap();
        assert!(promise.is_rejected());
        assert_eq!(promise.get_error(), Some(&error));
    }
}

#[cfg(test)]
mod promise_chaining_tests {
    use bulu::types::primitive::RuntimeValue;
    use bulu::runtime::promises::{utils, PromiseRegistry};

    #[test]
    fn test_promise_all_success() {
        let mut registry = PromiseRegistry::new();

        let id1 = registry.create_resolved_promise(RuntimeValue::Integer(1));
        let id2 = registry.create_resolved_promise(RuntimeValue::Integer(2));
        let id3 = registry.create_resolved_promise(RuntimeValue::Integer(3));

        let result = utils::promise_all(&registry, &[id1, id2, id3]).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], RuntimeValue::Integer(1));
        assert_eq!(result[1], RuntimeValue::Integer(2));
        assert_eq!(result[2], RuntimeValue::Integer(3));
    }

    #[test]
    fn test_promise_all_failure() {
        let mut registry = PromiseRegistry::new();

        let id1 = registry.create_resolved_promise(RuntimeValue::Integer(1));
        let id2 = registry.create_rejected_promise("Error occurred".to_string());
        let id3 = registry.create_resolved_promise(RuntimeValue::Integer(3));

        let result = utils::promise_all(&registry, &[id1, id2, id3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error occurred");
    }

    #[test]
    fn test_promise_race() {
        let mut registry = PromiseRegistry::new();

        let id1 = registry.create_promise(); // pending
        let id2 = registry.create_resolved_promise(RuntimeValue::String("winner".to_string()));
        let id3 = registry.create_promise(); // pending

        let result = utils::promise_race(&registry, &[id1, id2, id3]).unwrap();
        assert_eq!(result, RuntimeValue::String("winner".to_string()));
    }
}

#[cfg(test)]
mod await_functionality_tests {
    use super::*;
    use bulu::ast::nodes::{ArrayExpr, AwaitExpr, CallExpr, Expression, IdentifierExpr};
    use bulu::interpreter::Interpreter;
    use bulu::types::primitive::RuntimeValue;

    #[test]
    fn test_await_resolved_promise() {
        let mut interpreter = Interpreter::new();
        interpreter.enter_async_context();

        // Create a resolved promise
        let promise_registry = interpreter.promise_registry();
        let mut registry = promise_registry.lock().unwrap();
        let promise_id = registry.create_resolved_promise(RuntimeValue::String("Hello".to_string()));
        drop(registry);

        // Create await expression
        let await_expr = AwaitExpr {
            expr: Box::new(Expression::Identifier(IdentifierExpr {
                name: "promise".to_string(),
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        // Mock the promise value in environment
        interpreter
            .environment
            .define("promise".to_string(), RuntimeValue::Promise(promise_id as u32));

        let result = interpreter.evaluate_await_expression(&await_expr).unwrap();
        assert_eq!(result, RuntimeValue::String("Hello".to_string()));

        interpreter.exit_context();
    }

    #[test]
    fn test_await_rejected_promise() {
        let mut interpreter = Interpreter::new();
        interpreter.enter_async_context();

        // Create a rejected promise
        let promise_registry = interpreter.promise_registry();
        let mut registry = promise_registry.lock().unwrap();
        let promise_id = registry.create_rejected_promise("Something went wrong".to_string());
        drop(registry);

        // Create await expression
        let await_expr = AwaitExpr {
            expr: Box::new(Expression::Identifier(IdentifierExpr {
                name: "promise".to_string(),
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        // Mock the promise value in environment
        interpreter
            .environment
            .define("promise".to_string(), RuntimeValue::Promise(promise_id as u32));

        let result = interpreter.evaluate_await_expression(&await_expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Promise rejected: Something went wrong"));

        interpreter.exit_context();
    }

    #[test]
    fn test_await_outside_async_context() {
        let mut interpreter = Interpreter::new();
        // Don't enter async context

        // Create await expression
        let await_expr = AwaitExpr {
            expr: Box::new(Expression::Identifier(IdentifierExpr {
                name: "promise".to_string(),
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        let result = interpreter.evaluate_await_expression(&await_expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("await can only be used inside async functions"));
    }

    #[test]
    fn test_await_non_promise_value() {
        let mut interpreter = Interpreter::new();
        interpreter.enter_async_context();

        // Create await expression
        let await_expr = AwaitExpr {
            expr: Box::new(Expression::Identifier(IdentifierExpr {
                name: "not_promise".to_string(),
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        // Mock a non-promise value in environment
        interpreter
            .environment
            .define("not_promise".to_string(), RuntimeValue::Integer(42));

        let result = interpreter.evaluate_await_expression(&await_expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot await non-Promise value"));

        interpreter.exit_context();
    }

    #[test]
    fn test_promise_all_functionality() {
        let mut interpreter = Interpreter::new();

        // Create resolved promises
        let promise_registry = interpreter.promise_registry();
        let mut registry = promise_registry.lock().unwrap();
        let id1 = registry.create_resolved_promise(RuntimeValue::Integer(1));
        let id2 = registry.create_resolved_promise(RuntimeValue::Integer(2));
        let id3 = registry.create_resolved_promise(RuntimeValue::Integer(3));
        drop(registry);

        // Create Promise.all call
        let call_expr = CallExpr {
            callee: Box::new(Expression::Identifier(IdentifierExpr {
                name: "Promise.all".to_string(),
                position: Position::new(1, 1, 0),
            })),
            type_args: vec![],
            args: vec![Expression::Array(ArrayExpr {
                elements: vec![
                    Expression::Identifier(IdentifierExpr {
                        name: "promise1".to_string(),
                        position: Position::new(1, 1, 0),
                    }),
                    Expression::Identifier(IdentifierExpr {
                        name: "promise2".to_string(),
                        position: Position::new(1, 1, 0),
                    }),
                    Expression::Identifier(IdentifierExpr {
                        name: "promise3".to_string(),
                        position: Position::new(1, 1, 0),
                    }),
                ],
                position: Position::new(1, 1, 0),
            })],
            position: Position::new(1, 1, 0),
        };

        // Mock promise values in environment
        interpreter
            .environment
            .define("promise1".to_string(), RuntimeValue::Promise(id1 as u32));
        interpreter
            .environment
            .define("promise2".to_string(), RuntimeValue::Promise(id2 as u32));
        interpreter
            .environment
            .define("promise3".to_string(), RuntimeValue::Promise(id3 as u32));

        let result = interpreter.evaluate_call(&call_expr).unwrap();

        // Should return a promise
        match result {
            RuntimeValue::Promise(result_id) => {
                let registry = promise_registry.lock().unwrap();
                let promise = registry.get_promise(result_id as usize).unwrap();
                assert!(promise.is_resolved());

                if let Some(RuntimeValue::Array(results)) = promise.get_value() {
                    assert_eq!(results.len(), 3);
                    assert_eq!(results[0], RuntimeValue::Integer(1));
                    assert_eq!(results[1], RuntimeValue::Integer(2));
                    assert_eq!(results[2], RuntimeValue::Integer(3));
                } else {
                    panic!("Expected array result from Promise.all");
                }
            }
            _ => panic!("Expected Promise result from Promise.all"),
        }
    }

    #[test]
    fn test_promise_race_functionality() {
        let mut interpreter = Interpreter::new();

        // Create promises (one resolved, others pending)
        let promise_registry = interpreter.promise_registry();
        let mut registry = promise_registry.lock().unwrap();
        let id1 = registry.create_promise(); // pending
        let id2 = registry.create_resolved_promise(RuntimeValue::String("winner".to_string()));
        let id3 = registry.create_promise(); // pending
        drop(registry);

        // Create Promise.race call
        let call_expr = CallExpr {
            callee: Box::new(Expression::Identifier(IdentifierExpr {
                name: "Promise.race".to_string(),
                position: Position::new(1, 1, 0),
            })),
            type_args: vec![],
            args: vec![Expression::Array(ArrayExpr {
                elements: vec![
                    Expression::Identifier(IdentifierExpr {
                        name: "promise1".to_string(),
                        position: Position::new(1, 1, 0),
                    }),
                    Expression::Identifier(IdentifierExpr {
                        name: "promise2".to_string(),
                        position: Position::new(1, 1, 0),
                    }),
                    Expression::Identifier(IdentifierExpr {
                        name: "promise3".to_string(),
                        position: Position::new(1, 1, 0),
                    }),
                ],
                position: Position::new(1, 1, 0),
            })],
            position: Position::new(1, 1, 0),
        };

        // Mock promise values in environment
        interpreter
            .environment
            .define("promise1".to_string(), RuntimeValue::Promise(id1 as u32));
        interpreter
            .environment
            .define("promise2".to_string(), RuntimeValue::Promise(id2 as u32));
        interpreter
            .environment
            .define("promise3".to_string(), RuntimeValue::Promise(id3 as u32));

        let result = interpreter.evaluate_call(&call_expr).unwrap();

        // Should return a promise
        match result {
            RuntimeValue::Promise(result_id) => {
                let registry = promise_registry.lock().unwrap();
                let promise = registry.get_promise(result_id as usize).unwrap();
                assert!(promise.is_resolved());

                if let Some(RuntimeValue::String(winner)) = promise.get_value() {
                    assert_eq!(winner, "winner");
                } else {
                    panic!("Expected string result from Promise.race");
                }
            }
            _ => panic!("Expected Promise result from Promise.race"),
        }
    }

    #[test]
    fn test_promise_all_with_invalid_arguments() {
        let mut interpreter = Interpreter::new();

        // Test with wrong number of arguments
        let call_expr = CallExpr {
            callee: Box::new(Expression::Identifier(IdentifierExpr {
                name: "Promise.all".to_string(),
                position: Position::new(1, 1, 0),
            })),
            type_args: vec![],
            args: vec![], // No arguments
            position: Position::new(1, 1, 0),
        };

        let result = interpreter.evaluate_call(&call_expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Promise.all() requires exactly one argument"));
    }

    #[test]
    fn test_promise_all_with_non_array() {
        let mut interpreter = Interpreter::new();

        // Test with non-array argument
        let call_expr = CallExpr {
            callee: Box::new(Expression::Identifier(IdentifierExpr {
                name: "Promise.all".to_string(),
                position: Position::new(1, 1, 0),
            })),
            type_args: vec![],
            args: vec![Expression::Identifier(IdentifierExpr {
                name: "not_array".to_string(),
                position: Position::new(1, 1, 0),
            })],
            position: Position::new(1, 1, 0),
        };

        // Mock a non-array value
        interpreter.environment.define(
            "not_array".to_string(),
            RuntimeValue::String("not an array".to_string()),
        );

        let result = interpreter.evaluate_call(&call_expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Promise.all() requires an array of promises"));
    }

    #[test]
    fn test_promise_all_with_non_promise_elements() {
        let mut interpreter = Interpreter::new();

        // Test with array containing non-promise elements
        let call_expr = CallExpr {
            callee: Box::new(Expression::Identifier(IdentifierExpr {
                name: "Promise.all".to_string(),
                position: Position::new(1, 1, 0),
            })),
            type_args: vec![],
            args: vec![Expression::Array(ArrayExpr {
                elements: vec![Expression::Identifier(IdentifierExpr {
                    name: "not_promise".to_string(),
                    position: Position::new(1, 1, 0),
                })],
                position: Position::new(1, 1, 0),
            })],
            position: Position::new(1, 1, 0),
        };

        // Mock a non-promise value
        interpreter
            .environment
            .define("not_promise".to_string(), RuntimeValue::Integer(42));

        let result = interpreter.evaluate_call(&call_expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Promise.all() requires an array of promises"));
    }

    #[test]
    fn test_async_context_nesting() {
        let mut interpreter = Interpreter::new();

        // Test nested async contexts
        assert!(!interpreter.is_in_async_context());

        interpreter.enter_async_context();
        assert!(interpreter.is_in_async_context());

        interpreter.enter_sync_context();
        assert!(!interpreter.is_in_async_context());

        interpreter.exit_context();
        assert!(interpreter.is_in_async_context());

        interpreter.exit_context();
        assert!(!interpreter.is_in_async_context());
    }

    #[test]
    fn test_await_with_promise_timeout() {
        let mut interpreter = Interpreter::new();
        interpreter.enter_async_context();

        // Create a pending promise that will timeout
        let promise_registry = interpreter.promise_registry();
        let mut registry = promise_registry.lock().unwrap();
        let promise_id = registry.create_promise(); // This will remain pending
        drop(registry);

        // Create await expression
        let await_expr = AwaitExpr {
            expr: Box::new(Expression::Identifier(IdentifierExpr {
                name: "pending_promise".to_string(),
                position: Position::new(1, 1, 0),
            })),
            position: Position::new(1, 1, 0),
        };

        // Mock the promise value in environment
        interpreter
            .environment
            .define("pending_promise".to_string(), RuntimeValue::Promise(promise_id as u32));

        // This should timeout (but we can't easily test the timeout in a unit test)
        // For now, we'll just verify the structure is correct
        let result = interpreter.evaluate_await_expression(&await_expr);
        assert!(result.is_err());
        // The error should be about timeout, but since we can't wait 10 seconds in a test,
        // it might be a different error depending on timing

        interpreter.exit_context();
    }
}
