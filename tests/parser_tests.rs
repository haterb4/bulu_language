//! Parser tests for the Bulu language

use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::ast::*;
use bulu::error::BuluError;

/// Helper function to parse source code
fn parse_source(source: &str) -> Result<Program, BuluError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Helper function to parse a single statement
fn parse_statement(source: &str) -> Result<Statement, BuluError> {
    let program = parse_source(source)?;
    assert_eq!(program.statements.len(), 1, "Expected exactly one statement");
    Ok(program.statements.into_iter().next().unwrap())
}

/// Helper function to parse a single expression
fn parse_expression(source: &str) -> Result<Expression, BuluError> {
    let stmt = parse_statement(source)?;
    match stmt {
        Statement::Expression(expr_stmt) => Ok(expr_stmt.expr),
        _ => panic!("Expected expression statement"),
    }
}

#[cfg(test)]
mod variable_declaration_tests {
    use super::*;

    #[test]
    fn test_let_declaration_with_initializer() {
        let stmt = parse_statement("let x = 42").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert!(!decl.is_const);
                assert_eq!(decl.name, "x");
                assert!(decl.type_annotation.is_none());
                assert!(decl.initializer.is_some());
                
                if let Some(Expression::Literal(lit)) = decl.initializer {
                    assert_eq!(lit.value, LiteralValue::Integer(42));
                } else {
                    panic!("Expected integer literal initializer");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_const_declaration_with_initializer() {
        let stmt = parse_statement("const PI = 3.14").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert!(decl.is_const);
                assert_eq!(decl.name, "PI");
                assert!(decl.type_annotation.is_none());
                assert!(decl.initializer.is_some());
                
                if let Some(Expression::Literal(lit)) = decl.initializer {
                    assert_eq!(lit.value, LiteralValue::Float(3.14));
                } else {
                    panic!("Expected float literal initializer");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_let_declaration_with_type_annotation() {
        let stmt = parse_statement("let x: int32 = 42").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert!(!decl.is_const);
                assert_eq!(decl.name, "x");
                assert_eq!(decl.type_annotation, Some(Type::Int32));
                assert!(decl.initializer.is_some());
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_let_declaration_without_initializer() {
        let stmt = parse_statement("let x: int32").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert!(!decl.is_const);
                assert_eq!(decl.name, "x");
                assert_eq!(decl.type_annotation, Some(Type::Int32));
                assert!(decl.initializer.is_none());
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_const_declaration_without_initializer_fails() {
        let result = parse_statement("const PI: float64");
        assert!(result.is_err());
        
        if let Err(BuluError::ParseError { message, .. }) = result {
            assert!(message.contains("Constant declarations must have an initializer"));
        } else {
            panic!("Expected parse error about missing initializer");
        }
    }

    #[test]
    fn test_variable_declaration_with_string_literal() {
        let stmt = parse_statement("let name = \"Alice\"").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert_eq!(decl.name, "name");
                if let Some(Expression::Literal(lit)) = decl.initializer {
                    assert_eq!(lit.value, LiteralValue::String("Alice".to_string()));
                } else {
                    panic!("Expected string literal initializer");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_variable_declaration_with_boolean_literal() {
        let stmt = parse_statement("let active = true").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert_eq!(decl.name, "active");
                if let Some(Expression::Literal(lit)) = decl.initializer {
                    assert_eq!(lit.value, LiteralValue::Boolean(true));
                } else {
                    panic!("Expected boolean literal initializer");
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}

#[cfg(test)]
mod function_declaration_tests {
    use super::*;

    #[test]
    fn test_simple_function_declaration() {
        let stmt = parse_statement("func add(a: int32, b: int32): int32 { return a + b }").unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "add");
                assert!(!decl.is_async);
                assert_eq!(decl.params.len(), 2);
                assert_eq!(decl.params[0].name, "a");
                assert_eq!(decl.params[0].param_type, Type::Int32);
                assert_eq!(decl.params[1].name, "b");
                assert_eq!(decl.params[1].param_type, Type::Int32);
                assert_eq!(decl.return_type, Some(Type::Int32));
                assert_eq!(decl.body.statements.len(), 1);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_function_declaration_without_return_type() {
        let stmt = parse_statement("func greet(name: string) { print(name) }").unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "greet");
                assert!(!decl.is_async);
                assert_eq!(decl.params.len(), 1);
                assert_eq!(decl.params[0].name, "name");
                assert_eq!(decl.params[0].param_type, Type::String);
                assert!(decl.return_type.is_none());
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_function_declaration_without_parameters() {
        let stmt = parse_statement("func hello(): string { return \"Hello\" }").unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "hello");
                assert!(decl.params.is_empty());
                assert_eq!(decl.return_type, Some(Type::String));
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_async_function_declaration() {
        let stmt = parse_statement("async func fetchData(): string { return \"data\" }").unwrap();
        
        match stmt {
            Statement::FunctionDecl(decl) => {
                assert_eq!(decl.name, "fetchData");
                assert!(decl.is_async);
                assert_eq!(decl.return_type, Some(Type::String));
            }
            _ => panic!("Expected function declaration"),
        }
    }
}

#[cfg(test)]
mod expression_tests {
    use super::*;

    #[test]
    fn test_integer_literal() {
        let expr = parse_expression("42").unwrap();
        
        match expr {
            Expression::Literal(lit) => {
                assert_eq!(lit.value, LiteralValue::Integer(42));
            }
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_float_literal() {
        let expr = parse_expression("3.14").unwrap();
        
        match expr {
            Expression::Literal(lit) => {
                assert_eq!(lit.value, LiteralValue::Float(3.14));
            }
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_string_literal() {
        let expr = parse_expression("\"hello\"").unwrap();
        
        match expr {
            Expression::Literal(lit) => {
                assert_eq!(lit.value, LiteralValue::String("hello".to_string()));
            }
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_boolean_literals() {
        let expr_true = parse_expression("true").unwrap();
        let expr_false = parse_expression("false").unwrap();
        
        match expr_true {
            Expression::Literal(lit) => {
                assert_eq!(lit.value, LiteralValue::Boolean(true));
            }
            _ => panic!("Expected literal expression"),
        }
        
        match expr_false {
            Expression::Literal(lit) => {
                assert_eq!(lit.value, LiteralValue::Boolean(false));
            }
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_null_literal() {
        let expr = parse_expression("null").unwrap();
        
        match expr {
            Expression::Literal(lit) => {
                assert_eq!(lit.value, LiteralValue::Null);
            }
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_identifier() {
        let expr = parse_expression("variable").unwrap();
        
        match expr {
            Expression::Identifier(ident) => {
                assert_eq!(ident.name, "variable");
            }
            _ => panic!("Expected identifier expression"),
        }
    }

    #[test]
    fn test_binary_arithmetic_expressions() {
        let test_cases = vec![
            ("1 + 2", BinaryOperator::Add),
            ("5 - 3", BinaryOperator::Subtract),
            ("4 * 6", BinaryOperator::Multiply),
            ("8 / 2", BinaryOperator::Divide),
            ("10 % 3", BinaryOperator::Modulo),
        ];

        for (source, expected_op) in test_cases {
            let expr = parse_expression(source).unwrap();
            
            match expr {
                Expression::Binary(bin) => {
                    assert_eq!(bin.operator, expected_op);
                    // Check that left and right operands are literals
                    assert!(matches!(*bin.left, Expression::Literal(_)));
                    assert!(matches!(*bin.right, Expression::Literal(_)));
                }
                _ => panic!("Expected binary expression for: {}", source),
            }
        }
    }

    #[test]
    fn test_binary_comparison_expressions() {
        let test_cases = vec![
            ("a == b", BinaryOperator::Equal),
            ("a != b", BinaryOperator::NotEqual),
            ("a < b", BinaryOperator::Less),
            ("a > b", BinaryOperator::Greater),
            ("a <= b", BinaryOperator::LessEqual),
            ("a >= b", BinaryOperator::GreaterEqual),
        ];

        for (source, expected_op) in test_cases {
            let expr = parse_expression(source).unwrap();
            
            match expr {
                Expression::Binary(bin) => {
                    assert_eq!(bin.operator, expected_op);
                }
                _ => panic!("Expected binary expression for: {}", source),
            }
        }
    }

    #[test]
    fn test_binary_logical_expressions() {
        let test_cases = vec![
            ("a and b", BinaryOperator::And),
            ("a or b", BinaryOperator::Or),
        ];

        for (source, expected_op) in test_cases {
            let expr = parse_expression(source).unwrap();
            
            match expr {
                Expression::Binary(bin) => {
                    assert_eq!(bin.operator, expected_op);
                }
                _ => panic!("Expected binary expression for: {}", source),
            }
        }
    }

    #[test]
    fn test_unary_expressions() {
        let test_cases = vec![
            ("-x", UnaryOperator::Minus),
            ("+x", UnaryOperator::Plus),
            ("not x", UnaryOperator::Not),
        ];

        for (source, expected_op) in test_cases {
            let expr = parse_expression(source).unwrap();
            
            match expr {
                Expression::Unary(unary) => {
                    assert_eq!(unary.operator, expected_op);
                    assert!(matches!(*unary.operand, Expression::Identifier(_)));
                }
                _ => panic!("Expected unary expression for: {}", source),
            }
        }
    }

    #[test]
    fn test_function_call() {
        let expr = parse_expression("add(1, 2)").unwrap();
        
        match expr {
            Expression::Call(call) => {
                assert!(matches!(*call.callee, Expression::Identifier(_)));
                assert_eq!(call.args.len(), 2);
                assert!(matches!(call.args[0], Expression::Literal(_)));
                assert!(matches!(call.args[1], Expression::Literal(_)));
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_member_access() {
        let expr = parse_expression("obj.field").unwrap();
        
        match expr {
            Expression::MemberAccess(access) => {
                assert!(matches!(*access.object, Expression::Identifier(_)));
                assert_eq!(access.member, "field");
            }
            _ => panic!("Expected member access expression"),
        }
    }

    #[test]
    fn test_index_access() {
        let expr = parse_expression("arr[0]").unwrap();
        
        match expr {
            Expression::Index(index) => {
                assert!(matches!(*index.object, Expression::Identifier(_)));
                assert!(matches!(*index.index, Expression::Literal(_)));
            }
            _ => panic!("Expected index expression"),
        }
    }

    #[test]
    fn test_array_literal() {
        let expr = parse_expression("[1, 2, 3]").unwrap();
        
        match expr {
            Expression::Array(array) => {
                assert_eq!(array.elements.len(), 3);
                for element in array.elements {
                    assert!(matches!(element, Expression::Literal(_)));
                }
            }
            _ => panic!("Expected array expression"),
        }
    }

    #[test]
    fn test_empty_array_literal() {
        let expr = parse_expression("[]").unwrap();
        
        match expr {
            Expression::Array(array) => {
                assert_eq!(array.elements.len(), 0);
            }
            _ => panic!("Expected array expression"),
        }
    }

    #[test]
    fn test_parenthesized_expression() {
        let expr = parse_expression("(1 + 2)").unwrap();
        
        match expr {
            Expression::Parenthesized(paren) => {
                assert!(matches!(*paren.expr, Expression::Binary(_)));
            }
            _ => panic!("Expected parenthesized expression"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        // Test that multiplication has higher precedence than addition
        let expr = parse_expression("1 + 2 * 3").unwrap();
        
        match expr {
            Expression::Binary(bin) => {
                assert_eq!(bin.operator, BinaryOperator::Add);
                assert!(matches!(*bin.left, Expression::Literal(_)));
                assert!(matches!(*bin.right, Expression::Binary(_)));
                
                if let Expression::Binary(right_bin) = *bin.right {
                    assert_eq!(right_bin.operator, BinaryOperator::Multiply);
                }
            }
            _ => panic!("Expected binary expression with correct precedence"),
        }
    }

    #[test]
    fn test_assignment_expression() {
        let expr = parse_expression("x = 42").unwrap();
        
        match expr {
            Expression::Assignment(assign) => {
                assert_eq!(assign.operator, AssignmentOperator::Assign);
                assert!(matches!(*assign.target, Expression::Identifier(_)));
                assert!(matches!(*assign.value, Expression::Literal(_)));
            }
            _ => panic!("Expected assignment expression"),
        }
    }

    #[test]
    fn test_compound_assignment_expressions() {
        let test_cases = vec![
            ("x += 1", AssignmentOperator::AddAssign),
            ("x -= 1", AssignmentOperator::SubtractAssign),
            ("x *= 2", AssignmentOperator::MultiplyAssign),
            ("x /= 2", AssignmentOperator::DivideAssign),
            ("x %= 3", AssignmentOperator::ModuloAssign),
        ];

        for (source, expected_op) in test_cases {
            let expr = parse_expression(source).unwrap();
            
            match expr {
                Expression::Assignment(assign) => {
                    assert_eq!(assign.operator, expected_op);
                }
                _ => panic!("Expected assignment expression for: {}", source),
            }
        }
    }
}

#[cfg(test)]
mod statement_tests {
    use super::*;

    #[test]
    fn test_return_statement_with_value() {
        let stmt = parse_statement("return 42").unwrap();
        
        match stmt {
            Statement::Return(ret) => {
                assert!(ret.value.is_some());
                if let Some(Expression::Literal(lit)) = ret.value {
                    assert_eq!(lit.value, LiteralValue::Integer(42));
                }
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_return_statement_without_value() {
        let stmt = parse_statement("return").unwrap();
        
        match stmt {
            Statement::Return(ret) => {
                assert!(ret.value.is_none());
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_break_statement() {
        let stmt = parse_statement("break").unwrap();
        
        match stmt {
            Statement::Break(_) => {
                // Success
            }
            _ => panic!("Expected break statement"),
        }
    }

    #[test]
    fn test_continue_statement() {
        let stmt = parse_statement("continue").unwrap();
        
        match stmt {
            Statement::Continue(_) => {
                // Success
            }
            _ => panic!("Expected continue statement"),
        }
    }

    #[test]
    fn test_expression_statement() {
        let stmt = parse_statement("print(\"hello\")").unwrap();
        
        match stmt {
            Statement::Expression(expr_stmt) => {
                assert!(matches!(expr_stmt.expr, Expression::Call(_)));
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_if_statement() {
        let stmt = parse_statement("if x > 0 { print(\"positive\") }").unwrap();
        
        match stmt {
            Statement::If(if_stmt) => {
                assert!(matches!(if_stmt.condition, Expression::Binary(_)));
                assert_eq!(if_stmt.then_branch.statements.len(), 1);
                assert!(if_stmt.else_branch.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_if_else_statement() {
        let stmt = parse_statement("if x > 0 { print(\"positive\") } else { print(\"non-positive\") }").unwrap();
        
        match stmt {
            Statement::If(if_stmt) => {
                assert!(matches!(if_stmt.condition, Expression::Binary(_)));
                assert_eq!(if_stmt.then_branch.statements.len(), 1);
                assert!(if_stmt.else_branch.is_some());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_while_statement() {
        let stmt = parse_statement("while x < 10 { x = x + 1 }").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                assert!(matches!(while_stmt.condition, Expression::Binary(_)));
                assert_eq!(while_stmt.body.statements.len(), 1);
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_block_statement() {
        let stmt = parse_statement("{ let x = 1; let y = 2 }").unwrap();
        
        match stmt {
            Statement::Block(block) => {
                assert_eq!(block.statements.len(), 2);
                assert!(matches!(block.statements[0], Statement::VariableDecl(_)));
                assert!(matches!(block.statements[1], Statement::VariableDecl(_)));
            }
            _ => panic!("Expected block statement"),
        }
    }
}

#[cfg(test)]
mod program_tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let program = parse_source("").unwrap();
        assert!(program.statements.is_empty());
    }

    #[test]
    fn test_program_with_multiple_statements() {
        let source = r#"
            let x = 42
            const PI = 3.14
            func add(a: int32, b: int32): int32 {
                return a + b
            }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 3);
        
        assert!(matches!(program.statements[0], Statement::VariableDecl(_)));
        assert!(matches!(program.statements[1], Statement::VariableDecl(_)));
        assert!(matches!(program.statements[2], Statement::FunctionDecl(_)));
    }

    #[test]
    fn test_program_with_newlines() {
        let source = "let x = 1\n\nlet y = 2\n";
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 2);
    }
}

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_unexpected_token_error() {
        let result = parse_source("let x = @");
        assert!(result.is_err());
        
        if let Err(BuluError::LexError { message, .. }) = result {
            assert!(message.contains("Unexpected character"));
        }
    }

    #[test]
    fn test_missing_semicolon_or_newline() {
        // This should still work because we allow statements without explicit terminators
        let result = parse_source("let x = 42 let y = 43");
        // This might fail due to parsing the second 'let' as part of the expression
        // The exact behavior depends on our error recovery implementation
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_closing_paren() {
        let result = parse_source("func test(x: int32 { return x }");
        assert!(result.is_err());
        
        if let Err(BuluError::ParseError { message, .. }) = result {
            assert!(message.contains("Expected ')'"));
        }
    }

    #[test]
    fn test_missing_closing_brace() {
        let result = parse_source("func test() { let x = 1");
        assert!(result.is_err());
        
        if let Err(BuluError::ParseError { message, .. }) = result {
            assert!(message.contains("Expected '}'"));
        }
    }

    #[test]
    fn test_invalid_function_name() {
        let result = parse_source("func 123invalid() {}");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod type_parsing_tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let test_cases = vec![
            ("let x: int8", Type::Int8),
            ("let x: int16", Type::Int16),
            ("let x: int32", Type::Int32),
            ("let x: int64", Type::Int64),
            ("let x: uint8", Type::UInt8),
            ("let x: uint16", Type::UInt16),
            ("let x: uint32", Type::UInt32),
            ("let x: uint64", Type::UInt64),
            ("let x: float32", Type::Float32),
            ("let x: float64", Type::Float64),
            ("let x: bool", Type::Bool),
            ("let x: char", Type::Char),
            ("let x: string", Type::String),
            ("let x: any", Type::Any),
        ];

        for (source, expected_type) in test_cases {
            let stmt = parse_statement(source).unwrap();
            
            match stmt {
                Statement::VariableDecl(decl) => {
                    assert_eq!(decl.type_annotation, Some(expected_type));
                }
                _ => panic!("Expected variable declaration for: {}", source),
            }
        }
    }

    #[test]
    fn test_slice_type() {
        let stmt = parse_statement("let x: []int32").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                match decl.type_annotation {
                    Some(Type::Slice(slice_type)) => {
                        assert_eq!(*slice_type.element_type, Type::Int32);
                    }
                    _ => panic!("Expected slice type"),
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_named_type() {
        let stmt = parse_statement("let x: CustomType").unwrap();
        
        match stmt {
            Statement::VariableDecl(decl) => {
                assert_eq!(decl.type_annotation, Some(Type::Named("CustomType".to_string())));
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}