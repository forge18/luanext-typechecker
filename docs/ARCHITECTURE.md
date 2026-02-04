# Architecture

This document describes the high-level architecture and key algorithms of the TypedLua type checker.

## Overview

The type checker follows a **multi-pass compilation model** that processes TypedLua source code through several phases. It uses the **visitor pattern** extensively to traverse the AST and perform type checking operations.

```text
Source Code
    ↓
[Parser] → AST
    ↓
[Declaration Phase] → Symbol Table Setup
    ↓
[Type Inference] → Type Resolution
    ↓
[Validation] → Type Checking
    ↓
[Diagnostics] → Errors/Warnings
```

## Compilation Phases

### Phase 1: Declaration

The declaration phase processes all top-level declarations before type checking function bodies. This enables:

- **Function hoisting**: Functions can be called before they appear in source order
- **Forward references**: Types and classes can be used before their full definition
- **Symbol registration**: All symbols are registered in the symbol table

```rust
// First pass: Register function signatures
for statement in program.statements.iter() {
    if let Statement::Function(func_decl) = statement {
        self.register_function_signature(func_decl)?;
    }
}
```

### Phase 2: Type Inference & Checking

The second pass performs actual type checking:

- **Expression type inference**: Determine the type of each expression
- **Statement validation**: Verify type correctness of statements
- **Type compatibility**: Check assignments, returns, parameters

```rust
// Second pass: Check all statements
for statement in program.statements.iter_mut() {
    self.check_statement(statement)?;
}
```

### Phase 3: Validation

After type inference, validation checks complex relationships:

- **Class inheritance**: Verify `extends` relationships are valid
- **Interface implementation**: Ensure classes implement all interface members
- **Access control**: Check visibility modifiers
- **Circular dependencies**: Detect circular type references

## Visitor Pattern Architecture

The type checker uses three main visitors to traverse the AST:

### TypeInferenceVisitor

Handles type inference for expressions:

- **Literal types**: Extract types from literal values
- **Binary operations**: Determine result type from operands
- **Function calls**: Infer return type from function signature
- **Control flow**: Track types through if/else branches
- **Pattern matching**: Narrow types in match expressions

```rust
impl TypeInferenceVisitor {
    fn infer_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => self.infer_literal(lit),
            Expression::BinaryOp(binop) => self.infer_binary_op(binop),
            Expression::FunctionCall(call) => self.infer_function_call(call),
            // ...
        }
    }
}
```

### NarrowingVisitor

Performs control flow-based type refinement:

- **Condition analysis**: Analyze `if` conditions to narrow variable types
- **Truthiness narrowing**: Narrow `nil` vs non-nil types
- **Equality narrowing**: Narrow through `==` and `!=` checks
- **Exhaustiveness**: Ensure all cases are covered in pattern matching

```rust
impl NarrowingVisitor {
    fn narrow_from_condition(
        &self,
        condition: &Expression,
        context: NarrowingContext,
    ) -> NarrowingContext {
        match condition {
            Expression::BinaryOp(op) if op.operator.is_equality() => {
                // Narrow based on equality check
            }
            Expression::UnaryOp(op) if op.operator.is_not() => {
                // Narrow for nil checks
            }
            _ => context,
        }
    }
}
```

### AccessControlVisitor

Manages member visibility:

- **Private members**: Only accessible within declaring class
- **Protected members**: Accessible within declaring class and subclasses
- **Public members**: Accessible everywhere
- **Getter/setter**: Validate accessor pairs

## Type System Design

### Type Representation

Types are represented using a `Type` struct with a `TypeKind` enum:

```rust
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

pub enum TypeKind {
    Primitive(PrimitiveType),
    Reference(TypeReference),
    Object(ObjectType),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Function(FunctionType),
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    TypeParameter(String),
    TypePredicate(Type, Type),
    // ...
}
```

### TypeEnvironment

The `TypeEnvironment` manages type definitions and lookups:

- **Type registry**: Stores all defined types (interfaces, classes, aliases)
- **Type parameters**: Manages generic type parameters in scope
- **Utility types**: Provides built-in utility type transformations

```rust
impl TypeEnvironment {
    pub fn register_type(&mut self, name: String, typ: Type) -> Result<()> {
        // Add type to registry
        self.types.insert(name, typ);
        Ok(())
    }

    pub fn lookup_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
}
```

### Type Compatibility

Type compatibility uses **structural typing** (Types are compatible if they have compatible shapes):

