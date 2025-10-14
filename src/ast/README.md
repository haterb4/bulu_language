# Bulu AST (Abstract Syntax Tree) Documentation

The Bulu AST module provides comprehensive data structures and utilities for representing and manipulating the Abstract Syntax Tree of Bulu programs. This module is the foundation for all language processing operations including parsing, semantic analysis, optimization, and code generation.

## Overview

The AST module consists of four main components:

1. **Node Definitions** (`nodes.rs`) - Complete AST node type definitions
2. **Visitor Pattern** (`visitor.rs`) - Traversal and transformation utilities
3. **Builder Utilities** (`builder.rs`) - Programmatic AST construction
4. **Pretty Printer** (`printer.rs`) - Human-readable AST visualization

## Node Types

### Program Structure

The root of every AST is a `Program` node containing a list of top-level statements:

```rust
pub struct Program {
    pub statements: Vec<Statement>,
    pub position: Position,
}
```

### Statements

Statements represent executable code that doesn't return a value:

#### Declarations
- `VariableDecl` - Variable and constant declarations (`let x = 5`, `const PI = 3.14`)
- `FunctionDecl` - Function definitions with parameters, return types, and bodies
- `StructDecl` - Struct type definitions with fields and methods
- `InterfaceDecl` - Interface definitions with method signatures

#### Control Flow
- `IfStmt` - Conditional statements with optional else branches
- `WhileStmt` - While loops with condition and body
- `ForStmt` - For-in loops for iteration
- `MatchStmt` - Pattern matching statements
- `ReturnStmt` - Function return statements
- `BreakStmt` / `ContinueStmt` - Loop control statements
- `DeferStmt` - Deferred execution statements

#### Error Handling
- `TryStmt` - Try-catch blocks for error handling
- `FailStmt` - Error throwing statements

#### Module System
- `ImportStmt` - Module import statements
- `ExportStmt` - Module export statements

#### Other
- `ExpressionStmt` - Expression used as statement
- `BlockStmt` - Block of statements

### Expressions

Expressions represent code that evaluates to a value:

#### Literals and Identifiers
- `LiteralExpr` - Literal values (numbers, strings, booleans, null)
- `IdentifierExpr` - Variable and function references

#### Operations
- `BinaryExpr` - Binary operations (arithmetic, comparison, logical)
- `UnaryExpr` - Unary operations (negation, logical not)
- `AssignmentExpr` - Assignment operations

#### Function Calls and Access
- `CallExpr` - Function calls with arguments and type parameters
- `MemberAccessExpr` - Object member access (`obj.field`)
- `IndexExpr` - Array/map indexing (`arr[index]`)

#### Control Flow Expressions
- `IfExpr` - Conditional expressions (ternary-like)
- `MatchExpr` - Pattern matching expressions

#### Collections
- `ArrayExpr` - Array literals (`[1, 2, 3]`)
- `MapExpr` - Map literals (`{"key": "value"}`)

#### Functions
- `LambdaExpr` - Anonymous function expressions

#### Concurrency
- `AsyncExpr` - Async expressions
- `AwaitExpr` - Await expressions
- `RunExpr` - Goroutine spawn expressions
- `ChannelExpr` - Channel operations (send/receive)
- `SelectExpr` - Channel multiplexing

#### Type Operations
- `CastExpr` - Type casting (`expr as Type`)
- `TypeOfExpr` - Runtime type inspection

#### Other
- `RangeExpr` - Range expressions (`1..10`, `1...10`)
- `YieldExpr` - Generator yield expressions
- `ParenthesizedExpr` - Parenthesized expressions

### Types

The type system supports:

#### Primitive Types
- Integer types: `Int8`, `Int16`, `Int32`, `Int64`, `UInt8`, `UInt16`, `UInt32`, `UInt64`
- Floating point: `Float32`, `Float64`
- Other primitives: `Bool`, `Char`, `String`, `Any`

#### Composite Types
- `Array` - Fixed-size arrays with optional size specification
- `Slice` - Dynamic arrays
- `Map` - Key-value mappings
- `Function` - Function types with parameter and return types
- `Struct` - User-defined struct types
- `Interface` - Interface types
- `Generic` - Generic type parameters
- `Channel` - Channel types with direction and element type
- `Named` - Named type references

