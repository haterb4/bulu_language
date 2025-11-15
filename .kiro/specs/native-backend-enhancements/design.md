# Design Document: Native Backend Enhancements

## Overview

This document describes the design for enhancing the Bulu native backend compiler to support advanced string operations, improved memory management, and additional data types. The design builds upon the existing string system that uses dynamic memory allocation with a heap-based allocator.

The enhancements will be implemented incrementally, starting with the most impactful features (string operations and variables) and progressing to more complex features (garbage collection, arrays, structs).

## Architecture

### Current Architecture

The native backend currently consists of:

1. **IR to Assembly Translator**: Converts intermediate representation to x86-64 assembly
2. **String System**: Dynamic string structures with `[length:8][data:length]` format
3. **Heap Allocator**: Simple bump allocator using `brk` syscall (1MB heap)
4. **Runtime Functions**: `__string_create`, `__string_concat`, `__string_uppercase`, `__string_repeat`, `__string_print`

### Enhanced Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Native Backend Compiler                   │
├─────────────────────────────────────────────────────────────┤
│  IR Analyzer                                                 │
│  - Type inference                                            │
│  - Lifetime analysis                                         │
│  - Escape analysis                                           │
├─────────────────────────────────────────────────────────────┤
│  Code Generator                                              │
│  - Function prologue/epilogue                                │
│  - Register allocation                                       │
│  - Instruction selection                                     │
│  - String operation lowering                                 │
│  - Array/struct layout                                       │
├─────────────────────────────────────────────────────────────┤
│  Runtime System                                              │
│  ┌───────────────┬──────────────────┬────────────────────┐  │
│  │ Memory Mgmt   │ String Ops       │ Type Support       │  │
│  │ - Allocator   │ - uppercase      │ - Arrays           │  │
│  │ - GC          │ - lowercase      │ - Slices           │  │
│  │ - Free list   │ - concat         │ - Structs          │  │
│  │               │ - trim           │ - Tuples           │  │
│  │               │ - substring      │                    │  │
│  └───────────────┴──────────────────┴────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Enhanced String Operations

#### Design Decisions

- All string operations return new string structures (immutable strings)
- String operations handle null pointers gracefully
- ASCII-only operations for initial implementation (UTF-8 in future)

#### Component Structure

```rust
// In native_backend.rs
impl NativeBackend {
    fn generate_string_operation(&self, op: &str, args: &[IrValue]) -> String {
        match op {
            "uppercase" => self.gen_uppercase(args),
            "lowercase" => self.gen_lowercase(args),
            "trim" => self.gen_trim(args),
            "substring" => self.gen_substring(args),
            "length" => self.gen_length(args),
            _ => panic!("Unknown string operation: {}", op),
        }
    }
}
```

#### Runtime Functions

**`__string_lowercase`**
- Input: %rdi = string structure pointer
- Output: %rax = new lowercase string pointer
- Algorithm: Copy string, convert 'A'-'Z' to 'a'-'z'

**`__string_trim`**
- Input: %rdi = string structure pointer
- Output: %rax = new trimmed string pointer
- Algorithm: Find first/last non-whitespace, create substring

**`__string_substring`**
- Input: %rdi = string pointer, %rsi = start index, %rdx = end index
- Output: %rax = new substring pointer
- Algorithm: Bounds check, allocate new string, copy range

**`__string_length`**
- Input: %rdi = string structure pointer
- Output: %rax = length as int64
- Algorithm: Load length field from structure

### 2. String Variable Support

#### Design Decisions

- String variables store pointers to string structures
- Stack allocation for local string variables
- Heap allocation for string data
- Pass-by-reference semantics for strings

#### Variable Storage Layout

```
Stack Frame:
┌──────────────────┐ <- %rbp
│  Return Address  │
├──────────────────┤ <- %rbp - 8
│  String Var 1    │ (8 bytes: pointer to string structure)
├──────────────────┤ <- %rbp - 16
│  String Var 2    │ (8 bytes: pointer to string structure)
├──────────────────┤
│  Int Var 1       │ (8 bytes: integer value)
└──────────────────┘
```

#### Code Generation Pattern

