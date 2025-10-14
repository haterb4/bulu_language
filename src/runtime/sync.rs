//! Synchronization primitives for the Bulu language
//!
//! This module provides thread-safe synchronization primitives including:
//! - Mutex locks with acquire()/release() methods
//! - Block syntax for automatic lock management
//! - Atomic operations for basic types
//! - Sleep and yield functions

use crate::error::{BuluError, Result};
use crate::types::primitive::RuntimeValue;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Unique identifier for locks
pub type LockId = usize;

/// A mutual exclusion lock for the Bulu language
#[derive(Debug, Clone)]
pub struct Lock {
    id: LockId,
    inner: Arc<StdMutex<()>>,
    created_at: Instant,
}

impl Lock {
    /// Create a new lock with the given ID
    pub fn new(id: LockId) -> Self {
        Self {
            id,
            inner: Arc::new(StdMutex::new(())),
            created_at: Instant::now(),
        }
    }

    /// Get the lock ID
    pub fn id(&self) -> LockId {
        self.id
    }

    /// Acquire the lock (blocking)
    pub fn acquire(&self) -> Result<LockGuard<'_>> {
        match self.inner.lock() {
            Ok(guard) => Ok(LockGuard {
                _guard: guard,
                lock_id: self.id,
                acquired_at: Instant::now(),
            }),
            Err(_) => Err(BuluError::RuntimeError {
            file: None,
                message: format!("Failed to acquire lock {} (poisoned)", self.id),
            }),
        }
    }

    /// Try to acquire the lock (non-blocking)
    pub fn try_acquire(&self) -> Result<Option<LockGuard<'_>>> {
        match self.inner.try_lock() {
            Ok(guard) => Ok(Some(LockGuard {
                _guard: guard,
                lock_id: self.id,
                acquired_at: Instant::now(),
            })),
            Err(std::sync::TryLockError::WouldBlock) => Ok(None),
            Err(std::sync::TryLockError::Poisoned(_)) => Err(BuluError::RuntimeError {
            file: None,
                message: format!("Failed to acquire lock {} (poisoned)", self.id),
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

    /// Release method for explicit release (used in Bulu code)
    /// Note: This is mainly for API compatibility - the actual release
    /// happens when the LockGuard is dropped
    pub fn release(&self) -> Result<()> {
        // In the actual implementation, we would need to track which thread
        // holds the lock and only allow that thread to release it.
        // For now, this is a no-op since Rust's RAII handles the release.
        Ok(())
    }
}

/// RAII guard for a lock
pub struct LockGuard<'a> {
    _guard: std::sync::MutexGuard<'a, ()>,
    lock_id: LockId,
    acquired_at: Instant,
}

impl<'a> LockGuard<'a> {
    /// Get the lock ID this guard is for
    pub fn lock_id(&self) -> LockId {
        self.lock_id
    }

    /// Get the time when this lock was acquired
    pub fn acquired_at(&self) -> Instant {
        self.acquired_at
    }
}

/// Registry for managing locks
#[derive(Debug)]
pub struct LockRegistry {
    locks: HashMap<LockId, Lock>,
    next_id: LockId,
}

impl LockRegistry {
    /// Create a new lock registry
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new lock and return its ID
    pub fn create_lock(&mut self) -> LockId {
        let id = self.next_id;
        self.next_id += 1;
        
        let lock = Lock::new(id);
        self.locks.insert(id, lock);
        
        id
    }

    /// Get a lock by ID
    pub fn get_lock(&self, id: LockId) -> Option<&Lock> {
        self.locks.get(&id)
    }

    /// Remove a lock from the registry
    pub fn remove_lock(&mut self, id: LockId) -> Option<Lock> {
        self.locks.remove(&id)
    }

    /// Get the number of locks in the registry
    pub fn len(&self) -> usize {
        self.locks.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.locks.is_empty()
    }
}

/// Atomic operations for basic types
pub struct AtomicOperations;

impl AtomicOperations {
    /// Atomic load operation
    pub fn atomic_load(value: &RuntimeValue) -> Result<RuntimeValue> {
        match value {
            RuntimeValue::Int32(ptr) => {
                // In a real implementation, this would load from an atomic pointer
                // For now, we'll simulate it by returning the value
                Ok(RuntimeValue::Int32(*ptr))
            }
            RuntimeValue::Int64(ptr) => {
                Ok(RuntimeValue::Int64(*ptr))
            }
            RuntimeValue::UInt32(ptr) => {
                Ok(RuntimeValue::UInt32(*ptr))
            }
            RuntimeValue::UInt64(ptr) => {
                Ok(RuntimeValue::UInt64(*ptr))
            }
            RuntimeValue::Bool(ptr) => {
                Ok(RuntimeValue::Bool(*ptr))
            }
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: format!("Atomic operations not supported for type: {:?}", value.get_type()),
            }),
        }
    }

    /// Atomic store operation
    pub fn atomic_store(target: &mut RuntimeValue, value: RuntimeValue) -> Result<()> {
        match (target, value) {
            (RuntimeValue::Int32(target), RuntimeValue::Int32(val)) => {
                *target = val;
                Ok(())
            }
            (RuntimeValue::Int64(target), RuntimeValue::Int64(val)) => {
                *target = val;
                Ok(())
            }
            (RuntimeValue::UInt32(target), RuntimeValue::UInt32(val)) => {
                *target = val;
                Ok(())
            }
            (RuntimeValue::UInt64(target), RuntimeValue::UInt64(val)) => {
                *target = val;
                Ok(())
            }
            (RuntimeValue::Bool(target), RuntimeValue::Bool(val)) => {
                *target = val;
                Ok(())
            }
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Atomic store requires matching types".to_string(),
            }),
        }
    }

    /// Atomic compare-and-swap operation
    pub fn atomic_compare_and_swap(
        target: &mut RuntimeValue,
        expected: RuntimeValue,
        desired: RuntimeValue,
    ) -> Result<RuntimeValue> {
        match (target, expected, desired) {
            (RuntimeValue::Int32(target), RuntimeValue::Int32(exp), RuntimeValue::Int32(des)) => {
                let old_value = *target;
                if old_value == exp {
                    *target = des;
                }
                Ok(RuntimeValue::Int32(old_value))
            }
            (RuntimeValue::Int64(target), RuntimeValue::Int64(exp), RuntimeValue::Int64(des)) => {
                let old_value = *target;
                if old_value == exp {
                    *target = des;
                }
                Ok(RuntimeValue::Int64(old_value))
            }
            (RuntimeValue::UInt32(target), RuntimeValue::UInt32(exp), RuntimeValue::UInt32(des)) => {
                let old_value = *target;
                if old_value == exp {
                    *target = des;
                }
                Ok(RuntimeValue::UInt32(old_value))
            }
            (RuntimeValue::UInt64(target), RuntimeValue::UInt64(exp), RuntimeValue::UInt64(des)) => {
                let old_value = *target;
                if old_value == exp {
                    *target = des;
                }
                Ok(RuntimeValue::UInt64(old_value))
            }
            (RuntimeValue::Bool(target), RuntimeValue::Bool(exp), RuntimeValue::Bool(des)) => {
                let old_value = *target;
                if old_value == exp {
                    *target = des;
                }
                Ok(RuntimeValue::Bool(old_value))
            }
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Atomic compare-and-swap requires matching types".to_string(),
            }),
        }
    }

    /// Atomic add operation (returns old value)
    pub fn atomic_add(target: &mut RuntimeValue, value: RuntimeValue) -> Result<RuntimeValue> {
        match (target, value) {
            (RuntimeValue::Int32(target), RuntimeValue::Int32(val)) => {
                let old_value = *target;
                *target = target.wrapping_add(val);
                Ok(RuntimeValue::Int32(old_value))
            }
            (RuntimeValue::Int64(target), RuntimeValue::Int64(val)) => {
                let old_value = *target;
                *target = target.wrapping_add(val);
                Ok(RuntimeValue::Int64(old_value))
            }
            (RuntimeValue::UInt32(target), RuntimeValue::UInt32(val)) => {
                let old_value = *target;
                *target = target.wrapping_add(val);
                Ok(RuntimeValue::UInt32(old_value))
            }
            (RuntimeValue::UInt64(target), RuntimeValue::UInt64(val)) => {
                let old_value = *target;
                *target = target.wrapping_add(val);
                Ok(RuntimeValue::UInt64(old_value))
            }
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Atomic add requires numeric types".to_string(),
            }),
        }
    }

    /// Atomic subtract operation (returns old value)
    pub fn atomic_sub(target: &mut RuntimeValue, value: RuntimeValue) -> Result<RuntimeValue> {
        match (target, value) {
            (RuntimeValue::Int32(target), RuntimeValue::Int32(val)) => {
                let old_value = *target;
                *target = target.wrapping_sub(val);
                Ok(RuntimeValue::Int32(old_value))
            }
            (RuntimeValue::Int64(target), RuntimeValue::Int64(val)) => {
                let old_value = *target;
                *target = target.wrapping_sub(val);
                Ok(RuntimeValue::Int64(old_value))
            }
            (RuntimeValue::UInt32(target), RuntimeValue::UInt32(val)) => {
                let old_value = *target;
                *target = target.wrapping_sub(val);
                Ok(RuntimeValue::UInt32(old_value))
            }
            (RuntimeValue::UInt64(target), RuntimeValue::UInt64(val)) => {
                let old_value = *target;
                *target = target.wrapping_sub(val);
                Ok(RuntimeValue::UInt64(old_value))
            }
            _ => Err(BuluError::RuntimeError {
            file: None,
                message: "Atomic subtract requires numeric types".to_string(),
            }),
        }
    }
}

