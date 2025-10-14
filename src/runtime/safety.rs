//! Memory safety checks and runtime validation
//!
//! This module provides runtime memory safety checks including:
//! - Bounds checking for arrays and slices
//! - Null pointer dereference prevention
//! - Stack overflow detection
//! - Buffer overflow prevention

use std::sync::atomic::{AtomicUsize, Ordering};

/// Maximum stack size per thread (8MB default)
const DEFAULT_MAX_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Stack overflow detection threshold (1MB before limit)
const STACK_OVERFLOW_THRESHOLD: usize = 1024 * 1024;

/// Global stack size limit
static MAX_STACK_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_STACK_SIZE);

/// Memory safety error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyError {
    /// Array/slice index out of bounds
    IndexOutOfBounds {
        index: usize,
        length: usize,
        type_name: String,
    },
    /// Null pointer dereference attempt
    NullPointerDereference {
        operation: String,
        location: String,
    },
    /// Stack overflow detected
    StackOverflow {
        current_size: usize,
        max_size: usize,
    },
    /// Buffer overflow attempt
    BufferOverflow {
        attempted_size: usize,
        buffer_size: usize,
        operation: String,
    },
    /// Invalid memory access
    InvalidMemoryAccess {
        address: usize,
        operation: String,
    },
}

impl std::fmt::Display for SafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyError::IndexOutOfBounds { index, length, type_name } => {
                write!(f, "Index out of bounds: index {} is out of range for {} with length {}", 
                       index, type_name, length)
            }
            SafetyError::NullPointerDereference { operation, location } => {
                write!(f, "Null pointer dereference in {} at {}", operation, location)
            }
            SafetyError::StackOverflow { current_size, max_size } => {
                write!(f, "Stack overflow: current size {} bytes exceeds maximum {} bytes", 
                       current_size, max_size)
            }
            SafetyError::BufferOverflow { attempted_size, buffer_size, operation } => {
                write!(f, "Buffer overflow in {}: attempted to access {} bytes in buffer of {} bytes", 
                       operation, attempted_size, buffer_size)
            }
            SafetyError::InvalidMemoryAccess { address, operation } => {
                write!(f, "Invalid memory access in {}: address 0x{:x}", operation, address)
            }
        }
    }
}

impl std::error::Error for SafetyError {}

/// Result type for memory safety operations
pub type SafetyResult<T> = Result<T, SafetyError>;

/// Memory safety checker
#[derive(Debug, Clone)]
pub struct SafetyChecker {
    /// Whether bounds checking is enabled
    bounds_checking_enabled: bool,
    /// Whether null pointer checking is enabled
    null_checking_enabled: bool,
    /// Whether stack overflow checking is enabled
    stack_checking_enabled: bool,
}

impl SafetyChecker {
    /// Create a new safety checker with all checks enabled
    pub fn new() -> Self {
        Self {
            bounds_checking_enabled: true,
            null_checking_enabled: true,
            stack_checking_enabled: true,
        }
    }

    /// Create a safety checker with custom settings
    pub fn with_settings(bounds: bool, null: bool, stack: bool) -> Self {
        Self {
            bounds_checking_enabled: bounds,
            null_checking_enabled: null,
            stack_checking_enabled: stack,
        }
    }

    /// Check array/slice bounds access
    pub fn check_bounds(&self, index: usize, length: usize, type_name: &str) -> SafetyResult<()> {
        if !self.bounds_checking_enabled {
            return Ok(());
        }

        if index >= length {
            return Err(SafetyError::IndexOutOfBounds {
                index,
                length,
                type_name: type_name.to_string(),
            });
        }

        Ok(())
    }

