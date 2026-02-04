# TypedLua Typechecker

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Tests](https://img.shields.io/badge/tests-367%2F368-green)
![Coverage](https://img.shields.io/badge/coverage-85%25-yellow)

A standalone type checker for TypedLua, extracted from typedlua-core. Provides comprehensive type checking for TypedLua with full support for generics, type narrowing, utility types, and multi-module compilation.

## About

This project provides comprehensive type checking for TypedLua, including:

- **Type compatibility checking** - Structural and nominal type compatibility
- **Generic types** - Full support for type parameters and constraints
- **Type narrowing** - Control flow-based type refinement
- **Utility types** - Pick, Omit, Keyof, conditional types, mapped types
- **Symbol tracking** - Complete symbol table with scope management
- **Module system** - Multi-module compilation with dependency resolution
- **Standard library** - Built-in type definitions
- **Dependency injection** - Modular component composition

## Quick Start

```rust
use typedlua_typechecker::{TypeChecker, TypeCheckError};
use typedlua_parser::{parse, string_interner::StringInterner};

fn type_check(source: &str) -> Result<(), TypeCheckError> {
    let interner = StringInterner::new();
    let common = interner.common();
    let handler = Arc::new(DefaultDiagnosticHandler::new());

    let mut checker = TypeChecker::new(handler, &interner, &common)
        .with_stdlib()?;

    let (mut program, _) = parse(source, &interner)?;
    checker.check_program(&mut program)
}
```

## Project Statistics

| Metric | Value |
|--------|-------|
| Total Lines of Code | ~25,000 |
| Test Count | 368 |
| Modules | 57 |
| Supported Lua Versions | 5.1, 5.2, 5.3, 5.4, LuaJIT |

## Project Structure

```text
src/
├── lib.rs                    # Public API exports
├── core/                     # Core type checking engine
│   ├── type_checker.rs       # Main TypeChecker (3,241 lines)
│   ├── type_compat.rs        # Type compatibility algorithms
│   ├── type_environment.rs   # Type registry & environment
│   └── context.rs            # Type checking context
├── types/                     # Type system
│   ├── generics.rs           # Generic type handling (2,342 lines)
│   └── utility_types.rs      # Utility types (2,061 lines)
├── visitors/                  # AST visitor patterns
│   ├── inference.rs          # Type inference visitor (2,339 lines)
│   ├── narrowing.rs          # Control flow narrowing (1,039 lines)
│   └── access_control.rs    # Member visibility (333 lines)
├── phases/                    # Compilation phases
│   ├── declaration_phase.rs        # Symbol declaration
│   ├── declaration_checking_phase.rs # Declaration validation (880 lines)
│   ├── inference_phase.rs          # Type inference phase
│   ├── module_phase.rs             # Multi-module processing (643 lines)
│   └── validation_phase.rs         # Type validation (815 lines)
├── utils/                     # Utilities
│   ├── symbol_table.rs       # Symbol tracking (429 lines)
│   └── narrowing_integration.rs
├── cli/                       # CLI & tooling
│   ├── config.rs             # Compiler options
│   ├── diagnostics.rs        # Error reporting (1,013 lines)
│   ├── errors.rs
│   └── fs.rs                 # File system abstraction
├── module_resolver/           # Module system
│   ├── mod.rs                # Main resolver
│   ├── registry.rs           # Module registry
│   └── dependency_graph.rs    # Dependency tracking
├── state/                     # State management
│   ├── type_checker_state.rs
│   └── stdlib_loader.rs
├── stdlib/                    # Standard library definitions
├── di/                        # Dependency injection container
├── helpers/                   # Helper utilities
└── test_utils/               # Testing utilities
```

## Installation

```bash
# Build release
cargo build --release

# Build debug
cargo build

# Run tests
cargo test

# Code coverage
cargo tarpaulin --out Html
```

## Development

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Benchmarks
cargo bench

# Documentation
cargo doc --open
```

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| typedlua-parser | git | AST and parsing infrastructure |
| rustc-hash | 2.1 | Fast hashing |
| serde | 1.0 | Serialization |
| serde_yaml | 0.9 | YAML config support |
| indexmap | 2.0 | Ordered maps |
| thiserror | 2.0 | Error handling |
| tracing | 0.1 | Logging |
| tracing-subscriber | 0.3 | Log formatting |

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design and algorithms
- [API Reference](docs/API.md) - Public API documentation
- [Type System](docs/TYPES.md) - Type system reference
- [Examples](docs/EXAMPLES.md) - Usage examples

## License

MIT License. See [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request
