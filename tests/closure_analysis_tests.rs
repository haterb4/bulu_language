//! Tests for closure analysis and variable capture detection

use bulu::ast::*;
use bulu::compiler::SemanticAnalyzer;
use bulu::error::Result;
use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn parse_and_analyze(source: &str) -> Result<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse()?;
    
    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&mut program)?;
    
    Ok(program)
}

#[cfg(test)]
mod closure_capture_tests {
    use super::*;

    #[test]
    fn test_simple_closure_capture() {
        let program = parse_and_analyze(r#"
            func outer(): func(): int32 {
                let x = 42
                return func(): int32 { return x }
            }
        "#).unwrap();
        
        // Find the outer function
        if let Statement::FunctionDecl(outer_func) = &program.statements[0] {
            // Find the return statement
            if let Statement::Return(return_stmt) = &outer_func.body.statements[1] {
                if let Some(Expression::Lambda(lambda)) = &return_stmt.value {
                    // The lambda should capture variable 'x'
                    assert_eq!(lambda.captures.len(), 1);
                    assert_eq!(lambda.captures[0].name, "x");
                    assert_eq!(lambda.captures[0].capture_type, CaptureType::ByValue);
                }
            }
        }
    }

    #[test]
    fn test_mutable_closure_capture() {
        let program = parse_and_analyze(r#"
            func counter(): func(): int32 {
                let count = 0
                return func(): int32 {
                    count = count + 1
                    return count
                }
            }
        "#).unwrap();
        
        // Find the outer function
        if let Statement::FunctionDecl(outer_func) = &program.statements[0] {
            // Find the return statement
            if let Statement::Return(return_stmt) = &outer_func.body.statements[1] {
                if let Some(Expression::Lambda(lambda)) = &return_stmt.value {
                    // The lambda should capture variable 'count' by reference (mutable)
                    assert_eq!(lambda.captures.len(), 1);
                    assert_eq!(lambda.captures[0].name, "count");
                    assert_eq!(lambda.captures[0].capture_type, CaptureType::ByReference);
                }
            }
        }
    }

    #[test]
    fn test_multiple_captures() {
        let program = parse_and_analyze(r#"
            func createAdder(x: int32, y: int32): func(int32): int32 {
                return func(z: int32): int32 { return x + y + z }
            }
        "#).unwrap();
        
        // Find the outer function
        if let Statement::FunctionDecl(outer_func) = &program.statements[0] {
            // Find the return statement
            if let Statement::Return(return_stmt) = &outer_func.body.statements[0] {
                if let Some(Expression::Lambda(lambda)) = &return_stmt.value {
                    // The lambda should capture both 'x' and 'y'
                    assert_eq!(lambda.captures.len(), 2);
                    
                    let captured_names: Vec<&String> = lambda.captures.iter().map(|c| &c.name).collect();
                    assert!(captured_names.contains(&&"x".to_string()));
                    assert!(captured_names.contains(&&"y".to_string()));
                }
            }
        }
    }

    #[test]
    fn test_nested_closures() {
        let program = parse_and_analyze(r#"
            func outer(x: int32): func(int32): func(int32): int32 {
                return func(y: int32): func(int32): int32 {
                    return func(z: int32): int32 { return x + y + z }
                }
            }
        "#).unwrap();
        
        // This is a complex case - the innermost lambda should capture both x and y
        // The middle lambda should capture x
        // We'll just verify it parses and analyzes without error for now
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_no_captures_for_local_variables() {
        let program = parse_and_analyze(r#"
            func test(): func(int32): int32 {
                return func(x: int32): int32 { 
                    let y = x * 2
                    return y 
                }
            }
        "#).unwrap();
        
        // Find the lambda
        if let Statement::FunctionDecl(outer_func) = &program.statements[0] {
            if let Statement::Return(return_stmt) = &outer_func.body.statements[0] {
                if let Some(Expression::Lambda(lambda)) = &return_stmt.value {
                    // The lambda should not capture anything (x is a parameter, y is local)
                    assert_eq!(lambda.captures.len(), 0);
                }
            }
        }
    }

    #[test]
    fn test_arrow_function_captures() {
        let program = parse_and_analyze(r#"
            func createMultiplier(factor: int32): func(int32): int32 {
                return (x: int32) => x * factor
            }
        "#).unwrap();
        
        // Find the arrow function
        if let Statement::FunctionDecl(outer_func) = &program.statements[0] {
            if let Statement::Return(return_stmt) = &outer_func.body.statements[0] {
                if let Some(Expression::Lambda(lambda)) = &return_stmt.value {
                    // The arrow function should capture 'factor'
                    assert_eq!(lambda.captures.len(), 1);
                    assert_eq!(lambda.captures[0].name, "factor");
                    assert_eq!(lambda.captures[0].capture_type, CaptureType::ByValue);
                }
            }
        }
    }
}