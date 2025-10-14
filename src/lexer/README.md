# Bulu Lexer Documentation

The Bulu lexer is responsible for converting source code text into a stream of tokens that can be consumed by the parser. It implements a complete tokenizer for the Bulu programming language.

## Features

### Token Types

The lexer recognizes all 33 keywords defined in the Bulu language specification:

#### Control Flow Keywords (8)
- `if`, `else`, `while`, `for`, `break`, `continue`, `return`, `match`

#### Declaration Keywords (6)
- `let`, `const`, `func`, `struct`, `interface`, `as`

#### Literal Keywords (3)
- `true`, `false`, `null`

#### Logical Operator Keywords (3)
- `and`, `or`, `not`

#### Module System Keywords (2)
- `import`, `export`

#### Error Handling Keywords (3)
- `try`, `fail`, `defer`

#### Concurrency Keywords (6)
- `async`, `await`, `run`, `chan`, `lock`, `select`

#### Generator Keywords (1)
- `yield`

#### Special Keywords (1)
- `as` (type casting)

### Operators and Delimiters

The lexer supports all Bulu operators:

#### Arithmetic Operators
- `+`, `-`, `*`, `/`, `%`, `**` (power)

#### Comparison Operators
- `==`, `!=`, `<`, `>`, `<=`, `>=`

#### Assignment Operators
- `=`, `+=`, `-=`, `*=`, `/=`, `%=`

#### Bitwise Operators
- `&`, `|`, `^`, `~`, `<<`, `>>`

#### Arrow Operators
- `<-` (channel send), `->` (function return), `=>` (fat arrow)

#### Delimiters
- `(`, `)`, `{`, `}`, `[`, `]`
- `,`, `;`, `:`, `.`, `..`, `..<`, `...`, `?`

### Literals

#### Integer Literals
- Decimal: `42`, `123456789`
- Hexadecimal: `0xFF`, `0xDEADBEEF`
- Octal: `0o777`, `0o123`
- Binary: `0b1010`, `0b11111111`

#### Float Literals
- Basic: `3.14`, `0.5`, `123.456`
- Scientific notation: `1e10`, `1.5e-3`, `2E+5`
- Edge cases: `.5`, `5.`, `1.`, `.0`

#### String Literals
- Basic: `"hello world"`
- With escapes: `"line1\nline2\ttab"`
- Multiline strings supported
- Unicode content: `"Hello, 世界! 🚀"`

#### Character Literals
- Basic: `'a'`, `'Z'`, `'5'`
- Escaped: `'\n'`, `'\t'`, `'\\'`, `'\''`, `'\"'`, `'\0'`

#### Boolean Literals
- `true`, `false`

#### Null Literal
- `null`

### Comments

#### Single-line Comments
```bulu
// This is a single-line comment
let x = 42  // Comment at end of line
```

#### Multi-line Comments
```bulu
/* This is a
   multi-line comment */
```

#### Nested Comments
```bulu
/* Outer comment
   /* Nested comment */
   Still in outer comment
*/
```

### Identifiers

Identifiers can contain:
- Letters (including Unicode)
- Digits (not as first character)
- Underscores

Examples:
- `variable_name`
- `CamelCase`
- `_private`
- `café` (Unicode support)
- `变量` (Unicode support)

## Usage

### Basic Usage

```rust
use bulu::lexer::Lexer;

let source = r#"
func main() {
    let x = 42
    println("Hello, World!")
}
"#;

let mut lexer = Lexer::new(source);
let tokens = lexer.tokenize()?;

for token in tokens {
    println!("{:?}", token);
}
```

### Token Structure

Each token contains:
- `token_type`: The type of token (keyword, operator, literal, etc.)
- `lexeme`: The original text from the source
- `literal`: Optional parsed value for literals
- `position`: Line, column, and offset information for error reporting

```rust
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub position: Position,
}
```

### Error Handling

The lexer provides detailed error information:

```rust
match lexer.tokenize() {
    Ok(tokens) => { /* process tokens */ },
    Err(BuluError::LexError { message, line, column }) => {
        eprintln!("Lexical error at {}:{}: {}", line, column, message);
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Implementation Details

### Performance Characteristics

- **Time Complexity**: O(n) where n is the length of input
- **Space Complexity**: O(n) for storing tokens
- **Memory Usage**: Efficient string handling with minimal allocations
- **Unicode Support**: Full Unicode identifier and string support

### Position Tracking

The lexer maintains accurate position information:
- Line numbers (1-based)
- Column numbers (1-based)
- Byte offsets (0-based)

This enables precise error reporting and IDE integration.

### Comment Handling

Comments are filtered out during tokenization but can be preserved if needed for documentation generation or formatting tools.

### Number Parsing

The lexer handles various number formats:
- Automatic base detection (decimal, hex, octal, binary)
- Scientific notation for floats
- Proper overflow detection
- Accurate floating-point parsing

### String Processing

String literals support:
- Standard escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\0`)
- Unicode content
- Multiline strings
- Proper error handling for unterminated strings

## Testing

The lexer is thoroughly tested with:
- **40+ unit tests** covering all functionality
- **Edge case testing** for Unicode, large inputs, and error conditions
- **Performance testing** for large files and complex structures
- **Error handling testing** for all error conditions

### Test Categories

1. **Basic Functionality Tests** (`tests/lexer_tests.rs`)
   - Keywords, operators, literals
   - Comments and identifiers

2. **Comprehensive Tests** (`tests/lexer_comprehensive_tests.rs`)
   - All token types
   - Error handling
   - Position tracking
   - Complex expressions

3. **Edge Case Tests** (`tests/lexer_edge_cases.rs`)
   - Unicode support
   - Large inputs
   - Nested structures
   - Boundary conditions

4. **Performance Tests** (`tests/lexer_performance.rs`)
   - Large file handling
   - Memory usage
   - Processing speed

## Future Enhancements

Potential improvements for future versions:
- Incremental tokenization for IDE support
- Token stream caching
- Parallel tokenization for very large files
- Custom token types for language extensions
- Source map generation for transpilation

## Integration

The lexer integrates seamlessly with:
- **Parser**: Provides token stream for syntax analysis
- **Error Reporter**: Accurate position information
- **IDE Tools**: Token-based syntax highlighting
- **Formatter**: Preserves original formatting information
- **Debugger**: Source location mapping