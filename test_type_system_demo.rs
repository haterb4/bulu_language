use std::fs;
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::types::TypeChecker;

fn main() {
    let source = fs::read_to_string("test_type_system_demo.bu").expect("Failed to read file");
    
    println!("Source code:");
    println!("{}", source);
    println!("\n{}\n", "=".repeat(50));
    
    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            return;
        }
    };
    
    println!("Tokenization: SUCCESS ({} tokens)", tokens.len());
    
    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            return;
        }
    };
    
    println!("Parsing: SUCCESS ({} statements)", program.statements.len());
    
    // Type check
    let mut type_checker = TypeChecker::new();
    match type_checker.check(&program) {
        Ok(()) => {
            println!("Type checking: SUCCESS");
            println!("\nAll type checks passed! The program is well-typed.");
        }
        Err(e) => {
            eprintln!("Type checking error: {}", e);
            return;
        }
    }
    
    println!("\nType system demonstration completed successfully!");
}