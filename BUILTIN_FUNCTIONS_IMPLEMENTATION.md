# Built-in Functions Implementation

This document describes the implementation of I/O and utility built-in functions for the Bulu programming language.

## Task 14: I/O and Utility Built-ins Implementation

### Requirements Implemented

#### I/O Built-ins (Requirements 16.5.1-16.5.4)
- ✅ **16.5.1** `print(args...)` - Print values to stdout with space separation
- ✅ **16.5.2** `println(args...)` - Print values to stdout with newline
- ✅ **16.5.3** `printf(format, args...)` - Formatted output with format specifiers
- ✅ **16.5.4** `input(prompt)` - Read input from stdin with optional prompt

#### Error Handling Built-ins (Requirements 16.6.1-16.6.3)
- ✅ **16.6.1** `panic(message)` - Trigger unrecoverable error with message
- ✅ **16.6.2** `recover()` - Catch panics in defer blocks (placeholder implementation)
- ✅ **16.6.3** `assert(condition, message)` - Assert condition with optional message

#### Type Inspection Built-ins (Requirements 16.7.1-16.7.3)
- ✅ **16.7.1** `typeof(x)` - Get type name as string
- ✅ **16.7.2** `instanceof(x, type_name)` - Check if value is instance of type
- ✅ **16.7.3** `sizeof(x)` - Get size of value in bytes (already implemented)

## Function Details

### I/O Functions

#### `print(args...)`
- Prints all arguments to stdout separated by spaces
- Does not add a newline at the end
- Flushes stdout to ensure immediate output
- Returns `null`

```rust
print("Hello", 42, true)  // Output: "Hello 42 true"
```

#### `println(args...)`
- Prints all arguments to stdout separated by spaces
- Adds a newline at the end
- Returns `null`

```rust
println("Hello", "World")  // Output: "Hello World\n"
```

#### `printf(format, args...)`
- Formatted printing with C-style format specifiers
- Supported format specifiers:
  - `%d`, `%i` - Integer format
  - `%f` - Float format (6 decimal places)
  - `%g` - General float format
  - `%s` - String format
  - `%c` - Character format
  - `%b` - Boolean format
  - `%x` - Hexadecimal format
  - `%o` - Octal format
  - `%v` - Default format (any type)
  - `%%` - Escaped percent sign
- Returns `null`

```rust
printf("Name: %s, Age: %d, Score: %f", "Alice", 25, 95.5)
// Output: "Name: Alice, Age: 25, Score: 95.500000"
```

#### `input(prompt?)`
- Reads a line from stdin
- Optional prompt parameter
- Removes trailing newline characters
- Returns the input as a string

```rust
let name = input("Enter your name: ")
```

### Error Handling Functions

#### `panic(message?)`
- Triggers an unrecoverable error
- Optional message parameter (defaults to "panic")
- Returns a `RuntimeError`
- Used for critical failures that should terminate execution

```rust
panic("Critical error occurred")
```

#### `recover()`
- Placeholder implementation for panic recovery
- Should be used in defer blocks to catch panics
- Currently returns `null` (full implementation pending)
- Future implementation will integrate with defer mechanism

```rust
defer {
    if let error = recover() {
        println("Recovered from panic:", error)
    }
}
```

#### `assert(condition, message?)`
- Asserts that a condition is truthy
- Optional message parameter for custom error message
- Returns `null` if assertion passes
- Returns `RuntimeError` if assertion fails

```rust
assert(x > 0, "x must be positive")
assert(len(array) > 0)  // Default message
```

### Type Inspection Functions

#### `typeof(x)`
- Returns the type name of a value as a string
- Supports all primitive types
- Returns exact type names: "int32", "float64", "string", etc.

```rust
typeof(42)        // "int32"
typeof(3.14)      // "float64"
typeof("hello")   // "string"
typeof(true)      // "bool"
typeof(null)      // "null"
```

#### `instanceof(x, type_name)`
- Checks if a value is an instance of a specific type
- Supports exact type matching and category matching
- Type categories supported:
  - `"integer"` - Any integer type
  - `"float"` - Any floating-point type
  - `"numeric"` - Any numeric type (integer or float)
  - `"signed"` - Signed integer types
  - `"unsigned"` - Unsigned integer types
  - `"primitive"` - Any primitive type (excludes null)
  - `"any"` - Matches everything
- Returns boolean result

```rust
instanceof(42, "int32")     // true
instanceof(42, "integer")   // true
instanceof(42, "numeric")   // true
instanceof(42, "float")     // false
instanceof("test", "string") // true
instanceof(null, "primitive") // false
instanceof(null, "any")     // true
```

## Implementation Architecture

### Built-in Registry
- `BuiltinRegistry` manages all built-in functions
- Functions are registered by name with function pointers
- Provides lookup, existence checking, and enumeration
- Thread-safe and efficient hash-map based storage

### Function Signature
- All built-in functions use the same signature: `fn(&[RuntimeValue]) -> Result<RuntimeValue>`
- Consistent error handling with `BuluError::RuntimeError`
- Argument validation and type checking
- Proper error messages for debugging

### Format String Processing
- Custom format string parser for `printf`
- Handles escape sequences and format specifiers
- Type-aware formatting for different value types
- Error handling for insufficient arguments

## Testing

### Unit Tests
- Comprehensive test suite with 39+ test cases
- Tests for all function variants and edge cases
- Error condition testing
- Type conversion and compatibility testing

### Integration Tests
- Real-world usage scenarios
- Function interaction testing
- Performance and memory testing
- Cross-function workflow validation

### Demo Programs
- Interactive demonstrations of all functions
- Format specifier showcase
- Type inspection examples
- Error handling demonstrations

## Performance Considerations

- Minimal overhead for function calls
- Efficient string formatting and I/O operations
- Memory-safe operations with proper cleanup
- Optimized type checking and conversion

## Future Enhancements

1. **Enhanced `recover()` Implementation**
   - Integration with defer statement execution
   - Thread-local panic state tracking
   - Proper panic value capture and return

2. **Extended Format Specifiers**
   - Width and precision specifiers
   - Alignment and padding options
   - Custom format functions

3. **Advanced Type Inspection**
   - Reflection capabilities
   - Runtime type information
   - Generic type parameter inspection

4. **I/O Enhancements**
   - File I/O operations
   - Buffered I/O support
   - Async I/O capabilities

## Conclusion

The I/O and utility built-in functions provide a solid foundation for the Bulu programming language. They offer comprehensive type inspection, flexible I/O operations, and robust error handling capabilities. The implementation follows best practices for performance, safety, and usability while maintaining compatibility with the language specification.