```rust
impl TypeCompatibility {
    pub fn is_assignable(source: &Type, target: &Type) -> bool {
        match (source.kind, target.kind) {
            // Primitive types: exact match
            (Primitive(a), Primitive(b)) => a == b,

            // Object types: structural compatibility
            (Object(obj1), Object(obj2)) => {
                obj1.is_structurally_compatible(obj2)
            }

            // Functions: parameter and return type compatibility
            (Function(func1), Function(func2)) => {
                func1.is_assignable_to(func2)
            }

            // Arrays: element type compatibility
            (Array(elem1), Array(elem2)) => {
                Self::is_assignable(elem1, elem2)
            }

            // ...
        }
    }
}
```

## Module Resolution

### ModuleRegistry

Tracks all modules in a compilation:

```rust
pub struct ModuleRegistry {
    modules: FxHashMap<ModuleId, ModuleInfo>,
    next_id: AtomicUsize,
}

struct ModuleInfo {
    id: ModuleId,
    path: PathBuf,
    exports: FxHashMap<String, Symbol>,
    dependencies: Vec<ModuleId>,
}
```

### DependencyGraph

Computes compilation order based on dependencies:

```rust
impl DependencyGraph {
    pub fn build_order(&self) -> Result<Vec<ModuleId>, CycleError> {
        // Topological sort of modules
        // Ensures dependencies are compiled before dependents
    }
}
```

### ModuleResolver

Resolves import paths to module files:

```rust
pub struct ModuleResolver {
    base_paths: Vec<PathBuf>,
    extensions: Vec<String>,
}

impl ModuleResolver {
    pub fn resolve(&self, import: &str, from: &Path) -> Result<PathBuf, ResolveError> {
        // Try different resolution strategies:
        // 1. Relative imports (./foo)
        // 2. Non-relative imports (foo → node_modules)
        // 3. Index files (foo/index.lua)
    }
}
```

## Control Flow Analysis

### NarrowingContext

Tracks the type of each variable as it may be narrowed by conditions:

```rust
pub struct NarrowingContext {
    // Maps variable name to possible narrowed types
    variable_types: FxHashMap<String, Type>,
}

impl NarrowingContext {
    pub fn narrow_variable(&mut self, name: &str, typ: Type) {
        // Update variable's possible type
        self.variable_types.insert(name.to_string(), typ);
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.variable_types.get(name)
    }
}
```

### Narrowing Rules

1. **Nil Check Narrowing**:

   ```lua
   if x ~= nil then
       -- x is narrowed to non-nil type
   end
   ```

2. **Type Predicate Narrowing**:

   ```lua
   function isNumber(x: any): x is number
       return type(x) == "number"
   end

   if isNumber(x) then
       -- x is narrowed to number
   end
   ```

3. **Union Narrowing**:

   ```lua
   type Result = Success | Failure

   if result is Success then
       -- result is narrowed to Success
   end
   ```

## Error Reporting

### DiagnosticHandler

The `DiagnosticHandler` trait defines how errors are reported:

```rust
pub trait DiagnosticHandler: Send + Sync {
    fn error(&self, span: Span, message: &str);
    fn warning(&self, span: Span, message: &str);
    fn info(&self, span: Span, message: &str);
    fn error_count(&self) -> usize;
}
```

### Error Types

- **Type mismatch**: Incompatible types in assignment/operation
- **Unknown type**: Referenced type not defined
- **Unknown member**: Property/method not found on type
- **Access violation**: Private/protected member accessed incorrectly
- **Missing return**: Non-void function missing return on some path
- **Unreachable code**: Code that can never execute
- **Circular dependency**: Circular type reference detected

## Performance Considerations

### String Interning

The parser uses a `StringInterner` to deduplicate string storage:

```rust
let name_id = interner.intern("myFunction");
// Same string returns same ID
assert_eq!(interner.intern("myFunction"), name_id);
```

### Hash Collections

Uses `FxHashMap` (Rust's fast hash map) for performance-critical collections:

- Symbol tables
- Type registries
- Module dependencies

### Incremental Checking

Modules can be type-checked incrementally:

- Track file modification times
- Only recheck changed modules
- Reuse symbol tables from previous runs

## Extension Points

### Custom Diagnostic Handlers

Implement `DiagnosticHandler` to integrate with IDEs or build tools:

```rust
struct LspDiagnosticHandler {
    // Maps errors to LSP diagnostic protocol
}

impl DiagnosticHandler for LspDiagnosticHandler {
    fn error(&self, span: Span, message: &str) {
        // Send to LSP client
    }
}
```

### Custom Type Checkers

Extend the type system with custom type rules by implementing visitor traits:

```rust
pub trait TypeCheckVisitor {
    fn visit_type(&mut self, type_ref: &TypeReference) -> Result<Type>;
}
```
