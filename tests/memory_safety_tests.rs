//! Memory safety tests
//!
//! Tests for runtime memory safety checks including:
//! - Bounds checking for arrays and slices
//! - Null pointer dereference prevention
//! - Stack overflow detection
//! - Buffer overflow prevention

use bulu::runtime::safety::{
    SafetyChecker, SafetyError, SafetyResult, safe_array_get, safe_array_get_mut,
    safe_slice, safe_slice_mut, safe_deref, safe_deref_mut, set_max_stack_size, get_max_stack_size
};
use bulu::runtime::memory::MemoryManager;

#[test]
fn test_bounds_checking_enabled() {
    let checker = SafetyChecker::new();
    
    // Test valid bounds
    assert!(checker.check_bounds(0, 10, "test_array").is_ok());
    assert!(checker.check_bounds(5, 10, "test_array").is_ok());
    assert!(checker.check_bounds(9, 10, "test_array").is_ok());
    
    // Test invalid bounds
    let result = checker.check_bounds(10, 10, "test_array");
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::IndexOutOfBounds { index, length, type_name } => {
            assert_eq!(index, 10);
            assert_eq!(length, 10);
            assert_eq!(type_name, "test_array");
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
    
    // Test way out of bounds
    let result = checker.check_bounds(100, 10, "test_array");
    assert!(result.is_err());
}

#[test]
fn test_bounds_checking_disabled() {
    let checker = SafetyChecker::with_settings(false, true, true);
    
    // Should not error even with invalid bounds when disabled
    assert!(checker.check_bounds(100, 10, "test_array").is_ok());
    assert!(checker.check_bounds(usize::MAX, 10, "test_array").is_ok());
}

#[test]
fn test_slice_bounds_checking() {
    let checker = SafetyChecker::new();
    
    // Valid slice bounds
    assert!(checker.check_slice_bounds(0, 5, 10, "test_array").is_ok());
    assert!(checker.check_slice_bounds(2, 8, 10, "test_array").is_ok());
    assert!(checker.check_slice_bounds(0, 0, 10, "test_array").is_ok());
    assert!(checker.check_slice_bounds(10, 10, 10, "test_array").is_ok());
    
    // Invalid slice bounds - end beyond length
    let result = checker.check_slice_bounds(0, 11, 10, "test_array");
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::IndexOutOfBounds { index, length, type_name } => {
            assert_eq!(index, 11);
            assert_eq!(length, 10);
            assert!(type_name.contains("slice end"));
        }
        _ => panic!("Expected IndexOutOfBounds error for slice end"),
    }
    
    // Invalid slice bounds - start beyond length
    let result = checker.check_slice_bounds(11, 15, 10, "test_array");
    assert!(result.is_err());
    
    // Invalid slice bounds - start > end
    let result = checker.check_slice_bounds(8, 5, 10, "test_array");
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::IndexOutOfBounds { index, length, type_name } => {
            assert_eq!(index, 8);
            assert_eq!(length, 5);
            assert!(type_name.contains("slice range"));
        }
        _ => panic!("Expected IndexOutOfBounds error for slice range"),
    }
}

#[test]
fn test_null_pointer_checking() {
    let checker = SafetyChecker::new();
    
    // Valid pointer
    let value = 42i32;
    let ptr = &value as *const i32;
    assert!(checker.check_null_pointer(ptr, "field_access", "test.bu:10").is_ok());
    
    // Null pointer
    let null_ptr: *const i32 = std::ptr::null();
    let result = checker.check_null_pointer(null_ptr, "field_access", "test.bu:10");
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::NullPointerDereference { operation, location } => {
            assert_eq!(operation, "field_access");
            assert_eq!(location, "test.bu:10");
        }
        _ => panic!("Expected NullPointerDereference error"),
    }
}

#[test]
fn test_null_pointer_checking_disabled() {
    let checker = SafetyChecker::with_settings(true, false, true);
    
    // Should not error even with null pointer when disabled
    let null_ptr: *const i32 = std::ptr::null();
    assert!(checker.check_null_pointer(null_ptr, "field_access", "test.bu:10").is_ok());
}

