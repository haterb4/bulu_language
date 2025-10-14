use bulu::lexer::Lexer;
use bulu::parser::Parser;
use std::fs;

fn main() {
    println!("Testing exact content from complex example...");
    
    // Read just the first few lines to isolate the issue
    let source = r#"// Main entry point for the complex Bulu example
import { Calculator } from "./math/calculator.bu"
import { Logger } from "./utils/logger.bu"
import { User, UserManager } from "./models/user.bu"
import "std.io" as io

func main() {
    let logger = Logger.new("INFO")
}"#;
    
    println!("Testing with simplified content...");
    
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Lexing failed: {:?}", e);
            return;
        }
    };
    
    println!("Tokenization successful, {} tokens", tokens.len());
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("Parsing successful! {} statements", ast.statements.len());
        }
        Err(e) => {
            println!("Parsing failed: {:?}", e);
        }
    }
    
    // Now test with the actual file content
    println!("\nTesting with actual file content...");
    let actual_content = fs::read_to_string("complex-bulu-example/src/main.bu").unwrap();
    
    let mut lexer = Lexer::new(&actual_content);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Lexing failed: {:?}", e);
            return;
        }
    };
    
    println!("Tokenization successful, {} tokens", tokens.len());
    
    let tokens_clone = tokens.clone();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("Parsing successful! {} statements", ast.statements.len());
        }
        Err(e) => {
            println!("Parsing failed: {:?}", e);
            
            // Let's examine the tokens around the error position
            println!("\nExamining tokens around the error...");
            for (i, token) in tokens_clone.iter().enumerate() {
                if token.position.line >= 5 && token.position.line <= 8 {
                    println!("  Token {}: {:?} at line {}, col {}", i, token, token.position.line, token.position.column);
                }
            }
        }
    }
}