//! Unit tests for built-in functions
//!
//! This module tests all core built-in functions including:
//! - Type conversion functions
//! - Memory functions
//! - Collection operations
//! - I/O functions

use bulu::runtime::builtins::*;
use bulu::types::primitive::RuntimeValue;
use bulu::error::BuluError;

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_int8_conversion() {
        // Test valid conversions
        let result = builtin_int8(&[RuntimeValue::Int32(42)]).unwrap();
        assert_eq!(result, RuntimeValue::Int8(42));
        
        let result = builtin_int8(&[RuntimeValue::Float64(3.14)]).unwrap();
        assert_eq!(result, RuntimeValue::Int8(3));
        
        let result = builtin_int8(&[RuntimeValue::Bool(true)]).unwrap();
        assert_eq!(result, RuntimeValue::Int8(1));
        
        let result = builtin_int8(&[RuntimeValue::Bool(false)]).unwrap();
        assert_eq!(result, RuntimeValue::Int8(0));
        
        // Test error cases
        assert!(builtin_int8(&[]).is_err());
        assert!(builtin_int8(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
    }

    #[test]
    fn test_int16_conversion() {
        let result = builtin_int16(&[RuntimeValue::Int32(1000)]).unwrap();
        assert_eq!(result, RuntimeValue::Int16(1000));
        
        let result = builtin_int16(&[RuntimeValue::Float32(123.45)]).unwrap();
        assert_eq!(result, RuntimeValue::Int16(123));
    }

    #[test]
    fn test_int32_conversion() {
        let result = builtin_int32(&[RuntimeValue::Int64(100000)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(100000));
        
        let result = builtin_int32(&[RuntimeValue::Float64(42.7)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(42));
        
        let result = builtin_int32(&[RuntimeValue::Char('A')]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(65)); // ASCII value of 'A'
    }

    #[test]
    fn test_int64_conversion() {
        let result = builtin_int64(&[RuntimeValue::Int32(42)]).unwrap();
        assert_eq!(result, RuntimeValue::Int64(42));
        
        let result = builtin_int64(&[RuntimeValue::UInt64(12345)]).unwrap();
        assert_eq!(result, RuntimeValue::Int64(12345));
    }

    #[test]
    fn test_uint8_conversion() {
        let result = builtin_uint8(&[RuntimeValue::Int32(255)]).unwrap();
        assert_eq!(result, RuntimeValue::UInt8(255));
        
        let result = builtin_uint8(&[RuntimeValue::Bool(true)]).unwrap();
        assert_eq!(result, RuntimeValue::UInt8(1));
    }

    #[test]
    fn test_uint16_conversion() {
        let result = builtin_uint16(&[RuntimeValue::Int32(65535)]).unwrap();
        assert_eq!(result, RuntimeValue::UInt16(65535));
    }

    #[test]
    fn test_uint32_conversion() {
        let result = builtin_uint32(&[RuntimeValue::Int64(4294967295)]).unwrap();
        assert_eq!(result, RuntimeValue::UInt32(4294967295));
        
        let result = builtin_uint32(&[RuntimeValue::Char('Z')]).unwrap();
        assert_eq!(result, RuntimeValue::UInt32(90)); // ASCII value of 'Z'
    }

    #[test]
    fn test_uint64_conversion() {
        let result = builtin_uint64(&[RuntimeValue::Int32(42)]).unwrap();
        assert_eq!(result, RuntimeValue::UInt64(42));
    }

    #[test]
    fn test_float32_conversion() {
        let result = builtin_float32(&[RuntimeValue::Int32(42)]).unwrap();
        assert_eq!(result, RuntimeValue::Float32(42.0));
        
        let result = builtin_float32(&[RuntimeValue::Float64(3.14159)]).unwrap();
        assert_eq!(result, RuntimeValue::Float32(3.14159_f32));
    }

    #[test]
    fn test_float64_conversion() {
        let result = builtin_float64(&[RuntimeValue::Int32(42)]).unwrap();
        assert_eq!(result, RuntimeValue::Float64(42.0));
        
        let result = builtin_float64(&[RuntimeValue::Float32(3.14)]).unwrap();
        // Note: float32 to float64 conversion may have precision differences
        if let RuntimeValue::Float64(f) = result {
            assert!((f - 3.14).abs() < 0.001); // Allow small precision difference
        } else {
            panic!("Expected Float64 result");
        }
    }

    #[test]
    fn test_bool_conversion() {
        // Truthy values
        let result = builtin_bool(&[RuntimeValue::Int32(1)]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_bool(&[RuntimeValue::Int32(-5)]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_bool(&[RuntimeValue::Float64(3.14)]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_bool(&[RuntimeValue::String("hello".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        // Falsy values
        let result = builtin_bool(&[RuntimeValue::Int32(0)]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        let result = builtin_bool(&[RuntimeValue::Float64(0.0)]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        let result = builtin_bool(&[RuntimeValue::String("".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        let result = builtin_bool(&[RuntimeValue::Null]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
    }

    #[test]
    fn test_char_conversion() {
        let result = builtin_char(&[RuntimeValue::Int32(65)]).unwrap();
        assert_eq!(result, RuntimeValue::Char('A'));
        
        let result = builtin_char(&[RuntimeValue::UInt32(8364)]).unwrap(); // Euro symbol
        assert_eq!(result, RuntimeValue::Char('€'));
    }

    #[test]
    fn test_string_conversion() {
        let result = builtin_string(&[RuntimeValue::Int32(42)]).unwrap();
        assert_eq!(result, RuntimeValue::String("42".to_string()));
        
        let result = builtin_string(&[RuntimeValue::Float64(3.14)]).unwrap();
        assert_eq!(result, RuntimeValue::String("3.14".to_string()));
        
        let result = builtin_string(&[RuntimeValue::Bool(true)]).unwrap();
        assert_eq!(result, RuntimeValue::String("true".to_string()));
        
        let result = builtin_string(&[RuntimeValue::Char('A')]).unwrap();
        assert_eq!(result, RuntimeValue::String("A".to_string()));
        
        let result = builtin_string(&[RuntimeValue::Null]).unwrap();
        assert_eq!(result, RuntimeValue::String("null".to_string()));
    }
}

#[cfg(test)]
mod memory_function_tests {
    use super::*;

    #[test]
    fn test_len_function() {
        // Test string length
        let result = builtin_len(&[RuntimeValue::String("hello".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(5));
        
        let result = builtin_len(&[RuntimeValue::String("".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(0));
        
        let result = builtin_len(&[RuntimeValue::String("🦀".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(4)); // UTF-8 bytes
        
        // Test error cases
        assert!(builtin_len(&[]).is_err());
        assert!(builtin_len(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
    }

    #[test]
    fn test_cap_function() {
        // For now, cap returns 0 for all types (placeholder implementation)
        let result = builtin_cap(&[RuntimeValue::String("test".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(0));
        
        // Test error cases
        assert!(builtin_cap(&[]).is_err());
        assert!(builtin_cap(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
    }

    #[test]
    fn test_clone_function() {
        // Test cloning primitive values
        let original = RuntimeValue::Int32(42);
        let result = builtin_clone(&[original.clone()]).unwrap();
        assert_eq!(result, original);
        
        let original = RuntimeValue::String("hello".to_string());
        let result = builtin_clone(&[original.clone()]).unwrap();
        assert_eq!(result, original);
        
        let original = RuntimeValue::Bool(true);
        let result = builtin_clone(&[original.clone()]).unwrap();
        assert_eq!(result, original);
        
        // Test error cases
        assert!(builtin_clone(&[]).is_err());
        assert!(builtin_clone(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
    }

    #[test]
    fn test_sizeof_function() {
        // Test sizes of different types
        let result = builtin_sizeof(&[RuntimeValue::Int8(0)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(1));
        
        let result = builtin_sizeof(&[RuntimeValue::Int16(0)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(2));
        
        let result = builtin_sizeof(&[RuntimeValue::Int32(0)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(4));
        
        let result = builtin_sizeof(&[RuntimeValue::Int64(0)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(8));
        
        let result = builtin_sizeof(&[RuntimeValue::Float32(0.0)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(4));
        
        let result = builtin_sizeof(&[RuntimeValue::Float64(0.0)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(8));
        
        let result = builtin_sizeof(&[RuntimeValue::Bool(false)]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(1));
        
        let result = builtin_sizeof(&[RuntimeValue::Char('A')]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(4));
        
        let result = builtin_sizeof(&[RuntimeValue::String("hello".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(5));
        
        let result = builtin_sizeof(&[RuntimeValue::Null]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(0));
        
        // Test error cases
        assert!(builtin_sizeof(&[]).is_err());
        assert!(builtin_sizeof(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
    }
}

#[cfg(test)]
mod collection_function_tests {
    use super::*;

    #[test]
    fn test_make_function() {
        // For now, make returns null (placeholder implementation)
        let result = builtin_make(&[RuntimeValue::String("slice".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test error cases
        assert!(builtin_make(&[]).is_err());
    }

    #[test]
    fn test_append_function() {
        // For now, append returns the first argument (placeholder implementation)
        let slice = RuntimeValue::String("slice".to_string());
        let element = RuntimeValue::Int32(42);
        let result = builtin_append(&[slice.clone(), element]).unwrap();
        assert_eq!(result, slice);
        
        // Test error cases
        assert!(builtin_append(&[]).is_err());
        assert!(builtin_append(&[RuntimeValue::Int32(1)]).is_err());
    }

    #[test]
    fn test_copy_function() {
        // For now, copy returns 0 (placeholder implementation)
        let src = RuntimeValue::String("source".to_string());
        let dst = RuntimeValue::String("dest".to_string());
        let result = builtin_copy(&[dst, src]).unwrap();
        assert_eq!(result, RuntimeValue::Int32(0));
        
        // Test error cases
        assert!(builtin_copy(&[]).is_err());
        assert!(builtin_copy(&[RuntimeValue::Int32(1)]).is_err());
        assert!(builtin_copy(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2), RuntimeValue::Int32(3)]).is_err());
    }

    #[test]
    fn test_delete_function() {
        // For now, delete returns null (placeholder implementation)
        let map = RuntimeValue::String("map".to_string());
        let key = RuntimeValue::String("key".to_string());
        let result = builtin_delete(&[map, key]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test error cases
        assert!(builtin_delete(&[]).is_err());
        assert!(builtin_delete(&[RuntimeValue::Int32(1)]).is_err());
        assert!(builtin_delete(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2), RuntimeValue::Int32(3)]).is_err());
    }
}

#[cfg(test)]
mod io_function_tests {
    use super::*;

    #[test]
    fn test_print_function() {
        // Test printing single value
        let result = builtin_print(&[RuntimeValue::String("hello".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printing multiple values
        let result = builtin_print(&[
            RuntimeValue::String("hello".to_string()),
            RuntimeValue::Int32(42),
            RuntimeValue::Bool(true)
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printing no arguments
        let result = builtin_print(&[]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
    }

    #[test]
    fn test_println_function() {
        // Test println with values
        let result = builtin_println(&[RuntimeValue::String("hello world".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test println with no arguments
        let result = builtin_println(&[]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
    }

    #[test]
    fn test_printf_function() {
        // Test printf with format string only
        let result = builtin_printf(&[RuntimeValue::String("Hello World".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with integer format
        let result = builtin_printf(&[
            RuntimeValue::String("Number: %d".to_string()),
            RuntimeValue::Int32(42)
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with float format
        let result = builtin_printf(&[
            RuntimeValue::String("Pi: %f".to_string()),
            RuntimeValue::Float64(3.14159)
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with string format
        let result = builtin_printf(&[
            RuntimeValue::String("Hello %s".to_string()),
            RuntimeValue::String("World".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with character format
        let result = builtin_printf(&[
            RuntimeValue::String("Char: %c".to_string()),
            RuntimeValue::Char('A')
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with boolean format
        let result = builtin_printf(&[
            RuntimeValue::String("Bool: %b".to_string()),
            RuntimeValue::Bool(true)
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with hex format
        let result = builtin_printf(&[
            RuntimeValue::String("Hex: %x".to_string()),
            RuntimeValue::Int32(255)
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with multiple format specifiers
        let result = builtin_printf(&[
            RuntimeValue::String("Name: %s, Age: %d, Score: %f".to_string()),
            RuntimeValue::String("Alice".to_string()),
            RuntimeValue::Int32(25),
            RuntimeValue::Float64(95.5)
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test printf with escaped %
        let result = builtin_printf(&[
            RuntimeValue::String("100%% complete".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test error cases
        assert!(builtin_printf(&[]).is_err());
        assert!(builtin_printf(&[RuntimeValue::Int32(42)]).is_err()); // First arg must be string
        
        // Test insufficient arguments
        let result = builtin_printf(&[
            RuntimeValue::String("Number: %d".to_string())
            // Missing argument for %d
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_input_function() {
        // Note: This test doesn't actually test input reading since it would require stdin
        // In a real test environment, you would mock stdin or use a test harness
        
        // Test input with no prompt (would block in real usage)
        // let result = builtin_input(&[]).unwrap();
        // assert!(matches!(result, RuntimeValue::String(_)));
        
        // For now, just test that the function exists and has correct signature
        // We can't easily test interactive input in unit tests
    }
}

#[cfg(test)]
mod utility_function_tests {
    use super::*;

    #[test]
    fn test_typeof_function() {
        // Test typeof for different types
        let result = builtin_typeof(&[RuntimeValue::Int8(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("int8".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Int16(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("int16".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Int32(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("int32".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Int64(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("int64".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::UInt8(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("uint8".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::UInt16(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("uint16".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::UInt32(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("uint32".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::UInt64(0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("uint64".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Float32(0.0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("float32".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Float64(0.0)]).unwrap();
        assert_eq!(result, RuntimeValue::String("float64".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Bool(true)]).unwrap();
        assert_eq!(result, RuntimeValue::String("bool".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Char('A')]).unwrap();
        assert_eq!(result, RuntimeValue::String("char".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::String("test".to_string())]).unwrap();
        assert_eq!(result, RuntimeValue::String("string".to_string()));
        
        let result = builtin_typeof(&[RuntimeValue::Null]).unwrap();
        assert_eq!(result, RuntimeValue::String("null".to_string()));
        
        // Test error cases
        assert!(builtin_typeof(&[]).is_err());
        assert!(builtin_typeof(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
    }

    #[test]
    fn test_panic_function() {
        // Test panic with message
        let result = builtin_panic(&[RuntimeValue::String("test panic".to_string())]);
        assert!(result.is_err());
        if let Err(BuluError::RuntimeError { message, .. }) = result {
            assert!(message.contains("panic: test panic"));
        }
        
        // Test panic without message
        let result = builtin_panic(&[]);
        assert!(result.is_err());
        if let Err(BuluError::RuntimeError { message, .. }) = result {
            assert!(message.contains("panic: panic"));
        }
    }

    #[test]
    fn test_assert_function() {
        // Test successful assertion
        let result = builtin_assert(&[RuntimeValue::Bool(true)]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        let result = builtin_assert(&[RuntimeValue::Int32(1)]).unwrap(); // Truthy
        assert_eq!(result, RuntimeValue::Null);
        
        // Test failed assertion without message
        let result = builtin_assert(&[RuntimeValue::Bool(false)]);
        assert!(result.is_err());
        if let Err(BuluError::RuntimeError { message, .. }) = result {
            assert!(message.contains("assertion failed"));
        }
        
        // Test failed assertion with message
        let result = builtin_assert(&[
            RuntimeValue::Bool(false),
            RuntimeValue::String("custom message".to_string())
        ]);
        assert!(result.is_err());
        if let Err(BuluError::RuntimeError { message, .. }) = result {
            assert!(message.contains("assertion failed: custom message"));
        }
        
        // Test error cases
        assert!(builtin_assert(&[]).is_err());
    }

    #[test]
    fn test_instanceof_function() {
        // Test exact type matches
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("int32".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::String("hello".to_string()),
            RuntimeValue::String("string".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Bool(true),
            RuntimeValue::String("bool".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        // Test exact type mismatches
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("string".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        // Test category matches
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("integer".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::UInt16(100),
            RuntimeValue::String("integer".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Float64(3.14),
            RuntimeValue::String("float".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("number".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Float32(3.14),
            RuntimeValue::String("numeric".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(-42),
            RuntimeValue::String("signed".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::UInt32(42),
            RuntimeValue::String("unsigned".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::String("test".to_string()),
            RuntimeValue::String("primitive".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Null,
            RuntimeValue::String("primitive".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        // Test 'any' type - everything should match
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("any".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Null,
            RuntimeValue::String("any".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(true));
        
        // Test category mismatches
        let result = builtin_instanceof(&[
            RuntimeValue::String("hello".to_string()),
            RuntimeValue::String("integer".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("float".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        let result = builtin_instanceof(&[
            RuntimeValue::UInt32(42),
            RuntimeValue::String("signed".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(-42),
            RuntimeValue::String("unsigned".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        // Test unknown type
        let result = builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::String("unknown_type".to_string())
        ]).unwrap();
        assert_eq!(result, RuntimeValue::Bool(false));
        
        // Test error cases
        assert!(builtin_instanceof(&[]).is_err());
        assert!(builtin_instanceof(&[RuntimeValue::Int32(1)]).is_err());
        assert!(builtin_instanceof(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2), RuntimeValue::Int32(3)]).is_err());
        
        // Test invalid second argument (not a string)
        assert!(builtin_instanceof(&[
            RuntimeValue::Int32(42),
            RuntimeValue::Int32(123)
        ]).is_err());
    }

    #[test]
    fn test_recover_function() {
        // Test recover with no arguments
        let result = builtin_recover(&[]).unwrap();
        assert_eq!(result, RuntimeValue::Null);
        
        // Test error cases
        assert!(builtin_recover(&[RuntimeValue::Int32(1)]).is_err());
        assert!(builtin_recover(&[RuntimeValue::String("test".to_string())]).is_err());
    }
}

#[cfg(test)]
mod builtin_registry_tests {
    use super::*;

    #[test]
    fn test_builtin_registry_creation() {
        let registry = BuiltinRegistry::new();
        
        // Test that all expected built-ins are registered
        assert!(registry.is_builtin("int8"));
        assert!(registry.is_builtin("int16"));
        assert!(registry.is_builtin("int32"));
        assert!(registry.is_builtin("int64"));
        assert!(registry.is_builtin("uint8"));
        assert!(registry.is_builtin("uint16"));
        assert!(registry.is_builtin("uint32"));
        assert!(registry.is_builtin("uint64"));
        assert!(registry.is_builtin("float32"));
        assert!(registry.is_builtin("float64"));
        assert!(registry.is_builtin("bool"));
        assert!(registry.is_builtin("char"));
        assert!(registry.is_builtin("string"));
        
        assert!(registry.is_builtin("len"));
        assert!(registry.is_builtin("cap"));
        assert!(registry.is_builtin("clone"));
        assert!(registry.is_builtin("sizeof"));
        
        assert!(registry.is_builtin("make"));
        assert!(registry.is_builtin("append"));
        assert!(registry.is_builtin("copy"));
        assert!(registry.is_builtin("delete"));
        
        assert!(registry.is_builtin("print"));
        assert!(registry.is_builtin("println"));
        assert!(registry.is_builtin("printf"));
        assert!(registry.is_builtin("input"));
        
        assert!(registry.is_builtin("typeof"));
        assert!(registry.is_builtin("instanceof"));
        assert!(registry.is_builtin("panic"));
        assert!(registry.is_builtin("assert"));
        assert!(registry.is_builtin("recover"));
        
        // Test that non-existent functions are not registered
        assert!(!registry.is_builtin("nonexistent"));
        assert!(!registry.is_builtin(""));
    }

    #[test]
    fn test_builtin_registry_get_function() {
        let registry = BuiltinRegistry::new();
        
        // Test getting existing functions
        assert!(registry.get("int32").is_some());
        assert!(registry.get("len").is_some());
        assert!(registry.get("print").is_some());
        
        // Test getting non-existent functions
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_builtin_registry_get_all_names() {
        let registry = BuiltinRegistry::new();
        let names = registry.get_all_names();
        
        // Should have all the built-in functions
        assert!(names.len() >= 27); // At least 27 built-ins
        assert!(names.contains(&"int32".to_string()));
        assert!(names.contains(&"len".to_string()));
        assert!(names.contains(&"print".to_string()));
    }

    #[test]
    fn test_default_builtins() {
        let registry = get_default_builtins();
        
        // Should be the same as creating a new registry
        assert!(registry.is_builtin("int32"));
        assert!(registry.is_builtin("len"));
        assert!(registry.is_builtin("print"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_type_conversion_chain() {
        // Test converting through multiple types
        let original = RuntimeValue::Int32(42);
        
        // int32 -> float64 -> string -> back to numbers
        let as_float = builtin_float64(&[original]).unwrap();
        let as_string = builtin_string(&[as_float]).unwrap();
        
        if let RuntimeValue::String(s) = as_string {
            assert_eq!(s, "42");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_memory_operations_workflow() {
        let test_string = RuntimeValue::String("hello world".to_string());
        
        // Get length
        let length = builtin_len(&[test_string.clone()]).unwrap();
        assert_eq!(length, RuntimeValue::Int32(11));
        
        // Get size
        let size = builtin_sizeof(&[test_string.clone()]).unwrap();
        assert_eq!(size, RuntimeValue::Int32(11));
        
        // Clone
        let cloned = builtin_clone(&[test_string.clone()]).unwrap();
        assert_eq!(cloned, test_string);
        
        // Get type
        let type_name = builtin_typeof(&[test_string]).unwrap();
        assert_eq!(type_name, RuntimeValue::String("string".to_string()));
    }

    #[test]
    fn test_error_handling_consistency() {
        // All single-argument functions should fail with wrong argument count
        let functions = [
            builtin_int32 as BuiltinFunction,
            builtin_len,
            builtin_clone,
            builtin_sizeof,
            builtin_typeof,
        ];
        
        for func in &functions {
            // No arguments
            assert!(func(&[]).is_err());
            
            // Too many arguments
            assert!(func(&[RuntimeValue::Int32(1), RuntimeValue::Int32(2)]).is_err());
        }
    }

    #[test]
    fn test_io_and_utility_integration() {
        // Test a complete workflow using I/O and utility functions
        
        // Test typeof and instanceof together
        let value = RuntimeValue::Int32(42);
        let type_result = builtin_typeof(&[value.clone()]).unwrap();
        assert_eq!(type_result, RuntimeValue::String("int32".to_string()));
        
        let instanceof_result = builtin_instanceof(&[
            value.clone(),
            RuntimeValue::String("int32".to_string())
        ]).unwrap();
        assert_eq!(instanceof_result, RuntimeValue::Bool(true));
        
        let instanceof_category = builtin_instanceof(&[
            value.clone(),
            RuntimeValue::String("integer".to_string())
        ]).unwrap();
        assert_eq!(instanceof_category, RuntimeValue::Bool(true));
        
        // Test print functions with different types
        let print_result = builtin_print(&[
            RuntimeValue::String("Value:".to_string()),
            value.clone(),
            RuntimeValue::String("Type:".to_string()),
            type_result.clone()
        ]).unwrap();
        assert_eq!(print_result, RuntimeValue::Null);
        
        // Test printf with type information
        let printf_result = builtin_printf(&[
            RuntimeValue::String("Value %d has type %s".to_string()),
            value.clone(),
            type_result.clone()
        ]).unwrap();
        assert_eq!(printf_result, RuntimeValue::Null);
        
        // Test assertions
        let assert_result = builtin_assert(&[instanceof_result]).unwrap();
        assert_eq!(assert_result, RuntimeValue::Null);
        
        // Test recover (should return null in normal execution)
        let recover_result = builtin_recover(&[]).unwrap();
        assert_eq!(recover_result, RuntimeValue::Null);
        
        // Test with different value types
        let float_val = RuntimeValue::Float64(3.14159);
        let float_type = builtin_typeof(&[float_val.clone()]).unwrap();
        assert_eq!(float_type, RuntimeValue::String("float64".to_string()));
        
        let is_numeric = builtin_instanceof(&[
            float_val.clone(),
            RuntimeValue::String("numeric".to_string())
        ]).unwrap();
        assert_eq!(is_numeric, RuntimeValue::Bool(true));
        
        let is_integer = builtin_instanceof(&[
            float_val.clone(),
            RuntimeValue::String("integer".to_string())
        ]).unwrap();
        assert_eq!(is_integer, RuntimeValue::Bool(false));
        
        // Test printf with float
        let printf_float = builtin_printf(&[
            RuntimeValue::String("Pi is approximately %f".to_string()),
            float_val
        ]).unwrap();
        assert_eq!(printf_float, RuntimeValue::Null);
    }

    #[test]
    fn test_comprehensive_type_checking() {
        // Test instanceof with all primitive types
        let test_cases = vec![
            (RuntimeValue::Int8(1), "int8", true),
            (RuntimeValue::Int8(1), "integer", true),
            (RuntimeValue::Int8(1), "signed", true),
            (RuntimeValue::Int8(1), "numeric", true),
            (RuntimeValue::Int8(1), "unsigned", false),
            (RuntimeValue::Int8(1), "float", false),
            
            (RuntimeValue::UInt32(100), "uint32", true),
            (RuntimeValue::UInt32(100), "integer", true),
            (RuntimeValue::UInt32(100), "unsigned", true),
            (RuntimeValue::UInt32(100), "numeric", true),
            (RuntimeValue::UInt32(100), "signed", false),
            
            (RuntimeValue::Float32(3.14), "float32", true),
            (RuntimeValue::Float32(3.14), "float", true),
            (RuntimeValue::Float32(3.14), "numeric", true),
            (RuntimeValue::Float32(3.14), "integer", false),
            
            (RuntimeValue::String("test".to_string()), "string", true),
            (RuntimeValue::String("test".to_string()), "primitive", true),
            (RuntimeValue::String("test".to_string()), "numeric", false),
            
            (RuntimeValue::Bool(true), "bool", true),
            (RuntimeValue::Bool(true), "primitive", true),
            (RuntimeValue::Bool(true), "numeric", false),
            
            (RuntimeValue::Char('A'), "char", true),
            (RuntimeValue::Char('A'), "primitive", true),
            (RuntimeValue::Char('A'), "numeric", false),
            
            (RuntimeValue::Null, "null", true),
            (RuntimeValue::Null, "primitive", false),
            (RuntimeValue::Null, "any", true),
        ];
        
        for (value, type_name, expected) in test_cases {
            let result = builtin_instanceof(&[
                value.clone(),
                RuntimeValue::String(type_name.to_string())
            ]).unwrap();
            
            assert_eq!(
                result,
                RuntimeValue::Bool(expected),
                "instanceof({:?}, {}) should be {}",
                value, type_name, expected
            );
        }
    }
}