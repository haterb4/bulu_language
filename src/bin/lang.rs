//! Bulu Language Tool (lang)
//!
//! High-level command-line tool for Bulu project management

use bulu::build::{run_executable, BuildOptions, Builder};
use bulu::compiler::symbol_resolver::SymbolType;
use bulu::compiler::{IrGenerator, SemanticAnalyzer, SymbolResolver};
use bulu::docs::{DocFormat, DocGenerator, DocOptions};
use bulu::formatter::{create_default_format_config, load_format_config, Formatter};
use bulu::lexer::Lexer;
use bulu::linter::{create_default_lint_config, load_lint_config, Linter};
use bulu::package::commands::{PackageManager, PackageOptions};
use bulu::parser::Parser;
use bulu::project::{create_project, Project};
use bulu::runtime::{ast_interpreter::AstInterpreter, Interpreter};
use bulu::testing::{BenchmarkRunner, TestOptions, TestRunner};
use bulu::types::{primitive::RuntimeValue, TypeChecker};
use bulu::{BuluError, Result};
use clap::{Arg, Command};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() -> Result<()> {
    let matches = Command::new("lang")
        .version(bulu::VERSION)
        .about("Bulu Language Tool - High-level project management")
        .subcommand(
            Command::new("build")
                .about("Build the current project")
                .arg(
                    Arg::new("release")
                        .long("release")
                        .help("Build in release mode")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("target")
                        .long("target")
                        .help("Target architecture")
                        .value_name("TARGET"),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Run a Bulu program (bytecode by default, source with --source)")
                .trailing_var_arg(true)
                .arg(
                    Arg::new("file")
                        .help("Bulu file to run followed by program arguments")
                        .value_name("FILE")
                        .num_args(1..)
                        .required(false),
                )
                .arg(
                    Arg::new("source")
                        .long("source")
                        .help("Treat input as source code instead of bytecode")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("release")
                        .long("release")
                        .help("Run in release mode (only for source)")
                        .action(clap::ArgAction::SetTrue),
                )
                .allow_external_subcommands(false)
                .disable_help_subcommand(false),
        )
        .subcommand(
            Command::new("test")
                .about("Run tests")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose test output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("coverage")
                        .long("coverage")
                        .help("Generate coverage report")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .help("Filter tests by name")
                        .value_name("PATTERN"),
                ),
        )
        .subcommand(
            Command::new("fmt")
                .about("Format source code")
                .arg(
                    Arg::new("check")
                        .long("check")
                        .help("Check if files are formatted without modifying them")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("init")
                        .long("init")
                        .help("Create a default .langfmt.toml configuration file")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("lint")
                .about("Run linter")
                .arg(
                    Arg::new("fix")
                        .long("fix")
                        .help("Automatically fix issues where possible")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("init")
                        .long("init")
                        .help("Create a default .langlint.toml configuration file")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("doc")
                .about("Generate documentation")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output directory")
                        .value_name("DIR")
                        .default_value("docs"),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .help("Output format")
                        .value_parser(["html", "markdown", "json"])
                        .default_value("html"),
                )
                .arg(
                    Arg::new("serve")
                        .long("serve")
                        .help("Start local documentation server")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .help("Port for documentation server")
                        .value_name("PORT")
                        .default_value("8080"),
                ),
        )
        .subcommand(
            Command::new("clean").about("Clean build artifacts").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Verbose output")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("new")
                .about("Create a new Bulu project")
                .arg(
                    Arg::new("name")
                        .help("Project name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .help("Directory to create project in")
                        .value_name("PATH"),
                ),
        )
        .subcommand(
            Command::new("bench").about("Run benchmarks").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Verbose output")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("add")
                .about("Add a dependency")
                .arg(
                    Arg::new("package")
                        .help("Package name")
                        .required(true)
                        .index(1),
                )
                .arg(Arg::new("version").help("Version constraint").index(2))
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a dependency")
                .arg(
                    Arg::new("package")
                        .help("Package name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("update").about("Update dependencies").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Verbose output")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("install").about("Install dependencies").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Verbose output")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("list").about("List dependencies").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Verbose output")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("search")
                .about("Search for packages")
                .arg(
                    Arg::new("query")
                        .help("Search query")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .help("Maximum number of results")
                        .value_name("NUM")
                        .default_value("20"),
                ),
        )
        .subcommand(
            Command::new("publish")
                .about("Publish package to registry")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Show what would be published without actually publishing")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("vendor")
                .about("Vendor dependencies locally")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .help("Force update existing vendored dependencies")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("build", sub_matches)) => {
            let release = sub_matches.get_flag("release");
            let verbose = sub_matches.get_flag("verbose");
            let target = sub_matches.get_one::<String>("target").map(|s| s.as_str());
            build_project(release, verbose, target)
        }
        Some(("run", sub_matches)) => {
            let release = sub_matches.get_flag("release");
            let is_source = sub_matches.get_flag("source");
            
            // Get all positional arguments (file + args)
            let positional: Vec<String> = sub_matches
                .get_many::<String>("file")
                .map(|vals| vals.map(|s| s.to_string()).collect())
                .unwrap_or_default();
            
            let file = positional.first();
            let args = if positional.len() > 1 {
                positional[1..].to_vec()
            } else {
                Vec::new()
            };
            
            run_project(file, release, is_source, args)
        }
        Some(("test", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            let coverage = sub_matches.get_flag("coverage");
            let filter = sub_matches.get_one::<String>("filter").map(|s| s.as_str());
            run_tests(verbose, coverage, filter)
        }
        Some(("fmt", sub_matches)) => {
            let check = sub_matches.get_flag("check");
            let verbose = sub_matches.get_flag("verbose");
            let init = sub_matches.get_flag("init");
            format_code(check, verbose, init)
        }
        Some(("lint", sub_matches)) => {
            let fix = sub_matches.get_flag("fix");
            let verbose = sub_matches.get_flag("verbose");
            let init = sub_matches.get_flag("init");
            lint_code(fix, verbose, init)
        }
        Some(("doc", sub_matches)) => {
            let output = sub_matches.get_one::<String>("output").unwrap();
            let format = sub_matches.get_one::<String>("format").unwrap();
            let serve = sub_matches.get_flag("serve");
            let port = sub_matches
                .get_one::<String>("port")
                .unwrap()
                .parse()
                .unwrap_or(8080);
            generate_docs(output, format, serve, port)
        }
        Some(("clean", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            clean_project(verbose)
        }
        Some(("new", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let path = sub_matches.get_one::<String>("path").map(|s| Path::new(s));
            create_new_project(name, path)
        }
        Some(("bench", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            run_benchmarks(verbose)
        }
        Some(("add", sub_matches)) => {
            let package = sub_matches.get_one::<String>("package").unwrap();
            let version = sub_matches.get_one::<String>("version").map(|s| s.as_str());
            let verbose = sub_matches.get_flag("verbose");
            add_dependency(package, version, verbose)
        }
        Some(("remove", sub_matches)) => {
            let package = sub_matches.get_one::<String>("package").unwrap();
            let verbose = sub_matches.get_flag("verbose");
            remove_dependency(package, verbose)
        }
        Some(("update", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            update_dependencies(verbose)
        }
        Some(("install", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            install_dependencies(verbose)
        }
        Some(("list", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            list_dependencies(verbose)
        }
        Some(("search", sub_matches)) => {
            let query = sub_matches.get_one::<String>("query").unwrap();
            let limit = sub_matches.get_one::<String>("limit").unwrap().parse().ok();
            search_packages(query, limit)
        }
        Some(("publish", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            let dry_run = sub_matches.get_flag("dry-run");
            publish_package(verbose, dry_run)
        }
        Some(("vendor", sub_matches)) => {
            let verbose = sub_matches.get_flag("verbose");
            let force = sub_matches.get_flag("force");
            vendor_dependencies(verbose, force)
        }
        _ => {
            println!("No subcommand provided. Use 'lang --help' for usage information.");
            return Ok(());
        }
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

fn build_project(release: bool, verbose: bool, target: Option<&str>) -> Result<()> {
    let project = Project::load_current()?;

    let mut options = BuildOptions {
        release,
        verbose,
        target: target.map(|s| s.to_string()),
        ..BuildOptions::default()
    };

    // Override with project configuration
    options.parallel = project.config.build.parallel;
    options.incremental = project.config.build.incremental;

    let builder = Builder::new(project, options);
    let result = builder.build()?;

    if !result.success {
        let error_count = result.errors.len();
        let warning_count = result.warnings.len();

        if error_count > 0 {
            eprintln!(
                "{} {} compilation error{}",
                "Error:".red().bold(),
                error_count,
                if error_count == 1 { "" } else { "s" }
            );
        }

        if warning_count > 0 {
            eprintln!(
                "{} {} warning{}",
                "Warning:".yellow().bold(),
                warning_count,
                if warning_count == 1 { "" } else { "s" }
            );
        }

        return Err(BuluError::Other(format!(
            "Build failed with {} error{} and {} warning{}",
            error_count,
            if error_count == 1 { "" } else { "s" },
            warning_count,
            if warning_count == 1 { "" } else { "s" }
        )));
    }

    Ok(())
}

/// Find the project entrypoint file (main.bu in src directory)
fn find_project_entrypoint() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .map_err(|e| BuluError::Other(format!("Failed to get current directory: {}", e)))?;

    // Look for main.bu in src directory
    let main_path = current_dir.join("src").join("main.bu");
    if main_path.exists() {
        return Ok(main_path);
    }

    // Look for main.bu in current directory as fallback
    let main_path = current_dir.join("main.bu");
    if main_path.exists() {
        return Ok(main_path);
    }

    Err(BuluError::Other(
        "No entrypoint found. Expected 'main.bu' in 'src/' directory or current directory."
            .to_string(),
    ))
}

fn run_project(file: Option<&String>, _release: bool, is_source: bool, args: Vec<String>) -> Result<()> {
    if let Some(file_path) = file {
        // Run a specific file
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(BuluError::Other(format!("File '{}' not found", file_path)));
        }

        if is_source {
            // Treat as source code
            execute_source_file_with_args(path, Some(args))?;
        } else {
            // Treat as bytecode (default)
            execute_bytecode_file(path)?;
        }
        Ok(())
    } else {
        // No file specified - look for project entrypoint
        if is_source {
            let entrypoint = find_project_entrypoint()?;
            execute_source_file_with_args(&entrypoint, Some(args))?;
        } else {
            // Look for compiled bytecode in target/debug
            let bytecode_path = find_project_bytecode()?;
            execute_bytecode_file(&bytecode_path)?;
        }
        Ok(())
    }
}

/// Execute a Bulu source file with full compilation pipeline
fn execute_source_file(path: &Path) -> Result<RuntimeValue> {
    execute_source_file_with_args(path, None)
}

/// Execute a Bulu source file with optional program arguments
fn execute_source_file_with_args(path: &Path, extra_args: Option<Vec<String>>) -> Result<RuntimeValue> {
    // Initialize program arguments for os module
    let file_path_str = path.to_string_lossy().to_string();
    let mut program_args = vec![file_path_str.clone()];
    
    // Add extra arguments if provided
    if let Some(args) = extra_args {
        program_args.extend(args);
    }
    
    bulu::std::os::init_args(program_args);
    
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

    // Set the current directory for the module resolver to the file's directory
    // This ensures relative imports are resolved correctly regardless of where lang is executed
    if let Some(parent_dir) = path.parent() {
        symbol_resolver
            .module_resolver_mut()
            .set_current_dir(parent_dir.to_path_buf());
    }

    symbol_resolver.resolve_program(&mut ast)?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker.set_file_path(Some(file_path.clone()));

    // Import symbols from the symbol resolver
    type_checker.import_symbols_from_resolver(&symbol_resolver);

    // Re-add builtin functions to ensure they're not overwritten
    type_checker.add_builtin_functions_after_import();

    // Add standard library types and methods
    type_checker.add_std_types();

    type_checker.check(&ast)?;

    // Use AST interpreter for better module support
    use bulu::runtime::ast_interpreter::AstInterpreter;
    let mut ast_interpreter = AstInterpreter::with_file(file_path.clone());
    
    // Execute the program (defines functions, imports, etc.)
    ast_interpreter.execute_program(&ast)?;
    
    // Call main() if it exists
    if let Some(main_func) = ast_interpreter.get_function_definition("main") {
        ast_interpreter.call_user_function(&main_func, &[])
    } else {
        Ok(RuntimeValue::Null)
    }
}

/// Execute a Bulu executable or bytecode file
fn execute_bytecode_file(path: &Path) -> Result<RuntimeValue> {
    // Check if it's a bytecode file (.buc extension)
    if path.extension().map_or(false, |ext| ext == "buc") {
        // Execute bytecode with Rust interpreter
        execute_bytecode_with_interpreter(path)
    } else if is_native_executable(path) {
        // Execute the native binary directly
        execute_native_binary(path)
    } else {
        // Try to detect file type by reading header
        if let Ok(content) = std::fs::read(path) {
            if content.len() >= 4 && &content[0..4] == b"BULU" {
                // It's bytecode, execute with interpreter
                execute_bytecode_with_interpreter(path)
            } else {
                // Assume it's a native executable
                execute_native_binary(path)
            }
        } else {
            Err(BuluError::Other(format!(
                "Cannot read file: {}",
                path.display()
            )))
        }
    }
}

/// Check if a file is a native executable
fn is_native_executable(path: &Path) -> bool {
    // Check if file is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0; // Check if any execute bit is set
        }
    }

    #[cfg(windows)]
    {
        // On Windows, check if it has .exe extension or is executable
        if path.extension().map_or(false, |ext| ext == "exe") {
            return true;
        }
    }

    // Try to read the file header to detect if it's a native binary
    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;
        let mut header = [0u8; 4];
        if file.read_exact(&mut header).is_ok() {
            // Check for ELF magic number (Linux)
            if header == [0x7f, 0x45, 0x4c, 0x46] {
                return true;
            }
            // Check for PE magic number (Windows) - would be at offset 0x3c, but this is a simple check
            if header[0..2] == [0x4d, 0x5a] {
                // MZ header
                return true;
            }
            // Check for Mach-O magic numbers (macOS)
            if header == [0xfe, 0xed, 0xfa, 0xce] || header == [0xfe, 0xed, 0xfa, 0xcf] {
                return true;
            }
        }
    }

    false
}

/// Execute bytecode with the Rust interpreter
fn execute_bytecode_with_interpreter(path: &Path) -> Result<RuntimeValue> {
    // Read bytecode file
    let bytecode = std::fs::read(path)
        .map_err(|e| BuluError::Other(format!("Failed to read bytecode file: {}", e)))?;

    // Validate bytecode header
    if bytecode.len() < 4 || &bytecode[0..4] != b"BULU" {
        return Err(BuluError::Other("Invalid bytecode format".to_string()));
    }

    // Create and run interpreter
    let mut interpreter = Interpreter::new();

    // For now, we'll create a simple bytecode execution
    // This is a placeholder - in a real implementation, we'd parse and execute the bytecode
    println!("Executing Bulu bytecode...");

    // Parse the bytecode and execute it
    execute_simple_bytecode(&bytecode)
}

/// Simple bytecode executor (placeholder implementation)
fn execute_simple_bytecode(bytecode: &[u8]) -> Result<RuntimeValue> {
    if bytecode.len() < 8 {
        return Err(BuluError::Other("Bytecode too short".to_string()));
    }

    let mut pc = 8; // Skip header (BULU + version + reserved + function count)

    while pc < bytecode.len() {
        if pc >= bytecode.len() {
            break;
        }

        let opcode = bytecode[pc];
        pc += 1;

        match opcode {
            0x06 => {
                // LOAD_STRING
                if pc + 4 > bytecode.len() {
                    break;
                }
                let len = u32::from_le_bytes([
                    bytecode[pc],
                    bytecode[pc + 1],
                    bytecode[pc + 2],
                    bytecode[pc + 3],
                ]) as usize;
                pc += 4;

                if pc + len > bytecode.len() {
                    break;
                }
                let string_data = &bytecode[pc..pc + len];
                let string = String::from_utf8_lossy(string_data);
                print!("{}", string);
                pc += len;
            }
            0x40 => {
                // PRINTLN
                println!();
            }
            0x30 => {
                // RETURN
                break;
            }
            _ => {
                // Skip unknown opcodes
            }
        }
    }

    Ok(RuntimeValue::Null)
}

/// Execute a native binary
fn execute_native_binary(path: &Path) -> Result<RuntimeValue> {
    use std::process::Command;

    let output = Command::new(path)
        .output()
        .map_err(|e| BuluError::Other(format!("Failed to execute binary: {}", e)))?;

    // Print stdout
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    // Print stderr
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Check exit status
    if !output.status.success() {
        return Err(BuluError::Other(format!(
            "Program exited with code: {}",
            output.status.code().unwrap_or(-1)
        )));
    }

    // Return a dummy value since native executables don't return RuntimeValue
    Ok(RuntimeValue::Null)
}

/// Find the compiled executable or bytecode for the current project
fn find_project_bytecode() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .map_err(|e| BuluError::Other(format!("Failed to get current directory: {}", e)))?;

    // Get project name from directory name
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("main");

    // First, look for bytecode files (.buc) - these are generated in debug mode
    let bytecode_path = current_dir.join(format!("{}.buc", project_name));
    if bytecode_path.exists() {
        return Ok(bytecode_path);
    }

    let debug_bytecode_path = current_dir
        .join("target")
        .join("debug")
        .join(format!("{}.buc", project_name));
    if debug_bytecode_path.exists() {
        return Ok(debug_bytecode_path);
    }

    // Then look for native executables - these are generated in release mode
    let executable_path = current_dir.join(project_name);
    if executable_path.exists() {
        return Ok(executable_path);
    }

    let debug_path = current_dir.join("target").join("debug").join(project_name);
    if debug_path.exists() {
        return Ok(debug_path);
    }

    let release_path = current_dir
        .join("target")
        .join("release")
        .join(project_name);
    if release_path.exists() {
        return Ok(release_path);
    }

    // Also check with .exe extension on Windows
    #[cfg(windows)]
    {
        let exe_path = current_dir.join(format!("{}.exe", project_name));
        if exe_path.exists() {
            return Ok(exe_path);
        }

        let debug_exe_path = current_dir
            .join("target")
            .join("debug")
            .join(format!("{}.exe", project_name));
        if debug_exe_path.exists() {
            return Ok(debug_exe_path);
        }

        let release_exe_path = current_dir
            .join("target")
            .join("release")
            .join(format!("{}.exe", project_name));
        if release_exe_path.exists() {
            return Ok(release_exe_path);
        }
    }

    Err(BuluError::Other(format!(
        "No compiled executable or bytecode found. Run 'langc build' first to compile the project."
    )))
}

fn run_tests(verbose: bool, coverage: bool, filter: Option<&str>) -> Result<()> {
    let project = Project::load_current()?;

    let options = TestOptions {
        verbose,
        coverage,
        filter: filter.map(|s| s.to_string()),
        ..TestOptions::default()
    };

    let runner = TestRunner::new(project, options);
    let result = runner.run_tests()?;

    if coverage {
        runner.generate_coverage()?;
    }

    if result.failed > 0 {
        return Err(BuluError::Other(format!("{} tests failed", result.failed)));
    }

    Ok(())
}

fn format_code(check: bool, verbose: bool, init: bool) -> Result<()> {
    if init {
        // Create default configuration file
        let current_dir = std::env::current_dir()
            .map_err(|e| BuluError::Other(format!("Failed to get current directory: {}", e)))?;
        return create_default_format_config(&current_dir);
    }

    let project = Project::load_current()?;

    let mut options = load_format_config(&project.root)?;
    options.check_only = check;
    options.verbose = verbose;

    let formatter = Formatter::new(project, options);
    let results = formatter.format_project()?;

    if check {
        let needs_formatting = results.iter().any(|r| r.changed);
        if needs_formatting {
            return Err(BuluError::Other("Some files need formatting".to_string()));
        }
    }

    Ok(())
}

fn lint_code(fix: bool, verbose: bool, init: bool) -> Result<()> {
    if init {
        // Create default configuration file
        let current_dir = std::env::current_dir()
            .map_err(|e| BuluError::Other(format!("Failed to get current directory: {}", e)))?;
        return create_default_lint_config(&current_dir);
    }

    let project = Project::load_current()?;

    let mut options = load_lint_config(&project.root)?;
    options.fix = fix;
    options.verbose = verbose;

    let linter = Linter::new(project, options);
    let result = linter.lint_project()?;

    if result.errors > 0 {
        return Err(BuluError::Other(format!(
            "{} lint errors found",
            result.errors
        )));
    }

    Ok(())
}

fn generate_docs(output: &str, format: &str, serve: bool, port: u16) -> Result<()> {
    let project = Project::load_current()?;

    let doc_format = match format {
        "html" => DocFormat::Html,
        "markdown" => DocFormat::Markdown,
        "json" => DocFormat::Json,
        _ => return Err(BuluError::Other(format!("Unknown format: {}", format))),
    };

    let options = DocOptions {
        output_dir: PathBuf::from(output),
        format: doc_format,
        serve,
        port,
        verbose: true,
        ..DocOptions::default()
    };

    let generator = DocGenerator::new(project, options);
    generator.generate()?;

    Ok(())
}

fn clean_project(verbose: bool) -> Result<()> {
    let project = Project::load_current()?;

    let options = BuildOptions {
        verbose,
        ..BuildOptions::default()
    };

    let builder = Builder::new(project, options);
    builder.clean()?;

    Ok(())
}

fn create_new_project(name: &str, path: Option<&Path>) -> Result<()> {
    create_project(name, path)?;

    println!(
        "{} Created new Bulu project '{}'",
        "Success".green().bold(),
        name
    );
    println!();
    println!("To get started:");
    if let Some(path) = path {
        println!("  cd {}", path.join(name).display());
    } else {
        println!("  cd {}", name);
    }
    println!("  lang run");

    Ok(())
}

fn run_benchmarks(verbose: bool) -> Result<()> {
    let project = Project::load_current()?;

    if verbose {
        println!(
            "{} Running benchmarks for '{}'...",
            "Benchmarking".green().bold(),
            project.config.package.name
        );
    }

    let runner = BenchmarkRunner::new(project);
    runner.run_benchmarks()?;

    Ok(())
}
// Package management functions

fn add_dependency(package: &str, version: Option<&str>, verbose: bool) -> Result<()> {
    use bulu::package::http_client::RegistryHttpClient;
    use std::fs;
    use std::io::Write;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async {
        if verbose {
            println!("{} Adding dependency: {}", "Adding".green().bold(), package);
        }

        // Load project
        let mut project = Project::load_current()?;
        
        // Get registry URL from config or use default
        let registry_url = std::env::var("BULU_REGISTRY")
            .unwrap_or_else(|_| "https://bulu-language.onrender.com".to_string());

        // Create HTTP client
        let client = RegistryHttpClient::new(registry_url);

        // Find the version to use
        let version_to_use = if let Some(v) = version {
            v.to_string()
        } else {
            // Get latest version
            let versions = client.get_package_versions(package).await?;
            versions.last()
                .ok_or_else(|| BuluError::Other(format!("No versions found for {}", package)))?
                .clone()
        };

        if verbose {
            println!("  {} Using version: {}", "→".blue(), version_to_use);
        }

        // Add to dependencies in lang.toml
        let version_spec = if version.is_some() {
            version_to_use.clone()
        } else {
            format!("^{}", version_to_use)
        };

        project.config.dependencies.insert(
            package.to_string(),
            bulu::project::DependencySpec::Simple(version_spec.clone())
        );

        // Save lang.toml
        let config_content = toml::to_string_pretty(&project.config)
            .map_err(|e| BuluError::Other(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(project.root.join("lang.toml"), config_content)
            .map_err(|e| BuluError::Other(format!("Failed to write lang.toml: {}", e)))?;

        // Download and install the package
        if verbose {
            println!("  {} Downloading {}...", "→".blue(), package);
        }

        let tarball = client.download_package(package, &version_to_use).await?;

        // Extract to vendor directory
        let vendor_dir = project.root.join("vendor").join(package);
        fs::create_dir_all(&vendor_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create vendor directory: {}", e)))?;

        // Extract tarball
        use flate2::read::GzDecoder;
        use tar::Archive;
        use std::io::Cursor;

        let cursor = Cursor::new(tarball);
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);
        
        archive.unpack(&vendor_dir)
            .map_err(|e| BuluError::Other(format!("Failed to extract package: {}", e)))?;

        println!("{} Added {} v{}", "Success".green().bold(), package, version_to_use);

        Ok(())
    })
}

fn remove_dependency(package: &str, verbose: bool) -> Result<()> {
    use std::fs;

    if verbose {
        println!("{} Removing dependency: {}", "Removing".red().bold(), package);
    }

    let mut project = Project::load_current()?;

    if !project.config.dependencies.contains_key(package) {
        return Err(BuluError::Other(format!("Dependency {} not found", package)));
    }

    // Remove from dependencies
    project.config.dependencies.remove(package);

    // Save lang.toml
    let config_content = toml::to_string_pretty(&project.config)
        .map_err(|e| BuluError::Other(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(project.root.join("lang.toml"), config_content)
        .map_err(|e| BuluError::Other(format!("Failed to write lang.toml: {}", e)))?;

    // Remove from vendor directory
    let vendor_dir = project.root.join("vendor").join(package);
    if vendor_dir.exists() {
        fs::remove_dir_all(&vendor_dir)
            .map_err(|e| BuluError::Other(format!("Failed to remove vendor directory: {}", e)))?;
    }

    println!("{} Removed dependency: {}", "Success".green().bold(), package);

    Ok(())
}

fn update_dependencies(verbose: bool) -> Result<()> {
    use bulu::package::http_client::RegistryHttpClient;
    use std::fs;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async {
        if verbose {
            println!("{} Updating dependencies...", "Updating".blue().bold());
        }

        let project = Project::load_current()?;
        
        if project.config.dependencies.is_empty() {
            println!("No dependencies to update");
            return Ok(());
        }

        let registry_url = std::env::var("BULU_REGISTRY")
            .unwrap_or_else(|_| "https://bulu-language.onrender.com".to_string());

        let client = RegistryHttpClient::new(registry_url);

        let mut updated = 0;

        for (name, _spec) in &project.config.dependencies {
            if verbose {
                println!("  {} Updating {}...", "→".blue(), name);
            }

            // Get latest version
            let versions = client.get_package_versions(name).await?;
            let latest_version = versions.last()
                .ok_or_else(|| BuluError::Other(format!("No versions found for {}", name)))?;

            // Download
            let tarball = client.download_package(name, latest_version).await?;

            // Remove old version
            let vendor_dir = project.root.join("vendor").join(name);
            if vendor_dir.exists() {
                fs::remove_dir_all(&vendor_dir)
                    .map_err(|e| BuluError::Other(format!("Failed to remove old version: {}", e)))?;
            }

            // Extract new version
            fs::create_dir_all(&vendor_dir)
                .map_err(|e| BuluError::Other(format!("Failed to create vendor directory: {}", e)))?;

            use flate2::read::GzDecoder;
            use tar::Archive;
            use std::io::Cursor;

            let cursor = Cursor::new(tarball);
            let decoder = GzDecoder::new(cursor);
            let mut archive = Archive::new(decoder);
            
            archive.unpack(&vendor_dir)
                .map_err(|e| BuluError::Other(format!("Failed to extract package: {}", e)))?;

            if verbose {
                println!("    {} {} v{}", "✓".green(), name, latest_version);
            }

            updated += 1;
        }

        println!("{} Updated {} dependencies", "Success".green().bold(), updated);

        Ok(())
    })
}

fn install_dependencies(verbose: bool) -> Result<()> {
    use bulu::package::http_client::RegistryHttpClient;
    use std::fs;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async {
        if verbose {
            println!("{} Installing dependencies...", "Installing".blue().bold());
        }

        let project = Project::load_current()?;
        
        if project.config.dependencies.is_empty() {
            println!("No dependencies to install");
            return Ok(());
        }

        let registry_url = std::env::var("BULU_REGISTRY")
            .unwrap_or_else(|_| "https://bulu-language.onrender.com".to_string());

        let client = RegistryHttpClient::new(registry_url);

        let mut installed = 0;

        for (name, spec) in &project.config.dependencies {
            if verbose {
                println!("  {} Installing {}...", "→".blue(), name);
            }

            // Parse version spec
            let version_str = match spec {
                bulu::project::DependencySpec::Simple(v) => v.clone(),
                bulu::project::DependencySpec::Detailed { version, .. } => {
                    version.clone().unwrap_or_else(|| "*".to_string())
                }
            };

            // Get versions and find matching one
            let versions = client.get_package_versions(name).await?;
            let version_to_use = versions.last()
                .ok_or_else(|| BuluError::Other(format!("No versions found for {}", name)))?;

            // Download
            let tarball = client.download_package(name, version_to_use).await?;

            // Extract
            let vendor_dir = project.root.join("vendor").join(name);
            fs::create_dir_all(&vendor_dir)
                .map_err(|e| BuluError::Other(format!("Failed to create vendor directory: {}", e)))?;

            use flate2::read::GzDecoder;
            use tar::Archive;
            use std::io::Cursor;

            let cursor = Cursor::new(tarball);
            let decoder = GzDecoder::new(cursor);
            let mut archive = Archive::new(decoder);
            
            archive.unpack(&vendor_dir)
                .map_err(|e| BuluError::Other(format!("Failed to extract package: {}", e)))?;

            if verbose {
                println!("    {} {} v{}", "✓".green(), name, version_to_use);
            }

            installed += 1;
        }

        println!("{} Installed {} dependencies", "Success".green().bold(), installed);

        Ok(())
    })
}

fn list_dependencies(verbose: bool) -> Result<()> {
    let project = Project::load_current()?;

    if project.config.dependencies.is_empty() {
        println!("No dependencies");
        return Ok(());
    }

    println!("{}", "Dependencies:".bold());
    
    for (name, spec) in &project.config.dependencies {
        let version_str = match spec {
            bulu::project::DependencySpec::Simple(v) => v.clone(),
            bulu::project::DependencySpec::Detailed { version, path, git, .. } => {
                if let Some(path) = path {
                    format!("path:{}", path)
                } else if let Some(git) = git {
                    format!("git:{}", git)
                } else if let Some(version) = version {
                    version.clone()
                } else {
                    "*".to_string()
                }
            }
        };

        println!("  {} {}", 
            name.cyan(), 
            version_str.green()
        );

        if verbose {
            // Check if installed
            let vendor_path = project.root.join("vendor").join(name);
            if vendor_path.exists() {
                println!("    {} Installed at {}", "✓".green(), vendor_path.display());
            } else {
                println!("    {} Not installed", "✗".red());
            }
        }
    }

    Ok(())
}

fn search_packages(query: &str, limit: Option<usize>) -> Result<()> {
    use bulu::package::http_client::RegistryHttpClient;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async {
        println!("{} Searching for: {}", "Searching".blue().bold(), query);

        let registry_url = std::env::var("BULU_REGISTRY")
            .unwrap_or_else(|_| "https://bulu-language.onrender.com".to_string());

        let client = RegistryHttpClient::new(registry_url);
        let results = client.search(query, limit).await?;

        if results.packages.is_empty() {
            println!("No packages found matching '{}'", query);
            return Ok(());
        }

        println!("Found {} packages:", results.packages.len());
        
        for package in &results.packages {
            println!("  {} {} - {}", 
                package.name.cyan().bold(),
                package.version.green(),
                package.description.as_deref().unwrap_or("No description").dimmed()
            );
            println!("    {} downloads", 
                package.downloads.to_string().yellow()
            );
        }

        if results.total > results.packages.len() {
            println!("\n... and {} more results", results.total - results.packages.len());
        }

        Ok(())
    })
}

fn publish_package(verbose: bool, dry_run: bool) -> Result<()> {
    use bulu::package::http_client::{RegistryHttpClient, PublishRequest};
    use std::fs;
    use std::io::Read;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async {
        println!("{} Loading project configuration...", "→".blue());
        let project = Project::load_current()?;
        println!("{} Project loaded: {}", "✓".green(), project.config.package.name);

        println!("{} Publishing package: {} v{}", 
            "Publishing".blue().bold(), 
            project.config.package.name,
            project.config.package.version
        );

        // Create tarball
        println!("  {} Creating tarball...", "→".blue());

        let tarball_path = project.root.join(format!("{}-{}.tar.gz", 
            project.config.package.name, 
            project.config.package.version
        ));

        // Create tar.gz
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let tarball_file = fs::File::create(&tarball_path)
            .map_err(|e| BuluError::Other(format!("Failed to create tarball: {}", e)))?;
        
        let encoder = GzEncoder::new(tarball_file, Compression::default());
        let mut builder = Builder::new(encoder);

        // Add src directory
        let src_dir = project.root.join("src");
        if src_dir.exists() {
            println!("    {} Adding src directory: {}", "→".blue(), src_dir.display());
            builder.append_dir_all("src", &src_dir)
                .map_err(|e| BuluError::Other(format!("Failed to add src: {}", e)))?;
            println!("    {} src directory added", "✓".green());
        } else {
            println!("    {} src directory not found", "⚠".yellow());
        }

        // Add lang.toml
        println!("    {} Adding lang.toml", "→".blue());
        builder.append_path_with_name(project.root.join("lang.toml"), "lang.toml")
            .map_err(|e| BuluError::Other(format!("Failed to add lang.toml: {}", e)))?;
        println!("    {} lang.toml added", "✓".green());

        // Add README if exists
        let readme_path = project.root.join("README.md");
        if readme_path.exists() {
            println!("    {} Adding README.md", "→".blue());
            builder.append_path_with_name(&readme_path, "README.md")
                .map_err(|e| BuluError::Other(format!("Failed to add README: {}", e)))?;
            println!("    {} README.md added", "✓".green());
        }

        let encoder = builder.into_inner()
            .map_err(|e| BuluError::Other(format!("Failed to finish tar builder: {}", e)))?;
        
        encoder.finish()
            .map_err(|e| BuluError::Other(format!("Failed to finish gzip encoder: {}", e)))?;

        println!("  {} Tarball created: {}", "✓".green(), tarball_path.display());

        // Read and encode tarball
        println!("  {} Reading tarball...", "→".blue());
        let mut tarball_file = fs::File::open(&tarball_path)
            .map_err(|e| BuluError::Other(format!("Failed to read tarball: {}", e)))?;
        
        let mut tarball_data = Vec::new();
        tarball_file.read_to_end(&mut tarball_data)
            .map_err(|e| BuluError::Other(format!("Failed to read tarball data: {}", e)))?;

        println!("  {} Tarball size: {} bytes", "✓".green(), tarball_data.len());

        if dry_run {
            println!("Would publish: {} v{}", project.config.package.name, project.config.package.version);
            println!("  Tarball size: {} bytes", tarball_data.len());
            fs::remove_file(&tarball_path).ok();
            return Ok(());
        }

        // Prepare dependencies
        println!("  {} Preparing package metadata...", "→".blue());
        let mut dependencies = std::collections::HashMap::new();
        for (name, spec) in &project.config.dependencies {
            let version_str = match spec {
                bulu::project::DependencySpec::Simple(v) => v.clone(),
                bulu::project::DependencySpec::Detailed { version, .. } => {
                    version.clone().unwrap_or_else(|| "*".to_string())
                }
            };
            dependencies.insert(name.clone(), version_str);
        }
        println!("  {} Dependencies: {}", "✓".green(), dependencies.len());

        // Create publish request
        println!("  {} Creating publish request...", "→".blue());
        let request = PublishRequest {
            name: project.config.package.name.clone(),
            version: project.config.package.version.clone(),
            description: project.config.package.description.clone(),
            authors: project.config.package.authors.clone(),
            license: project.config.package.license.clone(),
            repository: project.config.package.repository.clone(),
            keywords: project.config.package.keywords.clone().unwrap_or_default(),
            dependencies,
            tarball: tarball_data,
        };

        // Publish
        let registry_url = std::env::var("BULU_REGISTRY")
            .unwrap_or_else(|_| "https://bulu-language.onrender.com".to_string());

        println!("  {} Uploading to registry: {}", "→".blue(), registry_url);
        println!("  {} Package: {} v{}", "→".blue(), request.name, request.version);

        let client = RegistryHttpClient::new(registry_url.clone());
        
        match client.publish(request).await {
            Ok(_) => {
                println!("  {} Upload successful!", "✓".green());
            }
            Err(e) => {
                eprintln!("  {} Upload failed: {}", "✗".red(), e);
                fs::remove_file(&tarball_path).ok();
                return Err(e);
            }
        }

        // Clean up tarball
        println!("  {} Cleaning up tarball...", "→".blue());
        fs::remove_file(&tarball_path).ok();

        println!("{} Published: {} v{}", 
            "Success".green().bold(), 
            project.config.package.name, 
            project.config.package.version
        );

        Ok(())
    })
}

fn vendor_dependencies(verbose: bool, force: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async {
        let project = Project::load_current()?;
        let package_manager = PackageManager::new(project)?;

        let options = PackageOptions {
            verbose,
            dry_run: false,
            force,
        };

        package_manager.vendor_dependencies(&options).await
    })
}
