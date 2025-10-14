//! Tests for import/export functionality

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::runtime::ast_interpreter::AstInterpreter;
    use crate::types::primitive::RuntimeValue;

    #[test]
    fn test_export_parsing() {
        let source = r#"
            export const PI = 3.14159
            export let message = "Hello"
            export func greet(name: string): string {
                return "Hello, " + name
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 3);
        
        // All statements should be exported declarations
        for (i, statement) in ast.statements.iter().enumerate() {
            match statement {
                crate::ast::nodes::Statement::VariableDecl(decl) => {
                    assert!(decl.is_exported, "Variable declaration at index {} should be exported", i);
                },
                crate::ast::nodes::Statement::FunctionDecl(decl) => {
                    assert!(decl.is_exported, "Function declaration at index {} should be exported", i);
                },
                crate::ast::nodes::Statement::Export(_) => {},
                _ => panic!("Expected exported declaration at index {}, got: {:?}", i, statement),
            }
        }
    }

    #[test]
    fn test_import_parsing() {
        let source = r#"
            import "std.io"
            import "test_module.bu" as test
            import { print, println } from "std.io"
            import { greet as hello } from "test_module.bu"
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 4);
        
        // All statements should be import statements
        for statement in &ast.statements {
            match statement {
                crate::ast::nodes::Statement::Import(_) => {},
                _ => panic!("Expected import statement"),
            }
        }
    }

    #[test]
    fn test_reexport_parsing() {
        let source = r#"export { PI, greet } from "test_module.bu"
export { Point as Coordinate } from "geometry.bu"
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 2);
        
        // All statements should be export statements containing imports
        for statement in &ast.statements {
            match statement {
                crate::ast::nodes::Statement::Export(export_stmt) => {
                    match export_stmt.item.as_ref() {
                        crate::ast::nodes::Statement::Import(_) => {},
                        _ => panic!("Expected import statement inside export"),
                    }
                },
                _ => panic!("Expected export statement"),
            }
        }
    }

    #[test]
    fn test_export_execution() {
        let source = r#"
            export const PI = 3.14159
            export let message = "Hello, World!"
            let private_var = "This is private"
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let mut interpreter = AstInterpreter::new();
        interpreter.execute_program(&ast).unwrap();

        // Exported symbols should be accessible
        assert!(interpreter.is_symbol_accessible("PI"));
        assert!(interpreter.is_symbol_accessible("message"));
        
        // Private symbols should not be accessible
        assert!(!interpreter.is_symbol_accessible("private_var"));
        
        // Check values in global environment
        let globals = interpreter.globals();
        if let Some(pi_value) = globals.get("PI") {
            match pi_value {
                RuntimeValue::Float64(f) => assert!((f - 3.14159).abs() < 0.0001),
                _ => panic!("Expected float value for PI"),
            }
        } else {
            panic!("PI not found in globals");
        }
        
        if let Some(msg_value) = globals.get("message") {
            match msg_value {
                RuntimeValue::String(s) => assert_eq!(s, "Hello, World!"),
                _ => panic!("Expected string value for message"),
            }
        } else {
            panic!("message not found in globals");
        }
    }

    #[test]
    fn test_std_import_execution() {
        let source = r#"
            import "std.io" as io
            import { print } from "std.io"
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let mut interpreter = AstInterpreter::new();
        interpreter.execute_program(&ast).unwrap();

        // Should have imported symbols
        let env = interpreter.environment();
        assert!(env.contains("io"));
        assert!(env.contains("print"));
    }

    #[test]
    fn test_selective_import() {
        let source = r#"
            import { print, println } from "std.io"
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let mut interpreter = AstInterpreter::new();
        interpreter.execute_program(&ast).unwrap();

        // Should have only imported specific symbols
        let env = interpreter.environment();
        assert!(env.contains("print"));
        assert!(env.contains("println"));
        // Should not have imported other symbols like "input"
        assert!(!env.contains("input"));
    }

    #[test]
    fn test_import_with_alias() {
        let source = r#"
            import "std.math" as mathematics
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let mut interpreter = AstInterpreter::new();
        interpreter.execute_program(&ast).unwrap();

        // Should have imported with alias
        let env = interpreter.environment();
        assert!(env.contains("mathematics"));
        assert!(!env.contains("math")); // Original name should not be available
    }

    #[test]
    fn test_function_export() {
        let source = r#"
            export func greet(name: string): string {
                return "Hello, " + name
            }
            
            func private_func() {
                return "private"
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let mut interpreter = AstInterpreter::new();
        interpreter.execute_program(&ast).unwrap();

        // Exported function should be accessible
        assert!(interpreter.is_symbol_accessible("greet"));
        
        // Private function should not be accessible
        assert!(!interpreter.is_symbol_accessible("private_func"));
    }

    #[test]
    fn test_struct_export() {
        let source = r#"
            export struct Point {
                x: float64
                y: float64
            }
            
            struct PrivateStruct {
                data: string
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let mut interpreter = AstInterpreter::new();
        interpreter.execute_program(&ast).unwrap();

        // Exported struct should be accessible
        assert!(interpreter.is_symbol_accessible("Point"));
        
        // Private struct should not be accessible
        assert!(!interpreter.is_symbol_accessible("PrivateStruct"));
    }
}