    /// Check slice bounds for range access
    pub fn check_slice_bounds(&self, start: usize, end: usize, length: usize, type_name: &str) -> SafetyResult<()> {
        if !self.bounds_checking_enabled {
            return Ok(());
        }

        if start > length {
            return Err(SafetyError::IndexOutOfBounds {
                index: start,
                length,
                type_name: format!("{} slice start", type_name),
            });
        }

        if end > length {
            return Err(SafetyError::IndexOutOfBounds {
                index: end,
                length,
                type_name: format!("{} slice end", type_name),
            });
        }

        if start > end {
            return Err(SafetyError::IndexOutOfBounds {
                index: start,
                length: end,
                type_name: format!("{} slice range", type_name),
            });
        }

        Ok(())
    }

    /// Check for null pointer dereference
    pub fn check_null_pointer<T>(&self, ptr: *const T, operation: &str, location: &str) -> SafetyResult<()> {
        if !self.null_checking_enabled {
            return Ok(());
        }

        if ptr.is_null() {
            return Err(SafetyError::NullPointerDereference {
                operation: operation.to_string(),
                location: location.to_string(),
            });
        }

        Ok(())
    }

    /// Check for null pointer dereference (mutable)
    pub fn check_null_pointer_mut<T>(&self, ptr: *mut T, operation: &str, location: &str) -> SafetyResult<()> {
        if !self.null_checking_enabled {
            return Ok(());
        }

        if ptr.is_null() {
            return Err(SafetyError::NullPointerDereference {
                operation: operation.to_string(),
                location: location.to_string(),
            });
        }

        Ok(())
    }

    /// Check for buffer overflow
    pub fn check_buffer_access(&self, offset: usize, size: usize, buffer_size: usize, operation: &str) -> SafetyResult<()> {
        if !self.bounds_checking_enabled {
            return Ok(());
        }

        if offset.saturating_add(size) > buffer_size {
            return Err(SafetyError::BufferOverflow {
                attempted_size: offset + size,
                buffer_size,
                operation: operation.to_string(),
            });
        }

        Ok(())
    }

    /// Check stack overflow
    pub fn check_stack_overflow(&self) -> SafetyResult<()> {
        if !self.stack_checking_enabled {
            return Ok(());
        }

        let current_stack_size = estimate_stack_usage();
        let max_size = MAX_STACK_SIZE.load(Ordering::Relaxed);

        if current_stack_size > max_size.saturating_sub(STACK_OVERFLOW_THRESHOLD) {
            return Err(SafetyError::StackOverflow {
                current_size: current_stack_size,
                max_size,
            });
        }

        Ok(())
    }

    /// Enable or disable bounds checking
    pub fn set_bounds_checking(&mut self, enabled: bool) {
        self.bounds_checking_enabled = enabled;
    }

    /// Enable or disable null pointer checking
    pub fn set_null_checking(&mut self, enabled: bool) {
        self.null_checking_enabled = enabled;
    }

    /// Enable or disable stack overflow checking
    pub fn set_stack_checking(&mut self, enabled: bool) {
        self.stack_checking_enabled = enabled;
    }

    /// Get current settings
    pub fn get_settings(&self) -> (bool, bool, bool) {
        (self.bounds_checking_enabled, self.null_checking_enabled, self.stack_checking_enabled)
    }
}

impl Default for SafetyChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Safe array access with bounds checking
pub fn safe_array_get<T>(array: &[T], index: usize) -> SafetyResult<&T> {
    let checker = SafetyChecker::new();
    checker.check_bounds(index, array.len(), "array")?;
    Ok(&array[index])
}

/// Safe array access with bounds checking (mutable)
pub fn safe_array_get_mut<T>(array: &mut [T], index: usize) -> SafetyResult<&mut T> {
    let checker = SafetyChecker::new();
    checker.check_bounds(index, array.len(), "array")?;
    Ok(&mut array[index])
}

/// Safe slice creation with bounds checking
pub fn safe_slice<T>(array: &[T], start: usize, end: usize) -> SafetyResult<&[T]> {
    let checker = SafetyChecker::new();
    checker.check_slice_bounds(start, end, array.len(), "array")?;
    Ok(&array[start..end])
}

