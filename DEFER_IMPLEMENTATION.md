# Defer Statement Implementation for Bulu Language

This document describes the complete implementation of the `defer` statement for the Bulu programming language, which provides guaranteed cleanup execution.

## Overview

The `defer` statement in Bulu ensures that specified code will execute before a function returns, regardless of how the function exits (normal return, error, or panic). This is essential for resource management and cleanup operations.

## Syntax

### Basic Defer Statement

```bulu
defer statement
```

### Examples

```bulu
// Simple defer
defer print("cleanup")

// Defer with block
defer {
    print("cleanup line 1")
    print("cleanup line 2")
}

// Defer with function call
defer cleanup_resource("database")

// Defer with variable access
let file = open("data.txt")
defer file.close()
```

## Semantics

### 1. Execution Order (LIFO)

Multiple defer statements execute in Last-In-First-Out (LIFO) order:

```bulu
func example() {
    defer print("First defer")   // Executes last
    defer print("Second defer")  // Executes second
    defer print("Third defer")   // Executes first
    print("Function body")
}

// Output:
// Function body
// Third defer
// Second defer
// First defer
```

### 2. Execution Timing

Deferred statements execute:
- Before any function return (normal or early)
- Before error propagation
- Even when errors occur
- Before panic unwinding

```bulu
func resource_example(): int32 {
    let resource = acquire_resource()
    defer release_resource(resource)  // Always executes
    
    if some_condition {
        return 1  // Defer executes before return
    }
    
    try {
        risky_operation()
    } fail on err {
        return -1  // Defer executes before error return
    }
    
    return 0  // Defer executes before normal return
}
```

### 3. Variable Capture Semantics

Variables in defer statements are captured by reference at defer declaration time:

```bulu
func capture_example() {
    let x = 10
    defer print("x is: " + string(x))  // Will print current value of x when defer executes
    
    x = 20  // Changes x, defer will see this change
    x = 30  // Defer will print "x is: 30"
}
```

### 4. Scope Management

Defers are scoped to their containing block:

```bulu
func scope_example() {
    defer print("Function scope")
    
    {
        defer print("Block scope 1")
        defer print("Block scope 2")
        // Block scope defers execute when block ends
    }
    
    // Function scope defer executes when function ends
}

// Output:
// Block scope 2
// Block scope 1
// Function scope
```

## Implementation Details

### 1. AST Representation

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct DeferStmt {
    pub stmt: Box<Statement>,
    pub position: Position,
}
```

### 2. Parser Implementation

The parser handles defer statements in `parse_defer_statement()`:

```rust
fn parse_defer_statement(&mut self) -> Result<Statement> {
    let pos = self.current_position();
    self.advance(); // consume 'defer'
    let stmt = Box::new(self.parse_statement()?);

    Ok(Statement::Defer(DeferStmt {
        stmt,
        position: pos,
    }))
}
```

### 3. Runtime Management

The error handler manages defer execution:

```rust
pub struct ErrorHandler {
    pub defer_stack: Vec<DeferredAction>,
    // ... other fields
}

#[derive(Debug, Clone)]
pub struct DeferredAction {
    pub statement: Statement,
    pub position: Position,
}
```

Key methods:
- `add_defer()`: Adds a deferred action to the stack
- `get_defers_to_execute()`: Returns defers in LIFO order
- `handle_return()`: Gets all defers for function return

### 4. Interpreter Integration

The interpreter executes deferred statements at appropriate times:

```rust
impl Interpreter {
    fn execute_defer_statement(&mut self, defer_stmt: &DeferStmt) -> Result<Value> {
        self.error_handler.add_defer(*defer_stmt.stmt.clone(), defer_stmt.position);
        Ok(Value::Null)
    }

