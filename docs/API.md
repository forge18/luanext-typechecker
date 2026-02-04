# API Reference

This document provides detailed documentation for the public API of the TypedLua type checker.

## TypeChecker

The main type checker entry point.

### Creating a TypeChecker

```rust
use typedlua_typechecker::{TypeChecker, TypeCheckError};
use typedlua_parser::string_interner::StringInterner;
use std::sync::Arc;

// Create string interner for deduplicating identifiers
let interner = StringInterner::new();
let common = interner.common();

// Create a diagnostic handler (see Diagnostics section)
let handler: Arc<dyn DiagnosticHandler> = Arc::new(DefaultDiagnosticHandler::new());

// Create type checker (stdlib not loaded yet)
let checker = TypeChecker::new(handler, &interner, &common);
```

### Methods

#### `new(diagnostic_handler, interner, common) -> Self`

Creates a new TypeChecker without loading the standard library.

**Arguments:**

- `diagnostic_handler: Arc<dyn DiagnosticHandler>` - Handler for errors/warnings
- `interner: &'a StringInterner` - String interner for identifier deduplication
- `common: &'a CommonIdentifiers` - Common identifier constants

**Returns:** A new `TypeChecker` instance

**Example:**

```rust
let handler = Arc::new(DefaultDiagnosticHandler::new());
let interner = StringInterner::new();
let common = interner.common();
let checker = TypeChecker::new(handler, &interner, &common);
```

---

#### `with_stdlib() -> Result<Self, String>`

Creates a TypeChecker with the standard library loaded.

**Returns:** `Ok(TypeChecker)` on success, `Err(String)` on failure

**Example:**

```rust
let checker = TypeChecker::new(handler, &interner, &common)
    .with_stdlib()?;
```

---

#### `new_with_stdlib(diagnostic_handler, internet, common) -> Result<Self, String>`

Convenience constructor that creates a TypeChecker with stdlib loaded.

**Arguments:**

- `diagnostic_handler: Arc<dyn DiagnosticHandler>`
- `interner: &'a StringInterner`
- `common: &'a CommonIdentifiers`

**Returns:** `Ok(TypeChecker)` or `Err(String)`

---

#### `with_options(options: CompilerOptions) -> Self`

Configures the type checker with compiler options.

**Arguments:**

- `options: CompilerOptions` - Configuration for target Lua version, etc.

**Returns:** Self for method chaining

**Example:**

```rust
let checker = TypeChecker::new(handler, &interner, &common)
    .with_options(CompilerOptions {
        target: LuaVersion::Lua54,
        strict_mode: true,
    });
```

---

#### `check_program(&mut self, program: &mut Program) -> Result<(), TypeCheckError>`

Type checks a parsed program.

**Arguments:**

- `program: &mut Program` - The AST to type check (mutated during checking)

**Returns:**

- `Ok(())` if type checking succeeds
- `Err(TypeCheckError)` with first error encountered

**Example:**

```rust
let (mut program, _) = parse(source, &interner)?;
checker.check_program(&mut program)?;
println!("Type checking passed!");
```

---

#### `check_statement(&mut self, statement: &mut Statement) -> Result<(), TypeCheckError>`

Type checks a single statement.

**Arguments:**

- `statement: &mut Statement` - The statement to check

**Returns:** `Ok(())` or `Err(TypeCheckError)`

**Note:** Most users should use `check_program` instead.

---

#### `infer_expression_type(&mut self, expr: &mut Expression) -> Result<Type, TypeCheckError>`

Infers the type of an expression.

**Arguments:**

- `expr: &mut Expression` - The expression to infer

**Returns:** The inferred `Type`

**Example:**

```rust
let expr: Expression = parse_expression("5 + 3", &interner)?;
let number_type = checker.infer_expression_type(&mut expr)?;
assert!(matches!(number_type.kind, TypeKind::Primitive(PrimitiveType::Number)));
```

---

## TypeEnvironment

Manages type definitions and lookups.

### Creating a TypeEnvironment

```rust
let env = TypeEnvironment::new();
```

### Methods

#### `register_type(&mut self, name: String, typ: Type) -> Result<(), String>`

Registers a new type in the environment.

**Arguments:**

- `name: String` - Type name
- `typ: Type` - Type definition

**Returns:** `Ok(())` or error if type already exists

**Example:**

```rust
let my_type = Type::new(
    TypeKind::Object(ObjectType { members: vec![] }),
    span,
);
env.register_type("MyType".to_string(), my_type)?;
```

---

#### `lookup_type(&self, name: &str) -> Option<&Type>`

Looks up a type by name.

**Arguments:**

- `name: &str` - Type name to look up

**Returns:** `Some(&Type)` if found, `None` otherwise

**Example:**

```rust
if let Some(user_type) = env.lookup_type("User") {
    println!("Found User type");
}
```

