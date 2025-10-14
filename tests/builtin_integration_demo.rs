//! Integration demo for built-in functions
//!
//! This test demonstrates how the built-in functions work together
//! in realistic scenarios.

use bulu::runtime::builtins::*;
use bulu::types::primitive::RuntimeValue;

#[cfg(test)]
mod integration_demo {
    use super::*;

    #[test]
    fn test_builtin_functions_demo() {
        println!("=== Bulu Built-in Functions Demo ===");
        
        // Create a registry of built-in functions
        let registry = get_default_builtins();
        
        // Demonstrate type conversions
        println!("\n--- Type Conversions ---");
        let int_val = RuntimeValue::Int32(42);
        let float_val = builtin_float64(&[int_val.clone()]).unwrap();
        let string_val = builtin_string(&[float_val.clone()]).unwrap();
        
        println!("Original int32: {:?}", int_val);
        println!("Converted to float64: {:?}", float_val);
        println!("Converted to string: {:?}", string_val);
        
        // Demonstrate memory functions
        println!("\n--- Memory Functions ---");
        let test_string = RuntimeValue::String("Hello, Bulu!".to_string());
        let length = builtin_len(&[test_string.clone()]).unwrap();
        let size = builtin_sizeof(&[test_string.clone()]).unwrap();
        let cloned = builtin_clone(&[test_string.clone()]).unwrap();
        
        println!("String: {:?}", test_string);
        println!("Length: {:?}", length);
        println!("Size: {:?}", size);
        println!("Cloned: {:?}", cloned);
        
        // Demonstrate utility functions
        println!("\n--- Utility Functions ---");
        let type_name = builtin_typeof(&[test_string.clone()]).unwrap();
        println!("Type of string: {:?}", type_name);
        
        // Test instanceof function
        let is_string = builtin_instanceof(&[
            test_string.clone(),
            RuntimeValue::String("string".to_string())
        ]).unwrap();
        println!("Is string instance of 'string': {:?}", is_string);
        
        let is_primitive = builtin_instanceof(&[
            test_string.clone(),
            RuntimeValue::String("primitive".to_string())
        ]).unwrap();
        println!("Is string instance of 'primitive': {:?}", is_primitive);
        
        let is_number = builtin_instanceof(&[
            int_val.clone(),
            RuntimeValue::String("numeric".to_string())
        ]).unwrap();
        println!("Is int32 instance of 'numeric': {:?}", is_number);
        
        // Test recover function
        let recover_result = builtin_recover(&[]).unwrap();
        println!("Recover result (normal execution): {:?}", recover_result);
        
        // Demonstrate boolean conversions
        println!("\n--- Boolean Conversions ---");
        let bool_from_int = builtin_bool(&[RuntimeValue::Int32(1)]).unwrap();
        let bool_from_float = builtin_bool(&[RuntimeValue::Float64(3.14)]).unwrap();
        let bool_from_string = builtin_bool(&[RuntimeValue::String("hello".to_string())]).unwrap();
        let bool_from_empty = builtin_bool(&[RuntimeValue::String("".to_string())]).unwrap();
        
        println!("bool(1): {:?}", bool_from_int);
        println!("bool(3.14): {:?}", bool_from_float);
        println!("bool(\"hello\"): {:?}", bool_from_string);
        println!("bool(\"\"): {:?}", bool_from_empty);
        
        // Demonstrate I/O functions (print to stdout)
        println!("\n--- I/O Functions ---");
        println!("Testing print functions:");
        let _ = builtin_print(&[RuntimeValue::String("Hello".to_string())]);
        let _ = builtin_print(&[RuntimeValue::String(" ".to_string())]);
        let _ = builtin_println(&[RuntimeValue::String("World!".to_string())]);
        
        // Test printf with various format specifiers
        println!("Testing printf with format specifiers:");
        let _ = builtin_printf(&[
            RuntimeValue::String("Integer: %d, Float: %f, String: %s, Bool: %b\n".to_string()),
            RuntimeValue::Int32(42),
            RuntimeValue::Float64(3.14159),
            RuntimeValue::String("test".to_string()),
            RuntimeValue::Bool(true)
        ]);
        
        let _ = builtin_printf(&[
            RuntimeValue::String("Hex: %x, Octal: %o, Character: %c\n".to_string()),
            RuntimeValue::Int32(255),
            RuntimeValue::Int32(64),
            RuntimeValue::Char('A')
        ]);
        
        let _ = builtin_printf(&[
            RuntimeValue::String("Escaped percent: 100%% complete\n".to_string())
        ]);
        
        // Demonstrate error handling
        println!("\n--- Error Handling ---");
        let assert_result = builtin_assert(&[RuntimeValue::Bool(true)]);
        println!("Assert true result: {:?}", assert_result);
        
        let panic_result = builtin_panic(&[RuntimeValue::String("Test panic".to_string())]);
        println!("Panic result (should be error): {:?}", panic_result.is_err());
        
        // Show registry functionality
        println!("\n--- Registry Functions ---");
        println!("Is 'int32' a built-in? {}", registry.is_builtin("int32"));
        println!("Is 'len' a built-in? {}", registry.is_builtin("len"));
        println!("Is 'nonexistent' a built-in? {}", registry.is_builtin("nonexistent"));
        
        let all_builtins = registry.get_all_names();
        println!("Total built-in functions: {}", all_builtins.len());
        println!("First 10 built-ins: {:?}", &all_builtins[..10.min(all_builtins.len())]);
        
        println!("\n=== Demo Complete ===");
    }
    
