use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::AstInterpreter;

fn main() {
    println!("Testing symbol access after import...");
    
    // Create a simple module to import from
    let module_source = r#"
export const PI = 3.14159
export let greeting = "Hello from module!"
"#;
    
    // Create the main program that imports and uses symbols
    let main_source = r#"
import { PI, greeting } from "./test_module.bu"

let result = PI + 1.0
let message = greeting + " PI is: " + PI
"#;
    
    // Parse and execute
    let mut lexer = Lexer::new(main_source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let main_ast = parser.parse().unwrap();
    
    let mut interpreter = AstInterpreter::with_file("main.bu".to_string());
    interpreter.module_resolver.add_memory_module("./test_module.bu".to_string(), module_source.to_string());
    
    match interpreter.execute_program(&main_ast) {
        Ok(result) => {
            println!("Execution successful! Result: {:?}", result);
            
            // Check if imported symbols are available in the environment
            println!("\nChecking imported symbols:");
            if let Some(pi_value) = interpreter.environment().get("PI") {
                println!("PI = {:?}", pi_value);
            } else {
                println!("PI not found in environment");
            }
            
            if let Some(greeting_value) = interpreter.environment().get("greeting") {
                println!("greeting = {:?}", greeting_value);
            } else {
                println!("greeting not found in environment");
            }
            
            if let Some(result_value) = interpreter.environment().get("result") {
                println!("result = {:?}", result_value);
            } else {
                println!("result not found in environment");
            }
            
            if let Some(message_value) = interpreter.environment().get("message") {
                println!("message = {:?}", message_value);
            } else {
                println!("message not found in environment");
            }
        }
        Err(e) => {
            println!("Execution failed: {:?}", e);
        }
    }
}