```assembly
; String variable declaration: let s = "hello"
lea str_0(%rip), %rdi      ; Load string literal address
mov $str_0_len, %rsi       ; Load string length
call __string_create       ; Create string structure
movq %rax, -8(%rbp)        ; Store pointer in variable

; String variable usage: println(s)
movq -8(%rbp), %rdi        ; Load string pointer
call __string_print        ; Print string
```

### 3. Memory Management Improvements

#### Design Decisions

- Implement simple mark-and-sweep garbage collector
- Use reference counting for immediate reclamation
- Maintain free list for efficient reallocation
- Trigger GC when heap usage exceeds 80%

#### Memory Block Structure

```
Memory Block:
┌──────────────────┐
│  Header (16B)    │
│  - size: 8B      │
│  - flags: 4B     │ (marked, type, etc.)
│  - refcount: 4B  │
├──────────────────┤
│  Data            │
│  (variable size) │
└──────────────────┘
```

#### Garbage Collection Algorithm

```
1. Mark Phase:
   - Start from stack roots (local variables)
   - Traverse all reachable objects
   - Set mark bit in header

2. Sweep Phase:
   - Scan entire heap
   - Free unmarked blocks
   - Add to free list
   - Clear mark bits

3. Compact Phase (optional):
   - Move live objects together
   - Update pointers
   - Reset heap pointer
```

#### Runtime Functions

**`__gc_alloc`**
- Allocate with GC support
- Check free list first
- Trigger GC if needed
- Update reference count

**`__gc_mark`**
- Mark object as reachable
- Recursively mark referenced objects

**`__gc_sweep`**
- Free unmarked objects
- Rebuild free list

**`__gc_collect`**
- Full garbage collection cycle
- Return amount of memory freed

### 4. Array and Slice Support

#### Design Decisions

- Arrays are fixed-size, allocated on stack or heap
- Slices are dynamic views into arrays
- Bounds checking on all accesses
- Slice structure: `[pointer:8][length:8][capacity:8]`

#### Array Layout

```
Fixed Array:
┌──────────────────┐
│  Element 0       │ (8 bytes per element for int64)
├──────────────────┤
│  Element 1       │
├──────────────────┤
│  ...             │
└──────────────────┘

Slice Structure:
┌──────────────────┐
│  Data Pointer    │ (8 bytes)
├──────────────────┤
│  Length          │ (8 bytes)
├──────────────────┤
│  Capacity        │ (8 bytes)
└──────────────────┘
```

#### Runtime Functions

**`__array_create`**
- Input: %rdi = element count, %rsi = element size
- Output: %rax = array pointer
- Allocate contiguous memory

**`__array_get`**
- Input: %rdi = array pointer, %rsi = index, %rdx = length
- Output: %rax = element pointer
- Bounds check, calculate offset

**`__slice_create`**
- Input: %rdi = array pointer, %rsi = start, %rdx = end, %rcx = capacity
- Output: %rax = slice structure pointer
- Create slice view

**`__slice_append`**
- Input: %rdi = slice pointer, %rsi = element pointer
- Output: %rax = new slice pointer (may reallocate)
- Check capacity, reallocate if needed, append element

### 5. Struct Type Support

#### Design Decisions

- Structs are value types (copied by default)
- Field offsets calculated at compile time
- Alignment requirements respected (8-byte alignment)
- Nested structs supported

#### Struct Layout

```
struct Person {
    age: int64,      // offset 0, size 8
    name: string,    // offset 8, size 8 (pointer)
    height: float64  // offset 16, size 8
}

Memory Layout:
┌──────────────────┐ offset 0
│  age (int64)     │
├──────────────────┤ offset 8
│  name (pointer)  │
├──────────────────┤ offset 16
│  height (float64)│
└──────────────────┘ total size: 24 bytes
```

#### Code Generation

```rust
struct StructLayout {
    name: String,
    fields: Vec<FieldLayout>,
    total_size: usize,
    alignment: usize,
}

struct FieldLayout {
    name: String,
    offset: usize,
    size: usize,
    type_info: TypeInfo,
}

impl NativeBackend {
    fn calculate_struct_layout(&self, struct_def: &StructDef) -> StructLayout {
        let mut offset = 0;
        let mut fields = Vec::new();
        
        for field in &struct_def.fields {
            let size = self.get_type_size(&field.type_);
            let alignment = self.get_type_alignment(&field.type_);
            
            // Align offset
            offset = (offset + alignment - 1) & !(alignment - 1);
            
            fields.push(FieldLayout {
                name: field.name.clone(),
                offset,
                size,
                type_info: field.type_.clone(),
            });
            
            offset += size;
        }
        
        StructLayout {
            name: struct_def.name.clone(),
            fields,
            total_size: offset,
            alignment: 8,
        }
    }
}
```

