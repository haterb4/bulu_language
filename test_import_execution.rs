use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::AstInterpreter;
use bulu::compiler::SymbolResolver;
use std::fs;

fn main() {
    // Read the main file
    let source = fs::read_to_string("simple_main.bu").expect("Failed to read file");
    
    // Parse the main file
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().expect("Failed to tokenize");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("Failed to parse");
    
    // Resolve symbols and imports
    let mut symbol_resolver = SymbolResolver::new();
    symbol_resolver.set_current_module("simple_main.bu".to_string());
    let mut ast_mut = ast.clone();
    symbol_resolver.resolve_program(&mut ast_mut).expect("Failed to resolve symbols");
    
    // Combine AST with imports (same logic as in langc.rs)
    let combined_ast = combine_ast_with_imports(&ast_mut, &mut symbol_resolver).expect("Failed to combine AST");
    
    // Execute with AST interpreter
    let mut interpreter = AstInterpreter::new();
    match interpreter.execute_program(&combined_ast) {
        Ok(_) => println!("Program executed successfully!"),
        Err(e) => eprintln!("Execution error: {}", e),
    }
}

/// Combine the main AST with all imported modules
fn combine_ast_with_imports(
    main_ast: &bulu::ast::Program, 
    symbol_resolver: &mut SymbolResolver
) -> Result<bulu::ast::Program, Box<dyn std::error::Error>> {
    use bulu::ast::*;
    
    let mut combined_statements = Vec::new();
    
    // First, add all statements from imported modules (excluding imports/exports)
    for module in symbol_resolver.module_resolver().get_loaded_modules() {
        for statement in &module.ast.statements {
            match statement {
                // Skip import statements as they're already resolved
                Statement::Import(_) => continue,
                
                // For export statements, add the wrapped statement
                Statement::Export(export_stmt) => {
                    combined_statements.push(export_stmt.item.as_ref().clone());
                }
                
                // Add all other statements (functions, variables, etc.)
                _ => {
                    combined_statements.push(statement.clone());
                }
            }
        }
    }
    
    // Then add statements from the main AST (excluding imports)
    for statement in &main_ast.statements {
        match statement {
            // Skip import statements as dependencies are already included
            Statement::Import(_) => continue,
            
            // Add all other statements
            _ => {
                combined_statements.push(statement.clone());
            }
        }
    }
    
    Ok(Program {
        statements: combined_statements,
        position: main_ast.position,
    })
}