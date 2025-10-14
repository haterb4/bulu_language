use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn main() {
    println!("Testing function parsing...");
    
    let test_cases = vec![
        "func main() {}",
        "func test() { return 42 }",
        "func add(a: int64, b: int64): int64 { return a + b }",
        "export func greet(name: string): string { return \"Hello\" }",
    ];
    
    for (i, source) in test_cases.iter().enumerate() {
        println!("\nTest case {}: {}", i + 1, source);
        
        let mut lexer = Lexer::new(source);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                println!("  Lexing failed: {:?}", e);
                continue;
            }
        };
        
        println!("  Tokens: {:?}", tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>());
        
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(ast) => {
                println!("  Parsing successful! {} statements", ast.statements.len());
            }
            Err(e) => {
                println!("  Parsing failed: {:?}", e);
            }
        }
    }
}