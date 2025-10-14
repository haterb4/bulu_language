//! Tests for concurrency features including 'run' keyword and goroutines

use bulu::ast::*;
use bulu::interpreter::{Interpreter, Value};
use bulu::lexer::token::Position;
use bulu::runtime::scheduler::{Scheduler, sync};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

fn create_test_position() -> Position {
    Position::new(1, 1, 0)
}

#[test]
fn test_run_expression_basic() {
    let scheduler = Scheduler::with_workers(1);
    let mut interpreter = Interpreter::with_scheduler(scheduler);
    
    // Create a run expression: run 42
    let run_expr = RunExpr {
        expr: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(42),
            position: create_test_position(),
        })),
        position: create_test_position(),
    };
    
    // Evaluate the run expression
    let result = interpreter.evaluate_run_expression(&run_expr).unwrap();
    
    // Should return a goroutine ID
    match result {
        Value::Goroutine(id) => {
            assert!(id > 0);
        }
        _ => panic!("Expected goroutine value, got: {:?}", result),
    }
    
    // Wait for goroutines to complete
    interpreter.wait_for_goroutines();
    
    // Check stats
    let stats = interpreter.scheduler_stats();
    assert_eq!(stats.total_goroutines, 1);
    assert_eq!(stats.completed_goroutines, 1);
    
    // Properly shutdown the scheduler
    interpreter.shutdown_scheduler();
}

#[test]
fn test_multiple_goroutines() {
    let scheduler = Scheduler::with_workers(2);
    let mut interpreter = Interpreter::with_scheduler(scheduler);
    let mut goroutine_ids = Vec::new();
    
    // Spawn multiple goroutines
    for i in 0..5 {
        let run_expr = RunExpr {
            expr: Box::new(Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(i),
                position: create_test_position(),
            })),
            position: create_test_position(),
        };
        
        let result = interpreter.evaluate_run_expression(&run_expr).unwrap();
        if let Value::Goroutine(id) = result {
            goroutine_ids.push(id);
        }
    }
    
    // All goroutines should have unique IDs
    assert_eq!(goroutine_ids.len(), 5);
    for i in 0..5 {
        for j in i+1..5 {
            assert_ne!(goroutine_ids[i], goroutine_ids[j]);
        }
    }
    
    // Wait for completion
    interpreter.wait_for_goroutines();
    
    // Check stats
    let stats = interpreter.scheduler_stats();
    assert_eq!(stats.total_goroutines, 5);
    assert_eq!(stats.completed_goroutines, 5);
    assert_eq!(stats.active_goroutines, 0);
    
    // Properly shutdown the scheduler
    interpreter.shutdown_scheduler();
}

#[test]
fn test_run_with_function_call() {
    let scheduler = Scheduler::with_workers(1);
    let mut interpreter = Interpreter::with_scheduler(scheduler);
    
    // Create a run expression with a function call: run add(1, 2)
    let run_expr = RunExpr {
        expr: Box::new(Expression::Call(CallExpr {
            callee: Box::new(Expression::Identifier(IdentifierExpr {
                name: "add".to_string(),
                position: create_test_position(),
            })),
            type_args: Vec::new(),
            args: vec![
                Expression::Literal(LiteralExpr {
                    value: LiteralValue::Integer(1),
                    position: create_test_position(),
                }),
                Expression::Literal(LiteralExpr {
                    value: LiteralValue::Integer(2),
                    position: create_test_position(),
                }),
            ],
            position: create_test_position(),
        })),
        position: create_test_position(),
    };
    
    // Evaluate the run expression
    let result = interpreter.evaluate_run_expression(&run_expr).unwrap();
    
    // Should return a goroutine ID
    match result {
        Value::Goroutine(id) => {
            assert!(id > 0);
        }
        _ => panic!("Expected goroutine value, got: {:?}", result),
    }
    
    // Wait for completion
    interpreter.wait_for_goroutines();
    
    // Properly shutdown the scheduler
    interpreter.shutdown_scheduler();
}

#[test]
fn test_scheduler_basic_functionality() {
    let scheduler = Scheduler::with_workers(2);
    
    // Test initial stats
    let stats = scheduler.stats();
    assert_eq!(stats.worker_threads, 2);
    assert_eq!(stats.total_goroutines, 0);
    assert_eq!(stats.active_goroutines, 0);
    
    // Spawn a goroutine
    let expr = Expression::Literal(LiteralExpr {
        value: LiteralValue::Integer(42),
        position: create_test_position(),
    });
    
    let id = scheduler.spawn_expression(expr);
    assert!(id > 0);
    
    // Wait for completion
    scheduler.wait_for_completion();
    
    // Check final stats
    let stats = scheduler.stats();
    assert_eq!(stats.total_goroutines, 1);
    assert_eq!(stats.active_goroutines, 0);
    assert_eq!(stats.completed_goroutines, 1);
    
    scheduler.shutdown();
}

