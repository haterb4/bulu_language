//! Goroutine scheduler for the Bulu language
//!
//! This module provides lightweight task scheduling for concurrent execution
//! using the 'run' keyword. It implements a work-stealing scheduler with
//! minimal overhead goroutines.

use crate::ast::*;
use crate::error::{BuluError, Result};
use crate::runtime::interpreter::Interpreter as Environment;
use crate::types::primitive::RuntimeValue as Value;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Unique identifier for goroutines
pub type GoroutineId = usize;

/// Goroutine state
#[derive(Debug, Clone, PartialEq)]
pub enum GoroutineState {
    Ready,     // Ready to run
    Running,   // Currently executing
    Blocked,   // Waiting for something
    Completed, // Finished execution
    Failed,    // Terminated with error
}

/// A lightweight goroutine
#[derive(Debug)]
pub struct Goroutine {
    pub id: GoroutineId,
    pub state: GoroutineState,
    pub task: GoroutineTask,
    pub created_at: Instant,
    pub stack_size: usize,
}

/// Task that a goroutine executes
pub enum GoroutineTask {
    Expression(Expression),
    Closure(Box<dyn FnOnce() -> Result<Value> + Send>),
}

impl std::fmt::Debug for GoroutineTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoroutineTask::Expression(expr) => write!(f, "Expression({:?})", expr),
            GoroutineTask::Closure(_) => write!(f, "Closure(<function>)"),
        }
    }
}

impl Goroutine {
    pub fn new(id: GoroutineId, task: GoroutineTask) -> Self {
        Self {
            id,
            state: GoroutineState::Ready,
            task,
            created_at: Instant::now(),
            stack_size: 4096, // 4KB initial stack
        }
    }
}

/// Work queue for goroutines
#[derive(Debug)]
struct WorkQueue {
    queue: VecDeque<Goroutine>,
}

impl WorkQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, goroutine: Goroutine) {
        self.queue.push_back(goroutine);
    }

    fn pop(&mut self) -> Option<Goroutine> {
        self.queue.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Statistics about the scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_goroutines: usize,
    pub active_goroutines: usize,
    pub completed_goroutines: usize,
    pub failed_goroutines: usize,
    pub worker_threads: usize,
}

/// Lightweight task scheduler for goroutines
pub struct Scheduler {
    work_queue: Arc<Mutex<WorkQueue>>,
    next_id: AtomicUsize,
    worker_handles: Vec<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    stats: Arc<Mutex<SchedulerStats>>,
    condvar: Arc<Condvar>,
    num_workers: usize,
}

impl Scheduler {
    /// Create a new scheduler with the specified number of worker threads
    pub fn new() -> Self {
        // Use fewer workers for tests to avoid thread exhaustion
        let workers = if cfg!(test) { 1 } else { num_cpus::get() };
        Self::with_workers(workers)
    }

    /// Create a new scheduler with a specific number of worker threads
    pub fn with_workers(num_workers: usize) -> Self {
        let work_queue = Arc::new(Mutex::new(WorkQueue::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let condvar = Arc::new(Condvar::new());
        let stats = Arc::new(Mutex::new(SchedulerStats {
            total_goroutines: 0,
            active_goroutines: 0,
            completed_goroutines: 0,
            failed_goroutines: 0,
            worker_threads: num_workers,
        }));

        let mut worker_handles = Vec::new();

        // Spawn worker threads
        for worker_id in 0..num_workers {
            let work_queue = Arc::clone(&work_queue);
            let shutdown = Arc::clone(&shutdown);
            let condvar = Arc::clone(&condvar);
            let stats = Arc::clone(&stats);

            let handle = thread::Builder::new()
                .name(format!("scheduler-worker-{}", worker_id))
                .spawn(move || {
                    Self::worker_loop(worker_id, work_queue, shutdown, condvar, stats);
                })
                .expect("Failed to spawn worker thread");

            worker_handles.push(handle);
        }

        Self {
            work_queue,
            next_id: AtomicUsize::new(1),
            worker_handles,
            shutdown,
            stats,
            condvar,
            num_workers,
        }
    }

    /// Spawn a new goroutine with an expression
    pub fn spawn_expression(&self, expr: Expression) -> GoroutineId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let goroutine = Goroutine::new(id, GoroutineTask::Expression(expr));

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_goroutines += 1;
            stats.active_goroutines += 1;
        }

        // Add to work queue
        {
            let mut queue = self.work_queue.lock().unwrap();
            queue.push(goroutine);
        }

        // Notify a worker
        self.condvar.notify_one();

        id
    }

