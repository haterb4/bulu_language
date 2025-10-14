//! Unit tests for synchronization primitives

#[cfg(test)]
mod tests {
    use super::super::sync::*;
    use crate::types::primitive::RuntimeValue;
    use std::time::{Duration, Instant};
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_lock_registry_basic_operations() {
        let mut registry = LockRegistry::new();
        
        // Test creating locks
        let lock1_id = registry.create_lock();
        let lock2_id = registry.create_lock();
        
        assert_eq!(lock1_id, 1);
        assert_eq!(lock2_id, 2);
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
        
        // Test getting locks
        assert!(registry.get_lock(lock1_id).is_some());
        assert!(registry.get_lock(lock2_id).is_some());
        assert!(registry.get_lock(999).is_none());
        
        // Test removing locks
        let removed = registry.remove_lock(lock1_id);
        assert!(removed.is_some());
        assert_eq!(registry.len(), 1);
        assert!(registry.get_lock(lock1_id).is_none());
    }

    #[test]
    fn test_lock_acquire_and_release() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = registry.get_lock(lock_id).unwrap().clone();

        // Test basic acquire
        let guard = lock.acquire().unwrap();
        assert_eq!(guard.lock_id(), lock_id);
        
        // Test that try_acquire fails when lock is held
        let try_result = lock.try_acquire().unwrap();
        assert!(try_result.is_none());
        
        // Release the lock by dropping the guard
        drop(guard);
        