#[test]
fn test_scheduler_concurrent_execution() {
    let scheduler = Scheduler::with_workers(4);
    let start_time = Instant::now();
    
    // Spawn multiple goroutines that should run concurrently
    for i in 0..10 {
        let expr = Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(i),
            position: create_test_position(),
        });
        scheduler.spawn_expression(expr);
    }
    
    // Wait for all to complete
    scheduler.wait_for_completion();
    let elapsed = start_time.elapsed();
    
    // Should complete relatively quickly due to parallel execution
    assert!(elapsed < Duration::from_secs(1));
    
    let stats = scheduler.stats();
    assert_eq!(stats.total_goroutines, 10);
    assert_eq!(stats.completed_goroutines, 10);
    assert_eq!(stats.active_goroutines, 0);
    
    scheduler.shutdown();
}

#[test]
fn test_lock_basic_usage() {
    let lock = sync::Lock::new();
    
    // Should be able to acquire lock
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
    assert!(elapsed < Duration::from_millis(100));
}

#[test]
fn test_lock_concurrent_access() {
    let lock = Arc::new(sync::Lock::new());
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    
    // Spawn multiple threads that increment a counter with lock protection
    for _ in 0..10 {
        let lock = Arc::clone(&lock);
        let counter = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            let _guard = lock.acquire().unwrap();
            let mut count = counter.lock().unwrap();
            *count += 1;
            // Simulate some work
            thread::sleep(Duration::from_millis(10));
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Counter should be exactly 10 (no race conditions)
    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, 10);
}

#[test]
fn test_goroutine_lifecycle() {
    let scheduler = Scheduler::with_workers(1);
    
    // Test goroutine creation and completion
    let expr = Expression::Literal(LiteralExpr {
        value: LiteralValue::String("test".to_string()),
        position: create_test_position(),
    });
    
    let id = scheduler.spawn_expression(expr);
    
    // Initially should be active
    let stats = scheduler.stats();
    assert_eq!(stats.active_goroutines, 1);
    assert_eq!(stats.completed_goroutines, 0);
    
    // Wait for completion
    scheduler.wait_for_completion();
    
    // Should now be completed
    let stats = scheduler.stats();
    assert_eq!(stats.active_goroutines, 0);
    assert_eq!(stats.completed_goroutines, 1);
    
    scheduler.shutdown();
}

#[test]
fn test_scheduler_with_many_goroutines() {
    let scheduler = Scheduler::with_workers(4);
    let num_goroutines = 100;
    
    // Spawn many goroutines
    for i in 0..num_goroutines {
        let expr = Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(i),
            position: create_test_position(),
        });
        scheduler.spawn_expression(expr);
    }
    
    // Wait for all to complete
    scheduler.wait_for_completion();
    
    let stats = scheduler.stats();
    assert_eq!(stats.total_goroutines, num_goroutines as usize);
    assert_eq!(stats.completed_goroutines, num_goroutines as usize);
    assert_eq!(stats.active_goroutines, 0);
    
    scheduler.shutdown();
}

#[test]
fn test_sync_primitives() {
    // Test yield_now (should not panic)
    sync::yield_now();
    
    // Test sleep
    let start = Instant::now();
    sync::sleep(Duration::from_millis(50));
    let elapsed = start.elapsed();
    
    assert!(elapsed >= Duration::from_millis(50));
    assert!(elapsed < Duration::from_millis(100));
}

#[test]
fn test_goroutine_error_handling() {
    let scheduler = Scheduler::with_workers(1);
    
    // Create an expression that would cause an error (division by zero)
    let expr = Expression::Binary(BinaryExpr {
        left: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(10),
            position: create_test_position(),
        })),
        operator: BinaryOperator::Divide,
        right: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(0),
            position: create_test_position(),
        })),
        position: create_test_position(),
    });
    
    let id = scheduler.spawn_expression(expr);
    assert!(id > 0);
    
    // Wait for completion
    scheduler.wait_for_completion();
    
    // Should have one failed goroutine
    let stats = scheduler.stats();
    assert_eq!(stats.total_goroutines, 1);
    assert_eq!(stats.failed_goroutines, 1);
    assert_eq!(stats.active_goroutines, 0);
    
    scheduler.shutdown();
}

#[test]
fn test_interpreter_with_custom_scheduler() {
    let scheduler = Scheduler::with_workers(2);
    let mut interpreter = Interpreter::with_scheduler(scheduler);
    
    // Test that the interpreter uses the custom scheduler
    let run_expr = RunExpr {
        expr: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(42),
            position: create_test_position(),
        })),
        position: create_test_position(),
    };
    
    let result = interpreter.evaluate_run_expression(&run_expr).unwrap();
    
    match result {
        Value::Goroutine(id) => {
            assert!(id > 0);
        }
        _ => panic!("Expected goroutine value"),
    }
    
    interpreter.wait_for_goroutines();
    
    let stats = interpreter.scheduler_stats();
    assert_eq!(stats.worker_threads, 2);
    assert_eq!(stats.total_goroutines, 1);
    assert_eq!(stats.completed_goroutines, 1);
    
    // Properly shutdown the scheduler
    interpreter.shutdown_scheduler();
}