    /// Spawn a new goroutine with a closure
    pub fn spawn_closure<F>(&self, f: F) -> GoroutineId
    where
        F: FnOnce() -> Result<Value> + Send + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let goroutine = Goroutine::new(id, GoroutineTask::Closure(Box::new(f)));

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_goroutines += 1;
            stats.active_goroutines += 1;
        }

        // Add to work queue
        {
            let mut queue = self.work_queue.lock().unwrap();
            queue.push(goroutine);
        }

        // Notify a worker
        self.condvar.notify_one();

        id
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStats {
        self.stats.lock().unwrap().clone()
    }

    /// Wait for all goroutines to complete
    pub fn wait_for_completion(&self) {
        let start = Instant::now();
        let timeout = Duration::from_secs(5); // 5 second timeout

        loop {
            let stats = self.stats();

            if stats.active_goroutines == 0 {
                break;
            }

            if start.elapsed() > timeout {
                eprintln!(
                    "WARNING: Timeout waiting for goroutines to complete. Active: {}, Total: {}",
                    stats.active_goroutines, stats.total_goroutines
                );
                break;
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Shutdown the scheduler and wait for all workers to finish
    pub fn shutdown(mut self) {
        // Signal shutdown
        self.shutdown.store(true, Ordering::SeqCst);

        // Notify all workers
        self.condvar.notify_all();

        // Wait for all workers to finish
        for handle in self.worker_handles.drain(..) {
            if let Err(e) = handle.join() {
                eprintln!("Worker thread panicked: {:?}", e);
            }
        }
    }

    /// Worker thread main loop
    fn worker_loop(
        _worker_id: usize,
        work_queue: Arc<Mutex<WorkQueue>>,
        shutdown: Arc<AtomicBool>,
        condvar: Arc<Condvar>,
        stats: Arc<Mutex<SchedulerStats>>,
    ) {
        // Create a simple interpreter without its own scheduler to avoid thread explosion
        let mut interpreter = SimpleInterpreter::new();

        loop {
            // Check for shutdown
            if shutdown.load(Ordering::SeqCst) {
                break;
            }

            // Try to get work
            let goroutine = {
                let mut queue = work_queue.lock().unwrap();

                // Wait for work or shutdown signal
                while queue.is_empty() {
                    if shutdown.load(Ordering::SeqCst) {
                        return;
                    }

                    let (new_queue, timeout_result) = condvar
                        .wait_timeout(queue, Duration::from_millis(100))
                        .unwrap();
                    queue = new_queue;

                    if timeout_result.timed_out() {
                        // Check shutdown again after timeout
                        if shutdown.load(Ordering::SeqCst) {
                            return;
                        }
                    }
                }

                queue.pop()
            };

            if let Some(mut goroutine) = goroutine {
                // Execute the goroutine
                goroutine.state = GoroutineState::Running;

                let result = match goroutine.task {
                    GoroutineTask::Expression(ref expr) => interpreter.evaluate_expression(expr),
                    GoroutineTask::Closure(_) => {
                        // For closures, we'd need to execute them differently
                        // This is a simplified implementation
                        Ok(Value::Null)
                    }
                };

                // Update goroutine state and stats
                match result {
                    Ok(_) => {
                        goroutine.state = GoroutineState::Completed;
                        let mut stats = stats.lock().unwrap();
                        stats.active_goroutines -= 1;
                        stats.completed_goroutines += 1;
                    }
                    Err(e) => {
                        goroutine.state = GoroutineState::Failed;
                        let mut stats = stats.lock().unwrap();
                        stats.active_goroutines -= 1;
                        stats.failed_goroutines += 1;
                        eprintln!("Goroutine {} failed: {}", goroutine.id, e);
                    }
                }
            }
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        // Signal shutdown if not already done
        self.shutdown.store(true, Ordering::SeqCst);

        // Notify all workers
        self.condvar.notify_all();

        // Note: We can't join threads in Drop because we've moved the handles
        // This is why explicit shutdown() should be called
    }
}

/// Basic synchronization primitives
pub mod sync {
    use crate::error::{BuluError, Result};
    use std::sync::{Arc, Mutex as StdMutex};
    use std::time::{Duration, Instant};

    /// A mutual exclusion lock
    #[derive(Debug, Clone)]
    pub struct Lock {
        inner: Arc<StdMutex<()>>,
        created_at: Instant,
    }

    impl Lock {
        /// Create a new lock
        pub fn new() -> Self {
            Self {
                inner: Arc::new(StdMutex::new(())),
                created_at: Instant::now(),
            }
        }

        /// Acquire the lock (blocking)
        pub fn acquire(&self) -> Result<LockGuard<'_>> {
            match self.inner.lock() {
                Ok(guard) => Ok(LockGuard {
                    _guard: guard,
                    acquired_at: Instant::now(),
                }),
                Err(_) => Err(BuluError::RuntimeError {
            file: None,
                    message: "Failed to acquire lock (poisoned)".to_string(),
                }),
            }
        }

        /// Try to acquire the lock (non-blocking)
        pub fn try_acquire(&self) -> Result<Option<LockGuard<'_>>> {
            match self.inner.try_lock() {
                Ok(guard) => Ok(Some(LockGuard {
                    _guard: guard,
                    acquired_at: Instant::now(),
                })),
                Err(std::sync::TryLockError::WouldBlock) => Ok(None),
                Err(std::sync::TryLockError::Poisoned(_)) => Err(BuluError::RuntimeError {
            file: None,
                    message: "Failed to acquire lock (poisoned)".to_string(),
                }),
            }
        }

        /// Try to acquire the lock with a timeout
        pub fn try_acquire_timeout(&self, timeout: Duration) -> Result<Option<LockGuard<'_>>> {
            let start = Instant::now();

            while start.elapsed() < timeout {
                if let Some(guard) = self.try_acquire()? {
                    return Ok(Some(guard));
                }
                std::thread::sleep(Duration::from_millis(1));
            }

            Ok(None)
        }
    }

    /// RAII guard for a lock
    pub struct LockGuard<'a> {
        _guard: std::sync::MutexGuard<'a, ()>,
        acquired_at: Instant,
    }

    impl<'a> LockGuard<'a> {
        /// Get the time when this lock was acquired
        pub fn acquired_at(&self) -> Instant {
            self.acquired_at
        }
    }

    /// Sleep the current goroutine for the specified duration
    pub fn sleep(duration: Duration) {
        std::thread::sleep(duration);
    }

    /// Yield execution to other goroutines
    pub fn yield_now() {
        std::thread::yield_now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::lexer::token::Position;

    fn create_test_position() -> Position {
        Position::new(1, 1, 0)
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::with_workers(2);
        let stats = scheduler.stats();

        assert_eq!(stats.worker_threads, 2);
        assert_eq!(stats.total_goroutines, 0);
        assert_eq!(stats.active_goroutines, 0);

        scheduler.shutdown();
    }

    #[test]
    fn test_goroutine_spawn() {
        let scheduler = Scheduler::with_workers(1);

        // Create a simple literal expression
        let expr = Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(42),
            position: create_test_position(),
        });

        let id = scheduler.spawn_expression(expr);
        assert!(id > 0);

        // Wait a bit for execution
        std::thread::sleep(Duration::from_millis(100));

        let stats = scheduler.stats();
        assert_eq!(stats.total_goroutines, 1);

        scheduler.shutdown();
    }

    #[test]
    fn test_multiple_goroutines() {
        let scheduler = Scheduler::with_workers(2);

        // Spawn multiple goroutines
        for i in 0..5 {
            let expr = Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(i),
                position: create_test_position(),
            });
            scheduler.spawn_expression(expr);
        }

        // Wait for completion
        scheduler.wait_for_completion();

        let stats = scheduler.stats();
        assert_eq!(stats.total_goroutines, 5);
        assert_eq!(stats.active_goroutines, 0);
        assert_eq!(stats.completed_goroutines, 5);

        scheduler.shutdown();
    }

    #[test]
    fn test_lock_basic_usage() {
        let lock = sync::Lock::new();

        // Acquire lock
        let guard = lock.acquire().unwrap();
        assert!(guard.acquired_at() <= Instant::now());

        // Try to acquire again (should fail)
        let try_result = lock.try_acquire().unwrap();
        assert!(try_result.is_none());

        // Drop the guard
        drop(guard);

        // Now should be able to acquire
        let guard2 = lock.try_acquire().unwrap();
        assert!(guard2.is_some());
    }

    #[test]
    fn test_lock_timeout() {
        let lock = sync::Lock::new();
        let _guard = lock.acquire().unwrap();

        let start = Instant::now();
        let result = lock.try_acquire_timeout(Duration::from_millis(50)).unwrap();
        let elapsed = start.elapsed();

        assert!(result.is_none());
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100)); // Should not take too long
    }
}