#[test]
fn test_buffer_overflow_checking() {
    let checker = SafetyChecker::new();
    
    // Valid buffer access
    assert!(checker.check_buffer_access(0, 10, 20, "memcpy").is_ok());
    assert!(checker.check_buffer_access(5, 10, 20, "memcpy").is_ok());
    assert!(checker.check_buffer_access(10, 10, 20, "memcpy").is_ok());
    assert!(checker.check_buffer_access(0, 20, 20, "memcpy").is_ok());
    
    // Invalid buffer access - overflow
    let result = checker.check_buffer_access(0, 25, 20, "memcpy");
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::BufferOverflow { attempted_size, buffer_size, operation } => {
            assert_eq!(attempted_size, 25);
            assert_eq!(buffer_size, 20);
            assert_eq!(operation, "memcpy");
        }
        _ => panic!("Expected BufferOverflow error"),
    }
    
    // Invalid buffer access - offset + size overflow
    let result = checker.check_buffer_access(15, 10, 20, "memcpy");
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::BufferOverflow { attempted_size, buffer_size, operation } => {
            assert_eq!(attempted_size, 25);
            assert_eq!(buffer_size, 20);
            assert_eq!(operation, "memcpy");
        }
        _ => panic!("Expected BufferOverflow error"),
    }
}

#[test]
fn test_stack_overflow_checking() {
    let checker = SafetyChecker::new();
    
    // Normal stack usage should be OK
    let result = checker.check_stack_overflow();
    // This might pass or fail depending on current stack usage, but shouldn't panic
    let _ = result;
    
    // Test with very small stack limit
    let original_size = get_max_stack_size();
    set_max_stack_size(1024); // Very small stack
    
    let result = checker.check_stack_overflow();
    // This should likely fail with such a small stack
    if result.is_err() {
        match result.unwrap_err() {
            SafetyError::StackOverflow { current_size, max_size } => {
                assert!(current_size > 0);
                assert_eq!(max_size, 1024);
            }
            _ => panic!("Expected StackOverflow error"),
        }
    }
    
    // Restore original stack size
    set_max_stack_size(original_size);
}

#[test]
fn test_stack_overflow_checking_disabled() {
    let checker = SafetyChecker::with_settings(true, true, false);
    
    // Should not error even with small stack when disabled
    let original_size = get_max_stack_size();
    set_max_stack_size(1024);
    
    assert!(checker.check_stack_overflow().is_ok());
    
    set_max_stack_size(original_size);
}

#[test]
fn test_safe_array_access() {
    let array = [1, 2, 3, 4, 5];
    
    // Valid access
    assert_eq!(*safe_array_get(&array, 0).unwrap(), 1);
    assert_eq!(*safe_array_get(&array, 2).unwrap(), 3);
    assert_eq!(*safe_array_get(&array, 4).unwrap(), 5);
    
    // Invalid access
    let result = safe_array_get(&array, 5);
    assert!(result.is_err());
    match result.unwrap_err() {
        SafetyError::IndexOutOfBounds { index, length, type_name } => {
            assert_eq!(index, 5);
            assert_eq!(length, 5);
            assert_eq!(type_name, "array");
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
    
    let result = safe_array_get(&array, 100);
    assert!(result.is_err());
}

#[test]
fn test_safe_array_access_mut() {
    let mut array = [1, 2, 3, 4, 5];
    
    // Valid mutable access
    *safe_array_get_mut(&mut array, 0).unwrap() = 10;
    assert_eq!(array[0], 10);
    
    *safe_array_get_mut(&mut array, 4).unwrap() = 50;
    assert_eq!(array[4], 50);
    
    // Invalid mutable access
    let result = safe_array_get_mut(&mut array, 5);
    assert!(result.is_err());
}

#[test]
fn test_safe_slice_creation() {
    let array = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Valid slices
    let slice1 = safe_slice(&array, 0, 5).unwrap();
    assert_eq!(slice1, &[1, 2, 3, 4, 5]);
    
    let slice2 = safe_slice(&array, 3, 7).unwrap();
    assert_eq!(slice2, &[4, 5, 6, 7]);
    
    let slice3 = safe_slice(&array, 0, 0).unwrap();
    assert_eq!(slice3, &[] as &[i32]);
    
    let slice4 = safe_slice(&array, 10, 10).unwrap();
    assert_eq!(slice4, &[] as &[i32]);
    
    // Invalid slices
    let result = safe_slice(&array, 0, 11);
    assert!(result.is_err());
    
    let result = safe_slice(&array, 5, 3);
    assert!(result.is_err());
    
    let result = safe_slice(&array, 11, 15);
    assert!(result.is_err());
}

#[test]
fn test_safe_slice_creation_mut() {
    let mut array = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Valid mutable slice
    let slice = safe_slice_mut(&mut array, 2, 6).unwrap();
    slice[0] = 100;
    slice[3] = 200;
    
    assert_eq!(array[2], 100);
    assert_eq!(array[5], 200);
    
    // Invalid mutable slice
    let result = safe_slice_mut(&mut array, 0, 11);
    assert!(result.is_err());
}

#[test]
fn test_safe_pointer_dereference() {
    let value = 42i32;
    let ptr = &value as *const i32;
    
    // Valid dereference
    unsafe {
        let result = safe_deref(ptr, "field_access", "test.bu:20");
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), 42);
    }
    
    // Null pointer dereference
    let null_ptr: *const i32 = std::ptr::null();
    unsafe {
        let result = safe_deref(null_ptr, "field_access", "test.bu:20");
        assert!(result.is_err());
        match result.unwrap_err() {
            SafetyError::NullPointerDereference { operation, location } => {
                assert_eq!(operation, "field_access");
                assert_eq!(location, "test.bu:20");
            }
            _ => panic!("Expected NullPointerDereference error"),
        }
    }
}