### 6. Arithmetic and Logical Operations

#### Design Decisions

- All arithmetic operations check for overflow/underflow
- Division checks for divide-by-zero
- Short-circuit evaluation for logical operators
- Comparison operations set flags, then convert to boolean

#### Overflow Checking

```assembly
; Addition with overflow check
mov -8(%rbp), %rax         ; Load first operand
add -16(%rbp), %rax        ; Add second operand
jo .overflow_error         ; Jump if overflow
movq %rax, -24(%rbp)       ; Store result

.overflow_error:
    lea .overflow_msg(%rip), %rdi
    call __panic
```

#### Short-Circuit Evaluation

```assembly
; Logical AND: a && b
movq -8(%rbp), %rax        ; Load a
test %rax, %rax            ; Test if a is false
jz .and_false              ; If false, skip b evaluation
movq -16(%rbp), %rax       ; Load b
test %rax, %rax            ; Test if b is false
jz .and_false              ; If false, result is false
mov $1, %rax               ; Both true, result is true
jmp .and_done
.and_false:
    mov $0, %rax            ; Result is false
.and_done:
    movq %rax, -24(%rbp)    ; Store result
```

### 7. Control Flow Structures

#### Design Decisions

- Use conditional jumps for if-else
- Use loop labels for while/for loops
- Support break/continue with label tracking
- Optimize tail calls where possible

#### If-Else Code Generation

```assembly
; if (condition) { then_block } else { else_block }
movq -8(%rbp), %rax        ; Load condition
test %rax, %rax            ; Test condition
jz .else_label             ; Jump to else if false
; then_block code here
jmp .endif_label           ; Skip else block
.else_label:
; else_block code here
.endif_label:
```

#### While Loop Code Generation

```assembly
; while (condition) { body }
.while_start:
    movq -8(%rbp), %rax    ; Load condition
    test %rax, %rax        ; Test condition
    jz .while_end          ; Exit if false
    ; body code here
    jmp .while_start       ; Loop back
.while_end:
```

### 8. Function Call Improvements

#### Design Decisions

- Follow System V AMD64 ABI calling convention
- First 6 integer/pointer args in registers
- Additional args on stack (right-to-left)
- Return values in %rax (or %rax:%rdx for 128-bit)
- Caller saves volatile registers
- Callee saves non-volatile registers

#### Calling Convention

```
Argument Registers (in order):
1. %rdi
2. %rsi
3. %rdx
4. %rcx
5. %r8
6. %r9

Stack Arguments (for 7+ parameters):
┌──────────────────┐ <- %rsp + 8
│  Arg 7           │
├──────────────────┤ <- %rsp + 16
│  Arg 8           │
├──────────────────┤
│  ...             │
└──────────────────┘

Return Values:
- Single value: %rax
- Two values: %rax, %rdx
- Large structs: pointer in %rax
```

#### Code Generation

```rust
impl NativeBackend {
    fn generate_function_call(&self, call: &IrCall) -> String {
        let mut asm = String::new();
        let arg_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
        
        // Pass first 6 args in registers
        for (i, arg) in call.args.iter().take(6).enumerate() {
            asm.push_str(&format!("    movq {}, {}\n", 
                self.load_value(arg), arg_regs[i]));
        }
        
        // Pass remaining args on stack (reverse order)
        for arg in call.args.iter().skip(6).rev() {
            asm.push_str(&format!("    pushq {}\n", self.load_value(arg)));
        }
        
        // Call function
        asm.push_str(&format!("    call {}\n", call.function_name));
        
        // Clean up stack args
        let stack_args = call.args.len().saturating_sub(6);
        if stack_args > 0 {
            asm.push_str(&format!("    add ${}, %rsp\n", stack_args * 8));
        }
        
        // Store return value
        if let Some(result) = &call.result {
            asm.push_str(&format!("    movq %rax, {}\n", 
                self.get_var_location(result)));
        }
        
        asm
    }
}
```

