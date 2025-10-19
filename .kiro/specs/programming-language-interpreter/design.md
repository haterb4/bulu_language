# Complete Programming Language Specification v1.0

## Table of Contents
1. [Keywords](#1-keywords)
2. [Syntax Specification](#2-syntax-specification)
3. [Type System](#3-type-system)
4. [Memory Model](#4-memory-model)
5. [Built-in Functions](#5-built-in-functions)
6. [Standard Library](#6-standard-library)
7. [Compilation & Toolchain](#7-compilation--toolchain)
8. [Example Programs](#8-example-programs)

---

## 1. Keywords (33 Total)

### Control Flow (8)
- `if` - conditional execution
- `else` - alternative branch
- `while` - loop with condition
- `for` - iteration loop
- `break` - exit loop early
- `continue` - skip to next iteration
- `return` - exit function with value
- `match` - pattern matching

### Declarations (6)
- `let` - mutable variable declaration
- `const` - immutable constant declaration
- `func` - function definition
- `struct` - composite type definition
- `interface` - interface definition
- `as` - type assertion/casting

### Literals (3)
- `true` - boolean true
- `false` - boolean false
- `null` - null/nil value

### Logical Operators (3)
- `and` - logical AND
- `or` - logical OR
- `not` - logical NOT

### Module System (2)
- `import` - import module/package
- `export` - export symbol from module

### Error Handling (3)
- `try` - begin error handling block
- `fail` - throw error/handle errors
- `defer` - guarantee cleanup execution

### Concurrency (6)
- `async` - asynchronous function
- `await` - wait for async operation
- `run` - create concurrent task
- `chan` - channel type keyword
- `lock` - mutual exclusion
- `select` - multiplex channel operations

### Generators (1)
- `yield` - yield value from generator

**Complete List:**
```
if else while for break continue return match
let const func struct interface as
true false null
and or not
import export
try fail defer
async await run chan lock select
yield
```

---

## 2. Syntax Specification

### 2.1 Comments
```
// Single line comment

/* 
   Multi-line comment
   spanning multiple lines
*/

/**
 * Documentation comment for functions/structs
 * @param name - parameter description
 * @return description
 */
```

### 2.2 Variables and Constants

```
// Mutable variable with type inference
let x = 42
let name = "Alice"
let prices = [10.5, 20.0, 15.75]

// Explicit type annotation
let age: int32 = 25
let pi: float64 = 3.14159
let active: bool = true

// Immutable constant
const MAX_SIZE = 1000
const API_KEY = "secret-key"

// Multiple declarations
let a = 1, b = 2, c = 3
let x: int32, y: int32, z: int32
```

### 2.3 Type System

#### Primitive Types
```
// Integer types
int8, int16, int32, int64      // signed integers
uint8, uint16, uint32, uint64  // unsigned integers
int                            // platform-dependent (int64 on 64-bit)
uint                           // platform-dependent (uint64 on 64-bit)

// Floating point types
float32                        // 32-bit float
float64                        // 64-bit float (default)
float                          // alias for float64

// Other primitives
bool                           // boolean (true/false)
char                           // UTF-8 character
string                         // UTF-8 string (immutable)
byte                           // alias for uint8
rune                           // alias for int32 (Unicode code point)
any                            // any type (type erasure)
```

#### Composite Types
```
// Arrays (fixed size, homogeneous)
let nums: [5]int32 = [1, 2, 3, 4, 5]
let matrix: [2][3]int32 = [[1,2,3], [4,5,6]]

// Slices (dynamic size, homogeneous)
let slice: []int32 = [1, 2, 3, 4, 5]
let empty: []string = []

// Maps
let ages: map[string]int32 = {"Alice": 30, "Bob": 25}

// Structs
struct Point {
    x: float64
    y: float64
}

// Interfaces
interface Shape {
    func area(): float64
    func perimeter(): float64
}

// Channels
let ch: chan int32                 // unbuffered channel
let buffered: chan string          // declare type only
let sendOnly: chan<- int32         // send-only
let recvOnly: <-chan int32         // receive-only

// Function types
let fn: func(int32, int32): int32 = add
let callback: func(): void
```

#### Type Inference
```
let x = 42           // inferred as int32
let y = 3.14         // inferred as float64
let s = "hello"      // inferred as string
let arr = [1, 2, 3]  // inferred as []int32
```

#### Type Casting with `as`
```
// Explicit type casting
let x: int32 = 42
let y = x as float64
let z = x as int64

// Safe casting with check
let result = value as int32
if result != null {
    print("Cast successful: " + result)
}

// Interface type assertion
let shape: Shape = circle
let c = shape as Circle
if c != null {
    print("It's a circle with radius: " + c.radius)
}
```

### 2.4 Logical Operators

```
// Using keyword operators
if age > 18 and age < 65 {
    print("working age")
}

if isWeekend or isHoliday {
    print("day off")
}

if not isValid {
    print("invalid input")
}

// Combined
if (x > 0 and x < 10) or (x > 90 and x < 100) {
    print("in range")
}

// Negation
if not (age < 18) {
    print("adult")
}
```

### 2.5 Functions

```
// Basic function
func add(a: int32, b: int32): int32 {
    return a + b
}

// No return type (void)
func greet(name: string) {
    print("Hello, " + name)
}

// Multiple return values
func divmod(a: int32, b: int32): (int32, int32) {
    return a / b, a % b
}

let quotient, remainder = divmod(10, 3)

// Anonymous function
let square = func(x: int32): int32 {
    return x * x
}

// Arrow function (single expression)
let double = (x: int32) => x * 2

// Arrow function extended
let max = (a:int23, b:int23): int32 => {
    if a >= b {
        return a
    }
    else {
        return b
    }
}

// Higher-order function
func map(arr: []int32, fn: func(int32): int32): []int32 {
    let result = []
    for x in arr {
        result.append(fn(x))
    }
    return result
}

// Variadic function
func sum(nums: ...int32): int32 {
    let total = 0
    for n in nums {
        total = total + n
    }
    return total
}

let s = sum(1, 2, 3, 4, 5)  // 15

// Default parameters
func power(base: float64, exp: int32 = 2): float64 {
    let result = 1.0
    for i in 0..<exp {
        result = result * base
    }
    return result
}

print(power(3.0))      // 9.0 (3^2)
print(power(3.0, 3))   // 27.0 (3^3)
```

### 2.6 Control Flow

#### If-Else
```
if x > 0 {
    print("positive")
} else if x < 0 {
    print("negative")
} else {
    print("zero")
}

// Single line
if condition { doSomething() }

// If as expression
let sign = if x > 0 { "positive" } else { "negative" }
```

#### While Loop
```
let i = 0
while i < 10 {
    print(i)
    i = i + 1
}

// Infinite loop
while true {
    if shouldBreak() {
        break
    }
}
```

#### For Loop
```
// Range iteration (exclusive)
for i in 0..<10 {
    print(i)  // 0 to 9
}

// Range iteration (inclusive)
for i in 0...10 {
    print(i)  // 0 to 10
}

// Array iteration
for item in items {
    print(item)
}

// With index
for i, item in items {
    print(i, item)
}

// Map iteration
for key, value in ages {
    print(key + ": " + value)
}

// Break and continue
for i in 0..<100 {
    if i == 50 { break }
    if i % 2 == 0 { continue }
    print(i)
}

// Step range
for i in 0..<100 step 5 {
    print(i)  // 0, 5, 10, 15, ...
}
```

#### Match (Pattern Matching)
```
// Basic match
match value {
    0 -> print("zero")
    1 -> print("one")
    2 | 3 -> print("two or three")
    _ -> print("other")
}

// Match with blocks
match status {
    "success" -> {
        log("Operation succeeded")
        return true
    }
    "error" -> {
        log("Operation failed")
        return false
    }
    _ -> {
        log("Unknown status")
        return null
    }
}

// Range matching
match age {
    0...12 -> print("child")
    13...19 -> print("teen")
    20...64 -> print("adult")
    _ -> print("senior")
}

// Destructuring match
match point {
    Point{x: 0, y: 0} -> print("origin")
    Point{x: 0, y: _} -> print("on y-axis")
    Point{x: _, y: 0} -> print("on x-axis")
    Point{x: x, y: y} -> print("at (" + x + ", " + y + ")")
}

// Type matching
match shape {
    Circle as c -> print("Circle with radius: " + c.radius)
    Rectangle as r -> print("Rectangle: " + r.width + "x" + r.height)
    _ -> print("Unknown shape")
}

// Match as expression
let category = match age {
    0...12 -> "child"
    13...19 -> "teen"
    20...64 -> "adult"
    _ -> "senior"
}
```

### 2.7 Structs

```
// Define struct
struct Point {
    x: float64
    y: float64
}

// Create instance
let p = Point{x: 10.0, y: 20.0}

// Access and modify fields
print(p.x)
p.y = 25.0

// Struct with methods
struct Rectangle {
    width: float64
    height: float64
    
    func area(): float64 {
        return this.width * this.height
    }
    
    func scale(factor: float64) {
        this.width = this.width * factor
        this.height = this.height * factor
    }
}

let rect = Rectangle{width: 5.0, height: 10.0}
print(rect.area())
rect.scale(2.0)

// Struct embedding (composition)
struct ColoredPoint {
    point: Point
    color: string
}

let cp = ColoredPoint{
    point: Point{x: 10.0, y: 20.0},
    color: "red"
}
print(cp.point.x)
print(cp.color)

// Constructor pattern
func Point.new(x: float64, y: float64): Point {
    return Point{x: x, y: y}
}

let p2 = Point.new(5.0, 10.0)
```

### 2.8 Interfaces

```
// Define interface
interface Shape {
    func area(): float64
    func perimeter(): float64
}

// Implement interface (implicit)
struct Circle {
    radius: float64
    
    func area(): float64 {
        return 3.14159 * this.radius * this.radius
    }
    
    func perimeter(): float64 {
        return 2.0 * 3.14159 * this.radius
    }
}

struct Rectangle {
    width: float64
    height: float64
    
    func area(): float64 {
        return this.width * this.height
    }
    
    func perimeter(): float64 {
        return 2.0 * (this.width + this.height)
    }
}

// Use interface
func printInfo(s: Shape) {
    print("Area: " + s.area())
    print("Perimeter: " + s.perimeter())
}

let circle = Circle{radius: 5.0}
let rect = Rectangle{width: 3.0, height: 4.0}

printInfo(circle)
printInfo(rect)

// Interface composition
interface Drawable {
    func draw()
}

interface Movable {
    func move(dx: float64, dy: float64)
}

interface GameObject {
    Drawable
    Movable
    func update()
}
```

### 2.9 Error Handling

```
// Try-fail block
func divide(a: float64, b: float64): float64 {
    try {
        if b == 0.0 {
            fail "Division by zero"
        }
        return a / b
    } fail on err {
        print("Error: " + err)
        return 0.0
    }
}

// Defer for cleanup
func readFile(path: string): string {
    let file = open(path)
    defer file.close()  // Always executes before return
    
    try {
        let content = file.read()
        return content
    } fail on err {
        print("Failed to read file: " + err)
        return ""
    }
}

// Multiple defers (execute in reverse order)
func transaction() {
    defer print("3: commit")
    defer print("2: validate")
    defer print("1: prepare")
    
    print("Executing transaction...")
}
// Output:
// Executing transaction...
// 1: prepare
// 2: validate
// 3: commit

// Fail without catch (propagates to caller)
func riskyOperation(): int32 {
    if someCondition {
        fail "Something went wrong"
    }
    return 42
}

func caller() {
    try {
        let result = riskyOperation()
        print(result)
    } fail on err {
        print("Caught error: " + err)
    }
}
```

### 2.10 Async/Await

```
// Async function declaration
async func fetchData(url: string): string {
    let response = await http.get(url)
    return response.text()
}

// Using async functions
async func main() {
    let data = await fetchData("https://api.example.com/data")
    print(data)
}

// Parallel async operations
async func fetchAll(urls: []string): []string {
    let tasks = []
    
    for url in urls {
        tasks.append(fetchData(url))
    }
    
    // Wait for all to complete
    let results = await Promise.all(tasks)
    return results
}

// Error handling with async
async func safeFetch(url: string): string {
    try {
        let data = await fetchData(url)
        return data
    } fail on err {
        print("Fetch failed: " + err)
        return ""
    }
}
```

### 2.11 Concurrency with Run

```
// Run concurrent task
func worker(id: int32, work: []string) {
    print("Worker " + id + " started")
    for item in work {
        process(item)
    }
    print("Worker " + id + " finished")
}

func main() {
    for i in 0..<5 {
        run worker(i, tasks[i])
    }
    
    print("All workers spawned")
}

// Run with anonymous function
run func() {
    while true {
        processQueue()
        sleep(1000)
    }
}()

// Run with return value (future/promise)
let future = run compute(1000000)
// ... do other work ...
let result = await future
```

### 2.12 Channels and Make Built-in

#### 2.12.1 Channel Creation with make()

The `make()` built-in function creates channels exactly like in Go:

```
// Unbuffered channels (synchronous)
let ch = make(chan int32)         // Blocks until receiver ready
let strCh = make(chan string)     // Blocks until receiver ready

// Buffered channels (asynchronous up to capacity)
let buffered = make(chan int32, 10)    // Can hold 10 values before blocking
let smallBuf = make(chan string, 1)    // Can hold 1 value before blocking

// Channel type annotations
let ch: chan int32                // Bidirectional channel
let sendOnly: chan<- int32        // Send-only channel
let recvOnly: <-chan int32        // Receive-only channel
```

#### 2.12.2 Channel Operations

```
// Send operation (blocks if channel full or no receiver for unbuffered)
ch <- 42
strCh <- "hello"

// Receive operation (blocks if channel empty)
let value = <-ch           // Blocks until value available
let msg = <-strCh          // Blocks until value available

// Receive with ok flag (like Go)
let value, ok = <-ch       // ok is false if channel closed and empty
if ok {
    print("Received: " + value)
} else {
    print("Channel closed")
}

// Close channel
close(ch)                  // No more values can be sent
close(strCh)              // Receivers will get zero value + false
```

#### 2.12.3 Channel Behavior (Go Semantics)

**Unbuffered Channels:**
- Send blocks until receiver ready
- Receive blocks until sender ready
- Synchronous communication (rendezvous)

**Buffered Channels:**
- Send blocks only when buffer full
- Receive blocks only when buffer empty
- Asynchronous up to buffer capacity

**Closed Channels:**
- Sending to closed channel panics
- Receiving from closed channel returns zero value + false
- Closing already closed channel panics

```
// Producer-Consumer pattern
func producer(ch: chan<- int32) {
    for i in 0..<10 {
        ch <- i                    // Send value
        print("Sent: " + i)
    }
    close(ch)                      // Signal completion
}

func consumer(ch: <-chan int32) {
    for {
        let value, ok = <-ch       // Receive with closed check
        if not ok {
            break                  // Channel closed
        }
        print("Received: " + value)
    }
}

// Alternative: range over channel (stops when closed)
func consumer2(ch: <-chan int32) {
    for value in ch {              // Automatically stops when closed
        print("Received: " + value)
    }
}

func main() {
    let ch = make(chan int32, 5)   // Buffered channel
    run producer(ch)               // Start producer goroutine
    consumer(ch)                   // Consume in main goroutine
}
```

#### 2.12.4 Make Built-in for All Types (Go Semantics)

The `make()` built-in works with channels, slices, and maps exactly like Go:

```
// Channels
let ch1 = make(chan int32)         // Unbuffered channel
let ch2 = make(chan string, 10)    // Buffered channel with capacity 10

// Slices
let slice1 = make([]int32, 5)      // Slice with length 5, capacity 5
let slice2 = make([]int32, 5, 10)  // Slice with length 5, capacity 10
let slice3 = make([]string, 0, 100) // Empty slice with capacity 100

// Maps
let map1 = make(map[string]int32)  // Empty map
let map2 = make(map[int32]string)  // Empty map with different types

// Zero values for make() with primitive types (Go compatibility)
let i = make(int32)                // 0
let f = make(float64)              // 0.0
let b = make(bool)                 // false
let s = make(string)               // ""

// All integer types
let i8 = make(int8)                // 0
let i16 = make(int16)              // 0
let i64 = make(int64)              // 0
let u8 = make(uint8)               // 0
let u16 = make(uint16)             // 0
let u32 = make(uint32)             // 0
let u64 = make(uint64)             // 0

// Float types
let f32 = make(float32)            // 0.0

// Character and byte types
let c = make(char)                 // '\0'
let by = make(byte)                // 0
let r = make(rune)                 // 0

// Any type
let a = make(any)                  // null
```

#### 2.12.5 Channel Directional Types

```
// Function parameters with directional channels
func sender(ch: chan<- int32) {    // Can only send
    ch <- 42
    // let x = <-ch                // Compile error: cannot receive
}

func receiver(ch: <-chan int32) {  // Can only receive
    let value = <-ch
    // ch <- 42                    // Compile error: cannot send
}

func processor(in: <-chan int32, out: chan<- int32) {
    for value in in {
        out <- value * 2           // Transform and forward
    }
    close(out)
}

// Bidirectional channels can be passed to directional parameters
func main() {
    let ch = make(chan int32)      // Bidirectional
    run sender(ch)                 // Implicitly converts to chan<-
    run receiver(ch)               // Implicitly converts to <-chan
}
```

#### 2.12.6 Channel Integration with Select

```
func multiplexer(ch1: <-chan string, ch2: <-chan int32, quit: <-chan bool) {
    for {
        select {
            msg := <-ch1 -> {
                print("String: " + msg)
            }
            num := <-ch2 -> {
                print("Number: " + num)
            }
            <-quit -> {
                print("Quitting")
                return
            }
            _ -> {
                // Default case (non-blocking)
                print("No messages")
                sleep(100)
            }
        }
    }
}

// Timeout pattern
func fetchWithTimeout(ch: <-chan string): string {
    let timeout = timer(5000)      // 5 second timeout
    
    select {
        data := <-ch -> {
            return data
        }
        <-timeout -> {
            return ""              // Timeout occurred
        }
    }
}
```

#### 2.12.7 Channel Implementation Details

**Internal Structure:**
- Unbuffered: Direct goroutine-to-goroutine handoff
- Buffered: Circular buffer with head/tail pointers
- Closed flag and waiting goroutine queues
- Type-safe element storage

**Memory Management:**
- Channels are reference types (heap allocated)
- Garbage collected when no references remain
- Buffer memory managed automatically

**Concurrency Safety:**
- All channel operations are thread-safe
- Uses internal mutexes and condition variables
- Lock-free fast paths for common cases

#### 2.12.8 Technical Implementation Specification

**Channel Runtime Structure:**
```rust
struct Channel<T> {
    buffer: Option<VecDeque<T>>,     // None for unbuffered, Some for buffered
    capacity: usize,                  // 0 for unbuffered, N for buffered
    closed: bool,                     // Channel closed flag
    send_queue: VecDeque<Sender<T>>,  // Waiting senders (unbuffered)
    recv_queue: VecDeque<Receiver<T>>, // Waiting receivers
    mutex: Mutex<()>,                 // Protects channel state
    send_cond: Condvar,               // Notifies waiting senders
    recv_cond: Condvar,               // Notifies waiting receivers
}
```

**Operation Semantics:**

*Send Operation (`ch <- value`):*
1. Acquire channel mutex
2. If channel closed: panic
3. If unbuffered and receiver waiting: direct handoff
4. If buffered and space available: add to buffer
5. Otherwise: block sender until space/receiver available

*Receive Operation (`value = <-ch`):*
1. Acquire channel mutex
2. If buffered and data available: return from buffer
3. If unbuffered and sender waiting: direct handoff
4. If channel closed and empty: return zero value + false
5. Otherwise: block receiver until data/close available

*Close Operation (`close(ch)`):*
1. Acquire channel mutex
2. If already closed: panic
3. Set closed flag
4. Wake all waiting receivers (they get zero value + false)
5. Panic any waiting senders

**Integration with Goroutine Scheduler:**
- Blocked goroutines are parked and removed from scheduler
- Channel operations wake parked goroutines
- Select statement uses channel readiness polling

#### 2.12.9 Make Built-in Implementation Specification

**Parser Integration:**
The parser must recognize type expressions in `make()` calls:

```
make(chan T)           // Channel type
make(chan T, N)        // Buffered channel
make([]T, len)         // Slice with length
make([]T, len, cap)    // Slice with length and capacity
make(map[K]V)          // Map type
make(PrimitiveType)    // Zero value of primitive type
```

**Runtime Implementation:**
```rust
fn execute_make_call(args: &[Expression]) -> Result<RuntimeValue> {
    match args.len() {
        1 => {
            // make(Type) - create zero value or empty collection
            match parse_type(&args[0]) {
                Type::Channel(elem_type) => create_unbuffered_channel(elem_type),
                Type::Slice(elem_type) => create_empty_slice(elem_type),
                Type::Map(key_type, val_type) => create_empty_map(key_type, val_type),
                Type::Primitive(prim_type) => create_zero_value(prim_type),
                _ => error("Invalid type for make()"),
            }
        },
        2 => {
            // make(Type, size_or_capacity)
            let size = evaluate_expression(&args[1])?;
            match parse_type(&args[0]) {
                Type::Channel(elem_type) => create_buffered_channel(elem_type, size),
                Type::Slice(elem_type) => create_slice_with_length(elem_type, size),
                _ => error("Invalid 2-argument make() call"),
            }
        },
        3 => {
            // make([]T, length, capacity)
            let length = evaluate_expression(&args[1])?;
            let capacity = evaluate_expression(&args[2])?;
            match parse_type(&args[0]) {
                Type::Slice(elem_type) => create_slice_with_length_capacity(elem_type, length, capacity),
                _ => error("Invalid 3-argument make() call"),
            }
        },
        _ => error("make() requires 1-3 arguments"),
    }
}
```

**Zero Value Semantics (Go Compatible):**
- Numeric types: 0
- Boolean: false
- String: ""
- Pointers/References: null
- Slices: empty slice (length 0, capacity 0)
- Maps: empty map
- Channels: nil (must use make to create)
- Structs: all fields set to their zero values
- Arrays: all elements set to zero value of element type

### 2.13 Select (Channel Multiplexing)

```
func multiplexer(ch1: <-chan string, ch2: <-chan string, done: <-chan bool) {
    while true {
        select {
            msg := <-ch1 -> {
                print("Channel 1: " + msg)
            }
            msg := <-ch2 -> {
                print("Channel 2: " + msg)
            }
            <-done -> {
                print("Done")
                return
            }
            _ -> {
                // Default case (non-blocking)
                print("No message")
                sleep(100)
            }
        }
    }
}

// Timeout pattern
func fetchWithTimeout(ch: <-chan string): string {
    let timeout = timer(5000)
    
    select {
        data := <-ch -> {
            return data
        }
        <-timeout -> {
            return ""  // Timeout
        }
    }
}

// Select without default (blocking)
select {
    data := <-ch1 -> handleData1(data)
    data := <-ch2 -> handleData2(data)
    <-quit -> return
}
```

### 2.14 Locks (Mutual Exclusion)

```
// Create lock
let mu = lock()

// Manual lock/unlock
mu.acquire()
counter = counter + 1
mu.release()

// Lock with block (auto-unlock)
lock mu {
    counter = counter + 1
}

// Lock with defer
func increment() {
    mu.acquire()
    defer mu.release()
    
    counter = counter + 1
}

// Thread-safe struct
struct SafeCounter {
    value: int32
    mu: lock
    
    func increment() {
        lock this.mu {
            this.value = this.value + 1
        }
    }
    
    func get(): int32 {
        lock this.mu {
            return this.value
        }
    }
    
    func reset() {
        lock this.mu {
            this.value = 0
        }
    }
}

let counter = SafeCounter{value: 0, mu: lock()}

for i in 0..<100 {
    run func() {
        counter.increment()
    }()
}
```

### 2.15 Generators (Yield)

```
// Generator function
func fibonacci() {
    let a = 0
    let b = 1
    
    while true {
        yield a
        let temp = a
        a = b
        b = temp + b
    }
}

// Using generator
let fib = fibonacci()
for i in 0..<10 {
    print(fib.next())
}

// Generator with parameters
func range(start: int32, end: int32) {
    let i = start
    while i < end {
        yield i
        i = i + 1
    }
}

for num in range(0, 5) {
    print(num)
}

// Generator pipeline
func map<T, U>(gen: Generator<T>, fn: func(T): U) {
    for value in gen {
        yield fn(value)
    }
}

let numbers = range(1, 6)
let squares = map(numbers, (x: int32) => x * x)

for sq in squares {
    print(sq)
}
```

### 2.16 Modules

```
// file: math.lang
export func add(a: int32, b: int32): int32 {
    return a + b
}

export func multiply(a: int32, b: int32): int32 {
    return a * b
}

export const PI: float64 = 3.14159

// Private function (not exported)
func helper(): int32 {
    return 42
}

// file: main.lang
import math

func main() {
    let sum = math.add(5, 3)
    let product = math.multiply(4, 7)
    print(math.PI)
}

// Import specific symbols
import { add, PI } from math

func calculate(): float64 {
    return add(10, 20) as float64 * PI
}

// Import with alias
import math as m

func compute(): int32 {
    return m.add(1, 2)
}

// Re-export
export { add, multiply } from math
```

---

## 3. Type System

### 3.1 Value vs Reference Semantics

**Value Types (copied on assignment):**
- All primitive types: `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `float32`, `float64`, `bool`, `char`
- Small structs (< 128 bytes by default)

**Reference Types (shared on assignment):**
- Strings (`string`)
- Arrays and slices (`[]T`)
- Maps (`map[K]V`)
- Large structs (≥ 128 bytes)
- Channels (`chan T`)
- Functions
- Interfaces

```
// Value types
let x: int32 = 42
let y = x
y = 100
print(x)  // 42 (unchanged)

// Reference types
let arr1 = [1, 2, 3]
let arr2 = arr1
arr2[0] = 999
print(arr1[0])  // 999 (changed!)

// Explicit copy
let arr3 = clone(arr1)
arr3[0] = 42
print(arr1[0])  // 999 (unchanged)
```

### 3.2 The `any` Type

The `any` type represents a value that can be of any type at runtime. It provides type erasure and dynamic typing capabilities within the statically typed system.

```
// any type declaration
let value: any = 42
value = "hello"
value = [1, 2, 3]
value = Point{x: 10.0, y: 20.0}

// Type checking with any
if value instanceof string {
    let s = value as string
    print("String: " + s)
}

// any in function parameters
func process(data: any) {
    match data {
        int32 as i -> print("Integer: " + i)
        string as s -> print("String: " + s)
        []any as arr -> print("Array with " + len(arr) + " items")
        _ -> print("Unknown type: " + typeof(data))
    }
}

// any in collections
let mixed: []any = [42, "hello", true, 3.14]
let anyMap: map[string]any = {
    "name": "Alice",
    "age": 30,
    "active": true
}

// any with null safety
let nullable: any = null
if nullable != null {
    process(nullable)
}
```

### 3.3 Generics System

#### 3.3.1 Generic Functions

```
// Basic generic function
func max<T>(a: T, b: T): T {
    return if a > b { a } else { b }
}

// Usage with explicit type parameters
let maxInt = max<int32>(5, 10)
let maxFloat = max<float64>(3.14, 2.71)

// Usage with type inference
let maxInferred = max(5, 10)  // T inferred as int32

// Multiple type parameters
func convert<T, U>(value: T, converter: func(T): U): U {
    return converter(value)
}

let result = convert<int32, string>(42, (x: int32) => string(x))

// Generic function with constraints (using interfaces)
interface Comparable<T> {
    func compare(other: T): int32
}

func sort<T>(arr: []T) where T: Comparable<T> {
    // Sort implementation using T.compare()
}

// Generic function with multiple constraints
interface Addable<T> {
    func add(other: T): T
}

interface Printable {
    func toString(): string
}

func sumAndPrint<T>(items: []T): T where T: Addable<T> and Printable {
    let sum = items[0]
    for i in 1..<len(items) {
        sum = sum.add(items[i])
    }
    print("Sum: " + sum.toString())
    return sum
}
```

#### 3.3.2 Generic Structs

```
// Basic generic struct
struct Box<T> {
    value: T
    
    func get(): T {
        return this.value
    }
    
    func set(v: T) {
        this.value = v
    }
    
    func map<U>(fn: func(T): U): Box<U> {
        return Box<U>{value: fn(this.value)}
    }
}

// Usage
let intBox = Box<int32>{value: 42}
let strBox = Box<string>{value: "hello"}
let doubledBox = intBox.map<int32>((x: int32) => x * 2)

// Generic struct with multiple type parameters
struct Pair<T, U> {
    first: T
    second: U
    
    func swap(): Pair<U, T> {
        return Pair<U, T>{first: this.second, second: this.first}
    }
}

let pair = Pair<string, int32>{first: "answer", second: 42}
let swapped = pair.swap()  // Pair<int32, string>

// Generic struct with constraints
struct SortedList<T> where T: Comparable<T> {
    items: []T
    
    func add(item: T) {
        // Insert in sorted order using T.compare()
    }
    
    func find(item: T): int32 {
        // Binary search using T.compare()
    }
}

// Generic struct with default type parameters
struct Result<T, E = string> {
    value: T
    error: E
    isSuccess: bool
    
    func unwrap(): T {
        if not this.isSuccess {
            panic("Attempted to unwrap error result")
        }
        return this.value
    }
    
    func unwrapOr(defaultValue: T): T {
        return if this.isSuccess { this.value } else { defaultValue }
    }
}

// Usage with default error type
let result1 = Result<int32>{value: 42, error: "", isSuccess: true}
// Usage with custom error type
let result2 = Result<int32, CustomError>{value: 0, error: CustomError{}, isSuccess: false}
```

#### 3.3.3 Generic Interfaces

```
// Basic generic interface
interface Container<T> {
    func add(item: T)
    func get(index: int32): T
    func size(): int32
    func isEmpty(): bool
}

// Generic interface with multiple type parameters
interface Map<K, V> {
    func put(key: K, value: V)
    func get(key: K): V
    func containsKey(key: K): bool
    func keys(): []K
    func values(): []V
}

// Generic interface with constraints
interface Serializable<T> where T: Printable {
    func serialize(): string
    func deserialize(data: string): T
}

// Interface inheritance with generics
interface ReadOnlyContainer<T> {
    func get(index: int32): T
    func size(): int32
}

interface MutableContainer<T> {
    ReadOnlyContainer<T>
    func set(index: int32, value: T)
    func add(item: T)
    func remove(index: int32): T
}

// Implementation
struct ArrayList<T> {
    items: []T
    
    func get(index: int32): T {
        return this.items[index]
    }
    
    func size(): int32 {
        return len(this.items)
    }
    
    func set(index: int32, value: T) {
        this.items[index] = value
    }
    
    func add(item: T) {
        this.items = append(this.items, item)
    }
    
    func remove(index: int32): T {
        let item = this.items[index]
        // Remove logic here
        return item
    }
}

// ArrayList<T> automatically implements MutableContainer<T>
```

#### 3.3.4 Generic Type Aliases

```
// Generic type aliases
type Optional<T> = T | null
type Result<T> = T | string  // T or error message
type Callback<T, U> = func(T): U
type Predicate<T> = func(T): bool
type Supplier<T> = func(): T

// Usage
let maybeValue: Optional<int32> = 42
let computation: Result<float64> = calculateSomething()
let transformer: Callback<string, int32> = (s: string) => len(s)
let filter: Predicate<int32> = (x: int32) => x > 0
let factory: Supplier<Point> = () => Point{x: 0.0, y: 0.0}

// Complex generic type aliases
type EventHandler<T> = func(event: T): bool
type AsyncResult<T> = Promise<Result<T>>
type Repository<T, ID> = interface {
    func findById(id: ID): Optional<T>
    func save(entity: T): T
    func delete(id: ID): bool
}
```

#### 3.3.5 Generic Constraints and Bounds

```
// Interface constraints
interface Numeric<T> {
    func add(other: T): T
    func subtract(other: T): T
    func multiply(other: T): T
    func divide(other: T): T
}

func calculate<T>(a: T, b: T): T where T: Numeric<T> {
    return a.add(b).multiply(a.subtract(b))
}

// Multiple constraints
interface Comparable<T> {
    func compare(other: T): int32
}

interface Hashable {
    func hash(): int32
}

func createSortedMap<K, V>(): SortedMap<K, V> 
    where K: Comparable<K> and Hashable {
    return SortedMap<K, V>{}
}

// Type constraints with specific types
func processNumbers<T>(values: []T): T 
    where T: int32 or int64 or float32 or float64 {
    let sum = values[0]
    for i in 1..<len(values) {
        sum = sum + values[i]
    }
    return sum
}

// Constraint inheritance
interface OrderedCollection<T> where T: Comparable<T> {
    func sort()
    func binarySearch(item: T): int32
    func min(): T
    func max(): T
}
```

#### 3.3.6 Generic Methods and Associated Types

```
// Generic methods in non-generic structs
struct Utils {
    func swap<T>(a: T, b: T): (T, T) {
        return b, a
    }
    
    func identity<T>(value: T): T {
        return value
    }
    
    func compose<A, B, C>(f: func(A): B, g: func(B): C): func(A): C {
        return (x: A) => g(f(x))
    }
}

// Associated types in interfaces
interface Iterator<T> {
    type Item = T
    
    func next(): Optional<Item>
    func hasNext(): bool
}

interface Collection<T> {
    type Element = T
    type Iter = Iterator<Element>
    
    func iterator(): Iter
    func size(): int32
    func contains(item: Element): bool
}

// Implementation with associated types
struct Vector<T> {
    items: []T
    
    func iterator(): VectorIterator<T> {
        return VectorIterator<T>{items: this.items, index: 0}
    }
    
    func size(): int32 {
        return len(this.items)
    }
    
    func contains(item: T): bool {
        for x in this.items {
            if x == item {
                return true
            }
        }
        return false
    }
}

struct VectorIterator<T> {
    items: []T
    index: int32
    
    func next(): Optional<T> {
        if this.index >= len(this.items) {
            return null
        }
        let item = this.items[this.index]
        this.index = this.index + 1
        return item
    }
    
    func hasNext(): bool {
        return this.index < len(this.items)
    }
}
```

#### 3.3.7 Generic Channels and Concurrency

```
// Generic channels
let intChan = make(chan int32)
let stringChan = make(chan string, 10)

// Generic channel functions
func fanOut<T>(input: <-chan T, outputs: []chan<- T) {
    for value in input {
        for output in outputs {
            output <- value
        }
    }
}

func fanIn<T>(inputs: []<-chan T, output: chan<- T) {
    for input in inputs {
        run func() {
            for value in input {
                output <- value
            }
        }()
    }
}

// Generic worker pattern
func worker<T, U>(id: int32, jobs: <-chan T, results: chan<- U, processor: func(T): U) {
    for job in jobs {
        result := processor(job)
        results <- result
    }
}

func processInParallel<T, U>(items: []T, processor: func(T): U, numWorkers: int32): []U {
    let jobs = make(chan T, len(items))
    let results = make(chan U, len(items))
    
    // Start workers
    for i in 0..<numWorkers {
        run worker<T, U>(i, jobs, results, processor)
    }
    
    // Send jobs
    for item in items {
        jobs <- item
    }
    close(jobs)
    
    // Collect results
    let output = []U{}
    for i in 0..<len(items) {
        output = append(output, <-results)
    }
    
    return output
}
```

#### 3.3.8 Generic Type Inference

```
// Type inference in function calls
func identity<T>(x: T): T { return x }

let a = identity(42)        // T inferred as int32
let b = identity("hello")   // T inferred as string
let c = identity([1, 2, 3]) // T inferred as []int32

// Type inference with constraints
func min<T>(a: T, b: T): T where T: Comparable<T> {
    return if a.compare(b) <= 0 { a } else { b }
}

let smaller = min(10, 20)   // T inferred as int32

// Partial type inference
func convert<T, U>(value: T): U {
    // Implementation depends on specific types
}

let result = convert<int32, string>(42)  // T=int32, U=string
// let result = convert<_, string>(42)   // T inferred, U explicit (future feature)

// Type inference in generic structs
let box1 = Box{value: 42}           // Box<int32>
let box2 = Box{value: "hello"}      // Box<string>

// Type inference with collections
let numbers = [1, 2, 3]             // []int32
let mixed: []any = [1, "hello", true] // []any (explicit annotation needed)

// Type inference in generic method calls
let doubled = box1.map((x: int32) => x * 2)  // Box<int32>
let stringified = box1.map((x: int32) => string(x))  // Box<string>
```

---

## 4. Memory Model

### 4.1 Memory Layout

```
Stack:
- Local variables (value types)
- Function call frames
- Small structs (< 128 bytes)

Heap:
- Large structs (≥ 128 bytes)
- Arrays/slices
- Maps
- Strings
- Channel buffers
- Interface values

Static/Data:
- Constants
- Global variables
- String literals
- Function code
```

### 4.2 Garbage Collection

**Algorithm:** Concurrent tri-color mark-and-sweep with generational collection

**Phases:**
1. **Mark:** Trace reachable objects from roots (stack, globals)
2. **Sweep:** Reclaim unreachable objects
3. **Compact:** Optional defragmentation for large objects

**GC Tuning:**
```bash
# Environment variables
LANG_GC_HEAP_SIZE=1024M     # Max heap size
LANG_GC_TARGET=80           # Target heap usage %
LANG_GC_THREADS=4           # Parallel GC threads
LANG_GC_DEBUG=false         # Enable GC logging
```

### 4.3 Memory Safety Guarantees

- ✅ **No null pointer dereferences** - runtime checks
- ✅ **No use-after-free** - guaranteed by GC
- ✅ **No buffer overflows** - bounds checking
- ✅ **No data races** - channel/lock synchronization
- ✅ **Type safety** - compile-time type checking

---

## 5. Built-in Functions

### 5.1 Core Built-ins

```
// Type conversion
int8(x), int16(x), int32(x), int64(x)
uint8(x), uint16(x), uint32(x), uint64(x)
float32(x), float64(x)
string(x), bool(x)

// Memory
len(x)              // Length of array/slice/string/map
cap(x)              // Capacity of slice/channel
clone(x)            // Deep copy
sizeof(T)           // Size of type in bytes

// Array/Slice operations
append(slice, item)                 // Append item to slice
append(slice, ...items)             // Append multiple items
make([]T, length)                   // Create slice with length
make([]T, length, capacity)         // Create slice with length and capacity
copy(dst, src)                      // Copy slice elements

// Map operations
make(map[K]V)                       // Create map
delete(m, key)                      // Delete key from map

// Channel operations
make(chan T)                        // Create unbuffered channel
make(chan T, capacity)              // Create buffered channel
close(ch)                           // Close channel

// Concurrency
run func()                          // Run function concurrently
lock()                              // Create new lock

// Panic/Recovery
panic(message)                      // Panic with message
recover()                           // Recover from panic

// Assertions
assert(condition, message)          // Assert condition

// Type checking
typeof(x)                           // Get type as string
instanceof(x, T)                    // Check if x is instance of T

// I/O
print(args...)                      // Print to stdout
println(args...)                    // Print with newline
printf(format, args...)             // Formatted print
input(prompt)                       // Read line from stdin

// Timing
sleep(milliseconds)                 // Sleep for duration
timer(milliseconds)                 // Create timer channel
```

### 5.2 String Built-ins

```
// String operations
string.len(s)                       // Length
string.charAt(s, index)             // Character at index
string.substr(s, start, end)        // Substring
string.contains(s, substr)          // Contains substring
string.startsWith(s, prefix)        // Starts with prefix
string.endsWith(s, suffix)          // Ends with suffix
string.indexOf(s, substr)           // Index of substring
string.split(s, sep)                // Split by separator
string.join(arr, sep)               // Join array with separator
string.replace(s, old, new)         // Replace substring
string.replaceAll(s, old, new)      // Replace all occurrences
string.trim(s)                      // Trim whitespace
string.toUpper(s)                   // To uppercase
string.toLower(s)                   // To lowercase
string.repeat(s, count)             // Repeat string
```

### 5.3 Array Built-ins

```
// Array operations
array.len(arr)                      // Length
array.push(arr, item)               // Push to end
array.pop(arr)                      // Pop from end
array.shift(arr)                    // Remove from start
array.unshift(arr, item)            // Add to start
array.slice(arr, start, end)        // Slice array
array.reverse(arr)                  // Reverse array
array.sort(arr)                     // Sort array
array.sort(arr, compareFn)          // Sort with comparator
array.map(arr, fn)                  // Map function
array.filter(arr, fn)               // Filter array
array.reduce(arr, fn, initial)      // Reduce array
array.forEach(arr, fn)              // For each element
array.find(arr, fn)                 // Find element
array.findIndex(arr, fn)            // Find index
array.contains(arr, item)           // Contains item
array.indexOf(arr, item)            // Index of item
array.concat(arr1, arr2)            // Concatenate arrays
```

### 5.4 Map Built-ins

```
// Map operations
map.len(m)                          // Size
map.get(m, key)                     // Get value (returns null if not found)
map.set(m, key, value)              // Set value
map.has(m, key)                     // Has key
map.delete(m, key)                  // Delete key
map.keys(m)                         // Get all keys
map.values(m)                       // Get all values
map.entries(m)                      // Get key-value pairs
map.clear(m)                        // Clear all entries
```

### 5.5 Math Built-ins

```
// Math operations
math.abs(x)                         // Absolute value
math.min(a, b)                      // Minimum
math.max(a, b)                      // Maximum
math.pow(base, exp)                 // Power
math.sqrt(x)                        // Square root
math.cbrt(x)                        // Cube root
math.ceil(x)                        // Ceiling
math.floor(x)                       // Floor
math.round(x)                       // Round
math.trunc(x)                       // Truncate
math.sin(x), math.cos(x), math.tan(x)
math.asin(x), math.acos(x), math.atan(x)
math.atan2(y, x)
math.sinh(x), math.cosh(x), math.tanh(x)
math.log(x)                         // Natural log
math.log10(x)                       // Base-10 log
math.log2(x)                        // Base-2 log
math.exp(x)                         // e^x
math.random()                       // Random [0, 1)
math.randomInt(min, max)            // Random integer

// Constants
math.PI, math.E, math.SQRT2, math.LN2, math.LN10
```

### 5.6 File I/O Built-ins

```
// File operations
file.open(path, mode)               // Open file ("r", "w", "a", "r+", "w+")
file.close(f)                       // Close file
file.read(f)                        // Read entire file
file.readLine(f)                    // Read one line
file.readBytes(f, n)                // Read n bytes
file.write(f, data)                 // Write string/bytes
file.writeLine(f, line)             // Write line with newline
file.seek(f, offset, whence)        // Seek position (0=start, 1=current, 2=end)
file.tell(f)                        // Current position
file.flush(f)                       // Flush buffer
file.exists(path)                   // Check if file exists
file.remove(path)                   // Delete file
file.rename(old, new)               // Rename file
file.copy(src, dst)                 // Copy file
file.stat(path)                     // Get file info (size, mtime, etc.)
```

### 5.7 Directory Built-ins

```
// Directory operations
dir.create(path)                    // Create directory
dir.createAll(path)                 // Create directory with parents
dir.remove(path)                    // Remove directory
dir.removeAll(path)                 // Remove directory recursively
dir.list(path)                      // List directory contents
dir.walk(path, fn)                  // Walk directory tree
dir.exists(path)                    // Check if directory exists
dir.current()                       // Get current working directory
dir.change(path)                    // Change working directory
dir.home()                          // Get home directory
dir.temp()                          // Get temp directory
```

### 5.8 JSON Built-ins

```
// JSON operations
json.encode(obj)                    // Object to JSON string
json.decode(str)                    // JSON string to object
json.pretty(obj)                    // Pretty-printed JSON
json.validate(str)                  // Validate JSON string
```

### 5.9 Regular Expression Built-ins

```
// Regex operations
regex.compile(pattern)              // Compile regex
regex.match(pattern, str)           // Test if matches
regex.find(pattern, str)            // Find first match
regex.findAll(pattern, str)         // Find all matches
regex.replace(pattern, str, repl)   // Replace first match
regex.replaceAll(pattern, str, repl) // Replace all matches
regex.split(pattern, str)           // Split by pattern
```

### 5.10 Date/Time Built-ins

```
// Time operations
time.now()                          // Current timestamp (milliseconds)
time.date()                         // Current date/time object
time.parse(str, format)             // Parse time string
time.format(time, format)           // Format time
time.unix(timestamp)                // Unix timestamp to date
time.add(time, duration)            // Add duration
time.diff(time1, time2)             // Difference between times
time.year(time), time.month(time), time.day(time)
time.hour(time), time.minute(time), time.second(time)

// Duration helpers
time.milliseconds(n), time.seconds(n), time.minutes(n)
time.hours(n), time.days(n)
```

### 5.11 Crypto Built-ins

```
// Cryptography operations
crypto.md5(data)                    // MD5 hash
crypto.sha1(data)                   // SHA-1 hash
crypto.sha256(data)                 // SHA-256 hash
crypto.sha512(data)                 // SHA-512 hash
crypto.randomBytes(n)               // Generate n random bytes
crypto.randomString(n)              // Generate random string
crypto.uuid()                       // Generate UUID v4
crypto.base64Encode(data)           // Base64 encode
crypto.base64Decode(str)            // Base64 decode
crypto.hexEncode(data)              // Hex encode
crypto.hexDecode(str)               // Hex decode
```

### 5.12 Process Built-ins

```
// Process operations
process.exit(code)                  // Exit with code
process.args()                      // Command line arguments
process.env(key)                    // Get environment variable
process.setEnv(key, value)          // Set environment variable
process.pid()                       // Process ID
process.exec(cmd, args)             // Execute command
process.spawn(cmd, args)            // Spawn process
process.kill(pid, signal)           // Kill process
```

### 5.13 Network Built-ins

```
// Network operations
net.resolve(hostname)               // Resolve hostname to IP
net.listen(protocol, address)       // Listen on address ("tcp", "udp")
net.dial(protocol, address)         // Connect to address
net.localIP()                       // Get local IP address
```

### 5.14 Encoding Built-ins

```
// Encoding operations
encoding.utf8Encode(str)            // String to UTF-8 bytes
encoding.utf8Decode(bytes)          // UTF-8 bytes to string
encoding.urlEncode(str)             // URL encode
encoding.urlDecode(str)             // URL decode
encoding.htmlEscape(str)            // HTML escape
encoding.htmlUnescape(str)          // HTML unescape
```

---

## 6. Standard Library

### 6.1 Core Modules

#### `std.io` - Input/Output
```
import std.io

// File operations
let f = io.open("file.txt", "r")
defer f.close()

let content = f.read()
let lines = f.readLines()

// Buffered I/O
let reader = io.BufferedReader.new(f)
let line = reader.readLine()

// Writer
let writer = io.BufferedWriter.new(f)
writer.write("Hello")
writer.flush()

// Standard streams
io.stdin.readLine()
io.stdout.write("Output")
io.stderr.write("Error")
```

#### `std.fmt` - Formatting
```
import std.fmt

// String formatting
let s = fmt.sprintf("Name: %s, Age: %d", name, age)
let formatted = fmt.format("{} + {} = {}", a, b, a+b)

// Parse
let num = fmt.parseInt("42")
let f = fmt.parseFloat("3.14")
```

#### `std.strings` - String Operations
```
import std.strings

let upper = strings.toUpper("hello")
let lower = strings.toLower("WORLD")
let parts = strings.split("a,b,c", ",")
let joined = strings.join(parts, "-")
let trimmed = strings.trim("  hello  ")
let replaced = strings.replaceAll("hello", "l", "L")
let repeated = strings.repeat("ab", 3)
let hasPrefix = strings.hasPrefix("hello", "he")
let hasSuffix = strings.hasSuffix("hello", "lo")
```

#### `std.arrays` - Array Operations
```
import std.arrays

let sorted = arrays.sort([3, 1, 2])
let reversed = arrays.reverse([1, 2, 3])
let filtered = arrays.filter([1,2,3,4], (x) => x % 2 == 0)
let mapped = arrays.map([1,2,3], (x) => x * 2)
let sum = arrays.reduce([1,2,3,4], (acc, x) => acc + x, 0)
let unique = arrays.unique([1,2,2,3,3,3])
let flattened = arrays.flatten([[1,2], [3,4]])
```

#### `std.math` - Mathematics
```
import std.math

// Basic operations
let sqrt = math.sqrt(16.0)
let pow = math.pow(2.0, 8.0)
let abs = math.abs(-42)
let min = math.min(5, 10)
let max = math.max(5, 10)

// Trigonometry
let sin = math.sin(math.PI / 2)
let cos = math.cos(0.0)
let tan = math.tan(math.PI / 4)

// Rounding
let ceil = math.ceil(3.2)
let floor = math.floor(3.8)
let round = math.round(3.5)

// Random
let rand = math.random()
let randInt = math.randomInt(1, 100)
let randChoice = math.choice([1, 2, 3, 4, 5])

// Constants
const PI = math.PI
const E = math.E
```

#### `std.time` - Time and Duration
```
import std.time

// Current time
let now = time.now()
let date = time.date()

// Formatting
let formatted = time.format(now, "2006-01-02 15:04:05")
let parsed = time.parse("2024-01-01", "2006-01-02")

// Duration
let dur = time.seconds(5)
time.sleep(dur)

// Timers
let timer = time.newTimer(time.seconds(10))
<-timer.channel()  // Wait

// Ticker
let ticker = time.newTicker(time.seconds(1))
for tick in ticker.channel() {
    print("Tick: " + tick)
}
```

#### `std.sync` - Synchronization
```
import std.sync

// Mutex
let mu = sync.mutex()
mu.lock()
defer mu.unlock()

// RWMutex (read-write lock)
let rwmu = sync.rwMutex()
rwmu.rLock()  // Read lock
defer rwmu.rUnlock()

rwmu.lock()   // Write lock
defer rwmu.unlock()

// WaitGroup
let wg = sync.waitGroup()
wg.add(10)

for i in 0..<10 {
    run func() {
        defer wg.done()
        doWork()
    }()
}

wg.wait()

// Once (execute once)
let once = sync.once()
once.do(func() {
    initialize()
})

// Semaphore
let sem = sync.semaphore(3)  // Max 3 concurrent
sem.acquire()
defer sem.release()

// Atomic operations
let counter = sync.atomic(0)
counter.add(1)
counter.load()
counter.store(42)
counter.compareAndSwap(42, 100)
```

#### `std.os` - Operating System
```
import std.os

// Environment
let path = os.getenv("PATH")
os.setenv("VAR", "value")
let allEnv = os.environ()

// Process
let args = os.args()
let pid = os.getpid()
os.exit(0)

// Working directory
let cwd = os.getcwd()
os.chdir("/tmp")

// System info
let hostname = os.hostname()
let username = os.username()
let homedir = os.homedir()
let tempdir = os.tempdir()
```

#### `std.path` - File Paths
```
import std.path

let joined = path.join("dir", "subdir", "file.txt")
let dir = path.dir("/path/to/file.txt")
let base = path.base("/path/to/file.txt")
let ext = path.ext("file.txt")
let abs = path.abs("relative/path")
let rel = path.rel("/a/b", "/a/c")
let exists = path.exists("/path/to/file")
let isDir = path.isDir("/path/to/dir")
let isFile = path.isFile("/path/to/file")
```

#### `std.http` - HTTP Client/Server
```
import std.http

// Client - GET
let resp = http.get("https://api.example.com/users")
print(resp.status)
print(resp.text())

// Client - POST
let resp = http.post("https://api.example.com/users", {
    headers: {"Content-Type": "application/json"},
    body: `{"name": "Alice", "age": 30}`
})

// Client - Custom request
let req = http.request("PUT", "https://api.example.com/users/1")
req.setHeader("Authorization", "Bearer token")
req.setBody(`{"name": "Bob"}`)
let resp = req.send()

// Server
let server = http.server()

server.handle("/", func(req, res) {
    res.setHeader("Content-Type", "text/html")
    res.write("<h1>Hello World</h1>")
})

server.handle("/api/users", func(req, res) {
    match req.method {
        "GET" -> {
            let users = getUsers()
            res.json(users)
        }
        "POST" -> {
            let body = req.json()
            let user = createUser(body)
            res.status(201).json(user)
        }
        _ -> {
            res.status(405).json({error: "Method not allowed"})
        }
    }
})

server.listen(8080)
```

#### `std.json` - JSON Encoding/Decoding
```
import std.json

// Encode
let obj = {
    name: "Alice",
    age: 30,
    active: true,
    scores: [85, 90, 95]
}
let jsonStr = json.encode(obj)
let pretty = json.encodePretty(obj)

// Decode
let parsed = json.decode(jsonStr)
print(parsed.name)
print(parsed.scores[0])

// Validate
if json.validate(jsonStr) {
    print("Valid JSON")
}

// Type-safe decoding
struct User {
    name: string
    age: int32
    active: bool
}

let user = json.decodeAs<User>(jsonStr)
```

#### `std.xml` - XML Processing
```
import std.xml

// Parse
let doc = xml.parse(`<root><item id="1">Hello</item></root>`)
let items = doc.find("//item")
for item in items {
    print(item.attr("id"))
    print(item.text())
}

// Create
let root = xml.element("root")
let item = xml.element("item")
item.setAttr("id", "1")
item.setText("Hello")
root.append(item)

let xmlStr = xml.toString(root)
```

#### `std.csv` - CSV Processing
```
import std.csv

// Read
let reader = csv.reader(file)
let rows = reader.readAll()
for row in rows {
    print(row[0], row[1])
}

// Write
let writer = csv.writer(file)
writer.write(["Name", "Age", "City"])
writer.write(["Alice", "30", "Paris"])
writer.flush()
```

#### `std.crypto` - Cryptography
```
import std.crypto

// Hashing
let hash = crypto.sha256("password")
let md5 = crypto.md5("data")

// Random
let bytes = crypto.randomBytes(32)
let token = crypto.randomString(16)
let uuid = crypto.uuid()

// Encoding
let b64 = crypto.base64Encode(data)
let decoded = crypto.base64Decode(b64)
let hex = crypto.hexEncode(data)

// Password hashing
let hashed = crypto.bcrypt("password", 10)
let valid = crypto.bcryptVerify("password", hashed)

// HMAC
let hmac = crypto.hmacSha256(message, key)
```

#### `std.regex` - Regular Expressions
```
import std.regex

// Compile
let pattern = regex.compile(`\d{3}-\d{4}`)

// Match
if pattern.match("123-4567") {
    print("Valid phone number")
}

// Find
let matches = pattern.findAll("Call 123-4567 or 890-1234")
for match in matches {
    print(match)
}

// Replace
let result = pattern.replace("Phone: 123-4567", "XXX-XXXX")

// Split
let parts = regex.split(`\s+`, "hello   world  foo")

// Groups
let pattern = regex.compile(`(\d{3})-(\d{4})`)
let groups = pattern.groups("123-4567")
print(groups[1])  // "123"
print(groups[2])  // "4567"
```

#### `std.net` - Networking
```
import std.net

// TCP Server
let listener = net.listen("tcp", ":8080")
print("Listening on :8080")

while true {
    let conn = listener.accept()
    run handleConnection(conn)
}

func handleConnection(conn: net.Conn) {
    defer conn.close()
    
    let buffer = make([]byte, 1024)
    let n = conn.read(buffer)
    let message = string(buffer[0..<n])
    
    conn.write("Echo: " + message)
}

// TCP Client
let conn = net.dial("tcp", "example.com:80")
defer conn.close()

conn.write("GET / HTTP/1.0\r\n\r\n")
let response = conn.readAll()
print(response)

// UDP
let udpConn = net.listenPacket("udp", ":9000")
let data, addr = udpConn.readFrom()
udpConn.writeTo("Pong", addr)
```

#### `std.db` - Database
```
import std.db

// Connect
let db = db.open("postgres", "user:pass@localhost/dbname")
defer db.close()

// Query
let rows = db.query("SELECT * FROM users WHERE age > $1", 18)
for row in rows {
    print(row.getString("name"))
    print(row.getInt("age"))
}

// Execute
let result = db.exec("INSERT INTO users (name, age) VALUES ($1, $2)", "Alice", 30)
print("Inserted rows: " + result.rowsAffected())

// Transaction
let tx = db.begin()
try {
    tx.exec("UPDATE accounts SET balance = balance - 100 WHERE id = $1", 1)
    tx.exec("UPDATE accounts SET balance = balance + 100 WHERE id = $1", 2)
    tx.commit()
} fail on err {
    tx.rollback()
    fail err
}

// Prepared statement
let stmt = db.prepare("SELECT * FROM users WHERE name = $1")
defer stmt.close()

let rows = stmt.query("Alice")
```

#### `std.log` - Logging
```
import std.log

// Basic logging
log.info("Application started")
log.debug("Debug information")
log.warn("Warning message")
log.error("Error occurred")
log.fatal("Fatal error")  // Exits program

// Formatted logging
log.infof("User %s logged in at %s", username, time.now())

// Custom logger
let logger = log.new("myapp")
logger.setLevel(log.DEBUG)
logger.setOutput(file)

logger.info("Custom log message")

// Structured logging
log.withFields({
    "user": "alice",
    "action": "login",
    "ip": "192.168.1.1"
}).info("User logged in")
```

#### `std.test` - Testing Framework
```
import std.test

func TestAdd(t: test.T) {
    let result = add(2, 3)
    t.assertEqual(result, 5)
    t.assertNotEqual(result, 4)
}

func TestDivide(t: test.T) {
    try {
        divide(10, 0)
        t.fail("Expected error for division by zero")
    } fail on err {
        t.assertContains(err, "division by zero")
    }
}

func TestUser(t: test.T) {
    let user = User{name: "Alice", age: 30}
    t.assertEqual(user.name, "Alice")
    t.assertTrue(user.age > 0)
    t.assertFalse(user.isAdmin())
}

// Benchmarks
func BenchmarkSort(b: test.B) {
    let data = generateRandomArray(1000)
    b.resetTimer()
    
    for i in 0..<b.iterations {
        sort(data)
    }
}

// Run tests: lang test ./...
// Run specific test: lang test TestAdd
// Run benchmarks: lang test -bench .
```

---

## 7. Compilation & Toolchain

### 7.1 Compiler Architecture

```
Source Code (.lang files)
    ↓
[Lexer] → Tokens
    ↓
[Parser] → Abstract Syntax Tree (AST)
    ↓
[Semantic Analysis]
    - Type checking
    - Name resolution
    - Interface satisfaction
    - Generic instantiation
    ↓
[Intermediate Representation (IR)]
    - Three-address code
    - SSA form
    ↓
[Optimization Passes]
    - Dead code elimination
    - Constant folding
    - Inline expansion
    - Loop optimization
    - Escape analysis
    ↓
[Code Generation]
    - Machine code (x86-64, ARM64, etc.)
    - Or LLVM IR for LLVM backend
    ↓
[Linking]
    - Standard library
    - User libraries
    - Runtime (GC, goroutine scheduler)
    ↓
Executable Binary
```

### 7.2 Compiler Commands

```bash
# Compile single file
langc main.lang -o program

# Compile with optimization levels
langc main.lang -O0 -o program  # No optimization (fast compile)
langc main.lang -O1 -o program  # Basic optimization
langc main.lang -O2 -o program  # Standard (recommended)
langc main.lang -O3 -o program  # Aggressive optimization

# Compile multiple files
langc main.lang utils.lang models.lang -o program

# Compile entire directory/package
langc ./src -o program

# Cross-compilation
langc main.lang --target linux-amd64 -o program
langc main.lang --target linux-arm64 -o program-arm
langc main.lang --target windows-amd64 -o program.exe
langc main.lang --target darwin-amd64 -o program-mac
langc main.lang --target darwin-arm64 -o program-m1
langc main.lang --target wasm -o program.wasm

# Show intermediate representations
langc main.lang --emit-tokens    # Show tokens
langc main.lang --emit-ast       # Show AST
langc main.lang --emit-ir        # Show IR
langc main.lang --emit-asm       # Show assembly

# Debug build
langc main.lang -g -o program    # Include debug symbols

# Static linking (no external dependencies)
langc main.lang --static -o program

# Optimize for size
langc main.lang -Os -o program

# Verbose output
langc main.lang -v -o program

# Check syntax only (no code generation)
langc main.lang --check

# Format code
langc --fmt main.lang

# Show compiler version
langc --version
```

### 7.3 Project Structure

```
myproject/
├── lang.yml              # Project configuration
├── src/
│   ├── main.lang          # Entry point (must have main() function)
│   ├── utils.lang
│   ├── models/
│   │   ├── user.lang
│   │   └── product.lang
│   └── handlers/
│       └── api.lang
├── tests/
│   ├── main_test.lang
│   └── utils_test.lang
├── vendor/                # Third-party dependencies
├── build/                 # Build artifacts
└── README.md
```

### 7.4 Project Configuration (`lang.toml`)

```toml
[package]
name = "myproject"
version = "1.0.0"
description = "My awesome project"
authors = ["Your Name <email@example.com>"]
license = "MIT"
repository = "https://github.com/user/myproject"

[build]
target = "executable"      # or "library"
entry = "src/main.lang"
output = "build/myproject"
optimization = "O2"        # O0, O1, O2, O3, Os
debug = false
static = false

[dependencies]
http = "1.2.0"
json = "2.0.1"
postgres = "3.1.0"

[dev-dependencies]
test = "1.0.0"

[compiler]
warnings = "all"           # all, none, error
experimental = false
```

### 7.5 Package Manager

```bash
# Initialize new project
lang init myproject
cd myproject

# Add dependency
lang add http@1.2.0
lang add json          # Latest version

# Remove dependency
lang remove http

# Update dependencies
lang update
lang update http       # Update specific package

# List dependencies
lang list

# Install dependencies (from lang.toml)
lang install

# Search packages
lang search http

# Publish package
lang publish

# Package registry
# Default: https://pkg.lang-lang.org
```

### 7.6 Build Commands

```bash
# Build project (uses lang.toml)
lang build

# Build with specific target
lang build --target linux-amd64

# Build for release (optimized)
lang build --release

# Clean build artifacts
lang clean

# Run directly (compile + execute)
lang run

# Run with arguments
lang run -- arg1 arg2

# Run tests
lang test
lang test ./tests
lang test TestUserCreate

# Run benchmarks
lang test --bench

# Format code
lang fmt
lang fmt src/

# Lint code
lang lint
lang lint src/

# Generate documentation
lang doc
lang doc --serve  # Start doc server on :6060

# Show project info
lang info
```

### 7.7 Executable Format

**Supported Platforms:**
- **Linux**: ELF (Executable and Linkable Format)
- **Windows**: PE (Portable Executable)
- **macOS**: Mach-O
- **WebAssembly**: WASM

**Binary Structure (ELF example):**
```
Executable File
├── ELF Header
├── Program Headers
├── .text          # Code segment (machine instructions)
├── .rodata        # Read-only data (string literals, constants)
├── .data          # Initialized global/static variables
├── .bss           # Uninitialized global/static variables
├── .symtab        # Symbol table (debug builds)
├── .strtab        # String table
├── .debug_*       # Debug information (if -g flag used)
└── Section Headers
```

**Runtime Components:**
```
Executable
├── User Code
├── Standard Library (static or dynamic)
├── Runtime System
│   ├── Garbage Collector
│   ├── Goroutine Scheduler
│   ├── Channel Implementation
│   └── Panic/Recover Handler
└── C Runtime (libc) - if not static
```

### 7.8 Debugging

```bash
# Compile with debug symbols
langc main.lang -g -o program

# Debug with GDB
gdb ./program

# Debug with LLDB (macOS)
lldb ./program

# Built-in debugger (future feature)
lang debug ./program
```

**Debug Symbols:**
- Function names
- Variable names
- Line number information
- Type information
- Source file paths

### 7.9 Performance Profiling

```bash
# CPU profiling
lang build --profile=cpu
./program
lang tool pprof cpu.prof

# Memory profiling
lang build --profile=mem
./program
lang tool pprof mem.prof

# Trace execution
lang build --trace
./program
lang tool trace trace.out

# Benchmark with profiling
lang test --bench --cpuprofile=cpu.prof
```

---

## 8. Example Programs

### 8.1 Hello World

```
func main() {
    println("Hello, World!")
}
```

### 8.2 HTTP Server

```
import std.{http, json}

struct User {
    id: int32
    name: string
    email: string
}

let users: []User = []
let userIdCounter = 0

func main() {
    let server = http.server()
    
    server.handle("/api/users", handleUsers)
    server.handle("/api/users/:id", handleUserById)
    
    println("Server listening on :8080")
    server.listen(8080)
}

func handleUsers(req, res) {
    match req.method {
        "GET" -> {
            res.json(users)
        }
        "POST" -> {
            let body = req.json()
            userIdCounter = userIdCounter + 1
            
            let user = User{
                id: userIdCounter,
                name: body.name,
                email: body.email
            }
            
            users.append(user)
            res.status(201).json(user)
        }
        _ -> {
            res.status(405).json({error: "Method not allowed"})
        }
    }
}

func handleUserById(req, res) {
    let id = req.param("id") as int32
    
    for user in users {
        if user.id == id {
            match req.method {
                "GET" -> res.json(user)
                "DELETE" -> {
                    users = arrays.filter(users, (u) => u.id != id)
                    res.status(204).send()
                }
                _ -> res.status(405).json({error: "Method not allowed"})
            }
            return
        }
    }
    
    res.status(404).json({error: "User not found"})
}
```

### 8.3 Concurrent Pipeline

```
import std.math

func stage1(input: <-chan int32, output: chan<- float64) {
    defer close(output)
    
    for value in input {
        let result = math.sqrt(value as float64)
        output <- result
    }
}

func stage2(input: <-chan float64, output: chan<- float64) {
    defer close(output)
    
    for value in input {
        output <- value * 2.0
    }
}

func stage3(input: <-chan float64, output: chan<- string) {
    defer close(output)
    
    for value in input {
        output <- "Result: " + value
    }
}

func main() {
    let ch1 = channel(int32, 10)
    let ch2 = channel(float64, 10)
    let ch3 = channel(float64, 10)
    let ch4 = channel(string, 10)
    
    // Start pipeline
    run stage1(ch1, ch2)
    run stage2(ch2, ch3)
    run stage3(ch3, ch4)
    
    // Feed input
    run func() {
        defer close(ch1)
        for i in 1...100 {
            ch1 <- i
        }
    }()
    
    // Collect results
    for result in ch4 {
        println(result)
    }
}
```

### 8.4 Web Scraper with Async

```
import std.http
import std.regex
import std.strings

async func fetchPage(url: string): string {
    try {
        let resp = await http.get(url)
        return resp.text()
    } fail on err {
        println("Error fetching " + url + ": " + err)
        return ""
    }
}

async func extractLinks(html: string): []string {
    let pattern = regex.compile(`href="(https?://[^"]+)"`)
    let matches = pattern.findAll(html)
    
    let links = []
    for match in matches {
        let groups = pattern.groups(match)
        if len(groups) > 1 {
            links.append(groups[1])
        }
    }
    return links
}

async func scrapeWebsite(startUrl: string, maxPages: int32) {
    let visited = map[string]bool{}
    let toVisit = [startUrl]
    let pageCount = 0
    
    while len(toVisit) > 0 and pageCount < maxPages {
        let url = toVisit[0]
        toVisit = toVisit[1..<len(toVisit)]
        
        if visited[url] {
            continue
        }
        
        visited[url] = true
        pageCount = pageCount + 1
        
        println("Scraping: " + url)
        let html = await fetchPage(url)
        
        if html != "" {
            let links = await extractLinks(html)
            for link in links {
                if not visited[link] {
                    toVisit.append(link)
                }
            }
        }
    }
    
    println("Scraped " + pageCount + " pages")
}

async func main() {
    await scrapeWebsite("https://example.com", 50)
}
```

### 8.5 Chat Server with Channels

```
import std.net
import std.strings
import std.time

struct Client {
    conn: net.Conn
    name: string
    messages: chan string
}

let clients: []Client = []
let clientMutex = lock()
let broadcast = channel(string, 100)

func main() {
    // Broadcast goroutine
    run broadcaster()
    
    let listener = net.listen("tcp", ":9000")
    println("Chat server listening on :9000")
    
    while true {
        let conn = listener.accept()
        run handleClient(conn)
    }
}

func broadcaster() {
    while true {
        let msg = <-broadcast
        
        lock clientMutex {
            for client in clients {
                select {
                    client.messages <- msg -> {}
                    _ -> {
                        // Client channel full, skip
                    }
                }
            }
        }
    }
}

func handleClient(conn: net.Conn) {
    defer conn.close()
    
    // Get username
    conn.write("Enter your name: ")
    let nameBytes = make([]byte, 256)
    let n = conn.read(nameBytes)
    let name = strings.trim(string(nameBytes[0..<n]))
    
    let client = Client{
        conn: conn,
        name: name,
        messages: channel(string, 10)
    }
    
    // Add client
    lock clientMutex {
        clients.append(client)
    }
    
    broadcast <- name + " joined the chat"
    
    // Start message sender
    run sendMessages(client)
    
    // Read messages
    let buffer = make([]byte, 1024)
    while true {
        let n = conn.read(buffer)
        if n == 0 {
            break
        }
        
        let message = strings.trim(string(buffer[0..<n]))
        if message == "/quit" {
            break
        }
        
        let timestamp = time.format(time.now(), "15:04:05")
        broadcast <- "[" + timestamp + "] " + name + ": " + message
    }
    
    // Remove client
    lock clientMutex {
        clients = arrays.filter(clients, (c) => c.name != name)
    }
    
    broadcast <- name + " left the chat"
    close(client.messages)
}

func sendMessages(client: Client) {
    for msg in client.messages {
        try {
            client.conn.write(msg + "\n")
        } fail on err {
            break
        }
    }
}
```

### 8.6 Task Queue with Worker Pool

```
import std.time
import std.math

struct Task {
    id: int32
    data: string
}

struct Result {
    taskId: int32
    result: string
    duration: int64
}

func worker(id: int32, tasks: <-chan Task, results: chan<- Result) {
    println("Worker " + id + " started")
    
    for task in tasks {
        let start = time.now()
        
        // Simulate work
        let workTime = math.randomInt(100, 1000)
        time.sleep(time.milliseconds(workTime))
        
        let result = Result{
            taskId: task.id,
            result: "Processed: " + task.data,
            duration: time.now() - start
        }
        
        results <- result
    }
    
    println("Worker " + id + " finished")
}

func main() {
    const NUM_WORKERS = 5
    const NUM_TASKS = 20
    
    let tasks = channel(Task, NUM_TASKS)
    let results = channel(Result, NUM_TASKS)
    
    // Start workers
    for i in 1...NUM_WORKERS {
        run worker(i, tasks, results)
    }
    
    // Send tasks
    run func() {
        for i in 1...NUM_TASKS {
            tasks <- Task{id: i, data: "Task " + i}
        }
        close(tasks)
    }()
    
    // Collect results
    let completed = 0
    while completed < NUM_TASKS {
        let result = <-results
        println("Task " + result.taskId + " completed in " + result.duration + "ms")
        completed = completed + 1
    }
    
    println("All tasks completed!")
}
```

### 8.7 Database CRUD Application

```
import std.db
import std.json
import std.time

struct User {
    id: int32
    name: string
    email: string
    createdAt: int64
}

func connectDB(): db.DB {
    let database = db.open("postgres", "user=admin password=secret dbname=myapp")
    
    // Create table if not exists
    database.exec(`
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            created_at BIGINT NOT NULL
        )
    `)
    
    return database
}

func createUser(db: db.DB, name: string, email: string): User {
    let now = time.now()
    
    let result = db.query(
        "INSERT INTO users (name, email, created_at) VALUES ($1, $2, $3) RETURNING id",
        name, email, now
    )
    
    let row = result.next()
    let id = row.getInt("id")
    
    return User{
        id: id,
        name: name,
        email: email,
        createdAt: now
    }
}

func getUser(db: db.DB, id: int32): User {
    let result = db.query("SELECT * FROM users WHERE id = $1", id)
    
    if result.hasNext() {
        let row = result.next()
        return User{
            id: row.getInt("id"),
            name: row.getString("name"),
            email: row.getString("email"),
            createdAt: row.getLong("created_at")
        }
    }
    
    fail "User not found"
}

func getAllUsers(db: db.DB): []User {
    let result = db.query("SELECT * FROM users ORDER BY created_at DESC")
    let users = []
    
    for row in result {
        users.append(User{
            id: row.getInt("id"),
            name: row.getString("name"),
            email: row.getString("email"),
            createdAt: row.getLong("created_at")
        })
    }
    
    return users
}

func updateUser(db: db.DB, id: int32, name: string, email: string) {
    db.exec(
        "UPDATE users SET name = $1, email = $2 WHERE id = $3",
        name, email, id
    )
}

func deleteUser(db: db.DB, id: int32) {
    db.exec("DELETE FROM users WHERE id = $1", id)
}

func main() {
    let db = connectDB()
    defer db.close()
    
    try {
        // Create users
        let user1 = createUser(db, "Alice", "alice@example.com")
        let user2 = createUser(db, "Bob", "bob@example.com")
        println("Created users: " + user1.id + ", " + user2.id)
        
        // Get user
        let user = getUser(db, user1.id)
        println("Retrieved user: " + user.name + " (" + user.email + ")")
        
        // Get all users
        let users = getAllUsers(db)
        println("Total users: " + len(users))
        
        // Update user
        updateUser(db, user1.id, "Alice Smith", "alice.smith@example.com")
        println("Updated user " + user1.id)
        
        // Delete user
        deleteUser(db, user2.id)
        println("Deleted user " + user2.id)
        
    } fail on err {
        println("Database error: " + err)
    }
}
```

### 8.8 File Watcher

```
import std.os
import std.path
import std.time
import std.crypto

struct FileInfo {
    path: string
    hash: string
    modTime: int64
}

let fileCache = map[string]FileInfo{}
let cacheMutex = lock()

func computeFileHash(filepath: string): string {
    try {
        let content = file.read(filepath)
        return crypto.sha256(content)
    } fail on err {
        return ""
    }
}

func scanDirectory(dirPath: string): []FileInfo {
    let files = []
    
    dir.walk(dirPath, func(path: string, info) {
        if path.isFile() {
            let hash = computeFileHash(path)
            let modTime = info.modTime()
            
            files.append(FileInfo{
                path: path,
                hash: hash,
                modTime: modTime
            })
        }
    })
    
    return files
}

func detectChanges(current: []FileInfo) {
    lock cacheMutex {
        let currentPaths = map[string]bool{}
        
        // Check for new and modified files
        for fileInfo in current {
            currentPaths[fileInfo.path] = true
            
            if not fileCache[fileInfo.path] {
                println("[NEW] " + fileInfo.path)
                fileCache[fileInfo.path] = fileInfo
            } else {
                let cached = fileCache[fileInfo.path]
                if cached.hash != fileInfo.hash {
                    println("[MODIFIED] " + fileInfo.path)
                    fileCache[fileInfo.path] = fileInfo
                }
            }
        }
        
        // Check for deleted files
        for path, _ in fileCache {
            if not currentPaths[path] {
                println("[DELETED] " + path)
                delete(fileCache, path)
            }
        }
    }
}

func watchDirectory(dirPath: string, interval: int32) {
    println("Watching directory: " + dirPath)
    println("Scan interval: " + interval + "ms")
    
    // Initial scan
    let files = scanDirectory(dirPath)
    lock cacheMutex {
        for fileInfo in files {
            fileCache[fileInfo.path] = fileInfo
        }
    }
    println("Initial scan complete: " + len(files) + " files")
    
    // Watch loop
    while true {
        time.sleep(time.milliseconds(interval))
        
        let current = scanDirectory(dirPath)
        detectChanges(current)
    }
}

func main() {
    if len(os.args()) < 2 {
        println("Usage: watcher <directory>")
        os.exit(1)
    }
    
    let dirPath = os.args()[1]
    
    if not path.exists(dirPath) {
        println("Error: Directory does not exist")
        os.exit(1)
    }
    
    if not path.isDir(dirPath) {
        println("Error: Path is not a directory")
        os.exit(1)
    }
    
    watchDirectory(dirPath, 1000)
}
```

### 8.9 JSON RPC Server

```
import std.net
import std.json
import std.strings

interface RPCHandler {
    func handle(params: map[string]any): any
}

struct EchoHandler {
    func handle(params: map[string]any): any {
        return params
    }
}

struct AddHandler {
    func handle(params: map[string]any): any {
        let a = params["a"] as int32
        let b = params["b"] as int32
        return a + b
    }
}

struct MultiplyHandler {
    func handle(params: map[string]any): any {
        let a = params["a"] as int32
        let b = params["b"] as int32
        return a * b
    }
}

let handlers = map[string]RPCHandler{
    "echo": EchoHandler{},
    "add": AddHandler{},
    "multiply": MultiplyHandler{}
}

struct RPCRequest {
    jsonrpc: string
    method: string
    params: map[string]any
    id: int32
}

struct RPCResponse {
    jsonrpc: string
    result: any
    error: string
    id: int32
}

func handleConnection(conn: net.Conn) {
    defer conn.close()
    
    let buffer = make([]byte, 4096)
    let n = conn.read(buffer)
    
    if n == 0 {
        return
    }
    
    let data = string(buffer[0..<n])
    
    try {
        let req = json.decodeAs<RPCRequest>(data)
        
        if req.jsonrpc != "2.0" {
            fail "Invalid JSON-RPC version"
        }
        
        let handler = handlers[req.method]
        if handler == null {
            fail "Method not found: " + req.method
        }
        
        let result = handler.handle(req.params)
        
        let response = RPCResponse{
            jsonrpc: "2.0",
            result: result,
            error: "",
            id: req.id
        }
        
        let responseJson = json.encode(response)
        conn.write(responseJson)
        
    } fail on err {
        let errorResponse = RPCResponse{
            jsonrpc: "2.0",
            result: null,
            error: err,
            id: 0
        }
        
        let responseJson = json.encode(errorResponse)
        conn.write(responseJson)
    }
}

func main() {
    let listener = net.listen("tcp", ":8080")
    println("JSON-RPC server listening on :8080")
    
    while true {
        let conn = listener.accept()
        run handleConnection(conn)
    }
}
```

### 8.10 Rate Limiter

```
import std.time
import std.sync

struct RateLimiter {
    rate: int32           // requests per second
    tokens: int32
    maxTokens: int32
    lastRefill: int64
    mu: lock
    
    func allow(): bool {
        lock this.mu {
            this.refill()
            
            if this.tokens > 0 {
                this.tokens = this.tokens - 1
                return true
            }
            
            return false
        }
    }
    
    func refill() {
        let now = time.now()
        let elapsed = now - this.lastRefill
        
        if elapsed >= 1000 {  // 1 second
            this.tokens = this.maxTokens
            this.lastRefill = now
        }
    }
    
    func wait() {
        while not this.allow() {
            time.sleep(time.milliseconds(10))
        }
    }
}

func RateLimiter.new(rate: int32): RateLimiter {
    return RateLimiter{
        rate: rate,
        tokens: rate,
        maxTokens: rate,
        lastRefill: time.now(),
        mu: lock()
    }
}

// Token bucket with goroutine refill
struct TokenBucket {
    capacity: int32
    tokens: chan bool
    
    func take(): bool {
        select {
            <-this.tokens -> {
                return true
            }
            _ -> {
                return false
            }
        }
    }
    
    func wait() {
        <-this.tokens
    }
}

func TokenBucket.new(capacity: int32, rate: int32): TokenBucket {
    let tokens = channel(bool, capacity)
    
    // Fill initial tokens
    for i in 0..<capacity {
        tokens <- true
    }
    
    // Refill goroutine
    run func() {
        let ticker = time.newTicker(time.milliseconds(1000 / rate))
        for _ in ticker.channel() {
            select {
                tokens <- true -> {}
                _ -> {}  // Bucket full, skip
            }
        }
    }()
    
    return TokenBucket{
        capacity: capacity,
        tokens: tokens
    }
}

func main() {
    println("Testing Rate Limiter (5 req/s)")
    let limiter = RateLimiter.new(5)
    
    for i in 1...20 {
        limiter.wait()
        println("Request " + i + " at " + time.format(time.now(), "15:04:05.000"))
    }
    
    println("\nTesting Token Bucket (10 req/s, burst 5)")
    let bucket = TokenBucket.new(5, 10)
    
    for i in 1...20 {
        bucket.wait()
        println("Request " + i + " at " + time.format(time.now(), "15:04:05.000"))
    }
}
```

---

## 9. Advanced Features

### 9.1 Reflection

```
import std.reflect

struct Person {
    name: string
    age: int32
    email: string
}

func main() {
    let p = Person{name: "Alice", age: 30, email: "alice@example.com"}
    
    // Get type information
    let t = reflect.typeOf(p)
    println("Type: " + t.name())
    println("Kind: " + t.kind())
    
    // Get fields
    let fields = t.fields()
    for field in fields {
        println("Field: " + field.name + " (" + field.type + ")")
    }
    
    // Get/Set field values
    let v = reflect.valueOf(p)
    let nameField = v.field("name")
    println("Name: " + nameField.get())
    
    nameField.set("Bob")
    println("Updated name: " + p.name)
    
    // Call methods
    let method = v.method("toString")
    let result = method.call([])
    println(result)
}
```

### 9.2 Code Generation (Macros)

```
// Macro definition (compile-time)
macro repeat(n, body) {
    for i in 0..<n {
        body
    }
}

func main() {
    // Expands at compile time to:
    // for i in 0..<5 { println(i) }
    repeat!(5, {
        println(i)
    })
}

// Derive macro for auto-implementing traits
@derive(Clone, Debug, Serialize)
struct User {
    name: string
    age: int32
}
```

### 9.3 FFI (Foreign Function Interface)

```
// Call C functions
extern "C" func printf(format: *char, ...): int32
extern "C" func malloc(size: uint64): *void
extern "C" func free(ptr: *void)

func main() {
    // Call C function
    printf("Hello from C!\n")
    
    // Allocate memory
    let ptr = malloc(1024)
    defer free(ptr)
}

// Link with C library
// lang build --link-lib=m main.lang
```

---

## 10. Language Design Principles

### 10.1 Core Principles

1. **Simplicity** - 33 keywords, easy to learn
2. **Expressiveness** - Pattern matching, generators, async/await
3. **Safety** - Type safety, memory safety, data race prevention
4. **Performance** - Compiled to native code, zero-cost abstractions
5. **Concurrency** - First-class channels and goroutines
6. **Practicality** - Rich standard library, good tooling

### 10.2 Influences

- **Go** - Simplicity, channels, goroutines
- **Rust** - Pattern matching, type safety
- **Python** - Readability, intuitive syntax
- **JavaScript** - Async/await, JSON support
- **Swift** - Range operators, clean syntax

### 10.3 Philosophy

```
"A language that tries to do everything ends up doing nothing well.
We focus on:
- Systems programming
- Web services
- Concurrent applications
- Command-line tools

We don't try to be:
- A scripting language (use Python)
- A data science language (use R/Julia)
- A low-level language (use C/Rust)
"
```

---

## 11. Roadmap

### Version 1.0 (Current)
- ✅ Core language features
- ✅ Standard library
- ✅ Compiler and toolchain
- ✅ Package manager
- ✅ Basic documentation

### Version 1.1 (Q2 2025)
- 🔄 Generics improvements
- 🔄 Better error messages
- 🔄 LSP (Language Server Protocol)
- 🔄 IDE plugins (VS Code, IntelliJ)

### Version 1.2 (Q3 2025)
- 📋 Reflection API
- 📋 Plugin system
- 📋 WebAssembly support
- 📋 Cross-compilation improvements

### Version 2.0 (Q4 2025)
- 📋 Compile-time macros
- 📋 Better generic constraints
- 📋 Async improvements
- 📋 Performance optimizations

---

**Language Name:** Lang  
**Version:** 1.0  
**License:** MIT  
**Website:** https://lang-lang.org  
**Repository:** https://github.com/lang-lang/lang  
**Last Updated:** September 30, 2025