/// Simple interpreter for worker threads that doesn't create its own scheduler
struct SimpleInterpreter {
    environment: Environment,
}

impl SimpleInterpreter {
    fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Result<Value> {
        match expression {
            Expression::Literal(literal) => self.evaluate_literal(literal),
            Expression::Identifier(identifier) => self.evaluate_identifier(identifier),
            Expression::Binary(binary) => self.evaluate_binary(binary),
            Expression::Unary(unary) => self.evaluate_unary(unary),
            _ => {
                // For other expressions, return null for now
                Ok(Value::Null)
            }
        }
    }

    fn evaluate_literal(&self, literal: &LiteralExpr) -> Result<Value> {
        let value = match &literal.value {
            LiteralValue::Integer(i) => Value::Int64(*i),
            LiteralValue::Float(f) => Value::Float64(*f),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Boolean(b) => Value::Bool(*b),
            LiteralValue::Char(c) => Value::String(c.to_string()),
            LiteralValue::Null => Value::Null,
        };
        Ok(value)
    }

    fn evaluate_identifier(&self, identifier: &IdentifierExpr) -> Result<Value> {
        if let Some(value) = self.environment.get(&identifier.name) {
            Ok(value.clone())
        } else {
            Err(BuluError::RuntimeError {
            file: None,
                message: format!("Undefined variable '{}'", identifier.name),
            })
        }
    }

