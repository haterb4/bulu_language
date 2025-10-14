//! Build system for Bulu projects

use crate::{BuluError, Result};
use crate::project::Project;
use crate::runtime::Interpreter;
use crate::error_reporter::ErrorReporter;
use std::path::{Path, PathBuf};
use std::process::Command;
use colored::*;

/// Build options
#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub release: bool,
    pub verbose: bool,
    pub target: Option<String>,
    pub parallel: bool,
    pub incremental: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            release: false,
            verbose: false,
            target: None,
            parallel: true,
            incremental: true,
        }
    }
}

/// Build result
#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Project builder
pub struct Builder {
    project: Project,
    options: BuildOptions,
}

impl Builder {
    pub fn new(project: Project, options: BuildOptions) -> Self {
        Self { project, options }
    }

    /// Build the project
    pub fn build(&self) -> Result<BuildResult> {
        if self.options.verbose {
            println!("{} Building project '{}'...", "Building".green().bold(), self.project.config.package.name);
        }

        // Get main source file
        let main_file = self.project.src_dir.join("main.bu");
        if !main_file.exists() {
            return Err(BuluError::Other("No main.bu file found in src directory".to_string()));
        }

        // Determine output path
        let output_name = if self.options.release {
            format!("{}-release", self.project.config.package.name)
        } else {
            self.project.config.package.name.clone()
        };
        
        let output_path = self.project.target_dir.join(&output_name);

        // Use langc to compile
        let langc_path = std::env::current_exe()?
            .parent()
            .unwrap()
            .join("langc");

        let mut cmd = Command::new(&langc_path);
        cmd.arg(&main_file)
            .arg("-o")
            .arg(&output_path);

        if self.options.release {
            cmd.arg("-O3");
        }

        if self.options.verbose {
            cmd.arg("--verbose");
        }

        let output = cmd.output()?;

        if output.status.success() {
            if self.options.verbose {
                println!("{} Build completed successfully", "Finished".green().bold());
            }
            Ok(BuildResult {
                success: true,
                output_path: Some(output_path),
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            let stdout_msg = String::from_utf8_lossy(&output.stdout);
            
            // The langc compiler already provides enhanced error reporting,
            // so we just need to pass through the output
            if !stdout_msg.is_empty() {
                eprintln!("{}", stdout_msg);
            }
            if !error_msg.is_empty() {
                eprintln!("{}", error_msg);
            }
            
            // Extract errors and warnings from compiler output
            let mut errors = Vec::new();
            let mut warnings = Vec::new();
            
            for line in error_msg.lines().chain(stdout_msg.lines()) {
                if line.contains("error") || line.contains("Error") {
                    errors.push(line.to_string());
                } else if line.contains("warning") || line.contains("Warning") {
                    warnings.push(line.to_string());
                }
            }
            
            Ok(BuildResult {
                success: false,
                output_path: None,
                errors,
                warnings,
            })
        }
    }

    /// Clean build artifacts
    pub fn clean(&self) -> Result<()> {
        if self.options.verbose {
            println!("{} Cleaning build artifacts...", "Cleaning".yellow().bold());
        }

        if self.project.target_dir.exists() {
            std::fs::remove_dir_all(&self.project.target_dir)?;
        }

        if self.options.verbose {
            println!("{} Clean completed", "Finished".green().bold());
        }

        Ok(())
    }
}

/// Run a Bulu program by interpreting the source directly
pub fn run_executable(exe_path: &Path, _args: &[String]) -> Result<()> {
    // For now, we'll interpret the source directly instead of trying to run compiled bytecode
    // This is a simpler approach that will work immediately
    
    // Find the project root by looking for lang.toml
    let mut current_dir = exe_path.parent().unwrap_or(exe_path);
    let mut project_root = None;
    
    // Walk up the directory tree to find lang.toml
    loop {
        if current_dir.join("lang.toml").exists() {
            project_root = Some(current_dir);
            break;
        }
        
        if let Some(parent) = current_dir.parent() {
            current_dir = parent;
        } else {
            break;
        }
    }
    
    let project_root = project_root.ok_or_else(|| {
        BuluError::Other("Could not find project root (lang.toml not found)".to_string())
    })?;
    
    // Look for main.bu in the src directory
    let source_path = project_root.join("src").join("main.bu");
    
    if !source_path.exists() {
        return Err(BuluError::Other(format!("Source file not found: {}", source_path.display())));
    }

    // Read and execute the source code
    let source = std::fs::read_to_string(&source_path)?;
    let mut interpreter = Interpreter::new();
    
    match interpreter.execute_source(&source) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            Err(e)
        }
    }
}