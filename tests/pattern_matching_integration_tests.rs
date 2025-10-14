//! Integration tests for pattern matching with real examples

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

/// Helper function to parse source code from string
fn parse_source(source: &str) -> Result<Program, BuluError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_pattern_matching_demo_parsing() {
        // Just test that the file parses without errors
        let result = parse_file("examples/pattern_matching_demo.bu");
        if let Err(e) = &result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Pattern matching demo should parse successfully");
        
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "main");
                assert!(func.params.is_empty());
                assert!(func.return_type.is_none());
                
                // The function body should contain multiple statements
                assert!(func.body.statements.len() > 10);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_comprehensive_pattern_matching() {
        let source = r#"
        func processData(data: any) {
            // Value matching
            match data {
                0 -> print("zero")
                1 | 2 | 3 -> print("small number")
                42 -> print("answer")
                _ -> print("other")
            }
            
            // Range matching
            let age = 25
            match age {
                0...17 -> print("minor")
                18..<65 -> print("adult")
                _ -> print("senior")
            }
            
            // Array matching
            let arr = [1, 2, 3]
            match arr {
                [] -> print("empty")
                [x] -> print("single")
                [1, 2, 3] -> print("specific")
                [first, _, last] -> print("first and last")
            }
            
            // Match with guards
            let num = 10
            match num {
                n if n > 0 -> print("positive")
                n if n < 0 -> print("negative")
                _ -> print("zero")
            }
            
            // Match as expression
            let category = match age {
                0...12 -> "child"
                13...19 -> "teen"
                _ -> "adult"
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "processData");
                assert_eq!(func.params.len(), 1);
                
                // Count different types of match constructs
                let mut match_statements = 0;
                let mut match_expressions = 0;
                let mut struct_decls = 0;
                
                fn count_matches_in_stmt(stmt: &Statement, match_stmts: &mut usize, match_exprs: &mut usize, struct_decls: &mut usize) {
                    match stmt {
                        Statement::Match(_) => *match_stmts += 1,
                        Statement::StructDecl(_) => *struct_decls += 1,
                        Statement::VariableDecl(var_decl) => {
                            if let Some(Expression::Match(_)) = &var_decl.initializer {
                                *match_exprs += 1;
                            }
                        }
                        Statement::Block(block) => {
                            for s in &block.statements {
                                count_matches_in_stmt(s, match_stmts, match_exprs, struct_decls);
                            }
                        }
                        _ => {}
                    }
                }
                
                for stmt in &func.body.statements {
                    count_matches_in_stmt(stmt, &mut match_statements, &mut match_expressions, &mut struct_decls);
                }
                
                assert_eq!(match_statements, 4, "Should have 4 match statements");
                assert_eq!(match_expressions, 1, "Should have 1 match expression");
                assert_eq!(struct_decls, 0, "Should have 0 struct declarations");
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_pattern_matching_with_control_flow() {
        let source = r#"
        func fibonacci(n: int32): int32 {
            return match n {
                0 -> 0
                1 -> 1
                _ -> fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        func factorial(n: int32): int32 {
            return match n {
                0 | 1 -> 1
                _ -> n * factorial(n - 1)
            }
        }
        
        func processArray(arr: []int32) {
            for item in arr {
                match item {
                    n if n < 0 -> {
                        print("Negative: " + n)
                        continue
                    }
                    0 -> {
                        print("Zero found")
                        break
                    }
                    n if n > 100 -> {
                        print("Large number: " + n)
                    }
                    _ -> print("Normal: " + item)
                }
            }
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 3);
        
        // Check fibonacci function
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "fibonacci");
                assert_eq!(func.body.statements.len(), 1);
                
                match &func.body.statements[0] {
                    Statement::Return(return_stmt) => {
                        match &return_stmt.value {
                            Some(Expression::Match(match_expr)) => {
                                assert_eq!(match_expr.arms.len(), 3);
                                
                                // Check recursive call in third arm
                                match &match_expr.arms[2].expr {
                                    Expression::Binary(_) => {}, // Should be addition of recursive calls
                                    _ => panic!("Expected binary expression"),
                                }
                            }
                            _ => panic!("Expected match expression in return"),
                        }
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
        
        // Check factorial function with OR pattern
        match &program.statements[1] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "factorial");
                
                match &func.body.statements[0] {
                    Statement::Return(return_stmt) => {
                        match &return_stmt.value {
                            Some(Expression::Match(match_expr)) => {
                                assert_eq!(match_expr.arms.len(), 2);
                                
                                // Check OR pattern in first arm
                                match &match_expr.arms[0].pattern {
                                    Pattern::Or(or_pattern) => {
                                        assert_eq!(or_pattern.patterns.len(), 2);
                                    }
                                    _ => panic!("Expected OR pattern"),
                                }
                            }
                            _ => panic!("Expected match expression in return"),
                        }
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
        
        // Check processArray function with match inside for loop
        match &program.statements[2] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "processArray");
                
                match &func.body.statements[0] {
                    Statement::For(for_stmt) => {
                        assert_eq!(for_stmt.body.statements.len(), 1);
                        
                        match &for_stmt.body.statements[0] {
                            Statement::Match(match_stmt) => {
                                assert_eq!(match_stmt.arms.len(), 4);
                                
                                // Check that some arms have guards
                                assert!(match_stmt.arms[0].guard.is_some());
                                assert!(match_stmt.arms[2].guard.is_some());
                                
                                // Check that some arms have block bodies with control flow
                                match &match_stmt.arms[0].body {
                                    Statement::Block(block) => {
                                        assert_eq!(block.statements.len(), 2);
                                        assert!(matches!(block.statements[1], Statement::Continue(_)));
                                    }
                                    _ => panic!("Expected block statement"),
                                }
                                
                                match &match_stmt.arms[1].body {
                                    Statement::Block(block) => {
                                        assert_eq!(block.statements.len(), 2);
                                        assert!(matches!(block.statements[1], Statement::Break(_)));
                                    }
                                    _ => panic!("Expected block statement"),
                                }
                            }
                            _ => panic!("Expected match statement"),
                        }
                    }
                    _ => panic!("Expected for statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_complex_nested_patterns() {
        let source = r#"
        func processNestedData() {
            let value = 42
            
            // Nested match expressions
            let result = match value {
                1 -> match value {
                    1 -> "one-one"
                    _ -> "one-other"
                }
                2 -> "two"
                _ -> "other"
            }
            
            print(result)
        }
        "#;
        
        let program = parse_source(source).unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::FunctionDecl(func) => {
                assert_eq!(func.name, "processNestedData");
                
                // Should have variable declarations and print statement
                assert_eq!(func.body.statements.len(), 3);
                
                // Check that we have a nested match expression
                match &func.body.statements[1] {
                    Statement::VariableDecl(var_decl) => {
                        match &var_decl.initializer {
                            Some(Expression::Match(outer_match)) => {
                                assert_eq!(outer_match.arms.len(), 3);
                                
                                // Check that first arm contains nested match
                                match &outer_match.arms[0].expr {
                                    Expression::Match(inner_match) => {
                                        assert_eq!(inner_match.arms.len(), 2);
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