### Patterns

Pattern matching supports:

- `Wildcard` - Catch-all pattern (`_`)
- `Literal` - Literal value patterns
- `Identifier` - Variable binding patterns
- `Struct` - Struct destructuring patterns
- `Array` - Array destructuring patterns
- `Range` - Range patterns (`1..10`)
- `Or` - Alternative patterns (`pattern1 | pattern2`)

## Visitor Pattern

The visitor pattern enables traversal and transformation of AST nodes:

### Read-Only Visitor

```rust
impl Visitor<ReturnType> for MyVisitor {
    fn visit_program(&mut self, program: &Program) -> ReturnType {
        // Process program node
    }
    
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> ReturnType {
        // Process binary expression
        let left_result = self.visit_expression(&expr.left);
        let right_result = self.visit_expression(&expr.right);
        // Combine results
    }
}
```

### Mutable Visitor

```rust
impl MutVisitor for MyTransformer {
    fn visit_identifier_expr(&mut self, expr: &mut IdentifierExpr) {
        // Transform identifier names
        if expr.name == "old_name" {
            expr.name = "new_name".to_string();
        }
    }
}
```

### Walker Functions

Helper functions for common traversal patterns:

```rust
// Walk through statements
walk_statement(&mut visitor, statement);

// Walk through expressions  
walk_expression(&mut visitor, expression);

// Mutable versions
walk_statement_mut(&mut visitor, statement);
walk_expression_mut(&mut visitor, expression);
```

## Builder Utilities

The `AstBuilder` provides convenient methods for programmatic AST construction:

```rust
use bulu::ast::AstBuilder;

// Create a simple program
let program = AstBuilder::program(vec![
    AstBuilder::variable_decl("x", Some(AstBuilder::int32_type()), Some(AstBuilder::literal_int(42))),
    AstBuilder::function_decl(
        "main",
        vec![],
        None,
        AstBuilder::block_stmt(vec![
            AstBuilder::expression_stmt(AstBuilder::call_expr(
                AstBuilder::identifier("println"),
                vec![AstBuilder::literal_string("Hello, World!")],
            )),
        ]),
    ),
]);

// Create expressions
let binary_expr = AstBuilder::binary_expr(
    AstBuilder::identifier("x"),
    BinaryOperator::Add,
    AstBuilder::literal_int(1),
);

// Create types
let array_type = AstBuilder::array_type(AstBuilder::int32_type(), Some(10));
let function_type = AstBuilder::function_type(
    vec![AstBuilder::int32_type(), AstBuilder::string_type()],
    Some(AstBuilder::bool_type()),
);
```

### Convenience Macros

```rust
// Binary expressions
let expr = binary!(ident!("x"), +, int!(42));

// Identifiers
let id = ident!("variable_name");

// Integer literals
let num = int!(123);
```

## Pretty Printer

The `AstPrinter` provides human-readable AST visualization:

```rust
use bulu::ast::AstPrinter;

let mut printer = AstPrinter::new();
let output = printer.print_program(&program);
println!("{}", output);
```

Output example:
```
Program {
  Let x: int32 = 42
  Func main() {
    ExprStmt(Ident(println)("Hello, World!"))
  }
}
```

### Customization

```rust
// Custom indentation
let mut printer = AstPrinter::with_indent_size(4);

// Print individual components
let expr_output = printer.print_expression(&expression);
let type_output = printer.print_type(&type_node);
let pattern_output = printer.print_pattern(&pattern);
```

## Position Information

All AST nodes include position information for error reporting and IDE integration:

```rust
pub trait HasPosition {
    fn position(&self) -> Position;
}

// Usage
let pos = statement.position();
println!("Error at line {}, column {}", pos.line, pos.column);
```

## Usage Examples

### Creating a Function Declaration

