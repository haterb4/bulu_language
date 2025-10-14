//! Integration tests for synchronization built-in functions

use bulu::runtime::builtins::*;
use bulu::types::primitive::RuntimeValue;
use bulu::error::Result;
use std::time::{Duration, Instant};

#[test]
fn test_builtin_lock() {
    let result = builtin_lock(&[]);
    assert!(result.is_ok());
    
    match result.unwrap() {
        RuntimeValue::Lock(id) => {
            assert_eq!(id, 1); // Should return lock ID 1
        }
        _ => panic!("lock() should return a Lock value"),
    }
    
    // Test with arguments (should fail)
    let result = builtin_lock(&[RuntimeValue::Int32(42)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_sleep() {
    // Test valid sleep
    let start = Instant::now();
    let result = builtin_sleep(&[RuntimeValue::Int32(50)]);
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RuntimeValue::Null);
    assert!(elapsed >= Duration::from_millis(50));
    assert!(elapsed < Duration::from_millis(100));
    
    // Test with different integer types
    let result = builtin_sleep(&[RuntimeValue::UInt64(10)]);
    assert!(result.is_ok());
    
    // Test with invalid argument count
    let result = builtin_sleep(&[]);
    assert!(result.is_err());
    
    let result = builtin_sleep(&[RuntimeValue::Int32(10), RuntimeValue::Int32(20)]);
    assert!(result.is_err());
    
    // Test with invalid argument type
    let result = builtin_sleep(&[RuntimeValue::String("invalid".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_yield() {
    let start = Instant::now();
    let result = builtin_yield(&[]);
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RuntimeValue::Null);
    assert!(elapsed < Duration::from_millis(10)); // Should return quickly
    
    // Test with arguments (should fail)
    let result = builtin_yield(&[RuntimeValue::Int32(42)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_timer() {
    // Test valid timer creation
    let result = builtin_timer(&[RuntimeValue::Int32(100)]);
    assert!(result.is_ok());
    
    // Should return some kind of channel identifier
    match result.unwrap() {
        RuntimeValue::Int32(_) => {}, // Expected
        _ => panic!("timer() should return a channel ID"),
    }
    
    // Test with different integer types
    let result = builtin_timer(&[RuntimeValue::UInt64(50)]);
    assert!(result.is_ok());
    
    // Test with invalid argument count
    let result = builtin_timer(&[]);
    assert!(result.is_err());
    
    let result = builtin_timer(&[RuntimeValue::Int32(10), RuntimeValue::Int32(20)]);
    assert!(result.is_err());
    
    // Test with invalid argument type
    let result = builtin_timer(&[RuntimeValue::String("invalid".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_atomic_load() {
    // Test valid atomic load
    let value = RuntimeValue::Int32(42);
    let result = builtin_atomic_load(&[value.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), value);
    
    // Test with different types
    let value = RuntimeValue::Bool(true);
    let result = builtin_atomic_load(&[value.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), value);
    
    // Test with invalid argument count
    let result = builtin_atomic_load(&[]);
    assert!(result.is_err());
    
    let result = builtin_atomic_load(&[RuntimeValue::Int32(10), RuntimeValue::Int32(20)]);
    assert!(result.is_err());
    
    // Test with unsupported type
    let result = builtin_atomic_load(&[RuntimeValue::String("test".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_atomic_store() {
    // Test valid atomic store
    let result = builtin_atomic_store(&[RuntimeValue::Int32(42), RuntimeValue::Int32(100)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RuntimeValue::Null);
    
    // Test with invalid argument count
    let result = builtin_atomic_store(&[RuntimeValue::Int32(42)]);
    assert!(result.is_err());
    
    let result = builtin_atomic_store(&[]);
    assert!(result.is_err());
    
    // Test with type mismatch
    let result = builtin_atomic_store(&[RuntimeValue::Int32(42), RuntimeValue::String("test".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_atomic_add() {
    // Test valid atomic add
    let result = builtin_atomic_add(&[RuntimeValue::Int32(42), RuntimeValue::Int32(10)]);
    assert!(result.is_ok());
    
    // Should return the old value
    match result.unwrap() {
        RuntimeValue::Int32(old_val) => assert_eq!(old_val, 42),
        _ => panic!("atomic_add should return the old value"),
    }
    
    // Test with different types
    let result = builtin_atomic_add(&[RuntimeValue::UInt64(100), RuntimeValue::UInt64(50)]);
    assert!(result.is_ok());
    
    // Test with invalid argument count
    let result = builtin_atomic_add(&[RuntimeValue::Int32(42)]);
    assert!(result.is_err());
    
    // Test with non-numeric types
    let result = builtin_atomic_add(&[RuntimeValue::String("test".to_string()), RuntimeValue::Int32(10)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_atomic_sub() {
    // Test valid atomic subtract
    let result = builtin_atomic_sub(&[RuntimeValue::Int32(42), RuntimeValue::Int32(10)]);
    assert!(result.is_ok());
    
    // Should return the old value
    match result.unwrap() {
        RuntimeValue::Int32(old_val) => assert_eq!(old_val, 42),
        _ => panic!("atomic_sub should return the old value"),
    }
    
    // Test with different types
    let result = builtin_atomic_sub(&[RuntimeValue::UInt32(100), RuntimeValue::UInt32(30)]);
    assert!(result.is_ok());
    
    // Test with invalid argument count
    let result = builtin_atomic_sub(&[RuntimeValue::Int32(42)]);
    assert!(result.is_err());
    
    // Test with non-numeric types
    let result = builtin_atomic_sub(&[RuntimeValue::Bool(true), RuntimeValue::Int32(10)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_atomic_cas() {
    // Test valid atomic compare-and-swap
    let result = builtin_atomic_cas(&[
        RuntimeValue::Int32(42),
        RuntimeValue::Int32(42), // Expected value
        RuntimeValue::Int32(100), // Desired value
    ]);
    assert!(result.is_ok());
    
    // Should return the old value
    match result.unwrap() {
        RuntimeValue::Int32(old_val) => assert_eq!(old_val, 42),
        _ => panic!("atomic_cas should return the old value"),
    }
    
    // Test with different types
    let result = builtin_atomic_cas(&[
        RuntimeValue::Bool(true),
        RuntimeValue::Bool(true),
        RuntimeValue::Bool(false),
    ]);
    assert!(result.is_ok());
    
    // Test with invalid argument count
    let result = builtin_atomic_cas(&[RuntimeValue::Int32(42), RuntimeValue::Int32(42)]);
    assert!(result.is_err());
    
    let result = builtin_atomic_cas(&[RuntimeValue::Int32(42)]);
    assert!(result.is_err());
    
    // Test with type mismatch
    let result = builtin_atomic_cas(&[
        RuntimeValue::Int32(42),
        RuntimeValue::String("wrong".to_string()),
        RuntimeValue::Int32(100),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_synchronization_functions_in_registry() {
    let registry = BuiltinRegistry::new();
    
    // Test that all synchronization functions are registered
    assert!(registry.is_builtin("lock"));
    assert!(registry.is_builtin("sleep"));
    assert!(registry.is_builtin("yield"));
    assert!(registry.is_builtin("timer"));
    assert!(registry.is_builtin("atomic_load"));
    assert!(registry.is_builtin("atomic_store"));
    assert!(registry.is_builtin("atomic_add"));
    assert!(registry.is_builtin("atomic_sub"));
    assert!(registry.is_builtin("atomic_cas"));
    
    // Test that we can get the functions
    assert!(registry.get("lock").is_some());
    assert!(registry.get("sleep").is_some());
    assert!(registry.get("yield").is_some());
    assert!(registry.get("timer").is_some());
    assert!(registry.get("atomic_load").is_some());
    assert!(registry.get("atomic_store").is_some());
    assert!(registry.get("atomic_add").is_some());
    assert!(registry.get("atomic_sub").is_some());
    assert!(registry.get("atomic_cas").is_some());
}

#[test]
fn test_lock_registry_access() {
    let registry = BuiltinRegistry::new();
    
    // Test that we can access the lock registry
    let lock_registry = registry.lock_registry();
    assert!(lock_registry.lock().is_ok());
    
    // Test creating a lock through the registry
    let mut lock_reg = lock_registry.lock().unwrap();
    let lock_id = lock_reg.create_lock();
    assert_eq!(lock_id, 1);
    assert_eq!(lock_reg.len(), 1);
}

#[test]
fn test_runtime_value_lock_formatting() {
    use bulu::runtime::builtins::format_runtime_value;
    
    let lock_value = RuntimeValue::Lock(42);
    let formatted = format_runtime_value(&lock_value);
    assert_eq!(formatted, "lock(42)");
}

#[test]
fn test_runtime_value_lock_type_operations() {
    let lock_value = RuntimeValue::Lock(1);
    
    // Test type checking
    assert_eq!(lock_value.get_type(), bulu::types::primitive::PrimitiveType::Any);
    
    // Test truthiness
    assert!(lock_value.is_truthy()); // Locks should always be truthy
    
    // Test sizeof
    let result = builtin_sizeof(&[lock_value.clone()]);
    assert!(result.is_ok());
    match result.unwrap() {
        RuntimeValue::Int32(size) => assert!(size > 0),
        _ => panic!("sizeof should return an int32"),
    }
    
    // Test typeof
    let result = builtin_typeof(&[lock_value.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RuntimeValue::String("lock".to_string()));
    
    // Test instanceof
    let result = builtin_instanceof(&[lock_value, RuntimeValue::String("lock".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RuntimeValue::Bool(true));
}