use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::AstInterpreter;

fn main() {
    println!("Testing integrated import/export system...");
    
    // Create a simple module to import from
    let module_source = r#"
export const PI = 3.14159
export let greeting = "Hello, World!"

export func add(a: int64, b: int64): int64 {
    return a + b
}
"#;
    
    // Create the main program that imports from the module
    let main_source = r#"
import { PI, greeting, add } from "./test_module.bu"

func main() {
    let result = add(10, 20)
    let message = greeting + " Result: " + result
    return message
}
"#;
    
    // First, let's test parsing the module
    println!("Parsing module...");
    let mut lexer = Lexer::new(module_source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let module_ast = parser.parse().unwrap();
    
    println!("Module AST has {} statements", module_ast.statements.len());
    
    // Now test parsing the main program
    println!("Parsing main program...");
    let mut lexer = Lexer::new(main_source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let main_ast = parser.parse().unwrap();
    
    println!("Main AST has {} statements", main_ast.statements.len());
    
    // Try to execute the main program
    println!("Executing main program...");
    let mut interpreter = AstInterpreter::with_file("main.bu".to_string());
    
    // Add the test module to the module resolver
    interpreter.module_resolver.add_memory_module("./test_module.bu".to_string(), module_source.to_string());
    
    match interpreter.execute_program(&main_ast) {
        Ok(result) => {
            println!("Execution successful! Result: {:?}", result);
        }
        Err(e) => {
            println!("Execution failed: {:?}", e);
        }
    }
}