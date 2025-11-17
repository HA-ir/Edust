# Edust Compiler

A complete educational compiler for the Edust programming language that compiles directly to native machine code using Cranelift JIT.

## Features

- **Complete language implementation** with integers, functions, control flow, and operators
- **Direct to machine code** - no intermediate language translation
- **Stack-based execution model** with proper variable scoping
- **Full semantic analysis** with error reporting
- **Optimized code generation** using Cranelift backend

## Language Overview

Edust is a simple imperative language designed for educational purposes:

```edust
func main() {
    let i = 0;
    while i < 5 {
        print(i);
        i = i + 1;
    }
    return 0;
}
```

### Supported Features

- **Variables**: `let x = 42;`
- **Functions**: `func add(a, b) { return a + b; }`
- **Control Flow**: `if/else`, `while` loops
- **Operators**: 
  - Arithmetic: `+`, `-`, `*`, `/`, `%`
  - Comparison: `<`, `<=`, `>`, `>=`, `==`, `!=`
  - Logical: `&&`, `||`, `!`
- **Built-in Functions**: `print(value)`
- **Entry Point**: Mandatory `main()` function

## Building

```bash
cargo build --release
```

## Running

```bash
# Compile and run a program
cargo run --release -- examples/test.edust

# Or use the binary directly
./target/release/edustc examples/test.edust
```

## Testing

```bash
cargo test
```

The test suite includes:
- Basic arithmetic and operations
- Control flow (if/else, while)
- Function calls and recursion
- All comparison and logical operators
- Variable scoping

## Architecture

### 1. Lexer (`lexer.rs`)
Tokenizes source code into a stream of tokens:
- Keywords: `func`, `let`, `if`, `else`, `while`, `return`
- Operators: arithmetic, comparison, logical
- Literals: integers
- Identifiers and delimiters

### 2. Parser (`parser.rs`)
Recursive descent parser that builds an Abstract Syntax Tree (AST):
- Implements operator precedence correctly
- Handles all language constructs
- Provides clear error messages with location info

### 3. Semantic Analyzer (`semantic.rs`)
Validates the AST before code generation:
- Checks for undefined variables
- Validates function signatures
- Ensures proper scoping rules
- Verifies `main()` exists and has correct signature

### 4. Code Generator (`codegen.rs`)
Generates native machine code using Cranelift:
- Creates SSA (Static Single Assignment) form
- Handles stack-based variables
- Generates efficient control flow
- Links to runtime functions

### 5. Runtime (`runtime.rs`)
Minimal runtime support:
- `print_int()`: Displays integer values

## Compilation Pipeline

```
Source Code
    ↓
[Lexer] → Tokens
    ↓
[Parser] → AST
    ↓
[Semantic Analysis] → Validated AST
    ↓
[Code Generator (Cranelift)] → Machine Code
    ↓
[JIT Execution]
```

## Example Programs

### Hello World (with numbers)
```edust
func main() {
    print(42);
    return 0;
}
```

### Factorial
```edust
func factorial(n) {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

func main() {
    print(factorial(5));  // Prints: 120
    return 0;
}
```

### Fibonacci
```edust
func fibonacci(n) {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

func main() {
    let i = 0;
    while i < 10 {
        print(fibonacci(i));
        i = i + 1;
    }
    return 0;
}
```

### Prime Number Check
```edust
func is_prime(n) {
    if n <= 1 {
        return 0;
    }
    let i = 2;
    while i * i <= n {
        if n % i == 0 {
            return 0;
        }
        i = i + 1;
    }
    return 1;
}

func main() {
    let num = 17;
    print(is_prime(num));  // Prints: 1 (true)
    return 0;
}
```

## Error Handling

The compiler provides clear error messages at each stage:

```
Lexer error: Unexpected character '@' at line 3, column 5
Parser error: Expected ')' at line 5, column 12
Semantic error: Undefined variable: x
Codegen error: Function 'foo' not found
```

## Limitations (MVP)

- Only integer type (no strings, floats, arrays)
- No standard library beyond `print()`
- No modules or imports
- No memory management (stack-only)
- Single file compilation

## Future Enhancements

Potential extensions for learning:
1. Add more types (strings, floats, booleans)
2. Implement arrays and pointers
3. Add a standard library
4. Implement proper optimizations
5. Add debugging information
6. Support multiple files
7. Create a REPL

## Technical Details

### Why Cranelift?

Cranelift is a fast, secure code generator used in production systems like Wasmtime. It provides:
- Fast compilation times
- Good code quality
- Safety guarantees
- Cross-platform support

### Variable Management

Variables are stored in Cranelift's SSA variables:
- Each variable declaration creates a new SSA variable
- Assignments update the SSA variable definition
- Scoping is handled by the semantic analyzer

### Control Flow

Control flow is implemented using basic blocks:
- **If/Else**: Three blocks (then, else, merge)
- **While**: Three blocks (header, body, exit)
- Proper block sealing ensures SSA form correctness

## Contributing

This is an educational project. Feel free to:
- Extend the language
- Improve error messages
- Add optimizations
- Write more examples

## License

Educational use - modify and learn freely!

## Resources

- [Cranelift Documentation](https://docs.rs/cranelift/)
- [Compiler Design Basics](https://www.craftinginterpreters.com/)
- [SSA Form](https://en.wikipedia.org/wiki/Static_single_assignment_form)