#[test]
fn test_safe_pointer_dereference_mut() {
    let mut value = 42i32;
    let ptr = &mut value as *mut i32;
    
    // Valid mutable dereference
    unsafe {
        let result = safe_deref_mut(ptr, "assignment", "test.bu:25");
        assert!(result.is_ok());
        *result.unwrap() = 100;
    }
    assert_eq!(value, 100);
    
    // Null pointer dereference
    let null_ptr: *mut i32 = std::ptr::null_mut();
    unsafe {
        let result = safe_deref_mut(null_ptr, "assignment", "test.bu:25");
        assert!(result.is_err());
    }
}

#[test]
fn test_memory_manager_safety_integration() {
    let mut mm = MemoryManager::new();
    
    // Test bounds checking through memory manager
    assert!(mm.check_array_bounds(5, 10, "test_array").is_ok());
    assert!(mm.check_array_bounds(10, 10, "test_array").is_err());
    
    // Test slice bounds checking
    assert!(mm.check_slice_bounds(2, 8, 10, "test_slice").is_ok());
    assert!(mm.check_slice_bounds(8, 5, 10, "test_slice").is_err());
    
    // Test null pointer checking
    let value = 42i32;
    let ptr = &value as *const i32;
    assert!(mm.check_null_pointer(ptr, "access", "test.bu:30").is_ok());
    
    let null_ptr: *const i32 = std::ptr::null();
    assert!(mm.check_null_pointer(null_ptr, "access", "test.bu:30").is_err());
    
    // Test buffer overflow checking
    assert!(mm.check_buffer_access(0, 10, 20, "copy").is_ok());
    assert!(mm.check_buffer_access(15, 10, 20, "copy").is_err());
    
    // Test stack overflow checking
    let _ = mm.check_stack_overflow(); // May pass or fail depending on current stack
}

#[test]
fn test_safety_checker_configuration() {
    let mut checker = SafetyChecker::with_settings(false, false, false);
    let (bounds, null, stack) = checker.get_settings();
    assert!(!bounds);
    assert!(!null);
    assert!(!stack);
    
    // All checks should pass when disabled
    assert!(checker.check_bounds(100, 10, "test").is_ok());
    let null_ptr: *const i32 = std::ptr::null();
    assert!(checker.check_null_pointer(null_ptr, "test", "test").is_ok());
    assert!(checker.check_stack_overflow().is_ok());
    
    // Enable checks
    checker.set_bounds_checking(true);
    checker.set_null_checking(true);
    checker.set_stack_checking(true);
    
    let (bounds, null, stack) = checker.get_settings();
    assert!(bounds);
    assert!(null);
    assert!(stack);
    
    // Now checks should fail
    assert!(checker.check_bounds(100, 10, "test").is_err());
    assert!(checker.check_null_pointer(null_ptr, "test", "test").is_err());
}

#[test]
fn test_stack_size_configuration() {
    let original_size = get_max_stack_size();
    
    // Test setting different stack sizes
    set_max_stack_size(2 * 1024 * 1024); // 2MB
    assert_eq!(get_max_stack_size(), 2 * 1024 * 1024);
    
    set_max_stack_size(512 * 1024); // 512KB
    assert_eq!(get_max_stack_size(), 512 * 1024);
    
    set_max_stack_size(16 * 1024 * 1024); // 16MB
    assert_eq!(get_max_stack_size(), 16 * 1024 * 1024);
    
    // Restore original size
    set_max_stack_size(original_size);
    assert_eq!(get_max_stack_size(), original_size);
}

