//! Control flow statement tests for the Bulu language

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

#[cfg(test)]
mod if_statement_tests {
    use super::*;

    #[test]
    fn test_simple_if_statement() {
        let stmt = parse_statement("if x > 0 { print(x) }").unwrap();
        
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
                
                if let Some(else_branch) = if_stmt.else_branch {
                    assert!(matches!(*else_branch, Statement::Block(_)));
                }
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_if_else_if_statement() {
        let stmt = parse_statement("if x > 0 { print(\"positive\") } else if x < 0 { print(\"negative\") } else { print(\"zero\") }").unwrap();
        
        match stmt {
            Statement::If(if_stmt) => {
                assert!(matches!(if_stmt.condition, Expression::Binary(_)));
                assert_eq!(if_stmt.then_branch.statements.len(), 1);
                assert!(if_stmt.else_branch.is_some());
                
                if let Some(else_branch) = if_stmt.else_branch {
                    // The else if should be parsed as another if statement
                    assert!(matches!(*else_branch, Statement::If(_)));
                }
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_if_with_complex_condition() {
        let stmt = parse_statement("if x > 0 and y < 10 { doSomething() }").unwrap();
        
        match stmt {
            Statement::If(if_stmt) => {
                if let Expression::Binary(bin) = if_stmt.condition {
                    assert_eq!(bin.operator, BinaryOperator::And);
                } else {
                    panic!("Expected binary expression for condition");
                }
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_nested_if_statements() {
        let stmt = parse_statement("if x > 0 { if y > 0 { print(\"both positive\") } }").unwrap();
        
        match stmt {
            Statement::If(if_stmt) => {
                assert_eq!(if_stmt.then_branch.statements.len(), 1);
                assert!(matches!(if_stmt.then_branch.statements[0], Statement::If(_)));
            }
            _ => panic!("Expected if statement"),
        }
    }
}

#[cfg(test)]
mod while_statement_tests {
    use super::*;

    #[test]
    fn test_simple_while_loop() {
        let stmt = parse_statement("while x < 10 { x = x + 1 }").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                assert!(matches!(while_stmt.condition, Expression::Binary(_)));
                assert_eq!(while_stmt.body.statements.len(), 1);
                assert!(matches!(while_stmt.body.statements[0], Statement::Expression(_)));
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_while_with_break() {
        let stmt = parse_statement("while true { if x > 10 { break } x = x + 1 }").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                if let Expression::Literal(lit) = while_stmt.condition {
                    assert_eq!(lit.value, LiteralValue::Boolean(true));
                } else {
                    panic!("Expected boolean literal condition");
                }
                assert_eq!(while_stmt.body.statements.len(), 2);
                assert!(matches!(while_stmt.body.statements[0], Statement::If(_)));
                assert!(matches!(while_stmt.body.statements[1], Statement::Expression(_)));
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_while_with_continue() {
        let stmt = parse_statement("while x < 10 {\nif x % 2 == 0 { continue }\nprint(x)\nx = x + 1\n}").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                assert_eq!(while_stmt.body.statements.len(), 3);
                assert!(matches!(while_stmt.body.statements[0], Statement::If(_)));
                assert!(matches!(while_stmt.body.statements[1], Statement::Expression(_)));
                assert!(matches!(while_stmt.body.statements[2], Statement::Expression(_)));
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_nested_while_loops() {
        let stmt = parse_statement("while i < 10 {\nwhile j < 10 {\nprint(i, j)\nj = j + 1\n}\ni = i + 1\n}").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                assert_eq!(while_stmt.body.statements.len(), 2);
                assert!(matches!(while_stmt.body.statements[0], Statement::While(_)));
                assert!(matches!(while_stmt.body.statements[1], Statement::Expression(_)));
            }
            _ => panic!("Expected while statement"),
        }
    }
}

#[cfg(test)]
mod for_statement_tests {
    use super::*;

    #[test]
    fn test_for_in_array() {
        let stmt = parse_statement("for item in items { print(item) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "item");
                assert!(for_stmt.index_variable.is_none());
                assert!(matches!(for_stmt.iterable, Expression::Identifier(_)));
                assert_eq!(for_stmt.body.statements.len(), 1);
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_with_index() {
        let stmt = parse_statement("for i, item in items { print(i, item) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "item");
                assert_eq!(for_stmt.index_variable, Some("i".to_string()));
                assert!(matches!(for_stmt.iterable, Expression::Identifier(_)));
                assert_eq!(for_stmt.body.statements.len(), 1);
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_range_exclusive() {
        let stmt = parse_statement("for i in 0..<10 { print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "i");
                assert!(for_stmt.index_variable.is_none());
                
                if let Expression::Range(range) = for_stmt.iterable {
                    assert!(!range.inclusive); // 0..<10 is exclusive
                    assert!(matches!(*range.start, Expression::Literal(_)));
                    assert!(matches!(*range.end, Expression::Literal(_)));
                } else {
                    panic!("Expected range expression");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_range_inclusive() {
        let stmt = parse_statement("for i in 0...10 { print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "i");
                assert!(for_stmt.index_variable.is_none());
                
                if let Expression::Range(range) = for_stmt.iterable {
                    assert!(range.inclusive); // 0...10 is inclusive
                    assert!(matches!(*range.start, Expression::Literal(_)));
                    assert!(matches!(*range.end, Expression::Literal(_)));
                } else {
                    panic!("Expected range expression");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_variable_range() {
        let stmt = parse_statement("for i in start..<end { print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                if let Expression::Range(range) = for_stmt.iterable {
                    assert!(!range.inclusive);
                    assert!(matches!(*range.start, Expression::Identifier(_)));
                    assert!(matches!(*range.end, Expression::Identifier(_)));
                } else {
                    panic!("Expected range expression");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_array_literal() {
        let stmt = parse_statement("for item in [1, 2, 3] { print(item) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "item");
                assert!(matches!(for_stmt.iterable, Expression::Array(_)));
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_nested_for_loops() {
        let stmt = parse_statement("for i in 0..<10 { for j in 0..<10 { print(i, j) } }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.body.statements.len(), 1);
                assert!(matches!(for_stmt.body.statements[0], Statement::For(_)));
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_with_break_and_continue() {
        let stmt = parse_statement("for i in 0..<10 { if i == 5 { break } if i % 2 == 0 { continue } print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.body.statements.len(), 3);
                assert!(matches!(for_stmt.body.statements[0], Statement::If(_)));
                assert!(matches!(for_stmt.body.statements[1], Statement::If(_)));
                assert!(matches!(for_stmt.body.statements[2], Statement::Expression(_)));
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_range_with_step() {
        let stmt = parse_statement("for i in 0..<100 step 5 { print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "i");
                assert!(for_stmt.index_variable.is_none());
                
                if let Expression::Range(range) = for_stmt.iterable {
                    assert!(!range.inclusive); // 0..<100 is exclusive
                    assert!(matches!(*range.start, Expression::Literal(_)));
                    assert!(matches!(*range.end, Expression::Literal(_)));
                    
                    // Check step
                    if let Some(step_expr) = range.step {
                        if let Expression::Literal(step_lit) = *step_expr {
                            assert_eq!(step_lit.value, LiteralValue::Integer(5));
                        } else {
                            panic!("Expected integer literal for step");
                        }
                    } else {
                        panic!("Expected step expression");
                    }
                } else {
                    panic!("Expected range expression");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_in_inclusive_range_with_step() {
        let stmt = parse_statement("for i in 1...20 step 3 { print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "i");
                assert!(for_stmt.index_variable.is_none());
                
                if let Expression::Range(range) = for_stmt.iterable {
                    assert!(range.inclusive); // 1...20 is inclusive
                    assert!(matches!(*range.start, Expression::Literal(_)));
                    assert!(matches!(*range.end, Expression::Literal(_)));
                    
                    // Check step
                    if let Some(step_expr) = range.step {
                        if let Expression::Literal(step_lit) = *step_expr {
                            assert_eq!(step_lit.value, LiteralValue::Integer(3));
                        } else {
                            panic!("Expected integer literal for step");
                        }
                    } else {
                        panic!("Expected step expression");
                    }
                } else {
                    panic!("Expected range expression");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }
}

#[cfg(test)]
mod range_expression_tests {
    use super::*;

    fn parse_expression(source: &str) -> Result<Expression, BuluError> {
        let stmt = parse_statement(source)?;
        match stmt {
            Statement::Expression(expr_stmt) => Ok(expr_stmt.expr),
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_exclusive_range() {
        let expr = parse_expression("0..<10").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                if let Expression::Literal(start_lit) = *range.start {
                    assert_eq!(start_lit.value, LiteralValue::Integer(0));
                } else {
                    panic!("Expected integer literal for start");
                }
                if let Expression::Literal(end_lit) = *range.end {
                    assert_eq!(end_lit.value, LiteralValue::Integer(10));
                } else {
                    panic!("Expected integer literal for end");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_inclusive_range() {
        let expr = parse_expression("1...5").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(range.inclusive);
                if let Expression::Literal(start_lit) = *range.start {
                    assert_eq!(start_lit.value, LiteralValue::Integer(1));
                } else {
                    panic!("Expected integer literal for start");
                }
                if let Expression::Literal(end_lit) = *range.end {
                    assert_eq!(end_lit.value, LiteralValue::Integer(5));
                } else {
                    panic!("Expected integer literal for end");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_variable_range() {
        let expr = parse_expression("start...end").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(range.inclusive);
                assert!(matches!(*range.start, Expression::Identifier(_)));
                assert!(matches!(*range.end, Expression::Identifier(_)));
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_expression_range() {
        let expr = parse_expression("(x + 1)..<(y * 2)").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                assert!(matches!(*range.start, Expression::Parenthesized(_)));
                assert!(matches!(*range.end, Expression::Parenthesized(_)));
                assert!(range.step.is_none());
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_exclusive_range_with_step() {
        let expr = parse_expression("0..<100 step 5").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                if let Expression::Literal(start_lit) = *range.start {
                    assert_eq!(start_lit.value, LiteralValue::Integer(0));
                } else {
                    panic!("Expected integer literal for start");
                }
                if let Expression::Literal(end_lit) = *range.end {
                    assert_eq!(end_lit.value, LiteralValue::Integer(100));
                } else {
                    panic!("Expected integer literal for end");
                }
                if let Some(step_expr) = range.step {
                    if let Expression::Literal(step_lit) = *step_expr {
                        assert_eq!(step_lit.value, LiteralValue::Integer(5));
                    } else {
                        panic!("Expected integer literal for step");
                    }
                } else {
                    panic!("Expected step expression");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_inclusive_range_with_step() {
        let expr = parse_expression("1...10 step 2").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(range.inclusive);
                if let Expression::Literal(start_lit) = *range.start {
                    assert_eq!(start_lit.value, LiteralValue::Integer(1));
                } else {
                    panic!("Expected integer literal for start");
                }
                if let Expression::Literal(end_lit) = *range.end {
                    assert_eq!(end_lit.value, LiteralValue::Integer(10));
                } else {
                    panic!("Expected integer literal for end");
                }
                if let Some(step_expr) = range.step {
                    if let Expression::Literal(step_lit) = *step_expr {
                        assert_eq!(step_lit.value, LiteralValue::Integer(2));
                    } else {
                        panic!("Expected integer literal for step");
                    }
                } else {
                    panic!("Expected step expression");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_variable_range_with_step() {
        let expr = parse_expression("start..<end step increment").unwrap();
        
        match expr {
            Expression::Range(range) => {
                assert!(!range.inclusive);
                assert!(matches!(*range.start, Expression::Identifier(_)));
                assert!(matches!(*range.end, Expression::Identifier(_)));
                if let Some(step_expr) = range.step {
                    assert!(matches!(*step_expr, Expression::Identifier(_)));
                } else {
                    panic!("Expected step expression");
                }
            }
            _ => panic!("Expected range expression"),
        }
    }
}

#[cfg(test)]
mod break_continue_tests {
    use super::*;

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
    fn test_break_in_while_loop() {
        let stmt = parse_statement("while true { if condition { break } doSomething() }").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                assert_eq!(while_stmt.body.statements.len(), 2);
                
                if let Statement::If(if_stmt) = &while_stmt.body.statements[0] {
                    assert_eq!(if_stmt.then_branch.statements.len(), 1);
                    assert!(matches!(if_stmt.then_branch.statements[0], Statement::Break(_)));
                } else {
                    panic!("Expected if statement containing break");
                }
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_continue_in_for_loop() {
        let stmt = parse_statement("for i in 0..<10 { if i % 2 == 0 { continue } print(i) }").unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.body.statements.len(), 2);
                
                if let Statement::If(if_stmt) = &for_stmt.body.statements[0] {
                    assert_eq!(if_stmt.then_branch.statements.len(), 1);
                    assert!(matches!(if_stmt.then_branch.statements[0], Statement::Continue(_)));
                } else {
                    panic!("Expected if statement containing continue");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }
}

#[cfg(test)]
mod complex_control_flow_tests {
    use super::*;

    #[test]
    fn test_fibonacci_loop() {
        let source = r#"
        {
            let a = 0
            let b = 1
            for i in 0..<10 {
                print(a)
                let temp = a
                a = b
                b = temp + b
            }
        }
        "#;
        
        let stmt = parse_statement(source).unwrap();
        
        match stmt {
            Statement::Block(block) => {
                assert_eq!(block.statements.len(), 3); // let a, let b, for loop
                assert!(matches!(block.statements[0], Statement::VariableDecl(_)));
                assert!(matches!(block.statements[1], Statement::VariableDecl(_)));
                assert!(matches!(block.statements[2], Statement::For(_)));
            }
            _ => panic!("Expected block statement"),
        }
    }

    #[test]
    fn test_nested_loops_with_break() {
        let source = r#"
        for i in 0..<10 {
            for j in 0..<10 {
                if i * j > 50 {
                    break
                }
                print(i, j)
            }
        }
        "#;
        
        let stmt = parse_statement(source).unwrap();
        
        match stmt {
            Statement::For(outer_for) => {
                assert_eq!(outer_for.body.statements.len(), 1);
                
                if let Statement::For(inner_for) = &outer_for.body.statements[0] {
                    assert_eq!(inner_for.body.statements.len(), 2);
                    assert!(matches!(inner_for.body.statements[0], Statement::If(_)));
                    assert!(matches!(inner_for.body.statements[1], Statement::Expression(_)));
                } else {
                    panic!("Expected nested for loop");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_while_with_complex_condition() {
        let stmt = parse_statement("while x > 0 and y < 100 and not done {\nprocessData()\nx = x - 1\n}").unwrap();
        
        match stmt {
            Statement::While(while_stmt) => {
                // The condition should be a complex binary expression with 'and' operators
                assert!(matches!(while_stmt.condition, Expression::Binary(_)));
                assert_eq!(while_stmt.body.statements.len(), 2);
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_for_loop_with_array_and_map_operations() {
        let source = r#"
        for i, item in items {
            if item.isValid {
                results[i] = process(item)
            } else {
                continue
            }
        }
        "#;
        
        let stmt = parse_statement(source).unwrap();
        
        match stmt {
            Statement::For(for_stmt) => {
                assert_eq!(for_stmt.variable, "item");
                assert_eq!(for_stmt.index_variable, Some("i".to_string()));
                assert_eq!(for_stmt.body.statements.len(), 1);
                assert!(matches!(for_stmt.body.statements[0], Statement::If(_)));
            }
            _ => panic!("Expected for statement"),
        }
    }
}