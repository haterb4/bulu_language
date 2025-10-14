use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn main() {
    println!("Testing main function with imports...");
    
    let source = r#"import { Calculator } from "./math/calculator.bu"
import { Logger } from "./utils/logger.bu"
import { User, UserManager } from "./models/user.bu"
import "std.io" as io

func main() {
    let logger = Logger.new("INFO")
}"#;
    
    println!("Source:");
    println!("{}", source);
    println!();
    
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Lexing failed: {:?}", e);
            return;
        }
    };
    
    println!("Tokens around 'func main()':");
    for (i, token) in tokens.iter().enumerate() {
        if token.lexeme == "func" || token.lexeme == "main" || i > 0 && tokens[i-1].lexeme == "main" {
            println!("  {}: {:?}", i, token);
        }
    }
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("Parsing successful! {} statements", ast.statements.len());
        }
        Err(e) => {
            println!("Parsing failed: {:?}", e);
        }
    }
}