/// Safe slice creation with bounds checking (mutable)
pub fn safe_slice_mut<T>(array: &mut [T], start: usize, end: usize) -> SafetyResult<&mut [T]> {
    let checker = SafetyChecker::new();
    checker.check_slice_bounds(start, end, array.len(), "array")?;
    Ok(&mut array[start..end])
}

/// Safe pointer dereference with null checking
pub unsafe fn safe_deref<'a, T>(ptr: *const T, operation: &str, location: &str) -> SafetyResult<&'a T> {
    let checker = SafetyChecker::new();
    checker.check_null_pointer(ptr, operation, location)?;
    Ok(&*ptr)
}

/// Safe pointer dereference with null checking (mutable)
pub unsafe fn safe_deref_mut<'a, T>(ptr: *mut T, operation: &str, location: &str) -> SafetyResult<&'a mut T> {
    let checker = SafetyChecker::new();
    checker.check_null_pointer_mut(ptr, operation, location)?;
    Ok(&mut *ptr)
}

/// Set maximum stack size for all threads
pub fn set_max_stack_size(size: usize) {
    MAX_STACK_SIZE.store(size, Ordering::Relaxed);
}

/// Get current maximum stack size
pub fn get_max_stack_size() -> usize {
    MAX_STACK_SIZE.load(Ordering::Relaxed)
}

/// Estimate current stack usage (platform-specific approximation)
fn estimate_stack_usage() -> usize {
    // This is a rough approximation - in a real implementation,
    // we would use platform-specific APIs to get accurate stack usage
    let stack_var = 0u8;
    let stack_ptr = &stack_var as *const u8 as usize;
    
    // Get the thread's stack bounds (simplified)
    thread_local! {
        static STACK_BASE: usize = {
            // In a real implementation, we would get the actual stack base
            // from the OS. For now, we'll use a rough estimate.
            let current_stack = &0u8 as *const u8 as usize;
            current_stack
        };
    }
    
    STACK_BASE.with(|base| {
        if stack_ptr < *base {
            base - stack_ptr
        } else {
            stack_ptr - base
        }
    })
}

/// Runtime bounds checking for array access
#[macro_export]
macro_rules! bounds_check {
    ($array:expr, $index:expr) => {
        if $index >= $array.len() {
            panic!("Index out of bounds: index {} is out of range for array with length {}", 
                   $index, $array.len());
        }
    };
}

/// Runtime bounds checking for slice access
#[macro_export]
macro_rules! slice_bounds_check {
    ($array:expr, $start:expr, $end:expr) => {
        if $start > $array.len() || $end > $array.len() || $start > $end {
            panic!("Slice bounds out of range: [{}..{}] for array with length {}", 
                   $start, $end, $array.len());
        }
    };
}

/// Runtime null pointer check
#[macro_export]
macro_rules! null_check {
    ($ptr:expr, $op:expr) => {
        if $ptr.is_null() {
            panic!("Null pointer dereference in operation: {}", $op);
        }
    };
}

