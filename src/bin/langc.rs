//! Bulu Language Compiler (langc)
//!
//! Command-line compiler for the Bulu programming language

use bulu::compiler::{
    CodeGenerator, IrGenerator, IrOptimizer, OptLevel as CompilerOptLevel, SemanticAnalyzer,
    SymbolResolver,
};
use bulu::error_reporter::ErrorReporter;
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::types::TypeChecker;
use bulu::{BuluError, Result};
use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

/// Optimization levels
#[derive(Debug, Clone, Copy)]
enum OptLevel {
    O0, // No optimization
    O1, // Basic optimization
    O2, // Standard optimization
    O3, // Aggressive optimization
    Os, // Optimize for size
}

impl OptLevel {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "0" => Ok(OptLevel::O0),
            "1" => Ok(OptLevel::O1),
            "2" => Ok(OptLevel::O2),
            "3" => Ok(OptLevel::O3),
            "s" => Ok(OptLevel::Os),
            _ => Err(BuluError::Other(format!(
                "Invalid optimization level: {}",
                s
            ))),
        }
    }
}

/// Emit types for intermediate representations
#[derive(Debug, Clone)]
enum EmitType {
    Tokens,
    Ast,
    Ir,
    Assembly,
    Executable,
}

impl EmitType {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "tokens" => Ok(EmitType::Tokens),
            "ast" => Ok(EmitType::Ast),
            "ir" => Ok(EmitType::Ir),
            "asm" | "assembly" => Ok(EmitType::Assembly),
            "exe" | "executable" => Ok(EmitType::Executable),
            _ => Err(BuluError::Other(format!("Invalid emit type: {}", s))),
        }
    }
}

/// Target platforms for cross-compilation
#[derive(Debug, Clone)]
enum Target {
    LinuxAmd64,
    LinuxArm64,
    WindowsAmd64,
    WindowsArm64,
    DarwinAmd64,
    DarwinArm64,
    Wasm,
    Native,
}

impl Target {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "linux-amd64" => Ok(Target::LinuxAmd64),
            "linux-arm64" => Ok(Target::LinuxArm64),
            "windows-amd64" => Ok(Target::WindowsAmd64),
            "windows-arm64" => Ok(Target::WindowsArm64),
            "darwin-amd64" => Ok(Target::DarwinAmd64),
            "darwin-arm64" => Ok(Target::DarwinArm64),
            "wasm" => Ok(Target::Wasm),
            "native" => Ok(Target::Native),
            _ => Err(BuluError::Other(format!("Unsupported target: {}", s))),
        }
    }

    fn default() -> Self {
        Target::Native
    }
}

/// Compiler configuration
#[derive(Debug)]
struct CompilerConfig {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
    opt_level: OptLevel,
    emit_type: EmitType,
    target: Target,
    debug: bool,
    static_link: bool,
}