---

#### `is_assignable(&self, source: &Type, target: &Type) -> bool`

Checks if a source type is assignable to a target type.

**Arguments:**

- `source: &Type` - The source type
- `target: &Type` - The target type

**Returns:** `true` if source is assignable to target

**Example:**

```rust
let string_type = Type::primitive(PrimitiveType::String);
let any_type = Type::primitive(PrimitiveType::Any);
assert!(env.is_assignable(&string_type, &any_type));
```

---

#### `register_interface(&mut self, name: String, typ: Type) -> Result<(), String>`

Registers an interface type.

---

#### `get_interface(&self, name: &str) -> Option<&Type>`

Retrieves an interface type by name.

---

## SymbolTable

Tracks symbols (variables, functions, types) in scopes.

### Creating a SymbolTable

```rust
let table = SymbolTable::new();
```

### Methods

#### `enter_scope(&mut self)`

Enters a new nested scope.

**Example:**

```rust
table.enter_scope();
let result = table.declare(my_symbol);
// ... use scope
table.exit_scope();
```

---

#### `exit_scope(&mut self)`

Exits the current scope, removing all symbols declared in it.

---

#### `declare(&mut self, symbol: Symbol) -> Result<(), String>`

Declares a new symbol in the current scope.

**Arguments:**

- `symbol: Symbol` - The symbol to declare

**Returns:** `Ok(())` or error if symbol already exists in scope

**Example:**

```rust
let symbol = Symbol::new(
    "myVariable".to_string(),
    SymbolKind::Variable,
    Type::number(),
    span,
)?;
table.declare(symbol)?;
```

---

#### `lookup(&self, name: &str) -> Option<&Symbol>`

Looks up a symbol by name, searching from innermost scope outward.

**Arguments:**

- `name: &str` - Symbol name

**Returns:** `Some(&Symbol)` if found, `None` otherwise

---

#### `all_visible_symbols(&self) -> impl Iterator<Item = (&String, &Symbol)>`

Returns all symbols visible in the current scope chain.

**Returns:** Iterator of (name, symbol) pairs

---

#### `current_scope_id(&self) -> usize`

Returns the ID of the current scope.

---

## Symbol

Represents a named entity in the program.

### Creating a Symbol

```rust
let symbol = Symbol {
    name: "count".to_string(),
    kind: SymbolKind::Variable,
    typ: Type::number(),
    span,
    is_exported: false,
    references: Vec::new(),
};
```

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `String` | Symbol name |
| `kind` | `SymbolKind` | Kind (Variable, Const, Function, etc.) |
| `typ` | `Type` | Symbol's type |
| `span` | `Span` | Source location |
| `is_exported` | `bool` | Whether symbol is exported |
| `references` | `Vec<Span>` | All reference locations |

### SymbolKind Enum

```rust
pub enum SymbolKind {
    Variable,
    Const,
    Function,
    Parameter,
    Type,
    Interface,
    Class,
    Enum,
    Module,
    Namespace,
    Property,
    Method,
    TypeParameter,
}
```

## Dependency Injection

### DiContainer

The dependency injection container for modular component composition.

#### `new() -> Self`

Creates a new container.

```rust
let container = DiContainer::new();
```

---

#### `register<F, T>(&mut self, factory: F, lifetime: ServiceLifetime) -> &mut Self`

Registers a service with the container.

**Arguments:**

- `factory: F` - Function that creates the service
- `lifetime: ServiceLifetime` - Service lifetime

**Example:**

```rust
container.register(
    |_| MyService::new(),
    ServiceLifetime::Singleton,
);
```

---

#### `resolve<T>(&self) -> Result<T, DiError>`

Resolves a service from the container.

**Type Parameters:**

- `T` - The service type to resolve

**Returns:** `Ok(T)` or `Err(DiError)`

**Example:**

```rust
let service: Arc<dyn MyTrait> = container.resolve()?;
```

---

#### `service_count(&self) -> usize`

Returns the number of registered services.

---

#### `singleton_count(&self) -> usize`

Returns the number of cached singleton instances.

---

### ServiceLifetime Enum

```rust
pub enum ServiceLifetime {
    /// New instance created each time
    Transient,
    /// Same instance returned each time
    Singleton,
    /// New instance per scoped region
    Scoped,
}
```

## Type

Represents a TypedLua type.

### Creating Types

```rust
// Primitive types
let number = Type::primitive(PrimitiveType::Number);
let string = Type::primitive(PrimitiveType::String);
let void = Type::primitive(PrimitiveType::Void);

// Array type
let numbers = Type::array(Type::primitive(PrimitiveType::Number));

// Function type
let func = Type::function(
    vec![Type::primitive(PrimitiveType::Number)],
    Type::primitive(PrimitiveType::Number),
);

// Object type
let obj = Type::object(vec![/* members */]);
```

