use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn main() {
    let source = "obj.method().getValue()";
    println!("Parsing: {}", source);
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    
    let mut parser = Parser::new(tokens);
    match parser.parse_expression() {
        Ok(expr) => println!("Success: {:#?}", expr),
        Err(e) => println!("Error: {:?}", e),
    }
}