fn main() -> Result<()> {
    let matches = Command::new("langc")
        .version(bulu::VERSION)
        .about("Bulu Language Compiler")
        .long_about("Bulu Language Compiler (langc) compiles .bu source files into executable binaries or intermediate representations.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("build")
                .about("Compile Bulu source files")
                .arg(
                    Arg::new("input")
                        .help("Input source file (.bu) or project directory")
                        .index(1)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file name")
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("optimization")
                        .short('O')
                        .long("opt-level")
                        .value_name("LEVEL")
                        .help("Optimization level: 0 (none), 1 (basic), 2 (standard), 3 (aggressive), s (size)")
                        .default_value("0")
                )

                .arg(
                    Arg::new("target")
                        .long("target")
                        .value_name("TARGET")
                        .help("Target platform: linux-amd64, linux-arm64, windows-amd64, windows-arm64, darwin-amd64, darwin-arm64, wasm, native")
                        .default_value("native")
                )
                .arg(
                    Arg::new("release")
                        .long("release")
                        .help("Build in release mode (equivalent to -O 2)")
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("debug")
                        .short('g')
                        .long("debug")
                        .help("Generate debug information")
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("static")
                        .long("static")
                        .help("Enable static linking")
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Enable verbose output")
                        .action(ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("emit")
                .about("Emit intermediate representations")
                .arg(
                    Arg::new("input")
                        .help("Input source file (.bu)")
                        .required(true)
                        .index(1)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("type")
                        .help("Type to emit: tokens, ast, ir, asm")
                        .required(true)
                        .index(2)
                        .value_parser(["tokens", "ast", "ir", "asm"])
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file name")
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            let config = parse_build_config(sub_matches)?;
            let verbose = sub_matches.get_flag("verbose");

            if verbose {
                println!("{}", "Bulu Language Compiler".bright_blue().bold());
                println!("Version: {}", bulu::VERSION);
                println!("Input: {}", config.input_file.display());
                if let Some(ref output) = config.output_file {
                    println!("Output: {}", output.display());
                }
                println!("Optimization: {:?}", config.opt_level);
                println!("Target: {:?}", config.target);
                println!();
            }

            // Compile the source file or project
            match compile(&config, verbose) {
                Ok(_) => {
                    if verbose {
                        println!("{}", "Compilation successful!".bright_green().bold());
                    }
                    Ok(())
                }
                Err(_) => {
                    // Error was already printed by the compile function with context
                    // Just exit with error code
                    process::exit(1);
                }
            }
        }
        Some(("emit", sub_matches)) => {
            let config = parse_emit_config(sub_matches)?;
            let verbose = false; // emit commands are typically not verbose

            // Compile with emit mode
            match compile(&config, verbose) {
                Ok(_) => Ok(()),
                Err(_) => {
                    process::exit(1);
                }
            }
        }
        _ => {
            unreachable!("Subcommand required")
        }
    }
}

fn parse_build_config(matches: &clap::ArgMatches) -> Result<CompilerConfig> {
    let (input_file, project_name) = if let Some(input) = matches.get_one::<PathBuf>("input") {
        // Explicit input file or directory provided
        if input.is_dir() {
            // Directory provided, look for main.bu
            let main_file = input.join("src").join("main.bu");
            if main_file.exists() {
                let project_name = input
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("main")
                    .to_string();
                (main_file, project_name)
            } else {
                return Err(BuluError::IoError(format!(
                    "No main.bu found in {}/src/",
                    input.display()
                )));
            }
        } else if input.is_file() {
            // File provided
            if !input.extension().map_or(false, |ext| ext == "bu") {
                return Err(BuluError::Other(
                    "Input file must have .bu extension".to_string(),
                ));
            }
            let project_name = input
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("main")
                .to_string();
            (input.clone(), project_name)
        } else {
            return Err(BuluError::IoError(format!(
                "Input '{}' not found",
                input.display()
            )));
        }
    } else {
        // No input provided, look for project in current directory
        let current_dir = std::env::current_dir()
            .map_err(|e| BuluError::IoError(format!("Cannot get current directory: {}", e)))?;

        // Look for main.bu in src/ directory
        let main_file = current_dir.join("src").join("main.bu");
        if main_file.exists() {
            let project_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("main")
                .to_string();
            (main_file, project_name)
        } else {
            // Look for main.bu in current directory
            let main_file = current_dir.join("main.bu");
            if main_file.exists() {
                let project_name = current_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("main")
                    .to_string();
                (main_file, project_name)
            } else {
                return Err(BuluError::IoError(
                    "No main.bu found in current directory or src/ subdirectory".to_string(),
                ));
            }
        }
    };

    // Parse optimization level and determine build mode
    let (opt_level, build_mode) = if matches.get_flag("release") {
        (OptLevel::O2, "release") // Release mode
    } else {
        let opt_level = OptLevel::from_str(matches.get_one::<String>("optimization").unwrap())?;
        let build_mode = match opt_level {
            OptLevel::O0 | OptLevel::O1 => "debug",
            OptLevel::O2 | OptLevel::O3 | OptLevel::Os => "release",
        };
        (opt_level, build_mode)
    };

    // Parse target
    let target = Target::from_str(matches.get_one::<String>("target").unwrap())?;

    // Determine output file
    let output_file = if let Some(output) = matches.get_one::<PathBuf>("output") {
        Some(output.clone())
    } else {
        // Create output in target/debug or target/release directory (like Rust)

        let target_dir = std::env::current_dir()
            .map_err(|e| BuluError::IoError(format!("Cannot get current directory: {}", e)))?
            .join("target")
            .join(build_mode);

        // Create target directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            return Err(BuluError::IoError(format!(
                "Cannot create target directory: {}",
                e
            )));
        }

        let extension = if cfg!(windows) { ".exe" } else { "" };
        let output_name = format!("{}{}", project_name, extension);
        Some(target_dir.join(output_name))
    };

    // Build mode is already determined above

    Ok(CompilerConfig {
        input_file,
        output_file,
        opt_level,
        emit_type: EmitType::Executable,
        target,
        debug: matches.get_flag("debug"),
        static_link: matches.get_flag("static"),
    })
}

fn parse_emit_config(matches: &clap::ArgMatches) -> Result<CompilerConfig> {
    let input_file = matches.get_one::<PathBuf>("input").unwrap().clone();

    // Validate input file
    if !input_file.exists() {
        return Err(BuluError::IoError(format!(
            "Input file '{}' not found",
            input_file.display()
        )));
    }

    if !input_file.extension().map_or(false, |ext| ext == "bu") {
        return Err(BuluError::Other(
            "Input file must have .bu extension".to_string(),
        ));
    }

    // Parse emit type
    let emit_type_str = matches.get_one::<String>("type").unwrap();
    let emit_type = EmitType::from_str(emit_type_str)?;

    // Determine output file
    let output_file = matches.get_one::<PathBuf>("output").cloned();

    Ok(CompilerConfig {
        input_file,
        output_file,
        opt_level: OptLevel::O0, // No optimization for emit
        emit_type,
        target: Target::Native,
        debug: false,
        static_link: false,
    })
}