```rust
let func_decl = AstBuilder::function_decl(
    "fibonacci",
    vec![
        AstBuilder::parameter("n", AstBuilder::int32_type()),
    ],
    Some(AstBuilder::int32_type()),
    AstBuilder::block_stmt(vec![
        AstBuilder::if_stmt(
            AstBuilder::binary_expr(
                AstBuilder::identifier("n"),
                BinaryOperator::LessEqual,
                AstBuilder::literal_int(1),
            ),
            AstBuilder::block_stmt(vec![
                AstBuilder::return_stmt(Some(AstBuilder::identifier("n"))),
            ]),
            Some(AstBuilder::return_stmt(Some(
                AstBuilder::binary_expr(
                    AstBuilder::call_expr(
                        AstBuilder::identifier("fibonacci"),
                        vec![AstBuilder::binary_expr(
                            AstBuilder::identifier("n"),
                            BinaryOperator::Subtract,
                            AstBuilder::literal_int(1),
                        )],
                    ),
                    BinaryOperator::Add,
                    AstBuilder::call_expr(
                        AstBuilder::identifier("fibonacci"),
                        vec![AstBuilder::binary_expr(
                            AstBuilder::identifier("n"),
                            BinaryOperator::Subtract,
                            AstBuilder::literal_int(2),
                        )],
                    ),
                ),
            ))),
        ),
    ]),
);
```

### Implementing a Node Counter Visitor

```rust
struct NodeCounter {
    function_count: usize,
    variable_count: usize,
}

impl Visitor<()> for NodeCounter {
    fn visit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
    }
    
    fn visit_function_decl(&mut self, _decl: &FunctionDecl) {
        self.function_count += 1;
    }
    
    fn visit_variable_decl(&mut self, _decl: &VariableDecl) {
        self.variable_count += 1;
    }
    
    // ... implement other required methods
}
```

### Creating Complex Types

```rust
// Generic function type: func<T>(T, T) -> T
let generic_func_type = Type::Function(FunctionType {
    param_types: vec![
        Type::Generic(GenericType {
            name: "T".to_string(),
            constraints: vec![],
        }),
        Type::Generic(GenericType {
            name: "T".to_string(),
            constraints: vec![],
        }),
    ],
    return_type: Some(Box::new(Type::Generic(GenericType {
        name: "T".to_string(),
        constraints: vec![],
    }))),
    is_async: false,
});

// Channel type: chan<- int32 (send-only channel of integers)
let send_channel_type = Type::Channel(ChannelType {
    element_type: Box::new(Type::Int32),
    direction: ChannelDirection::Send,
});
```

## Integration with Parser

The AST nodes are designed to be constructed by the parser:

```rust
// Parser creates AST nodes during parsing
impl Parser {
    fn parse_binary_expression(&mut self) -> Result<Expression> {
        let left = self.parse_primary()?;
        let operator = self.parse_binary_operator()?;
        let right = self.parse_primary()?;
        
        Ok(Expression::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            position: self.current_position(),
        }))
    }
}
```

## Testing

The AST module includes comprehensive tests:

- **33 AST tests** covering all node types and functionality
- **Builder tests** for programmatic construction
- **Visitor tests** for traversal and transformation
- **Printer tests** for visualization
- **Type system tests** for type representations

### Running Tests

```bash
# Run all AST tests
cargo test ast

# Run specific test categories
cargo test ast_tests
cargo test ast_printer_tests
cargo test ast_visitor_tests
```

## Performance Considerations

- **Memory Efficiency**: AST nodes use `Box<T>` for recursive structures to minimize memory usage
- **Clone Performance**: All nodes implement `Clone` for easy duplication during transformations
- **Position Tracking**: Position information is included in all nodes for precise error reporting
- **Visitor Optimization**: Walker functions provide efficient traversal patterns

## Future Enhancements

Potential improvements for future versions:

- **Serialization**: Add serde support for AST serialization/deserialization
- **Source Maps**: Enhanced position tracking for better IDE integration
- **Incremental Updates**: Support for incremental AST modifications
- **Memory Pools**: Custom allocators for large AST structures
- **Parallel Visitors**: Support for parallel AST traversal
- **AST Diffing**: Utilities for comparing and merging AST structures

## Integration Points

The AST module integrates with:

- **Lexer**: Position information from tokens
- **Parser**: AST construction during parsing
- **Semantic Analyzer**: Type checking and symbol resolution
- **Code Generator**: AST traversal for code emission
- **Optimizer**: AST transformations for optimization
- **IDE Tools**: AST-based language services