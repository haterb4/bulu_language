//! Testing framework for Bulu programs
//! 
//! This module provides unit testing capabilities including:
//! - Test functions and assertions
//! - Benchmarking with performance metrics
//! - Test fixtures and setup/teardown
//! - Code coverage reporting

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::Result;

/// Test context passed to test functions
#[derive(Debug, Clone)]
pub struct TestContext {
    pub name: String,
    pub passed: bool,
    pub failed: bool,
    pub skipped: bool,
    pub error_message: Option<String>,
    pub start_time: Option<Instant>,
    pub duration: Option<Duration>,
}

impl TestContext {
    pub fn new(name: String) -> Self {
        Self {
            name,
            passed: false,
            failed: false,
            skipped: false,
            error_message: None,
            start_time: None,
            duration: None,
        }
    }

    /// Mark test as passed
    pub fn pass(&mut self) {
        self.passed = true;
        self.failed = false;
    }

    /// Mark test as failed with message
    pub fn fail(&mut self, message: String) {
        self.failed = true;
        self.passed = false;
        self.error_message = Some(message);
    }

    /// Mark test as skipped
    pub fn skip(&mut self, reason: String) {
        self.skipped = true;
        self.error_message = Some(reason);
    }

    /// Start timing the test
    pub fn start_timer(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stop timing the test
    pub fn stop_timer(&mut self) {
        if let Some(start) = self.start_time {
            self.duration = Some(start.elapsed());
        }
    }
}

/// Benchmark context passed to benchmark functions
#[derive(Debug, Clone)]
pub struct BenchmarkContext {
    pub name: String,
    pub iterations: u64,
    pub total_duration: Duration,
    pub allocations: u64,
    pub bytes_allocated: u64,
}

impl BenchmarkContext {
    pub fn new(name: String) -> Self {
        Self {
            name,
            iterations: 0,
            total_duration: Duration::new(0, 0),
            allocations: 0,
            bytes_allocated: 0,
        }
    }

    /// Get operations per second
    pub fn ops_per_sec(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.iterations as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get nanoseconds per operation
    pub fn ns_per_op(&self) -> f64 {
        if self.iterations > 0 {
            self.total_duration.as_nanos() as f64 / self.iterations as f64
        } else {
            0.0
        }
    }

    /// Get bytes allocated per operation
    pub fn bytes_per_op(&self) -> f64 {
        if self.iterations > 0 {
            self.bytes_allocated as f64 / self.iterations as f64
        } else {
            0.0
        }
    }
}

/// Test result summary
#[derive(Debug, Clone)]
pub struct TestResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
    pub failed_tests: Vec<String>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration: Duration::new(0, 0),
            failed_tests: Vec::new(),
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total > 0 {
            self.passed as f64 / self.total as f64 * 100.0
        } else {
            0.0
        }
    }
}

/// Test runner for executing test functions
pub struct TestRunner {
    tests: HashMap<String, Box<dyn Fn(&mut TestContext)>>,
    benchmarks: HashMap<String, Box<dyn Fn(&mut BenchmarkContext)>>,
    setup_functions: Vec<Box<dyn Fn()>>,
    teardown_functions: Vec<Box<dyn Fn()>>,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            tests: HashMap::new(),
            benchmarks: HashMap::new(),
            setup_functions: Vec::new(),
            teardown_functions: Vec::new(),
        }
    }

    /// Register a test function
    pub fn register_test<F>(&mut self, name: String, test_fn: F)
    where
        F: Fn(&mut TestContext) + 'static,
    {
        self.tests.insert(name, Box::new(test_fn));
    }

    /// Register a benchmark function
    pub fn register_benchmark<F>(&mut self, name: String, bench_fn: F)
    where
        F: Fn(&mut BenchmarkContext) + 'static,
    {
        self.benchmarks.insert(name, Box::new(bench_fn));
    }

    /// Register setup function
    pub fn register_setup<F>(&mut self, setup_fn: F)
    where
        F: Fn() + 'static,
    {
        self.setup_functions.push(Box::new(setup_fn));
    }

    /// Register teardown function
    pub fn register_teardown<F>(&mut self, teardown_fn: F)
    where
        F: Fn() + 'static,
    {
        self.teardown_functions.push(Box::new(teardown_fn));
    }

    /// Run all tests
    pub fn run_tests(&self) -> TestResults {
        let mut results = TestResults::new();
        let start_time = Instant::now();

        println!("Running {} tests...", self.tests.len());

        for (name, test_fn) in &self.tests {
            // Run setup functions
            for setup in &self.setup_functions {
                setup();
            }

            let mut context = TestContext::new(name.clone());
            context.start_timer();

            // Run the test
            test_fn(&mut context);

            context.stop_timer();

            // Update results
            results.total += 1;
            if context.passed {
                results.passed += 1;
                println!("✓ {}", name);
            } else if context.failed {
                results.failed += 1;
                results.failed_tests.push(name.clone());
                if let Some(msg) = &context.error_message {
                    println!("✗ {} - {}", name, msg);
                } else {
                    println!("✗ {}", name);
                }
            } else if context.skipped {
                results.skipped += 1;
                if let Some(reason) = &context.error_message {
                    println!("- {} (skipped: {})", name, reason);
                } else {
                    println!("- {} (skipped)", name);
                }
            }

            // Run teardown functions
            for teardown in &self.teardown_functions {
                teardown();
            }
        }

        results.duration = start_time.elapsed();
        results
    }

    /// Run all benchmarks
    pub fn run_benchmarks(&self) -> Vec<BenchmarkContext> {
        let mut results = Vec::new();

        println!("Running {} benchmarks...", self.benchmarks.len());

        for (name, bench_fn) in &self.benchmarks {
            let mut context = BenchmarkContext::new(name.clone());
            
            // Run benchmark multiple times to get accurate measurements
            let start_time = Instant::now();
            let mut iterations = 0u64;
            
            // Run for at least 1 second or 1000 iterations, whichever comes first
            while start_time.elapsed() < Duration::from_secs(1) && iterations < 1000 {
                bench_fn(&mut context);
                iterations += 1;
            }
            
            context.iterations = iterations;
            context.total_duration = start_time.elapsed();
            
            println!("Benchmark {}: {} ops/sec, {:.2} ns/op", 
                name, 
                context.ops_per_sec() as u64,
                context.ns_per_op()
            );
            
            results.push(context);
        }

        results
    }
}