fn compile(config: &CompilerConfig, verbose: bool) -> Result<()> {
    // Read source code
    let source = fs::read_to_string(&config.input_file).map_err(|e| {
        BuluError::IoError(format!(
            "Failed to read {}: {}",
            config.input_file.display(),
            e
        ))
    })?;

    // Create error reporter for enhanced error messages
    let error_reporter = ErrorReporter::from_source(
        &source,
        Some(config.input_file.to_string_lossy().to_string()),
    );

    if verbose {
        println!("{}", "Lexical analysis...".bright_yellow());
    }

    // Tokenization with file information
    let file_path = config.input_file.to_string_lossy().to_string();
    let mut lexer = Lexer::with_file(&source, file_path.clone());
    let tokens = lexer.tokenize().map_err(|e| {
        eprintln!("{}", error_reporter.format_error(&e));
        e
    })?;

    if matches!(config.emit_type, EmitType::Tokens) {
        return emit_tokens(&tokens, &config.output_file);
    }

    if verbose {
        println!("{}", "Parsing...".bright_yellow());
    }

    // Parsing with file information
    let mut parser = Parser::with_file(tokens, file_path.clone());
    let mut ast = parser.parse().map_err(|e| {
        eprintln!("{}", error_reporter.format_error(&e));
        e
    })?;

    if matches!(config.emit_type, EmitType::Ast) {
        return emit_ast(&ast, &config.output_file);
    }

    if verbose {
        println!("{}", "Symbol resolution...".bright_yellow());
    }

    // Symbol resolution for imports/exports
    let mut symbol_resolver = SymbolResolver::new();
    symbol_resolver.set_current_module(file_path.clone());
    symbol_resolver.resolve_program(&mut ast).map_err(|e| {
        eprintln!("{}", error_reporter.format_error(&e));
        e
    })?;

    if verbose {
        println!("{}", "Symbol resolution...".bright_yellow());
    }

    // Symbol resolution for imports/exports
    let mut symbol_resolver = SymbolResolver::new();
    symbol_resolver.set_current_module(file_path.clone());

    // Set the current directory for the module resolver
    if let Some(parent_dir) = config.input_file.parent() {
        symbol_resolver
            .module_resolver()
            .set_current_dir(parent_dir.to_path_buf());
    }

    symbol_resolver
        .resolve_program(&mut ast.clone())
        .map_err(|e| {
            eprintln!("{}", error_reporter.format_error(&e));
            e
        })?;

    if verbose {
        let symbol_table = symbol_resolver.symbol_table();
        println!(
            "Imported symbols: {:?}",
            symbol_table.imported_symbols.keys().collect::<Vec<_>>()
        );
        println!(
            "Local symbols: {:?}",
            symbol_table.local_symbols.keys().collect::<Vec<_>>()
        );
    }

    if verbose {
        println!("{}", "Type checking...".bright_yellow());
    }

    // Type checking and semantic analysis with enhanced error reporting
    let mut type_checker = TypeChecker::new();

    // Import symbols from the symbol resolver
    type_checker.import_symbols_from_resolver(&symbol_resolver);

    if verbose {
        println!(
            "TypeChecker global scope symbols: {:?}",
            type_checker
                .scopes
                .first()
                .map(|s| s.keys().collect::<Vec<_>>())
                .unwrap_or_default()
        );
    }

    type_checker.check(&ast).map_err(|e| {
        eprintln!("{}", error_reporter.format_error(&e));
        e
    })?;

    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.analyze(&mut ast.clone()).map_err(|e| {
        eprintln!("{}", error_reporter.format_error(&e));
        e
    })?;

    if verbose {
        println!("{}", "IR generation...".bright_yellow());
    }

    // Combine main AST with all imported modules
    let combined_ast = combine_ast_with_imports(&ast, &symbol_resolver)?;

    // IR generation with enhanced error reporting
    let mut ir_generator = IrGenerator::new();
    let mut ir_program = ir_generator.generate(&combined_ast).map_err(|e| {
        eprintln!("{}", error_reporter.format_error(&e));
        e
    })?;

    // IR optimization
    if !matches!(config.opt_level, OptLevel::O0) {
        if verbose {
            println!("{}", "IR optimization...".bright_yellow());
        }

        let mut optimizer = IrOptimizer::new();
        let compiler_opt_level = match config.opt_level {
            OptLevel::O0 => CompilerOptLevel::O0,
            OptLevel::O1 => CompilerOptLevel::O1,
            OptLevel::O2 => CompilerOptLevel::O2,
            OptLevel::O3 => CompilerOptLevel::O3,
            OptLevel::Os => CompilerOptLevel::Os,
        };
        optimizer.set_level(compiler_opt_level);
        ir_program = optimizer.optimize(ir_program).map_err(|e| {
            eprintln!("{}", error_reporter.format_error(&e));
            e
        })?;
    }

    if matches!(config.emit_type, EmitType::Ir) {
        return emit_ir(&ir_program, &config.output_file);
    }

    if verbose {
        println!("{}", "Code generation...".bright_yellow());
    }

    // Code generation with enhanced error reporting
    let mut code_generator = CodeGenerator::new();
    let target_str = match config.target {
        Target::LinuxAmd64 => "linux-amd64",
        Target::LinuxArm64 => "linux-arm64",
        Target::WindowsAmd64 => "windows-amd64",
        Target::WindowsArm64 => "windows-arm64",
        Target::DarwinAmd64 => "darwin-amd64",
        Target::DarwinArm64 => "darwin-arm64",
        Target::Wasm => "wasm",
        Target::Native => "native",
    };
    code_generator.set_target(target_str);
    code_generator.set_debug(config.debug);
    code_generator.set_static_link(config.static_link);

    match config.emit_type {
        EmitType::Assembly => {
            let assembly = code_generator.generate_assembly(&ir_program).map_err(|e| {
                eprintln!("{}", error_reporter.format_error(&e));
                e
            })?;
            emit_assembly(&assembly, &config.output_file)
        }
        EmitType::Executable => {
            let executable = code_generator
                .generate_executable(&ir_program)
                .map_err(|e| {
                    eprintln!("{}", error_reporter.format_error(&e));
                    e
                })?;
            
            if code_generator.is_bytecode_output() {
                emit_bytecode(&executable, &config.output_file)
            } else {
                emit_executable(&executable, &config.output_file)
            }
        }
        _ => unreachable!(),
    }
}