### 9. Error Handling and Debugging

#### Design Decisions

- Runtime errors print to stderr and exit with code 1
- Stack traces use return address inspection
- Debug mode adds line number information
- Panic function never returns

#### Error Handling Functions

**`__panic`**
- Input: %rdi = error message pointer
- Print error message to stderr
- Print stack trace
- Exit with code 1

**`__bounds_check_failed`**
- Input: %rdi = index, %rsi = length
- Print bounds error message
- Call __panic

**`__division_by_zero`**
- Print division by zero error
- Call __panic

**`__stack_trace`**
- Walk stack frames using %rbp chain
- Print return addresses
- Resolve function names from symbol table (if available)

## Data Models

### String Structure

```c
struct String {
    uint64_t length;
    char data[];  // Flexible array member
};
```

### Slice Structure

```c
struct Slice {
    void* data;
    uint64_t length;
    uint64_t capacity;
};
```

### Memory Block Header

```c
struct BlockHeader {
    uint64_t size;
    uint32_t flags;  // bit 0: marked, bits 1-7: type
    uint32_t refcount;
};
```

## Error Handling

### Runtime Errors

1. **Out of Memory**: Return null pointer, caller checks and panics
2. **Bounds Check Failure**: Immediate panic with index and length
3. **Division by Zero**: Immediate panic
4. **Null Pointer Dereference**: Segfault (caught by OS)
5. **Stack Overflow**: Segfault (caught by OS)

### Error Messages

All error messages follow this format:
```
Runtime Error: <error type>
  at <function>:<line>
  <additional context>

Stack trace:
  0: <function> at <file>:<line>
  1: <function> at <file>:<line>
  ...
```

## Testing Strategy

### Unit Tests

1. **String Operations**
   - Test each string function with various inputs
   - Test null pointer handling
   - Test empty strings
   - Test very long strings

2. **Memory Management**
   - Test allocation and deallocation
   - Test garbage collection triggers
   - Test memory leak detection
   - Test fragmentation handling

3. **Arrays and Slices**
   - Test array creation and access
   - Test bounds checking
   - Test slice operations
   - Test slice reallocation

4. **Arithmetic Operations**
   - Test overflow detection
   - Test underflow detection
   - Test division by zero
   - Test all operators

### Integration Tests

1. **String Processing Program**
   - Read input, process with string functions, output result
   - Test with various string operations combined

2. **Array Manipulation Program**
   - Create arrays, sort, search, modify
   - Test with different array sizes

3. **Struct Usage Program**
   - Define structs, create instances, access fields
   - Test nested structs

4. **Complex Control Flow**
   - Nested loops, conditionals, function calls
   - Test break/continue behavior

### Performance Tests

1. **String Concatenation**
   - Measure time for concatenating N strings
   - Compare with interpreted version

2. **Array Operations**
   - Measure time for array creation, access, modification
   - Test with various array sizes

3. **Memory Allocation**
   - Measure allocation/deallocation speed
   - Test GC overhead

4. **Function Calls**
   - Measure function call overhead
   - Test with various parameter counts

## Implementation Phases

### Phase 1: Enhanced String Operations (Week 1)
- Implement lowercase, trim, substring, length
- Add string variable support
- Test with string-utils example

### Phase 2: Memory Management (Week 2)
- Implement memory block headers
- Add reference counting
- Implement simple mark-and-sweep GC
- Test with memory-intensive programs

### Phase 3: Arrays and Slices (Week 3)
- Implement array creation and access
- Add bounds checking
- Implement slice operations
- Test with array manipulation programs

### Phase 4: Structs (Week 4)
- Implement struct layout calculation
- Add struct field access
- Support nested structs
- Test with struct-heavy programs

### Phase 5: Arithmetic and Control Flow (Week 5)
- Implement overflow checking
- Add comparison and logical operations
- Implement if-else and loops
- Test with computational programs

### Phase 6: Function Calls and Error Handling (Week 6)
- Improve function calling convention
- Add stack argument support
- Implement error handling and debugging
- Comprehensive integration testing
