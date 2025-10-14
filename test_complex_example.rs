use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::AstInterpreter;
use std::fs;

fn main() {
    println!("Testing complex Bulu example...");
    
    // Read the main.bu file
    let main_path = "complex-example/src/main.bu";
    let source = match fs::read_to_string(main_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read {}: {}", main_path, e);
            return;
        }
    };
    
    println!("Source code:");
    println!("{}", source);
    println!("\n{}\n", "=".repeat(50));
    
    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Lexing failed: {:?}", e);
            return;
        }
    };
    
    println!("Tokens generated: {} tokens", tokens.len());
    
    // Parse
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("Parsing failed: {:?}", e);
            return;
        }
    };
    
    println!("AST generated with {} statements", ast.statements.len());
    
    // Print AST structure
    for (i, stmt) in ast.statements.iter().enumerate() {
        println!("Statement {}: {:?}", i, stmt);
    }
    
    // Try to execute with AST interpreter
    let mut interpreter = AstInterpreter::new();
    match interpreter.execute_program(&ast) {
        Ok(result) => {
            println!("Execution successful! Result: {:?}", result);
        }
        Err(e) => {
            println!("Execution failed: {:?}", e);
        }
    }
}