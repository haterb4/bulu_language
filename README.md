# Bulu Programming Language

A modern programming language with strong concurrency support, memory safety, and expressive syntax.

## Features

- **33 Keywords**: Carefully designed minimal keyword set
- **Strong Type System**: Static typing with type inference and generics
- **Memory Safety**: Garbage collection with concurrent mark-and-sweep
- **Concurrency**: Built-in goroutines, channels, and async/await
- **Pattern Matching**: Powerful match expressions with destructuring
- **Error Handling**: Try-fail blocks with defer for cleanup
- **Interoperability**: C FFI support for existing libraries

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/bulu-lang/bulu.git
cd bulu

# Build the compiler
cargo build --release

# Add to PATH (optional)
export PATH=$PATH:$(pwd)/target/release
```

### Hello World

Create a new project:
```bash
lang new hello-world
cd hello-world
```

Edit `src/main.bu`:
```bulu
func main() {
    println("Hello, World!")
}
```

Run the program:
```bash
lang run
```

## Language Overview

### Variables and Types

```bulu
// Variables
let x = 42              // int32 (inferred)
let name = "Alice"      // string
let active: bool = true // explicit type

// Constants
const PI = 3.14159
const MAX_SIZE = 1000
```

### Functions

```bulu
func add(a: int32, b: int32): int32 {
    return a + b
}

// Multiple return values
func divmod(a: int32, b: int32): (int32, int32) {
    return a / b, a % b
}

// Anonymous functions
let square = (x: int32) => x * x
```

### Structs and Interfaces

```bulu
struct Point {
    x: float64
    y: float64
    
    func distance(): float64 {
        return sqrt(this.x * this.x + this.y * this.y)
    }
}

interface Shape {
    func area(): float64
}
```

### Concurrency

```bulu
// Goroutines
run processData(data)

// Channels
let ch = make(chan int32)
run producer(ch)
let value = <-ch

// Async/await
async func fetchData(url: string): string {
    let response = await http.get(url)
    return response.text()
}
```

### Error Handling

```bulu
func divide(a: float64, b: float64): float64 {
    try {
        if b == 0.0 {
            fail "Division by zero"
        }
        return a / b
    } fail on err {
        println("Error: " + err)
        return 0.0
    }
}
```

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/bulu-lang/bulu.git
cd bulu

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Development

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --open
```

## Project Structure

```
bulu/
├── src/
│   ├── lexer/          # Tokenization
│   ├── parser/         # Syntax analysis
│   ├── ast/            # Abstract syntax tree
│   ├── compiler/       # Semantic analysis & codegen
│   ├── runtime/        # Runtime system & GC
│   ├── types/          # Type system
│   └── bin/            # Command-line tools
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
├── examples/           # Example programs
└── docs/               # Documentation
```

## Command-Line Tools

### `langc` - Compiler

```bash
# Compile a Bulu program
langc main.bu -o main

# Optimization levels
langc main.bu -O3 -o main

# Emit intermediate representations
langc main.bu --emit tokens
langc main.bu --emit ast
langc main.bu --emit ir

# Cross-compilation
langc main.bu --target linux-amd64
langc main.bu --target windows-amd64
```

### `lang` - Project Tool

```bash
# Create new project
lang new my-project

# Build project
lang build
lang build --release

# Run project
lang run
lang run -- arg1 arg2

# Development tools
lang test           # Run tests
lang fmt            # Format code
lang lint           # Run linter
lang doc            # Generate docs
lang clean          # Clean artifacts
```

## Language Specification

For detailed language specification, see [docs/specification.md](docs/specification.md).

## Examples

See the [examples/](examples/) directory for sample programs demonstrating various language features.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [x] Lexical analysis
- [ ] Parser implementation
- [ ] AST construction
- [ ] Type system
- [ ] Code generation
- [ ] Runtime system
- [ ] Standard library
- [ ] Package manager
- [ ] IDE support

## Community

- [GitHub Discussions](https://github.com/bulu-lang/bulu/discussions)
- [Discord Server](https://discord.gg/bulu-lang)
- [Reddit Community](https://reddit.com/r/bulu)

---

**Note**: Bulu is currently in early development. The language specification and implementation are subject to change.