/// Sleep the current thread for the specified number of milliseconds
pub fn sleep(milliseconds: u64) {
    std::thread::sleep(Duration::from_millis(milliseconds));
}

/// Yield execution to other threads
pub fn yield_now() {
    std::thread::yield_now();
}

/// Create a timer channel that sends a value after the specified duration
pub fn timer(milliseconds: u64) -> Result<RuntimeValue> {
    // This would create a channel that sends a value after the timeout
    // For now, we'll return a placeholder channel ID
    // In a full implementation, this would integrate with the channel system
    
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(milliseconds));
        // Send value to channel
    });
    
    // Return a channel ID (placeholder)
    Ok(RuntimeValue::Int32(0))
}

#[cfg(test)]
// Comprehensive tests are included inline

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_creation() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        
        assert_eq!(lock_id, 1);
        assert!(registry.get_lock(lock_id).is_some());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_lock_acquire_release() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = registry.get_lock(lock_id).unwrap().clone();

        // Acquire lock
        let guard = lock.acquire().unwrap();
        assert_eq!(guard.lock_id(), lock_id);

        // Try to acquire again (should fail)
        let try_result = lock.try_acquire().unwrap();
        assert!(try_result.is_none());

        // Drop the guard (releases the lock)
        drop(guard);

        // Now should be able to acquire
        let guard2 = lock.try_acquire().unwrap();
        assert!(guard2.is_some());
    }

    #[test]
    fn test_lock_timeout() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        let lock = registry.get_lock(lock_id).unwrap().clone();

        let _guard = lock.acquire().unwrap();

        let start = Instant::now();
        let result = lock.try_acquire_timeout(Duration::from_millis(50)).unwrap();
        let elapsed = start.elapsed();

        assert!(result.is_none());
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100)); // Should not take too long
    }

    #[test]
    fn test_atomic_operations() {
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
        
        // Test atomic compare-and-swap
        let old_value = AtomicOperations::atomic_compare_and_swap(
            &mut value,
            RuntimeValue::Int32(105),
            RuntimeValue::Int32(200),
        ).unwrap();
        assert_eq!(old_value, RuntimeValue::Int32(105));
        assert_eq!(value, RuntimeValue::Int32(200));
        
        // Test failed compare-and-swap
        let old_value = AtomicOperations::atomic_compare_and_swap(
            &mut value,
            RuntimeValue::Int32(999), // Wrong expected value
            RuntimeValue::Int32(300),
        ).unwrap();
        assert_eq!(old_value, RuntimeValue::Int32(200));
        assert_eq!(value, RuntimeValue::Int32(200)); // Should not change
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
    }

    #[test]
    fn test_sleep_and_yield() {
        let start = Instant::now();
        sleep(50);
        let elapsed = start.elapsed();
        
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100));
        
        // yield_now() should not panic
        yield_now();
    }

    #[test]
    fn test_multiple_locks() {
        let mut registry = LockRegistry::new();
        
        let lock1_id = registry.create_lock();
        let lock2_id = registry.create_lock();
        
        assert_ne!(lock1_id, lock2_id);
        assert_eq!(registry.len(), 2);
        
        let lock1 = registry.get_lock(lock1_id).unwrap().clone();
        let lock2 = registry.get_lock(lock2_id).unwrap().clone();
        
        // Should be able to acquire both locks independently
        let guard1 = lock1.acquire().unwrap();
        let guard2 = lock2.acquire().unwrap();
        
        assert_eq!(guard1.lock_id(), lock1_id);
        assert_eq!(guard2.lock_id(), lock2_id);
    }

    #[test]
    fn test_lock_registry_remove() {
        let mut registry = LockRegistry::new();
        let lock_id = registry.create_lock();
        
        assert_eq!(registry.len(), 1);
        
        let removed_lock = registry.remove_lock(lock_id);
        assert!(removed_lock.is_some());
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        
        // Should not be able to get the removed lock
        assert!(registry.get_lock(lock_id).is_none());
    }
}