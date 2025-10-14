//! Integration tests for control flow parsing with real examples

use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::ast::*;
use bulu::error::BuluError;
use std::fs;

/// Helper function to parse source code from file
fn parse_file(path: &str) -> Result<Program, BuluError> {
    let source = fs::read_to_string(path)
        .expect(&format!("Failed to read file: {}", path));
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_control_flow_demo_parsing() {
        let program = parse_file("examples/control_flow_demo.bu").unwrap();
        
        // Should have one main function
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "main");
                assert!(func.params.is_empty());
                assert!(func.return_type.is_none());
                
                // The function body should contain multiple statements
                assert!(func.body.statements.len() > 10);
                
                // Check that we have various control flow statements
                let mut has_if = false;
                let mut has_while = false;
                let mut has_for = false;
                
                for stmt in &func.body.statements {
                    match stmt {
                        Statement::If(_) => has_if = true,
                        Statement::While(_) => has_while = true,
                        Statement::For(_) => has_for = true,
                        _ => {}
                    }
                }
                
                assert!(has_if, "Should contain if statements");
                assert!(has_while, "Should contain while statements");
                assert!(has_for, "Should contain for statements");
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_fibonacci_example() {
        let source = r#"
        func fibonacci(n: int32): int32 {
            if n <= 1 {
                return n
            }
            
            let a = 0
            let b = 1
            for i in 2...n {
                let temp = a + b
                a = b
                b = temp
            }
            return b
        }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "fibonacci");
                assert_eq!(func.params.len(), 1);
                assert_eq!(func.params[0].name, "n");
                assert_eq!(func.return_type, Some(Type::Int32));
                
                // Should have if statement, variable declarations, and for loop
                let statements = &func.body.statements;
                assert!(statements.len() >= 4);
                assert!(matches!(statements[0], Statement::If(_)));
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_nested_loops_example() {
        let source = r#"
        func printMatrix(size: int32) {
            for i in 0..<size {
                for j in 0..<size {
                    if i == j {
                        print("1")
                    } else {
                        print("0")
                    }
                }
                print("\n")
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "printMatrix");
                
                // Should have outer for loop
                assert_eq!(func.body.statements.len(), 1);
                
                if let Statement::For(outer_for) = &func.body.statements[0] {
                    // Outer for should have range expression
                    assert!(matches!(outer_for.iterable, Expression::Range(_)));
                    
                    // Should have inner for loop and print statement
                    assert_eq!(outer_for.body.statements.len(), 2);
                    assert!(matches!(outer_for.body.statements[0], Statement::For(_)));
                    assert!(matches!(outer_for.body.statements[1], Statement::Expression(_)));
                    
                    // Check inner for loop
                    if let Statement::For(inner_for) = &outer_for.body.statements[0] {
                        assert!(matches!(inner_for.iterable, Expression::Range(_)));
                        assert_eq!(inner_for.body.statements.len(), 1);
                        assert!(matches!(inner_for.body.statements[0], Statement::If(_)));
                    }
                } else {
                    panic!("Expected for statement");
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_break_continue_example() {
        let source = r#"
        func processNumbers(numbers: []int32) {
            for i, num in numbers {
                if num < 0 {
                    continue  // Skip negative numbers
                }
                if num > 100 {
                    break     // Stop at first number > 100
                }
                print("Processing:", num)
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "processNumbers");
                
                // Should have for loop
                assert_eq!(func.body.statements.len(), 1);
                
                if let Statement::For(for_stmt) = &func.body.statements[0] {
                    // Should have index and value variables
                    assert_eq!(for_stmt.index_variable, Some("i".to_string()));
                    assert_eq!(for_stmt.variable, "num");
                    
                    // Should have three statements in body
                    assert_eq!(for_stmt.body.statements.len(), 3);
                    
                    // First should be if with continue
                    if let Statement::If(if_stmt) = &for_stmt.body.statements[0] {
                        assert_eq!(if_stmt.then_branch.statements.len(), 1);
                        assert!(matches!(if_stmt.then_branch.statements[0], Statement::Continue(_)));
                    }
                    
                    // Second should be if with break
                    if let Statement::If(if_stmt) = &for_stmt.body.statements[1] {
                        assert_eq!(if_stmt.then_branch.statements.len(), 1);
                        assert!(matches!(if_stmt.then_branch.statements[0], Statement::Break(_)));
                    }
                } else {
                    panic!("Expected for statement");
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_range_expressions_in_context() {
        let source = r#"
        func testRanges() {
            // Exclusive range
            for i in 0..<10 {
                print(i)
            }
            
            // Inclusive range
            for j in 1...5 {
                print(j)
            }
            
            // Variable range
            let start = 5
            let end = 15
            for k in start..<end {
                print(k)
            }
            
            // Range with step
            for m in 0..<100 step 10 {
                print(m)
            }
            
            // Inclusive range with step
            for n in 1...20 step 3 {
                print(n)
            }
        }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "testRanges");
                
                // Should have 5 for loops and 2 variable declarations
                assert_eq!(func.body.statements.len(), 7);
                
                // Check first for loop (exclusive range)
                if let Statement::For(for1) = &func.body.statements[0] {
                    if let Expression::Range(range) = &for1.iterable {
                        assert!(!range.inclusive);
                    } else {
                        panic!("Expected range expression");
                    }
                }
                
                // Check second for loop (inclusive range)
                if let Statement::For(for2) = &func.body.statements[1] {
                    if let Expression::Range(range) = &for2.iterable {
                        assert!(range.inclusive);
                    } else {
                        panic!("Expected range expression");
                    }
                }
                
                // Check third for loop (variable range)
                if let Statement::For(for3) = &func.body.statements[4] {
                    if let Expression::Range(range) = &for3.iterable {
                        assert!(!range.inclusive);
                        assert!(matches!(*range.start, Expression::Identifier(_)));
                        assert!(matches!(*range.end, Expression::Identifier(_)));
                        assert!(range.step.is_none());
                    } else {
                        panic!("Expected range expression");
                    }
                }
                
                // Check fourth for loop (range with step)
                if let Statement::For(for4) = &func.body.statements[5] {
                    if let Expression::Range(range) = &for4.iterable {
                        assert!(!range.inclusive);
                        assert!(range.step.is_some());
                    } else {
                        panic!("Expected range expression");
                    }
                }
                
                // Check fifth for loop (inclusive range with step)
                if let Statement::For(for5) = &func.body.statements[6] {
                    if let Expression::Range(range) = &for5.iterable {
                        assert!(range.inclusive);
                        assert!(range.step.is_some());
                    } else {
                        panic!("Expected range expression");
                    }
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }
}