/// Stack overflow check
#[macro_export]
macro_rules! stack_check {
    () => {
        let checker = $crate::runtime::safety::SafetyChecker::new();
        if let Err(e) = checker.check_stack_overflow() {
            panic!("Stack overflow detected: {}", e);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_checking() {
        let checker = SafetyChecker::new();
        
        // Valid bounds
        assert!(checker.check_bounds(0, 5, "test_array").is_ok());
        assert!(checker.check_bounds(4, 5, "test_array").is_ok());
        
        // Invalid bounds
        assert!(checker.check_bounds(5, 5, "test_array").is_err());
        assert!(checker.check_bounds(10, 5, "test_array").is_err());
    }

    #[test]
    fn test_slice_bounds_checking() {
        let checker = SafetyChecker::new();
        
        // Valid slice bounds
        assert!(checker.check_slice_bounds(0, 5, 10, "test_array").is_ok());
        assert!(checker.check_slice_bounds(2, 7, 10, "test_array").is_ok());
        assert!(checker.check_slice_bounds(0, 0, 10, "test_array").is_ok());
        
        // Invalid slice bounds
        assert!(checker.check_slice_bounds(0, 11, 10, "test_array").is_err());
        assert!(checker.check_slice_bounds(11, 15, 10, "test_array").is_err());
        assert!(checker.check_slice_bounds(5, 3, 10, "test_array").is_err());
    }

    #[test]
    fn test_null_pointer_checking() {
        let checker = SafetyChecker::new();
        
        // Valid pointer
        let value = 42;
        let ptr = &value as *const i32;
        assert!(checker.check_null_pointer(ptr, "test", "test_location").is_ok());
        
        // Null pointer
        let null_ptr: *const i32 = std::ptr::null();
        assert!(checker.check_null_pointer(null_ptr, "test", "test_location").is_err());
    }

    #[test]
    fn test_buffer_overflow_checking() {
        let checker = SafetyChecker::new();
        
        // Valid buffer access
        assert!(checker.check_buffer_access(0, 10, 20, "test").is_ok());
        assert!(checker.check_buffer_access(5, 10, 20, "test").is_ok());
        assert!(checker.check_buffer_access(10, 10, 20, "test").is_ok());
        
        // Invalid buffer access
        assert!(checker.check_buffer_access(0, 25, 20, "test").is_err());
        assert!(checker.check_buffer_access(15, 10, 20, "test").is_err());
    }

    #[test]
    fn test_safe_array_access() {
        let array = [1, 2, 3, 4, 5];
        
        // Valid access
        assert_eq!(*safe_array_get(&array, 0).unwrap(), 1);
        assert_eq!(*safe_array_get(&array, 4).unwrap(), 5);
        
        // Invalid access
        assert!(safe_array_get(&array, 5).is_err());
        assert!(safe_array_get(&array, 10).is_err());
    }

    #[test]
    fn test_safe_slice_creation() {
        let array = [1, 2, 3, 4, 5];
        
        // Valid slices
        let slice1 = safe_slice(&array, 0, 3).unwrap();
        assert_eq!(slice1, &[1, 2, 3]);
        
        let slice2 = safe_slice(&array, 2, 5).unwrap();
        assert_eq!(slice2, &[3, 4, 5]);
        
        // Invalid slices
        assert!(safe_slice(&array, 0, 6).is_err());
        assert!(safe_slice(&array, 3, 2).is_err());
    }

    #[test]
    fn test_safety_checker_settings() {
        let mut checker = SafetyChecker::with_settings(false, true, false);
        let (bounds, null, stack) = checker.get_settings();
        assert!(!bounds);
        assert!(null);
        assert!(!stack);
        
        checker.set_bounds_checking(true);
        checker.set_stack_checking(true);
        let (bounds, null, stack) = checker.get_settings();
        assert!(bounds);
        assert!(null);
        assert!(stack);
    }

    #[test]
    fn test_disabled_bounds_checking() {
        let checker = SafetyChecker::with_settings(false, true, true);
        
        // Should not error even with invalid bounds when disabled
        assert!(checker.check_bounds(10, 5, "test").is_ok());
        assert!(checker.check_slice_bounds(10, 15, 5, "test").is_ok());
    }

    #[test]
    fn test_stack_size_configuration() {
        let original_size = get_max_stack_size();
        
        set_max_stack_size(1024 * 1024); // 1MB
        assert_eq!(get_max_stack_size(), 1024 * 1024);
        
        // Restore original size
        set_max_stack_size(original_size);
    }

    #[test]
    fn test_error_display() {
        let error = SafetyError::IndexOutOfBounds {
            index: 5,
            length: 3,
            type_name: "array".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Index out of bounds"));
        assert!(display.contains("5"));
        assert!(display.contains("3"));

        let error = SafetyError::NullPointerDereference {
            operation: "field_access".to_string(),
            location: "main.bu:42".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Null pointer dereference"));
        assert!(display.contains("field_access"));
        assert!(display.contains("main.bu:42"));
    }
}