//! Bulu Language Tool (lang)
//!
//! High-level command-line tool for Bulu project management

use bulu::build::{run_executable, BuildOptions, Builder};
use bulu::compiler::{IrGenerator, SemanticAnalyzer, SymbolResolver};
use bulu::docs::{DocFormat, DocGenerator, DocOptions};
use bulu::formatter::{create_default_format_config, load_format_config, Formatter};
use bulu::linter::{create_default_lint_config, load_lint_config, Linter};
use bulu::lexer::Lexer;
use bulu::package::commands::{PackageManager, PackageOptions};
use bulu::parser::Parser;
use bulu::project::{create_project, Project};
use bulu::runtime::Interpreter;
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
                .about("Build and run the current project or a specific file")
                .arg(
                    Arg::new("file")
                        .help("Bulu source file to run (optional)")
                        .value_name("FILE")
                        .index(1)
                        .required(false),
                )
                .arg(
                    Arg::new("release")
                        .long("release")
                        .help("Run in release mode")
                        .action(clap::ArgAction::SetTrue),
                ),
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
                .arg(
                    Arg::new("version")
                        .help("Version constraint")
                        .index(2),
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
            Command::new("update")
                .about("Update dependencies")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("install")
                .about("Install dependencies")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List dependencies")
                .arg(
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
            let file = sub_matches.get_one::<String>("file");
            let release = sub_matches.get_flag("release");
            run_project(file, release)
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
            eprintln!("{} {} compilation error{}", 
                "Error:".red().bold(), 
                error_count, 
                if error_count == 1 { "" } else { "s" }
            );
        }
        
        if warning_count > 0 {
            eprintln!("{} {} warning{}", 
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
        "No entrypoint found. Expected 'main.bu' in 'src/' directory or current directory.".to_string()
    ))
}

fn run_project(file: Option<&String>, release: bool) -> Result<()> {
    if let Some(file_path) = file {
        // Run a specific file directly (like bulu_run)
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(BuluError::Other(format!("File '{}' not found", file_path)));
        }
        
        execute_source_file(path)?;
        Ok(())
    } else {
        // No file specified - look for project entrypoint
        let entrypoint = find_project_entrypoint()?;
        execute_source_file(&entrypoint)?;
        Ok(())
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

    // Set the current directory for the module resolver to the file's directory
    // This ensures relative imports are resolved correctly regardless of where lang is executed
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

    // Execute with interpreter
    let mut interpreter = Interpreter::new();
    interpreter.load_program(combined_ir_program);
    interpreter.execute()
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
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let mut package_manager = PackageManager::new(project)?;
        
        let options = PackageOptions {
            verbose,
            dry_run: false,
            force: false,
        };
        
        package_manager.add_dependency(package, version, &options).await
    })
}

fn remove_dependency(package: &str, verbose: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let mut package_manager = PackageManager::new(project)?;
        
        let options = PackageOptions {
            verbose,
            dry_run: false,
            force: false,
        };
        
        package_manager.remove_dependency(package, &options).await
    })
}

fn update_dependencies(verbose: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let mut package_manager = PackageManager::new(project)?;
        
        let options = PackageOptions {
            verbose,
            dry_run: false,
            force: false,
        };
        
        package_manager.update_dependencies(&options).await
    })
}

fn install_dependencies(verbose: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let mut package_manager = PackageManager::new(project)?;
        
        let options = PackageOptions {
            verbose,
            dry_run: false,
            force: false,
        };
        
        package_manager.install_dependencies(&options).await
    })
}

fn list_dependencies(verbose: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let package_manager = PackageManager::new(project)?;
        
        let options = PackageOptions {
            verbose,
            dry_run: false,
            force: false,
        };
        
        package_manager.list_dependencies(&options).await
    })
}

fn search_packages(query: &str, limit: Option<usize>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let package_manager = PackageManager::new(project)?;
        
        package_manager.search_packages(query, limit).await
    })
}

fn publish_package(verbose: bool, dry_run: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| BuluError::Other(format!("Failed to create async runtime: {}", e)))?;
    
    rt.block_on(async {
        let project = Project::load_current()?;
        let package_manager = PackageManager::new(project)?;
        
        let options = PackageOptions {
            verbose,
            dry_run,
            force: false,
        };
        
        package_manager.publish_package(&options).await
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