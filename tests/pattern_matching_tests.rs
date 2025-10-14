//! Unit tests for pattern matching functionality

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

#[cfg(test)]
mod pattern_matching_tests {
    use super::*;

    #[test]
    fn test_basic_match_statement() {
        let source = r#"
        func test() {
            match x {
                1 -> print("one")
                2 -> print("two")
                _ -> print("other")
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "test");
                assert_eq!(func.body.statements.len(), 1);
                
                match &func.body.statements[0] {
                    Statement::Match(match_stmt) => {
                        assert!(matches!(match_stmt.expr, Expression::Identifier(_)));
                        assert_eq!(match_stmt.arms.len(), 3);
                        
                        // Check first arm (literal pattern)
                        match &match_stmt.arms[0].pattern {
                            Pattern::Literal(LiteralValue::Integer(1), _) => {},
                            _ => panic!("Expected integer literal pattern"),
                        }
                        
                        // Check third arm (wildcard pattern)
                        match &match_stmt.arms[2].pattern {
                            Pattern::Wildcard(_) => {},
                            _ => panic!("Expected wildcard pattern"),
                        }
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_match_expression() {
        let source = r#"
        func test() {
            let result = match x {
                1 -> "one"
                2 -> "two"
                _ -> "other"
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.body.statements.len(), 1);
                
                match &func.body.statements[0] {
                    Statement::VariableDecl(var_decl) => {
                        match &var_decl.initializer {
                            Some(Expression::Match(match_expr)) => {
                                assert!(matches!(*match_expr.expr, Expression::Identifier(_)));
                                assert_eq!(match_expr.arms.len(), 3);
                            }
                            _ => panic!("Expected match expression"),
                        }
                    }
                    _ => panic!("Expected variable declaration"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_range_patterns() {
        let source = r#"
        func test() {
            match age {
                0...12 -> print("child")
                13...19 -> print("teen")
                20..<65 -> print("adult")
                _ -> print("senior")
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                match &func.body.statements[0] {
                    Statement::Match(match_stmt) => {
                        assert_eq!(match_stmt.arms.len(), 4);
                        
                        // Check first range pattern (inclusive)
                        match &match_stmt.arms[0].pattern {
                            Pattern::Range(range_pattern) => {
                                assert!(range_pattern.inclusive);
                                assert_eq!(range_pattern.start, LiteralValue::Integer(0));
                                assert_eq!(range_pattern.end, LiteralValue::Integer(12));
                            }
                            _ => panic!("Expected range pattern"),
                        }
                        
                        // Check third range pattern (exclusive)
                        match &match_stmt.arms[2].pattern {
                            Pattern::Range(range_pattern) => {
                                assert!(!range_pattern.inclusive);
                                assert_eq!(range_pattern.start, LiteralValue::Integer(20));
                                assert_eq!(range_pattern.end, LiteralValue::Integer(65));
                            }
                            _ => panic!("Expected range pattern"),
                        }
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_struct_patterns() {
        let source = r#"
        func test() {
            match point {
                Point{x: 0, y: 0} -> print("origin")
                Point{x: 0, y: _} -> print("on y-axis")
                Point{x: _, y: 0} -> print("on x-axis")
                Point{x: x, y: y} -> print("general point")
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                match &func.body.statements[0] {
                    Statement::Match(match_stmt) => {
                        assert_eq!(match_stmt.arms.len(), 4);
                        
                        // Check first struct pattern
                        match &match_stmt.arms[0].pattern {
                            Pattern::Struct(struct_pattern) => {
                                assert_eq!(struct_pattern.name, "Point");
                                assert_eq!(struct_pattern.fields.len(), 2);
                                
                                // Check x field pattern
                                assert_eq!(struct_pattern.fields[0].name, "x");
                                match &*struct_pattern.fields[0].pattern {
                                    Pattern::Literal(LiteralValue::Integer(0), _) => {},
                                    _ => panic!("Expected integer literal pattern for x"),
                                }
                                
                                // Check y field pattern
                                assert_eq!(struct_pattern.fields[1].name, "y");
                                match &*struct_pattern.fields[1].pattern {
                                    Pattern::Literal(LiteralValue::Integer(0), _) => {},
                                    _ => panic!("Expected integer literal pattern for y"),
                                }
                            }
                            _ => panic!("Expected struct pattern"),
                        }
                        
                        // Check second struct pattern with wildcard
                        match &match_stmt.arms[1].pattern {
                            Pattern::Struct(struct_pattern) => {
                                assert_eq!(struct_pattern.name, "Point");
                                assert_eq!(struct_pattern.fields.len(), 2);
                                
                                // Check y field is wildcard
                                assert_eq!(struct_pattern.fields[1].name, "y");
                                match &*struct_pattern.fields[1].pattern {
                                    Pattern::Wildcard(_) => {},
                                    _ => panic!("Expected wildcard pattern for y"),
                                }
                            }
                            _ => panic!("Expected struct pattern"),
                        }
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_or_patterns() {
        let source = r#"
        func test() {
            match value {
                1 | 2 | 3 -> print("small")
                10 | 20 -> print("round")
                _ -> print("other")
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                match &func.body.statements[0] {
                    Statement::Match(match_stmt) => {
                        assert_eq!(match_stmt.arms.len(), 3);
                        
                        // Check first OR pattern
                        match &match_stmt.arms[0].pattern {
                            Pattern::Or(or_pattern) => {
                                assert_eq!(or_pattern.patterns.len(), 3);
                                
                                // Check individual patterns
                                match &or_pattern.patterns[0] {
                                    Pattern::Literal(LiteralValue::Integer(1), _) => {},
                                    _ => panic!("Expected integer literal 1"),
                                }
                                match &or_pattern.patterns[1] {
                                    Pattern::Literal(LiteralValue::Integer(2), _) => {},
                                    _ => panic!("Expected integer literal 2"),
                                }
                                match &or_pattern.patterns[2] {
                                    Pattern::Literal(LiteralValue::Integer(3), _) => {},
                                    _ => panic!("Expected integer literal 3"),
                                }
                            }
                            _ => panic!("Expected OR pattern"),
                        }
                        
                        // Check second OR pattern
                        match &match_stmt.arms[1].pattern {
                            Pattern::Or(or_pattern) => {
                                assert_eq!(or_pattern.patterns.len(), 2);
                            }
                            _ => panic!("Expected OR pattern"),
                        }
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_array_patterns() {
        let source = r#"
        func test() {
            match arr {
                [] -> print("empty")
                [x] -> print("single element")
                [1, 2, 3] -> print("specific array")
                [first, _, last] -> print("first and last")
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                match &func.body.statements[0] {
                    Statement::Match(match_stmt) => {
                        assert_eq!(match_stmt.arms.len(), 4);
                        
                        // Check empty array pattern
                        match &match_stmt.arms[0].pattern {
                            Pattern::Array(array_pattern) => {
                                assert_eq!(array_pattern.elements.len(), 0);
                            }
                            _ => panic!("Expected array pattern"),
                        }
                        
                        // Check single element pattern
                        match &match_stmt.arms[1].pattern {
                            Pattern::Array(array_pattern) => {
                                assert_eq!(array_pattern.elements.len(), 1);
                                match &array_pattern.elements[0] {
                                    Pattern::Identifier(name, _) => {
                                        assert_eq!(name, "x");
                                    }
                                    _ => panic!("Expected identifier pattern"),
                                }
                            }
                            _ => panic!("Expected array pattern"),
                        }
                        
                        // Check specific array pattern
                        match &match_stmt.arms[2].pattern {
                            Pattern::Array(array_pattern) => {
                                assert_eq!(array_pattern.elements.len(), 3);
                                match &array_pattern.elements[0] {
                                    Pattern::Literal(LiteralValue::Integer(1), _) => {},
                                    _ => panic!("Expected integer literal 1"),
                                }
                            }
                            _ => panic!("Expected array pattern"),
                        }
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_match_with_guards() {
        let source = r#"
        func test() {
            match x {
                n if n > 0 -> print("positive")
                n if n < 0 -> print("negative")
                _ -> print("zero")
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                match &func.body.statements[0] {
                    Statement::Match(match_stmt) => {
                        assert_eq!(match_stmt.arms.len(), 3);
                        
                        // Check first arm has guard
                        assert!(match_stmt.arms[0].guard.is_some());
                        match &match_stmt.arms[0].guard {
                            Some(Expression::Binary(binary_expr)) => {
                                assert_eq!(binary_expr.operator, BinaryOperator::Greater);
                            }
                            _ => panic!("Expected binary expression guard"),
                        }
                        
                        // Check second arm has guard
                        assert!(match_stmt.arms[1].guard.is_some());
                        
                        // Check third arm has no guard
                        assert!(match_stmt.arms[2].guard.is_none());
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_nested_match_expressions() {
        let source = r#"
        func test() {
            let result = match outer {
                1 -> match inner {
                    1 -> "one"
                    2 -> "two"
                    _ -> "other"
                }
                2 -> "nothing"
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                match &func.body.statements[0] {
                    Statement::VariableDecl(var_decl) => {
                        match &var_decl.initializer {
                            Some(Expression::Match(outer_match)) => {
                                assert_eq!(outer_match.arms.len(), 2);
                                
                                // Check that first arm contains nested match
                                match &outer_match.arms[0].expr {
                                    Expression::Match(inner_match) => {
                                        assert_eq!(inner_match.arms.len(), 3);
                                    }
                                    _ => panic!("Expected nested match expression"),
                                }
                            }
                            _ => panic!("Expected match expression"),
                        }
                    }
                    _ => panic!("Expected variable declaration"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }
}