#[test]
fn test_error_types_and_display() {
    // Test IndexOutOfBounds error
    let error = SafetyError::IndexOutOfBounds {
        index: 15,
        length: 10,
        type_name: "my_array".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("Index out of bounds"));
    assert!(display.contains("15"));
    assert!(display.contains("10"));
    assert!(display.contains("my_array"));
    
    // Test NullPointerDereference error
    let error = SafetyError::NullPointerDereference {
        operation: "struct_field_access".to_string(),
        location: "main.bu:42".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("Null pointer dereference"));
    assert!(display.contains("struct_field_access"));
    assert!(display.contains("main.bu:42"));
    
    // Test StackOverflow error
    let error = SafetyError::StackOverflow {
        current_size: 9 * 1024 * 1024,
        max_size: 8 * 1024 * 1024,
    };
    let display = format!("{}", error);
    assert!(display.contains("Stack overflow"));
    assert!(display.contains("9437184")); // 9MB in bytes
    assert!(display.contains("8388608")); // 8MB in bytes
    
    // Test BufferOverflow error
    let error = SafetyError::BufferOverflow {
        attempted_size: 150,
        buffer_size: 100,
        operation: "string_copy".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("Buffer overflow"));
    assert!(display.contains("150"));
    assert!(display.contains("100"));
    assert!(display.contains("string_copy"));
    
    // Test InvalidMemoryAccess error
    let error = SafetyError::InvalidMemoryAccess {
        address: 0xdeadbeef,
        operation: "pointer_arithmetic".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("Invalid memory access"));
    assert!(display.contains("deadbeef"));
    assert!(display.contains("pointer_arithmetic"));
}

#[test]
fn test_comprehensive_bounds_checking_scenarios() {
    let checker = SafetyChecker::new();
    
    // Test edge cases for bounds checking
    assert!(checker.check_bounds(0, 1, "single_element").is_ok());
    assert!(checker.check_bounds(1, 1, "single_element").is_err());
    
    // Test empty array
    assert!(checker.check_bounds(0, 0, "empty_array").is_err());
    
    // Test large indices
    assert!(checker.check_bounds(usize::MAX - 1, usize::MAX, "max_array").is_ok());
    assert!(checker.check_bounds(usize::MAX, usize::MAX, "max_array").is_err());
}

#[test]
fn test_comprehensive_slice_bounds_scenarios() {
    let checker = SafetyChecker::new();
    
    // Test edge cases for slice bounds
    assert!(checker.check_slice_bounds(0, 0, 10, "empty_slice").is_ok());
    assert!(checker.check_slice_bounds(5, 5, 10, "empty_slice_mid").is_ok());
    assert!(checker.check_slice_bounds(10, 10, 10, "empty_slice_end").is_ok());
    
    // Test full slice
    assert!(checker.check_slice_bounds(0, 10, 10, "full_slice").is_ok());
    
    // Test single element slices
    assert!(checker.check_slice_bounds(0, 1, 10, "first_element").is_ok());
    assert!(checker.check_slice_bounds(9, 10, 10, "last_element").is_ok());
}

#[test]
fn test_memory_safety_with_different_types() {
    // Test with different array types
    let int_array = [1i32, 2, 3, 4, 5];
    assert_eq!(*safe_array_get(&int_array, 2).unwrap(), 3);
    assert!(safe_array_get(&int_array, 5).is_err());
    
    let float_array = [1.0f64, 2.0, 3.0];
    assert_eq!(*safe_array_get(&float_array, 1).unwrap(), 2.0);
    assert!(safe_array_get(&float_array, 3).is_err());
    
    let string_array = ["hello", "world", "test"];
    assert_eq!(*safe_array_get(&string_array, 0).unwrap(), "hello");
    assert!(safe_array_get(&string_array, 3).is_err());
    
    // Test with different slice types
    let slice1 = safe_slice(&int_array, 1, 4).unwrap();
    assert_eq!(slice1, &[2, 3, 4]);
    
    let slice2 = safe_slice(&string_array, 0, 2).unwrap();
    assert_eq!(slice2, &["hello", "world"]);
}

// Note: We can't easily test actual stack overflow in unit tests without
// risking crashing the test runner, so we test the detection mechanism
// with artificial limits instead.