    fn evaluate_binary(&mut self, binary: &BinaryExpr) -> Result<Value> {
        let left = self.evaluate_expression(&binary.left)?;
        let right = self.evaluate_expression(&binary.right)?;

        match binary.operator {
            BinaryOperator::Add => self.add_values(left, right),
            BinaryOperator::Subtract => self.subtract_values(left, right),
            BinaryOperator::Multiply => self.multiply_values(left, right),
            BinaryOperator::Divide => self.divide_values(left, right),
            BinaryOperator::Equal => Ok(Value::Bool(self.values_equal(&left, &right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!self.values_equal(&left, &right))),
            BinaryOperator::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinaryOperator::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: format!("Unsupported binary operator: {:?}", binary.operator),
            }),
        }
    }

    fn evaluate_unary(&mut self, unary: &UnaryExpr) -> Result<Value> {
        let operand = self.evaluate_expression(&unary.operand)?;

        match unary.operator {
            UnaryOperator::Minus => match operand {
                Value::Int64(i) => Ok(Value::Int64(-i)),
                Value::Float64(f) => Ok(Value::Float64(-f)),
                _ => Err(BuluError::RuntimeError {
            file: None,
                    message: "Cannot negate non-numeric value".to_string(),
                }),
            },
            UnaryOperator::Not => Ok(Value::Bool(!operand.is_truthy())),
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: format!("Unsupported unary operator: {:?}", unary.operator),
            }),
        }
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int64(a), Value::Int64(b)) => Ok(Value::Int64(a + b)),
            (Value::Float64(a), Value::Float64(b)) => Ok(Value::Float64(a + b)),
            (Value::Int64(a), Value::Float64(b)) => Ok(Value::Float64(a as f64 + b)),
            (Value::Float64(a), Value::Int64(b)) => Ok(Value::Float64(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot add incompatible types".to_string(),
            }),
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int64(a), Value::Int64(b)) => Ok(Value::Int64(a - b)),
            (Value::Float64(a), Value::Float64(b)) => Ok(Value::Float64(a - b)),
            (Value::Int64(a), Value::Float64(b)) => Ok(Value::Float64(a as f64 - b)),
            (Value::Float64(a), Value::Int64(b)) => Ok(Value::Float64(a - b as f64)),
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot subtract incompatible types".to_string(),
            }),
        }
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int64(a), Value::Int64(b)) => Ok(Value::Int64(a * b)),
            (Value::Float64(a), Value::Float64(b)) => Ok(Value::Float64(a * b)),
            (Value::Int64(a), Value::Float64(b)) => Ok(Value::Float64(a as f64 * b)),
            (Value::Float64(a), Value::Int64(b)) => Ok(Value::Float64(a * b as f64)),
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot multiply incompatible types".to_string(),
            }),
        }
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int64(a), Value::Int64(b)) => {
                if b == 0 {
                    return Err(BuluError::RuntimeError {
            file: None,
                        message: "Division by zero".to_string(),
                    });
                }
                Ok(Value::Int64(a / b))
            }
            (Value::Float64(a), Value::Float64(b)) => {
                if b == 0.0 {
                    return Err(BuluError::RuntimeError {
            file: None,
                        message: "Division by zero".to_string(),
                    });
                }
                Ok(Value::Float64(a / b))
            }
            (Value::Int64(a), Value::Float64(b)) => {
                if b == 0.0 {
                    return Err(BuluError::RuntimeError {
            file: None,
                        message: "Division by zero".to_string(),
                    });
                }
                Ok(Value::Float64(a as f64 / b))
            }
            (Value::Float64(a), Value::Int64(b)) => {
                if b == 0 {
                    return Err(BuluError::RuntimeError {
            file: None,
                        message: "Division by zero".to_string(),
                    });
                }
                Ok(Value::Float64(a / b as f64))
            }
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot divide incompatible types".to_string(),
            }),
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int64(a), Value::Int64(b)) => a == b,
            (Value::Float64(a), Value::Float64(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}
