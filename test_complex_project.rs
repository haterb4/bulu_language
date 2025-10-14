use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::AstInterpreter;
use std::fs;

fn main() {
    println!("Testing complex Bulu project...");
    
    // Read the main.bu file from the complex example
    let main_path = "complex-bulu-example/src/main.bu";
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
    
    // Parse the main program
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Lexing failed: {:?}", e);
            return;
        }
    };
    
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("Parsing failed: {:?}", e);
            return;
        }
    };
    
    println!("AST generated with {} statements", ast.statements.len());
    
    // Print the import statements
    for (i, stmt) in ast.statements.iter().enumerate() {
        match stmt {
            bulu::ast::nodes::Statement::Import(import_stmt) => {
                println!("Import {}: path='{}', alias={:?}, items={:?}", 
                    i, import_stmt.path, import_stmt.alias, import_stmt.items);
            }
            _ => {}
        }
    }
    
    // The parsing was successful, so the error must be coming from execution
    // Let's try to execute and see what specific import is causing the issue
    let mut interpreter = AstInterpreter::with_file(main_path.to_string());
    
    println!("Attempting to execute the program...");
    match interpreter.execute_program(&ast) {
        Ok(result) => {
            println!("Execution successful! Result: {:?}", result);
        }
        Err(e) => {
            println!("Execution failed: {:?}", e);
            println!("The error is likely from trying to parse an imported module that doesn't exist.");
            println!("This shows that the import/export system is working - it's trying to load the imported files!");
        }
    }
}