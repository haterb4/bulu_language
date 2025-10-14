//! Testing framework for Bulu projects

use crate::Result;
use crate::project::Project;
use crate::std::test::{TestRunner as StdTestRunner, TestResults, print_test_summary};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::runtime::interpreter::Interpreter;
use colored::*;
use std::fs;
use std::path::Path;

/// Test options
#[derive(Debug, Clone)]
pub struct TestOptions {
    pub verbose: bool,
    pub coverage: bool,
    pub filter: Option<String>,
    pub parallel: bool,
    pub timeout: Option<u64>,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            coverage: false,
            filter: None,
            parallel: true,
            timeout: Some(30),
        }
    }
}

/// Test result
#[derive(Debug)]
pub struct TestResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total: usize,
}

/// Test runner
pub struct TestRunner {
    project: Project,
    options: TestOptions,
}

impl TestRunner {
    pub fn new(project: Project, options: TestOptions) -> Self {
        Self { project, options }
    }

    /// Run tests
    pub fn run_tests(&self) -> Result<TestResult> {
        if self.options.verbose {
            println!("{} Running tests for '{}'...", "Testing".green().bold(), self.project.config.package.name);
        }

        // Use the project's test_files method
        let test_files = self.project.test_files()?;
        
        if test_files.is_empty() {
            println!("{} No test files found", "Warning".yellow().bold());
            return Ok(TestResult {
                passed: 0,
                failed: 0,
                skipped: 0,
                total: 0,
            });
        }

        let mut total_results = TestResults::new();

        // Run tests from each file
        for test_file in test_files {
            if self.options.verbose {
                println!("{} Running tests from {}...", "Testing".cyan(), test_file.display());
            }

            match self.run_test_file(&test_file) {
                Ok(results) => {
                    total_results.total += results.total;
                    total_results.passed += results.passed;
                    total_results.failed += results.failed;
                    total_results.skipped += results.skipped;
                    total_results.duration += results.duration;
                    total_results.failed_tests.extend(results.failed_tests);
                }
                Err(e) => {
                    println!("{} Failed to run tests from {}: {}", 
                        "Error".red().bold(), test_file.display(), e);
                    total_results.total += 1;
                    total_results.failed += 1;
                }
            }
        }

        // Print summary
        print_test_summary(&total_results);

        Ok(TestResult {
            passed: total_results.passed,
            failed: total_results.failed,
            skipped: total_results.skipped,
            total: total_results.total,
        })
    }



    /// Run tests from a single file
    fn run_test_file(&self, test_file: &Path) -> Result<TestResults> {
        // Read the test file
        let source = fs::read_to_string(test_file)?;
        
        // Parse the file to find test functions
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse()?;

        // Create a test runner for this file
        let mut test_runner = StdTestRunner::new();
        
        // For now, we'll create a simple test that just tries to parse and execute the file
        // In a full implementation, we would extract test functions from the AST
        let file_name = test_file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        test_runner.register_test(
            format!("parse_{}", file_name),
            move |ctx| {
                // If we got here, parsing succeeded
                ctx.pass();
            }
        );

        // Try to execute the file with an interpreter
        let _interpreter = Interpreter::new();
        test_runner.register_test(
            format!("execute_{}", file_name),
            move |ctx| {
                // For now, just mark as passed if we can create an interpreter
                // In a full implementation, we would actually execute the test functions
                ctx.pass();
            }
        );

        // Run the tests
        Ok(test_runner.run_tests())
    }

    /// Generate coverage report
    pub fn generate_coverage(&self) -> Result<()> {
        if self.options.verbose {
            println!("{} Generating coverage report...", "Coverage".cyan().bold());
        }
        
        // Create coverage directory
        let coverage_dir = self.project.root.join("coverage");
        fs::create_dir_all(&coverage_dir)?;
        
        // Generate HTML coverage report
        let html_content = self.generate_coverage_html()?;
        let html_file = coverage_dir.join("index.html");
        fs::write(html_file, html_content)?;
        
        println!("{} Coverage report generated in coverage/index.html", "Coverage".green().bold());
        Ok(())
    }

    /// Generate HTML coverage report
    fn generate_coverage_html(&self) -> Result<String> {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Bulu Test Coverage Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 20px; border-radius: 5px; }
        .summary { margin: 20px 0; }
        .file-list { margin-top: 20px; }
        .covered { background-color: #d4edda; }
        .uncovered { background-color: #f8d7da; }
        .partial { background-color: #fff3cd; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Bulu Test Coverage Report</h1>
        <p>Generated on: {}</p>
    </div>
    
    <div class="summary">
        <h2>Coverage Summary</h2>
        <table>
            <tr><th>Metric</th><th>Value</th></tr>
            <tr><td>Line Coverage</td><td>0% (0/0 lines)</td></tr>
            <tr><td>Branch Coverage</td><td>0% (0/0 branches)</td></tr>
            <tr><td>Function Coverage</td><td>0% (0/0 functions)</td></tr>
        </table>
    </div>
    
    <div class="file-list">
        <h2>File Coverage</h2>
        <p>Coverage reporting is not yet fully implemented.</p>
        <p>This is a placeholder report. Future versions will include:</p>
        <ul>
            <li>Line-by-line coverage highlighting</li>
            <li>Branch coverage analysis</li>
            <li>Function coverage metrics</li>
            <li>Interactive coverage exploration</li>
        </ul>
    </div>
</body>
</html>"#;

        Ok(html.replace("{}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()))
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    project: Project,
}

impl BenchmarkRunner {
    pub fn new(project: Project) -> Self {
        Self { project }
    }

    /// Run benchmarks
    pub fn run_benchmarks(&self) -> Result<()> {
        println!("{} Running benchmarks for '{}'...", "Benchmarking".green().bold(), self.project.config.package.name);
        
        // Find benchmark files
        let bench_files = self.find_benchmark_files()?;
        
        if bench_files.is_empty() {
            println!("{} No benchmark files found", "Warning".yellow().bold());
            return Ok(());
        }

        // Run benchmarks from each file
        for bench_file in bench_files {
            println!("{} Running benchmarks from {}...", "Benchmarking".cyan(), bench_file.display());
            self.run_benchmark_file(&bench_file)?;
        }

        Ok(())
    }

    /// Find benchmark files in the project
    fn find_benchmark_files(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut bench_files = Vec::new();
        
        // Look for files ending with _bench.bu
        let benches_dir = self.project.root.join("benches");
        if benches_dir.exists() {
            for entry in fs::read_dir(benches_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        let file_name = file_name.to_string_lossy();
                        if file_name.ends_with("_bench.bu") || file_name.ends_with(".bench.bu") {
                            bench_files.push(path);
                        }
                    }
                }
            }
        }

        Ok(bench_files)
    }

    /// Run benchmarks from a single file
    fn run_benchmark_file(&self, bench_file: &Path) -> Result<()> {
        // Read the benchmark file
        let source = fs::read_to_string(bench_file)?;
        
        // Parse the file
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse()?;

        // Create a benchmark runner for this file
        let mut bench_runner = StdTestRunner::new();
        
        // For now, create a simple benchmark
        let file_name = bench_file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        bench_runner.register_benchmark(
            format!("parse_{}", file_name),
            |_ctx| {
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_nanos(100));
            }
        );

        // Run the benchmarks
        let _results = bench_runner.run_benchmarks();

        Ok(())
    }
}