fn emit_tokens(tokens: &[bulu::lexer::Token], output_file: &Option<PathBuf>) -> Result<()> {
    let content = tokens
        .iter()
        .map(|token| format!("{:?}", token))
        .collect::<Vec<_>>()
        .join("\n");

    if let Some(output) = output_file {
        fs::write(output, content)?;
        println!("Tokens written to {}", output.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn emit_ast(ast: &bulu::ast::Program, output_file: &Option<PathBuf>) -> Result<()> {
    let content = format!("{:#?}", ast);

    if let Some(output) = output_file {
        fs::write(output, content)?;
        println!("AST written to {}", output.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn emit_ir(ir_program: &bulu::compiler::IrProgram, output_file: &Option<PathBuf>) -> Result<()> {
    let content = format!("{:#?}", ir_program);

    if let Some(output) = output_file {
        fs::write(output, content)?;
        println!("IR written to {}", output.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn emit_assembly(assembly: &str, output_file: &Option<PathBuf>) -> Result<()> {
    if let Some(output) = output_file {
        fs::write(output, assembly)?;
        println!("Assembly written to {}", output.display());
    } else {
        println!("{}", assembly);
    }

    Ok(())
}

fn emit_executable(executable: &[u8], output_file: &Option<PathBuf>) -> Result<()> {
    if let Some(output) = output_file {
        fs::write(output, executable)?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(output)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(output, perms)?;
        }

        println!("Executable written to {}", output.display());
    } else {
        return Err(BuluError::Other(
            "Cannot output executable to stdout".to_string(),
        ));
    }

    Ok(())
}

fn emit_bytecode(bytecode: &[u8], output_file: &Option<PathBuf>) -> Result<()> {
    if let Some(output) = output_file {
        // Change extension to .buc (Bulu Compiled) for bytecode
        let bytecode_path = if output.extension().is_some() {
            output.with_extension("buc")
        } else {
            output.with_extension("buc")
        };
        
        fs::write(&bytecode_path, bytecode)?;
        println!("Bytecode written to {}", bytecode_path.display());
    } else {
        return Err(BuluError::Other(
            "Cannot output bytecode to stdout".to_string(),
        ));
    }

    Ok(())
}

/// Combine the main AST with all imported modules
fn combine_ast_with_imports(
    main_ast: &bulu::ast::Program,
    symbol_resolver: &SymbolResolver,
) -> Result<bulu::ast::Program> {
    use bulu::ast::*;

    let mut combined_statements = Vec::new();

    // First, add all statements from imported modules (excluding imports/exports)
    for module in symbol_resolver.get_loaded_modules() {
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
