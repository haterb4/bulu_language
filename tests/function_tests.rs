//! Tests for function definitions and calls

use bulu::ast::*;
use bulu::error::Result;
use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn parse_source(source: &str) -> Result<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn parse_statement(source: &str) -> Result<Statement> {
    let program = parse_source(source)?;
    Ok(program.statements.into_iter().next().unwrap())
}

fn parse_expression(source: &str) -> Result<Expression> {
    let mut lexer = Lexer::new(source.trim());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_expression()
}

#[cfg(test)]
mod function_declaration_tests {
    use super::*;

    #[test]
    fn test_function_with_default_parameters() {
        let stmt = parse_statement(r#"
            func greet(name: string, greeting: string = "Hello"): string {
                return greeting + " " + name
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "greet");
                assert_eq!(decl.params.len(), 2);
                
                // First parameter (no default)
                assert_eq!(decl.params[0].name, "name");
                assert_eq!(decl.params[0].param_type, Type::String);
                assert!(decl.params[0].default_value.is_none());
                assert!(!decl.params[0].is_variadic);
                
                // Second parameter (with default)
                assert_eq!(decl.params[1].name, "greeting");
                assert_eq!(decl.params[1].param_type, Type::String);
                assert!(decl.params[1].default_value.is_some());
                assert!(!decl.params[1].is_variadic);
                
                if let Some(Expression::Literal(lit)) = &decl.params[1].default_value {
                    assert_eq!(lit.value, LiteralValue::String("Hello".to_string()));
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_variadic_function() {
        let stmt = parse_statement(r#"
            func sum(...nums: int32): int32 {
                let total = 0
                for num in nums {
                    total = total + num
                }
                return total
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "sum");
                assert_eq!(decl.params.len(), 1);
                
                // Variadic parameter
                assert_eq!(decl.params[0].name, "nums");
                assert_eq!(decl.params[0].param_type, Type::Int32);
                assert!(decl.params[0].default_value.is_none());
                assert!(decl.params[0].is_variadic);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_function_with_tuple_return_type() {
        let stmt = parse_statement(r#"
            func divmod(a: int32, b: int32): (int32, int32) {
                return a / b, a % b
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "divmod");
                assert_eq!(decl.params.len(), 2);
                
                // Check tuple return type
                if let Some(Type::Tuple(tuple_type)) = &decl.return_type {
                    assert_eq!(tuple_type.element_types.len(), 2);
                    assert_eq!(tuple_type.element_types[0], Type::Int32);
                    assert_eq!(tuple_type.element_types[1], Type::Int32);
                } else {
                    panic!("Expected tuple return type");
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_function_with_mixed_parameters() {
        let stmt = parse_statement(r#"
            func process(required: string, optional: int32 = 42, ...extra: string): bool {
                return true
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "process");
                assert_eq!(decl.params.len(), 3);
                
                // Required parameter
                assert_eq!(decl.params[0].name, "required");
                assert!(!decl.params[0].is_variadic);
                assert!(decl.params[0].default_value.is_none());
                
                // Optional parameter
                assert_eq!(decl.params[1].name, "optional");
                assert!(!decl.params[1].is_variadic);
                assert!(decl.params[1].default_value.is_some());
                
                // Variadic parameter
                assert_eq!(decl.params[2].name, "extra");
                assert!(decl.params[2].is_variadic);
                assert!(decl.params[2].default_value.is_none());
            }
            _ => panic!("Expected function declaration"),
        }
    }
}

#[cfg(test)]
mod lambda_expression_tests {
    use super::*;

    #[test]
    fn test_lambda_expression() {
        let expr = parse_expression(r#"
            func(x: int32, y: int32): int32 { return x + y }
        "#).unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 2);
                assert_eq!(lambda.params[0].name, "x");
                assert_eq!(lambda.params[0].param_type, Type::Int32);
                assert_eq!(lambda.params[1].name, "y");
                assert_eq!(lambda.params[1].param_type, Type::Int32);
                
                if let Some(Type::Int32) = lambda.return_type {
                    // Expected
                } else {
                    panic!("Expected int32 return type");
                }
                
                assert!(matches!(*lambda.body, Expression::Block(_)));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_lambda_expression_body() {
        let expr = parse_expression(r#"
            func(x: int32): int32 { x * 2 }
        "#).unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 1);
                assert!(matches!(*lambda.body, Expression::Block(_)));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_arrow_function_with_parentheses() {
        let expr = parse_expression(r#"
            (x: int32, y: int32) => x + y
        "#).unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 2);
                assert_eq!(lambda.params[0].name, "x");
                assert_eq!(lambda.params[0].param_type, Type::Int32);
                assert_eq!(lambda.params[1].name, "y");
                assert_eq!(lambda.params[1].param_type, Type::Int32);
                
                // Body should be a binary expression
                assert!(matches!(*lambda.body, Expression::Binary(_)));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_single_parameter_arrow_function() {
        let expr = parse_expression("x => x * 2").unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 1);
                assert_eq!(lambda.params[0].name, "x");
                assert_eq!(lambda.params[0].param_type, Type::Any); // Inferred
                
                // Body should be a binary expression
                assert!(matches!(*lambda.body, Expression::Binary(_)));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_arrow_function_with_block() {
        let expr = parse_expression(r#"
            (x: int32) => {
                let doubled = x * 2
                return doubled
            }
        "#).unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 1);
                assert!(matches!(*lambda.body, Expression::Block(_)));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_lambda_no_parameters() {
        let expr = parse_expression("func(): int32 { return 42 }").unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 0);
                if let Some(Type::Int32) = lambda.return_type {
                    // Expected
                } else {
                    panic!("Expected int32 return type");
                }
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_arrow_function_no_parameters() {
        let expr = parse_expression("() => 42").unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 0);
                assert!(lambda.return_type.is_none()); // Inferred
                assert!(matches!(*lambda.body, Expression::Literal(_)));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_lambda_with_default_parameters() {
        let expr = parse_expression(r#"
            func(x: int32, y: int32 = 10): int32 { return x + y }
        "#).unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 2);
                assert_eq!(lambda.params[0].name, "x");
                assert!(lambda.params[0].default_value.is_none());
                assert_eq!(lambda.params[1].name, "y");
                assert!(lambda.params[1].default_value.is_some());
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_lambda_with_variadic_parameters() {
        let expr = parse_expression(r#"
            func(...args: int32): int32 { return 0 }
        "#).unwrap();
        
        match expr {
            Expression::Lambda(lambda) => {
                assert_eq!(lambda.params.len(), 1);
                assert_eq!(lambda.params[0].name, "args");
                assert!(lambda.params[0].is_variadic);
            }
            _ => panic!("Expected lambda expression"),
        }
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_variable_capture() {
        // Test that we can parse a closure that references outer variables
        let program = parse_source(r#"
            func outer(): func(): int32 {
                let x = 42
                return func(): int32 { return x }
            }
        "#).unwrap();
        
        // Should parse successfully - actual closure semantics will be handled by semantic analysis
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "outer");
                // The function body should have let statement and return statement
                assert_eq!(decl.body.statements.len(), 2);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_nested_closures() {
        let program = parse_source(r#"
            func createAdder(x: int32): func(int32): func(int32): int32 {
                return func(y: int32): func(int32): int32 {
                    return func(z: int32): int32 { return x + y + z }
                }
            }
        "#).unwrap();
        
        // Should parse successfully
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "createAdder");
                // Check return type is a function type
                if let Some(Type::Function(func_type)) = &decl.return_type {
                    assert_eq!(func_type.param_types.len(), 1);
                    assert_eq!(func_type.param_types[0], Type::Int32);
                    
                    // Return type should be another function type
                    if let Some(return_type) = &func_type.return_type {
                        assert!(matches!(**return_type, Type::Function(_)));
                    }
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_closure_with_mutable_capture() {
        let program = parse_source(r#"
            func counter(): func(): int32 {
                let count = 0
                return func(): int32 {
                    count = count + 1
                    return count
                }
            }
        "#).unwrap();
        
        // Should parse successfully
        assert_eq!(program.statements.len(), 1);
    }
}

#[cfg(test)]
mod higher_order_function_tests {
    use super::*;

    #[test]
    fn test_function_taking_function_parameter() {
        let stmt = parse_statement(r#"
            func map(arr: []int32, fn: func(int32): int32): []int32 {
                let result = []
                for x in arr {
                    result.append(fn(x))
                }
                return result
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "map");
                assert_eq!(decl.params.len(), 2);
                
                // Second parameter should be a function type
                if let Type::Function(func_type) = &decl.params[1].param_type {
                    assert_eq!(func_type.param_types.len(), 1);
                    assert_eq!(func_type.param_types[0], Type::Int32);
                    
                    if let Some(return_type) = &func_type.return_type {
                        assert_eq!(**return_type, Type::Int32);
                    }
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_function_returning_function() {
        let stmt = parse_statement(r#"
            func createMultiplier(factor: int32): func(int32): int32 {
                return func(x: int32): int32 { return x * factor }
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "createMultiplier");
                
                // Return type should be a function type
                if let Some(Type::Function(func_type)) = &decl.return_type {
                    assert_eq!(func_type.param_types.len(), 1);
                    assert_eq!(func_type.param_types[0], Type::Int32);
                    
                    if let Some(return_type) = &func_type.return_type {
                        assert_eq!(**return_type, Type::Int32);
                    }
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_function_composition() {
        let program = parse_source(r#"
            func compose(f: func(int32): int32, g: func(int32): int32): func(int32): int32 {
                return func(x: int32): int32 { return f(g(x)) }
            }
        "#).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "compose");
                assert_eq!(decl.params.len(), 2);
                
                // Both parameters should be function types
                for param in &decl.params {
                    assert!(matches!(param.param_type, Type::Function(_)));
                }
                
                // Return type should be a function type
                assert!(matches!(decl.return_type, Some(Type::Function(_))));
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_callback_pattern() {
        let stmt = parse_statement(r#"
            func processAsync(data: string, callback: func(string)) {
                // Process data asynchronously
                callback(data + " processed")
            }
        "#).unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "processAsync");
                assert_eq!(decl.params.len(), 2);
                
                // Second parameter should be a function type with no return type (void)
                if let Type::Function(func_type) = &decl.params[1].param_type {
                    assert_eq!(func_type.param_types.len(), 1);
                    assert_eq!(func_type.param_types[0], Type::String);
                    assert!(func_type.return_type.is_none()); // void
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }
}

#[cfg(test)]
mod function_call_tests {
    use super::*;

    #[test]
    fn test_function_call_with_arguments() {
        let expr = parse_expression("add(1, 2, 3)").unwrap();
        
        match expr {
            Expression::Call(call) => {
                assert!(matches!(*call.callee, Expression::Identifier(_)));
                assert_eq!(call.args.len(), 3);
                
                for arg in &call.args {
                    assert!(matches!(arg, Expression::Literal(_)));
                }
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_function_call_no_arguments() {
        let expr = parse_expression("getValue()").unwrap();
        
        match expr {
            Expression::Call(call) => {
                assert!(matches!(*call.callee, Expression::Identifier(_)));
                assert_eq!(call.args.len(), 0);
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_chained_function_calls() {
        let expr = parse_expression("obj.method().getValue()").unwrap();
        
        match expr {
            Expression::Call(call) => {
                // The callee should be a member access on a call expression
                assert!(matches!(*call.callee, Expression::MemberAccess(_)));
                assert_eq!(call.args.len(), 0);
                
                // Check that the member access object is a call
                if let Expression::MemberAccess(member_access) = &*call.callee {
                    assert!(matches!(*member_access.object, Expression::Call(_)));
                    assert_eq!(member_access.member, "getValue");
                }
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_lambda_call() {
        let expr = parse_expression("(x => x * 2)(5)").unwrap();
        
        match expr {
            Expression::Call(call) => {
                // The callee should be a parenthesized lambda
                assert!(matches!(*call.callee, Expression::Parenthesized(_)));
                assert_eq!(call.args.len(), 1);
            }
            _ => panic!("Expected call expression"),
        }
    }
}

#[cfg(test)]
mod function_type_tests {
    use super::*;

    #[test]
    fn test_function_type_parsing() {
        let stmt = parse_statement("let callback: func(int32, string): bool").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                if let Some(Type::Function(func_type)) = &decl.type_annotation {
                    assert_eq!(func_type.param_types.len(), 2);
                    assert_eq!(func_type.param_types[0], Type::Int32);
                    assert_eq!(func_type.param_types[1], Type::String);
                    
                    if let Some(return_type) = &func_type.return_type {
                        assert_eq!(**return_type, Type::Bool);
                    } else {
                        panic!("Expected return type");
                    }
                    
                    assert!(!func_type.is_async);
                } else {
                    panic!("Expected function type");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_function_type_no_return() {
        let stmt = parse_statement("let action: func(string)").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                if let Some(Type::Function(func_type)) = &decl.type_annotation {
                    assert_eq!(func_type.param_types.len(), 1);
                    assert_eq!(func_type.param_types[0], Type::String);
                    assert!(func_type.return_type.is_none());
                } else {
                    panic!("Expected function type");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_function_type_no_parameters() {
        let stmt = parse_statement("let supplier: func(): int32").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                if let Some(Type::Function(func_type)) = &decl.type_annotation {
                    assert_eq!(func_type.param_types.len(), 0);
                    
                    if let Some(return_type) = &func_type.return_type {
                        assert_eq!(**return_type, Type::Int32);
                    } else {
                        panic!("Expected return type");
                    }
                } else {
                    panic!("Expected function type");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}

#[cfg(test)]
mod tuple_type_tests {
    use super::*;

    #[test]
    fn test_tuple_type_parsing() {
        let stmt = parse_statement("let pair: (int32, string)").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                if let Some(Type::Tuple(tuple_type)) = &decl.type_annotation {
                    assert_eq!(tuple_type.element_types.len(), 2);
                    assert_eq!(tuple_type.element_types[0], Type::Int32);
                    assert_eq!(tuple_type.element_types[1], Type::String);
                } else {
                    panic!("Expected tuple type");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_single_element_tuple_is_not_tuple() {
        let stmt = parse_statement("let value: (int32)").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                // Single element in parentheses should be treated as parenthesized type, not tuple
                assert_eq!(decl.type_annotation, Some(Type::Int32));
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_empty_tuple() {
        let stmt = parse_statement("let unit: ()").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                if let Some(Type::Tuple(tuple_type)) = &decl.type_annotation {
                    assert_eq!(tuple_type.element_types.len(), 0);
                } else {
                    panic!("Expected tuple type");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_nested_tuple_types() {
        let stmt = parse_statement("let nested: ((int32, string), bool)").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                if let Some(Type::Tuple(tuple_type)) = &decl.type_annotation {
                    assert_eq!(tuple_type.element_types.len(), 2);
                    
                    // First element should be a tuple
                    if let Type::Tuple(inner_tuple) = &tuple_type.element_types[0] {
                        assert_eq!(inner_tuple.element_types.len(), 2);
                        assert_eq!(inner_tuple.element_types[0], Type::Int32);
                        assert_eq!(inner_tuple.element_types[1], Type::String);
                    } else {
                        panic!("Expected nested tuple type");
                    }
                    
                    // Second element should be bool
                    assert_eq!(tuple_type.element_types[1], Type::Bool);
                } else {
                    panic!("Expected tuple type");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}