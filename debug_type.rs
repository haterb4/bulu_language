use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn main() {
    let source = "let callback: func(int32, string): bool";
    println!("Parsing: {}", source);
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    
    println!("Tokens:");
    for token in &tokens {
        println!("  {:?}", token);
    }
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => println!("Success: {:?}", program),
        Err(e) => println!("Error: {:?}", e),
    }
}