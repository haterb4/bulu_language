//! Step syntax tests for the Bulu language

use bulu::lexer::{Lexer, TokenType};
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
mod step_syntax_tests {
    use super::*;

    #[test]
    fn test_step_keyword_lexing() {
        let mut lexer = Lexer::new("step");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2); // step + EOF
        assert_eq!(tokens[0].token_type, TokenType::Step);
        assert_eq!(tokens[0].lexeme, "step");
    }

    #[test]
    fn test_exclusive_range_with_literal_step() {
        let expr = parse_expression("0..<100 step 5").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                
                // Check start
                if let Expression::Literal(start_lit) = *range.start {
                    assert_eq!(start_lit.value, LiteralValue::Integer(0));
                } else {
                    panic!("Expected integer literal for start");
                }
                
                // Check end
                if let Expression::Literal(end_lit) = *range.end {
                    assert_eq!(end_lit.value, LiteralValue::Integer(100));
                } else {
                    panic!("Expected integer literal for end");
                }
                
                // Check step
                assert!(range.step.is_some());
                if let Some(step_expr) = range.step {
                    if let Expression::Literal(step_lit) = *step_expr {
                        assert_eq!(step_lit.value, LiteralValue::Integer(5));
                    } else {
                        panic!("Expected integer literal for step");
                    }
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_inclusive_range_with_literal_step() {
        let expr = parse_expression("1...20 step 3").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(range.inclusive);
                
                // Check start
                if let Expression::Literal(start_lit) = *range.start {
                    assert_eq!(start_lit.value, LiteralValue::Integer(1));
                } else {
                    panic!("Expected integer literal for start");
                }
                
                // Check end
                if let Expression::Literal(end_lit) = *range.end {
                    assert_eq!(end_lit.value, LiteralValue::Integer(20));
                } else {
                    panic!("Expected integer literal for end");
                }
                
                // Check step
                assert!(range.step.is_some());
                if let Some(step_expr) = range.step {
                    if let Expression::Literal(step_lit) = *step_expr {
                        assert_eq!(step_lit.value, LiteralValue::Integer(3));
                    } else {
                        panic!("Expected integer literal for step");
                    }
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_range_with_variable_step() {
        let expr = parse_expression("start..<end step increment").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                assert!(matches!(*range.start, Expression::Identifier(_)));
                assert!(matches!(*range.end, Expression::Identifier(_)));
                
                // Check step
                assert!(range.step.is_some());
                if let Some(step_expr) = range.step {
                    assert!(matches!(*step_expr, Expression::Identifier(_)));
                } else {
                    panic!("Expected step expression");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_range_with_expression_step() {
        let expr = parse_expression("0..<100 step (x * 2)").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                
                // Check step is a parenthesized expression
                assert!(range.step.is_some());
                if let Some(step_expr) = range.step {
                    assert!(matches!(*step_expr, Expression::Parenthesized(_)));
                } else {
                    panic!("Expected step expression");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_for_loop_with_step_syntax() {
        let stmt = parse_statement("for i in 0..<100 step 10 { print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "i");
                assert!(for_stmt.index_variable.is_none());
                
                // Check that iterable is a range with step
                if let Expression::Range(range) = for_stmt.iterable {
                    assert!(!range.inclusive);
                    assert!(range.step.is_some());
                    
                    if let Some(step_expr) = range.step {
                        if let Expression::Literal(step_lit) = *step_expr {
                            assert_eq!(step_lit.value, LiteralValue::Integer(10));
                        } else {
                            panic!("Expected integer literal for step");
                        }
                    }
                } else {
                    panic!("Expected range expression");
                }
                
                // Check body
                assert_eq!(for_stmt.body.statements.len(), 1);
                assert!(matches!(for_stmt.body.statements[0], Statement::Expression(_)));
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_nested_for_loops_with_step() {
        let stmt = parse_statement("for i in 0..<10 step 2 {\nfor j in 0...5 step 1 {\nprint(i, j)\n}\n}").unwrap();
        
        match stmt {
            Statement::For(outer_for) => {
                // Check outer loop
                if let Expression::Range(outer_range) = outer_for.iterable {
                    assert!(!outer_range.inclusive);
                    assert!(outer_range.step.is_some());
                } else {
                    panic!("Expected range expression for outer loop");
                }
                
                // Check inner loop
                assert_eq!(outer_for.body.statements.len(), 1);
                if let Statement::For(inner_for) = &outer_for.body.statements[0] {
                    if let Expression::Range(inner_range) = &inner_for.iterable {
                        assert!(inner_range.inclusive);
                        assert!(inner_range.step.is_some());
                    } else {
                        panic!("Expected range expression for inner loop");
                    }
                } else {
                    panic!("Expected inner for loop");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_range_without_step_still_works() {
        let expr = parse_expression("0..<10").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                assert!(range.step.is_none());
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_step_syntax_in_complex_program() {
        let source = r#"
        func printEvens(max: int32) {
            for i in 0..<max step 2 {
                print("Even:", i)
            }
        }
        
        func printOdds(start: int32, end: int32) {
            for i in start...end step 2 {
                if i % 2 == 1 {
                    print("Odd:", i)
                }
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 2);
        
        // Check first function
        if let Statement::FunctionDecl(func1) = &program.statements[0] {
            assert_eq!(func1.name, "printEvens");
            assert_eq!(func1.body.statements.len(), 1);
            
            if let Statement::For(for_stmt) = &func1.body.statements[0] {
                if let Expression::Range(range) = &for_stmt.iterable {
                    assert!(!range.inclusive);
                    assert!(range.step.is_some());
                } else {
                    panic!("Expected range expression");
                }
            } else {
                panic!("Expected for statement");
            }
        } else {
            panic!("Expected function declaration");
        }
        
        // Check second function
        if let Statement::FunctionDecl(func2) = &program.statements[1] {
            assert_eq!(func2.name, "printOdds");
            assert_eq!(func2.body.statements.len(), 1);
            
            if let Statement::For(for_stmt) = &func2.body.statements[0] {
                if let Expression::Range(range) = &for_stmt.iterable {
                    assert!(range.inclusive);
                    assert!(range.step.is_some());
                } else {
                    panic!("Expected range expression");
                }
            } else {
                panic!("Expected for statement");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_step_with_negative_values() {
        let expr = parse_expression("10..<0 step -1").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                
                // Check step is negative
                assert!(range.step.is_some());
                if let Some(step_expr) = range.step {
                    if let Expression::Unary(unary) = *step_expr {
                        assert_eq!(unary.operator, UnaryOperator::Minus);
                        if let Expression::Literal(lit) = *unary.operand {
                            assert_eq!(lit.value, LiteralValue::Integer(1));
                        } else {
                            panic!("Expected integer literal in unary expression");
                        }
                    } else {
                        panic!("Expected unary expression for negative step");
                    }
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_step_with_float_values() {
        let expr = parse_expression("0.0..<10.0 step 0.5").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                
                // Check step is float
                assert!(range.step.is_some());
                if let Some(step_expr) = range.step {
                    if let Expression::Literal(step_lit) = *step_expr {
                        assert_eq!(step_lit.value, LiteralValue::Float(0.5));
                    } else {
                        panic!("Expected float literal for step");
                    }
                }
            }
            _ => panic!("Expected range expression"),
        }
    }
}