    #[test]
    fn test_type_conversion_chain_demo() {
        println!("\n=== Type Conversion Chain Demo ===");
        
        // Start with an integer
        let original = RuntimeValue::Int32(255);
        println!("Original: {:?}", original);
        
        // Convert through various types
        let as_uint8 = builtin_uint8(&[original.clone()]).unwrap();
        println!("As uint8: {:?}", as_uint8);
        
        let as_float = builtin_float32(&[as_uint8.clone()]).unwrap();
        println!("As float32: {:?}", as_float);
        
        let as_string = builtin_string(&[as_float.clone()]).unwrap();
        println!("As string: {:?}", as_string);
        
        let string_len = builtin_len(&[as_string.clone()]).unwrap();
        println!("String length: {:?}", string_len);
        
        let len_as_bool = builtin_bool(&[string_len.clone()]).unwrap();
        println!("Length as bool: {:?}", len_as_bool);
        
        // Show type information at each step
        println!("\nType information:");
        println!("typeof(original): {:?}", builtin_typeof(&[original]).unwrap());
        println!("typeof(as_uint8): {:?}", builtin_typeof(&[as_uint8]).unwrap());
        println!("typeof(as_float): {:?}", builtin_typeof(&[as_float]).unwrap());
        println!("typeof(as_string): {:?}", builtin_typeof(&[as_string]).unwrap());
        println!("typeof(string_len): {:?}", builtin_typeof(&[string_len]).unwrap());
        println!("typeof(len_as_bool): {:?}", builtin_typeof(&[len_as_bool]).unwrap());
    }
    
    #[test]
    fn test_memory_operations_demo() {
        println!("\n=== Memory Operations Demo ===");
        
        // Test with different data types
        let test_values = vec![
            RuntimeValue::Int8(127),
            RuntimeValue::Int32(2147483647),
            RuntimeValue::Float64(3.141592653589793),
            RuntimeValue::String("The quick brown fox jumps over the lazy dog".to_string()),
            RuntimeValue::Bool(true),
            RuntimeValue::Char('🦀'),
        ];
        
        for (i, value) in test_values.iter().enumerate() {
            println!("\n--- Value {} ---", i + 1);
            println!("Value: {:?}", value);
            
            let type_name = builtin_typeof(&[value.clone()]).unwrap();
            println!("Type: {:?}", type_name);
            
            let size = builtin_sizeof(&[value.clone()]).unwrap();
            println!("Size: {:?} bytes", size);
            
            if matches!(value, RuntimeValue::String(_)) {
                let length = builtin_len(&[value.clone()]).unwrap();
                println!("Length: {:?}", length);
            }
            
            let cloned = builtin_clone(&[value.clone()]).unwrap();
            println!("Clone successful: {}", cloned == *value);
        }
    }
    
