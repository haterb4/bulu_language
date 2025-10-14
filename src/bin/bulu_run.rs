//! Bulu Source Code Runner
//!
//! This executable can run Bulu source code files directly with full import/export support.

use bulu::compiler::{IrGenerator, SemanticAnalyzer, SymbolResolver};
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::runtime::Interpreter;
use bulu::types::{primitive::RuntimeValue, TypeChecker};
use bulu::{BuluError, Result};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source-file>", args[0]);
        eprintln!("");
        eprintln!("Bulu Source Code Runner");
        eprintln!("Executes Bulu source code files with full import/export support.");
        eprintln!("");
        eprintln!("Example:");
        eprintln!("  {} program.bu", args[0]);
        process::exit(1);
    }

    let source_file = &args[1];
    let path = Path::new(source_file);

    if !path.exists() {
        eprintln!("Error: File '{}' not found", source_file);
        process::exit(1);
    }

    match execute_source_file(path) {
        Ok(_result) => {
            // Program executed successfully - no output needed
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Execute a Bulu source file with full compilation pipeline
fn execute_source_file(path: &Path) -> Result<RuntimeValue> {
    // Read source code
    let source = fs::read_to_string(path)
        .map_err(|e| BuluError::Other(format!("Failed to read file: {}", e)))?;

    // Get file path for module resolution
    let file_path = path.to_string_lossy().to_string();

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    // Parse
    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse()?;

    // Symbol resolution for imports/exports
    let mut symbol_resolver = SymbolResolver::new();
    symbol_resolver.set_current_module(file_path.clone());

    // Set the current directory for the module resolver
    if let Some(parent_dir) = path.parent() {
        symbol_resolver
            .module_resolver()
            .set_current_dir(parent_dir.to_path_buf());
    }

    symbol_resolver.resolve_program(&mut ast)?;

    // Type checking
    let mut type_checker = TypeChecker::new();

    // Import symbols from the symbol resolver
    type_checker.import_symbols_from_resolver(&symbol_resolver);

    type_checker.check(&ast)?;

    // Semantic analysis
    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.analyze(&mut ast.clone())?;

    // Get all loaded modules from the symbol resolver
    let loaded_modules = symbol_resolver.get_loaded_modules();

    // Create a combined IR program that includes all modules
    let mut ir_generator = IrGenerator::new();
    let mut combined_ir_program = ir_generator.generate(&ast)?;

    // Generate IR for all imported modules and merge them
    for module in loaded_modules {
        let module_ir = ir_generator.generate(&module.ast)?;

        // Merge functions, globals, structs, and interfaces
        combined_ir_program.functions.extend(module_ir.functions);
        combined_ir_program.globals.extend(module_ir.globals);
        combined_ir_program.structs.extend(module_ir.structs);
        combined_ir_program.interfaces.extend(module_ir.interfaces);
    }

    // Debug logs removed - only program output should be shown
    
    // Execute with interpreter
    let mut interpreter = Interpreter::new();
    interpreter.load_program(combined_ir_program);
    interpreter.execute()
}