### Type Properties

| Property | Type | Description |
|----------|------|-------------|
| `kind` | `TypeKind` | The type variant |
| `span` | `Span` | Source location |

### TypeKind Enum

```rust
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
    TypePredicate(Box<Type>, Box<Type>),
    ThisType(Type),
}
```

## Diagnostics

### DiagnosticHandler Trait

```rust
pub trait DiagnosticHandler: Send + Sync {
    fn error(&self, span: Span, message: &str);
    fn warning(&self, span: Span, message: &str);
    fn info(&self, span: Span, message: &str);
    fn error_count(&self) -> usize;
    fn has_errors(&self) -> bool;
    fn get_diagnostics(&self) -> &[Diagnostic];
}
```

### DefaultDiagnosticHandler

The default diagnostic handler that collects errors.

```rust
use typedlua_typechecker::cli::diagnostics::DefaultDiagnosticHandler;

let handler = DefaultDiagnosticHandler::new();
```

---

### Diagnostic Struct

```rust
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub span: Span,
    pub message: String,
    pub code: Option<String>,
}

pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}
```

---

### DiagnosticBuilder

Builder pattern for creating diagnostics.

```rust
handler.error(span, "Type mismatch");
handler.warning(span, "Unused variable");

// Or with builder:
Diagnostic::error()
    .with_span(span)
    .with_message("Custom error")
    .with_code("E001")
    .report(handler);
```

## CompilerOptions

Configuration for the type checker.

```rust
pub struct CompilerOptions {
    pub target: LuaVersion,
    pub strict_mode: bool,
    pub no_stdlib: bool,
}

pub enum LuaVersion {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    Luajit,
}
```

## Error Types

### TypeCheckError

```rust
pub struct TypeCheckError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.message, self.span.line, self.span.column)
    }
}

impl std::error::Error for TypeCheckError {}
```

### Span

```rust
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub file_id: Option<FileId>,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}
```

## Module Resolver

### ModuleResolver

```rust
pub struct ModuleResolver {
    base_paths: Vec<PathBuf>,
    extensions: Vec<String>,
}

impl ModuleResolver {
    pub fn new() -> Self;
    pub fn add_base_path(&mut self, path: PathBuf);
    pub fn resolve(&self, import: &str, from: &Path) -> Result<PathBuf, ResolveError>;
}
```

### ModuleRegistry

```rust
pub struct ModuleRegistry {
    // Internal implementation
}

impl ModuleRegistry {
    pub fn register(&mut self, id: ModuleId, info: ModuleInfo);
    pub fn get(&self, id: ModuleId) -> Option<&ModuleInfo>;
    pub fn lookup_export(&self, module_id: ModuleId, name: &str) -> Option<&Symbol>;
}
```

## Utility Types API

### Generics

```rust
pub fn build_substitutions<T>(
    type_params: &[TypeParameter],
    type_args: &[Type],
    env: &TypeEnvironment,
) -> Result<FxHashMap<String, Type>, String>;

pub fn check_type_constraints(
    substitutions: &FxHashMap<String, Type>,
    type_params: &[TypeParameter],
    env: &TypeEnvironment,
) -> Result<(), String>;

pub fn instantiate_function_declaration(
    decl: &FunctionDeclaration,
    substitutions: &FxHashMap<String, Type>,
) -> FunctionDeclaration;

pub fn instantiate_type(
    typ: &Type,
    type_params: &[TypeParameter],
    type_args: &[Type],
) -> Result<Type, String>;
```

### Utility Types

```rust
pub fn apply_utility_type(
    type_name: &str,
    type_args: &[Type],
    env: &TypeEnvironment,
    span: Span,
) -> Result<Type, String>;

pub fn evaluate_keyof(typ: &Type, env: &TypeEnvironment) -> Result<Type, String>;

pub fn evaluate_mapped_type(
    source: &Type,
    mapping: &MappedType,
    env: &TypeEnvironment,
) -> Result<Type, String>;

pub fn evaluate_conditional_type(
    condition: &ConditionalType,
    env: &TypeEnvironment,
) -> Result<Type, String>;
```

## Narrowing API

### NarrowingContext

```rust
pub struct NarrowingContext {
    variable_types: FxHashMap<String, Type>,
}

impl NarrowingContext {
    pub fn new() -> Self;
    pub fn get_type(&self, name: &str) -> Option<&Type>;
    pub fn narrow(&mut self, name: &str, typ: Type);
}
```

### Narrowing Functions

```rust
pub fn narrow_type_from_condition(
    condition: &Expression,
    context: &NarrowingContext,
    variable_types: &FxHashMap<String, Type>,
    interner: &StringInterner,
) -> NarrowingContext;
```