    #[test]
    fn test_io_and_utility_comprehensive_demo() {
        println!("\n=== I/O and Utility Functions Comprehensive Demo ===");
        
        // Test all I/O functions
        println!("\n--- I/O Functions Test ---");
        
        // Basic print functions
        let _ = builtin_print(&[
            RuntimeValue::String("Testing print:".to_string()),
            RuntimeValue::Int32(123),
            RuntimeValue::Float64(45.67),
            RuntimeValue::Bool(true)
        ]);
        println!(); // Add newline after print
        
        let _ = builtin_println(&[
            RuntimeValue::String("Testing println with multiple values:".to_string()),
            RuntimeValue::String("Hello".to_string()),
            RuntimeValue::Char('W'),
            RuntimeValue::String("orld".to_string())
        ]);
        
        // Comprehensive printf testing
        println!("Testing printf format specifiers:");
        
        // Integer formats
        let _ = builtin_printf(&[
            RuntimeValue::String("Decimal: %d, Hex: %x, Octal: %o\n".to_string()),
            RuntimeValue::Int32(255),
            RuntimeValue::Int32(255),
            RuntimeValue::Int32(255)
        ]);
        
        // Float formats
        let _ = builtin_printf(&[
            RuntimeValue::String("Float (f): %f, Float (g): %g\n".to_string()),
            RuntimeValue::Float64(3.14159265359),
            RuntimeValue::Float64(3.14159265359)
        ]);
        
        // String and character formats
        let _ = builtin_printf(&[
            RuntimeValue::String("String: '%s', Character: '%c'\n".to_string()),
            RuntimeValue::String("Hello World".to_string()),
            RuntimeValue::Char('A')
        ]);
        
        // Boolean format
        let _ = builtin_printf(&[
            RuntimeValue::String("Boolean true: %b, Boolean false: %b\n".to_string()),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(false)
        ]);
        
        // Mixed formats
        let _ = builtin_printf(&[
            RuntimeValue::String("Name: %s, Age: %d, Score: %f, Active: %b\n".to_string()),
            RuntimeValue::String("Alice".to_string()),
            RuntimeValue::Int32(25),
            RuntimeValue::Float64(95.5),
            RuntimeValue::Bool(true)
        ]);
        
        // Test all utility functions
        println!("\n--- Utility Functions Test ---");
        
        let test_values = vec![
            RuntimeValue::Int8(-128),
            RuntimeValue::UInt16(65535),
            RuntimeValue::Float32(2.718),
            RuntimeValue::String("Bulu Language".to_string()),
            RuntimeValue::Bool(false),
            RuntimeValue::Char('🚀'),
            RuntimeValue::Null,
        ];
        
        for (i, value) in test_values.iter().enumerate() {
            println!("\n--- Testing Value {} ---", i + 1);
            
            // Test typeof
            let type_result = builtin_typeof(&[value.clone()]).unwrap();
            let _ = builtin_printf(&[
                RuntimeValue::String("Value: %v, Type: %s\n".to_string()),
                value.clone(),
                type_result.clone()
            ]);
            
            // Test instanceof with exact type
            if let RuntimeValue::String(type_name) = type_result {
                let instanceof_exact = builtin_instanceof(&[
                    value.clone(),
                    RuntimeValue::String(type_name.clone())
                ]).unwrap();
                let _ = builtin_printf(&[
                    RuntimeValue::String("instanceof('%s'): %b\n".to_string()),
                    RuntimeValue::String(type_name),
                    instanceof_exact
                ]);
            }
            
            // Test instanceof with categories
            let categories = vec!["integer", "float", "numeric", "signed", "unsigned", "primitive", "any"];
            for category in categories {
                let instanceof_result = builtin_instanceof(&[
                    value.clone(),
                    RuntimeValue::String(category.to_string())
                ]).unwrap();
                if let RuntimeValue::Bool(true) = instanceof_result {
                    let _ = builtin_printf(&[
                        RuntimeValue::String("  - instanceof('%s'): %b\n".to_string()),
                        RuntimeValue::String(category.to_string()),
                        instanceof_result
                    ]);
                }
            }
        }
        
        // Test error handling functions
        println!("\n--- Error Handling Functions Test ---");
        
        // Test successful assertions
        let _ = builtin_assert(&[RuntimeValue::Bool(true)]);
        println!("Assert(true) - passed");
        
        let _ = builtin_assert(&[RuntimeValue::Int32(42)]);
        println!("Assert(42) - passed (truthy)");
        
        let _ = builtin_assert(&[
            RuntimeValue::String("test".to_string())
        ]);
        println!("Assert(\"test\") - passed (truthy)");
        
        // Test recover function
        let recover_result = builtin_recover(&[]).unwrap();
        let _ = builtin_printf(&[
            RuntimeValue::String("Recover result: %v\n".to_string()),
            recover_result
        ]);
        
        // Test panic function (this will return an error)
        let panic_result = builtin_panic(&[
            RuntimeValue::String("This is a test panic".to_string())
        ]);
        println!("Panic function returned error: {}", panic_result.is_err());
        
        // Test failed assertion (this will return an error)
        let failed_assert = builtin_assert(&[RuntimeValue::Bool(false)]);
        println!("Failed assertion returned error: {}", failed_assert.is_err());
        
        println!("\n=== I/O and Utility Demo Complete ===");
    }
}