/// Assertion functions for tests

/// Assert that a condition is true
pub fn assert(condition: bool, message: &str) -> Result<()> {
    if !condition {
        return Err(format!("Assertion failed: {}", message).into());
    }
    Ok(())
}

/// Assert that two values are equal
pub fn assert_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T, message: &str) -> Result<()> {
    if left != right {
        return Err(format!("Assertion failed: {} - Expected {:?}, got {:?}", message, right, left).into());
    }
    Ok(())
}

/// Assert that two values are not equal
pub fn assert_ne<T: PartialEq + std::fmt::Debug>(left: T, right: T, message: &str) -> Result<()> {
    if left == right {
        return Err(format!("Assertion failed: {} - Values should not be equal: {:?}", message, left).into());
    }
    Ok(())
}

/// Assert that a value is null/None
pub fn assert_null<T>(value: Option<T>, message: &str) -> Result<()> {
    if value.is_some() {
        return Err(format!("Assertion failed: {} - Expected null, got Some(_)", message).into());
    }
    Ok(())
}

/// Assert that a value is not null/None
pub fn assert_not_null<T>(value: Option<T>, message: &str) -> Result<()> {
    if value.is_none() {
        return Err(format!("Assertion failed: {} - Expected non-null value, got None", message).into());
    }
    Ok(())
}

/// Assert that a floating point value is close to expected (within epsilon)
pub fn assert_float_eq(left: f64, right: f64, epsilon: f64, message: &str) -> Result<()> {
    if (left - right).abs() > epsilon {
        return Err(format!("Assertion failed: {} - Expected {}, got {} (difference: {})", 
            message, right, left, (left - right).abs()).into());
    }
    Ok(())
}

/// Assert that a function panics
pub fn assert_panics<F>(f: F, message: &str) -> Result<()>
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let result = std::panic::catch_unwind(f);
    if result.is_ok() {
        return Err(format!("Assertion failed: {} - Expected panic, but function completed normally", message).into());
    }
    Ok(())
}

/// Built-in functions that will be available in Bulu test code

/// Create a new test context (called from Bulu code)
pub fn create_test_context(name: String) -> TestContext {
    TestContext::new(name)
}

/// Create a new benchmark context (called from Bulu code)
pub fn create_benchmark_context(name: String) -> BenchmarkContext {
    BenchmarkContext::new(name)
}

/// Print test results summary
pub fn print_test_summary(results: &TestResults) {
    println!("\nTest Results:");
    println!("=============");
    println!("Total: {}", results.total);
    println!("Passed: {} ({:.1}%)", results.passed, results.success_rate());
    println!("Failed: {}", results.failed);
    println!("Skipped: {}", results.skipped);
    println!("Duration: {:.2}s", results.duration.as_secs_f64());
    
    if !results.failed_tests.is_empty() {
        println!("\nFailed tests:");
        for test in &results.failed_tests {
            println!("  - {}", test);
        }
    }
    
    if results.failed > 0 {
        println!("\n❌ Tests failed");
    } else {
        println!("\n✅ All tests passed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_functions() {
        assert!(assert(true, "should pass").is_ok());
        assert!(assert(false, "should fail").is_err());
        
        assert!(assert_eq(1, 1, "equal values").is_ok());
        assert!(assert_eq(1, 2, "unequal values").is_err());
        
        assert!(assert_ne(1, 2, "different values").is_ok());
        assert!(assert_ne(1, 1, "same values").is_err());
        
        assert!(assert_null(None::<i32>, "null value").is_ok());
        assert!(assert_null(Some(1), "non-null value").is_err());
        
        assert!(assert_not_null(Some(1), "non-null value").is_ok());
        assert!(assert_not_null(None::<i32>, "null value").is_err());
        
        assert!(assert_float_eq(1.0, 1.0001, 0.001, "close floats").is_ok());
        assert!(assert_float_eq(1.0, 1.1, 0.001, "distant floats").is_err());
    }

    #[test]
    fn test_context_operations() {
        let mut ctx = TestContext::new("test".to_string());
        assert!(!ctx.passed);
        assert!(!ctx.failed);
        
        ctx.pass();
        assert!(ctx.passed);
        assert!(!ctx.failed);
        
        ctx.fail("test error".to_string());
        assert!(!ctx.passed);
        assert!(ctx.failed);
        assert_eq!(ctx.error_message, Some("test error".to_string()));
    }

    #[test]
    fn test_benchmark_context() {
        let mut ctx = BenchmarkContext::new("bench".to_string());
        ctx.iterations = 1000;
        ctx.total_duration = Duration::from_millis(100);
        
        assert!(ctx.ops_per_sec() > 0.0);
        assert!(ctx.ns_per_op() > 0.0);
    }
}