        // Now try_acquire should succeed
        let guard2 = lock.try_acquire().unwrap();
        assert!(guard2.is_some());
    }

    #[test]
    fn test_lock_timeout() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = registry.get_lock(lock_id).unwrap().clone();

        // Hold the lock
        let _guard = lock.acquire().unwrap();

        // Test timeout
        let start = Instant::now();
        let result = lock.try_acquire_timeout(Duration::from_millis(50)).unwrap();
        let elapsed = start.elapsed();

        assert!(result.is_none());
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100)); // Should not take too long
    }

    #[test]
    fn test_concurrent_lock_access() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = Arc::new(registry.get_lock(lock_id).unwrap().clone());
        
        let lock1 = Arc::clone(&lock);
        let lock2 = Arc::clone(&lock);
        
        let counter = Arc::new(std::sync::Mutex::new(0));
        let counter1 = Arc::clone(&counter);
        let counter2 = Arc::clone(&counter);
        
        // Spawn two threads that try to increment a counter
        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                let _guard = lock1.acquire().unwrap();
                let mut count = counter1.lock().unwrap();
                *count += 1;
                // Small delay to increase chance of race condition without lock
                thread::sleep(Duration::from_micros(1));
            }
        });
        
        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                let _guard = lock2.acquire().unwrap();
                let mut count = counter2.lock().unwrap();
                *count += 1;
                thread::sleep(Duration::from_micros(1));
            }
        });
        
        handle1.join().unwrap();
        handle2.join().unwrap();
        
        // Counter should be exactly 200 if locking worked correctly
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 200);
    }

    #[test]
    fn test_atomic_operations_int32() {
        let mut value = RuntimeValue::Int32(42);
        
        // Test atomic load
        let loaded = AtomicOperations::atomic_load(&value).unwrap();
        assert_eq!(loaded, RuntimeValue::Int32(42));
        
        // Test atomic store
        AtomicOperations::atomic_store(&mut value, RuntimeValue::Int32(100)).unwrap();
        assert_eq!(value, RuntimeValue::Int32(100));
        
        // Test atomic add
        let old_value = AtomicOperations::atomic_add(&mut value, RuntimeValue::Int32(5)).unwrap();
        assert_eq!(old_value, RuntimeValue::Int32(100));
        assert_eq!(value, RuntimeValue::Int32(105));
        
        // Test atomic subtract
        let old_value = AtomicOperations::atomic_sub(&mut value, RuntimeValue::Int32(10)).unwrap();
        assert_eq!(old_value, RuntimeValue::Int32(105));
        assert_eq!(value, RuntimeValue::Int32(95));
        
        // Test atomic compare-and-swap (successful)
        let old_value = AtomicOperations::atomic_compare_and_swap(
            &mut value,
            RuntimeValue::Int32(95),
            RuntimeValue::Int32(200),
        ).unwrap();
        assert_eq!(old_value, RuntimeValue::Int32(95));
        assert_eq!(value, RuntimeValue::Int32(200));
        
        // Test atomic compare-and-swap (failed)
        let old_value = AtomicOperations::atomic_compare_and_swap(
            &mut value,
            RuntimeValue::Int32(999), // Wrong expected value
            RuntimeValue::Int32(300),
        ).unwrap();
        assert_eq!(old_value, RuntimeValue::Int32(200));
        assert_eq!(value, RuntimeValue::Int32(200)); // Should not change
    }

    #[test]
    fn test_atomic_operations_different_types() {
        // Test with int64
        let mut value = RuntimeValue::Int64(1000);
        let old_value = AtomicOperations::atomic_add(&mut value, RuntimeValue::Int64(500)).unwrap();
        assert_eq!(old_value, RuntimeValue::Int64(1000));
        assert_eq!(value, RuntimeValue::Int64(1500));
        
        // Test with uint32
        let mut value = RuntimeValue::UInt32(50);
        let old_value = AtomicOperations::atomic_sub(&mut value, RuntimeValue::UInt32(20)).unwrap();
        assert_eq!(old_value, RuntimeValue::UInt32(50));
        assert_eq!(value, RuntimeValue::UInt32(30));
        
        // Test with bool
        let mut value = RuntimeValue::Bool(false);
        AtomicOperations::atomic_store(&mut value, RuntimeValue::Bool(true)).unwrap();
        assert_eq!(value, RuntimeValue::Bool(true));
    }

    #[test]
    fn test_atomic_operations_type_mismatch() {
        let mut value = RuntimeValue::Int32(42);
        
        // Test atomic store with wrong type
        let result = AtomicOperations::atomic_store(&mut value, RuntimeValue::String("test".to_string()));
        assert!(result.is_err());
        
        // Test atomic add with wrong type
        let result = AtomicOperations::atomic_add(&mut value, RuntimeValue::Bool(true));
        assert!(result.is_err());
        
        // Test atomic compare-and-swap with mismatched types
        let result = AtomicOperations::atomic_compare_and_swap(
            &mut value,
            RuntimeValue::String("wrong".to_string()),
            RuntimeValue::Int32(100),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_atomic_operations_unsupported_types() {
        let value = RuntimeValue::String("test".to_string());
        
        // Atomic operations should fail on unsupported types
        let result = AtomicOperations::atomic_load(&value);
        assert!(result.is_err());
        
        let mut value = RuntimeValue::Char('a');
        let result = AtomicOperations::atomic_add(&mut value, RuntimeValue::Char('b'));
        assert!(result.is_err());
    }

    #[test]
    fn test_sleep_function() {
        let start = Instant::now();
        sleep(50); // Sleep for 50ms
        let elapsed = start.elapsed();
        
        // Should sleep for at least 50ms, but allow some tolerance
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100)); // Should not take too long
    }

    #[test]
    fn test_yield_function() {
        // yield_now() should not panic and should return quickly
        let start = Instant::now();
        yield_now();
        let elapsed = start.elapsed();
        
        // Should return very quickly (less than 10ms)
        assert!(elapsed < Duration::from_millis(10));
    }

    #[test]
    fn test_timer_function() {
        // Test that timer function returns a channel ID
        let result = timer(100);
        assert!(result.is_ok());
        
        // Should return some kind of channel identifier
        match result.unwrap() {
            RuntimeValue::Int32(_) => {}, // Expected
            _ => panic!("Timer should return a channel ID"),
        }
    }

    #[test]
    fn test_multiple_locks_independence() {
        let mut registry = LockRegistry::new();
        
        let lock1_id = registry.create_lock();
        let lock2_id = registry.create_lock();
        
        let lock1 = registry.get_lock(lock1_id).unwrap().clone();
        let lock2 = registry.get_lock(lock2_id).unwrap().clone();
        
        // Should be able to acquire both locks independently
        let guard1 = lock1.acquire().unwrap();
        let guard2 = lock2.acquire().unwrap();
        
        assert_eq!(guard1.lock_id(), lock1_id);
        assert_eq!(guard2.lock_id(), lock2_id);
        
        // Both locks should still be held
        assert!(lock1.try_acquire().unwrap().is_none());
        assert!(lock2.try_acquire().unwrap().is_none());
        
        // Drop one guard
        drop(guard1);
        
        // First lock should be available, second should still be held
        assert!(lock1.try_acquire().unwrap().is_some());
        assert!(lock2.try_acquire().unwrap().is_none());
    }

    #[test]
    fn test_lock_guard_timing() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = registry.get_lock(lock_id).unwrap().clone();

        let start = Instant::now();
        let guard = lock.acquire().unwrap();
        
        // Guard should record when it was acquired
        let acquired_time = guard.acquired_at();
        assert!(acquired_time >= start);
        assert!(acquired_time <= Instant::now());
    }

    #[test]
    fn test_atomic_operations_overflow() {
        // Test wrapping behavior for overflow
        let mut value = RuntimeValue::UInt8(255);
        let old_value = AtomicOperations::atomic_add(&mut value, RuntimeValue::UInt8(1)).unwrap();
        assert_eq!(old_value, RuntimeValue::UInt8(255));
        assert_eq!(value, RuntimeValue::UInt8(0)); // Should wrap around
        
        // Test underflow
        let mut value = RuntimeValue::UInt8(0);
        let old_value = AtomicOperations::atomic_sub(&mut value, RuntimeValue::UInt8(1)).unwrap();
        assert_eq!(old_value, RuntimeValue::UInt8(0));
        assert_eq!(value, RuntimeValue::UInt8(255)); // Should wrap around
    }

    #[test]
    fn test_lock_release_method() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = registry.get_lock(lock_id).unwrap().clone();

        // Test that release method doesn't panic
        let result = lock.release();
        assert!(result.is_ok());
        
        // Note: The actual release happens when LockGuard is dropped,
        // so this is mainly testing the API compatibility
    }
}