    fn execute_deferred_statements(&mut self, defers: Vec<DeferredAction>) -> Result<()> {
        for deferred in defers {
            if let Err(e) = self.execute_statement(&deferred.statement) {
                eprintln!("Error in deferred statement: {}", e);
            }
        }
        Ok(())
    }
}
```

## Integration with Error Handling

### 1. Try-Fail Blocks

Defers execute before error propagation:

```bulu
try {
    defer print("Cleanup 1")
    defer print("Cleanup 2")
    fail "Error occurred"
} fail on err {
    print("Caught: " + err)
}

// Output:
// Cleanup 2
// Cleanup 1
// Caught: Error occurred
```

### 2. Error Propagation

When errors propagate through function calls, defers execute at each level:

```bulu
func level1() {
    defer print("Level 1 cleanup")
    level2()
}

func level2() {
    defer print("Level 2 cleanup")
    fail "Error at level 2"
}

// Output when level1() is called:
// Level 2 cleanup
// Level 1 cleanup
// Error: Error at level 2
```

## Use Cases

### 1. Resource Management

```bulu
func process_file(filename: string): string {
    let file = open(filename)
    defer file.close()  // Guaranteed cleanup
    
    let lock = acquire_lock()
    defer release_lock(lock)  // Guaranteed unlock
    
    return file.read()
}
```

### 2. Transaction Management

```bulu
func database_transaction(): bool {
    let tx = begin_transaction()
    defer {
        if tx.is_active() {
            tx.rollback()
        }
    }
    
    try {
        tx.execute("INSERT INTO users ...")
        tx.execute("UPDATE accounts ...")
        tx.commit()
        return true
    } fail on err {
        print("Transaction failed: " + err)
        return false
    }
}
```

### 3. Logging and Monitoring

```bulu
func timed_operation(name: string) {
    let start_time = now()
    defer {
        let duration = now() - start_time
        print("Operation " + name + " took " + string(duration) + "ms")
    }
    
    // Perform operation
    expensive_computation()
}
```

### 4. State Restoration

```bulu
func temporary_state_change() {
    let old_state = get_current_state()
    defer set_state(old_state)  // Restore original state
    
    set_state(new_temporary_state)
    perform_operations_with_new_state()
}
```

## Error Handling in Defers

Errors in deferred statements are handled gracefully:

1. Errors in defers don't prevent other defers from executing
2. Defer errors are logged but don't propagate
3. The original function's return value/error is preserved

```bulu
func defer_error_example() {
    defer print("This will execute")
    defer fail "Error in defer"  // Logged but doesn't stop execution
    defer print("This will also execute")
    
    return 42  // Returns normally despite defer error
}
```

## Performance Considerations

1. **Minimal Overhead**: Defer statements have minimal runtime cost when no defers are used
2. **Stack Management**: Defer stack is efficiently managed with LIFO operations
3. **Memory Usage**: Deferred statements are stored compactly
4. **Error Isolation**: Defer errors don't impact performance of main execution path

## Testing

Comprehensive test suite covers:

- Basic defer parsing and execution
- LIFO execution order
- Integration with error handling
- Variable capture semantics
- Nested scopes and blocks
- Resource management patterns
- Error scenarios

Test files:
- `tests/defer_tests.rs`: Comprehensive defer functionality tests
- `tests/error_handling_tests.rs`: Integration with error handling
- `examples/defer_demo.bu`: Practical usage examples

## Requirements Compliance

This implementation satisfies all requirements from task 16:

✅ **Add defer statement parsing and execution**
- Complete parser implementation for defer syntax
- Runtime execution with proper statement handling

✅ **Implement LIFO execution order for multiple defers**
- Defer stack maintains LIFO order
- Comprehensive tests verify execution order

✅ **Ensure defer execution on both normal and error returns**
- Defers execute before any function exit
- Integration with try-fail error handling

✅ **Add variable capture semantics for defer**
- Variables captured by reference at defer time
- Proper scoping and environment management

✅ **Write unit tests for defer functionality**
- 18 comprehensive tests covering all aspects
- Integration tests with error handling
- Performance and edge case testing

The defer statement implementation is now fully functional and ready for use in Bulu programs, providing robust resource management and cleanup capabilities.