use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::ast_interpreter::AstInterpreter;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing complex Bulu example with import/export functionality...");
    
    // Read the main file
    let main_source = fs::read_to_string("complex-bulu-example/src/main.bu")?;
    println!("Read main.bu: {} characters", main_source.len());
    
    // Parse the main file
    let mut lexer = Lexer::with_file(&main_source, "complex-bulu-example/src/main.bu".to_string());
    let tokens = lexer.tokenize()?;
    println!("Tokenized main.bu: {} tokens", tokens.len());
    
    let mut parser = Parser::with_file(tokens, "complex-bulu-example/src/main.bu".to_string());
    let ast = parser.parse()?;
    println!("Parsed main.bu: {} statements", ast.statements.len());
    
    // Create AST interpreter with file context
    let mut interpreter = AstInterpreter::with_file("complex-bulu-example/src/main.bu".to_string());
    
    // Set the current directory for relative imports
    interpreter.module_resolver.set_current_dir(std::path::PathBuf::from("complex-bulu-example/src"));
    
    // Add the supporting modules to the module resolver's memory for testing
    let calculator_source = fs::read_to_string("complex-bulu-example/src/math/calculator.bu")?;
    let logger_source = fs::read_to_string("complex-bulu-example/src/utils/logger.bu")?;
    let user_source = fs::read_to_string("complex-bulu-example/src/models/user.bu")?;
    
    interpreter.module_resolver.add_memory_module("./math/calculator.bu".to_string(), calculator_source);
    interpreter.module_resolver.add_memory_module("./utils/logger.bu".to_string(), logger_source);
    interpreter.module_resolver.add_memory_module("./models/user.bu".to_string(), user_source);
    
    println!("Added supporting modules to memory");
    
    // Execute the program
    match interpreter.execute_program(&ast) {
        Ok(result) => {
            println!("Program executed successfully!");
            println!("Result: {:?}", result);
        }
        Err(e) => {
            println!("Execution error: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}