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

### Cross-File Type Resolution

LuaNext implements sophisticated cross-file type resolution enabling TypeScript-style imports across modules. This system was built in phases from February 2026 and provides full support for lazy type resolution, circular type dependencies, and re-exports.

#### Overview

The cross-file type system enables:
- **Lazy type resolution**: Types are resolved on-demand when imported
- **Type-only imports**: Import types without generating runtime code
- **Circular type dependencies**: Mutually referential types across files
- **Re-exports**: Export symbols from other modules
- **Deep re-export chains**: Follow transitive re-exports to original definitions

#### Architecture

```text
Module A (user.luax)              Module B (post.luax)
┌─────────────────────┐          ┌─────────────────────┐
│ import type { Post }│─────────▶│ export interface    │
│   from './post'     │          │   Post { ... }      │
│                     │          │                     │
│ interface User {    │          │ export interface    │
│   posts: Post[]     │◀─────────│   Author extends    │
│ }                   │  type    │   User { ... }      │
│                     │  only    │                     │
│ export { User }     │          └─────────────────────┘
└─────────────────────┘
         │
         │ value (circular dependency ERROR)
         ▼
    [Error: Cannot have circular value dependencies]
```

#### Core Components

##### 1. Lazy Type Resolution (Phase 1 - 2026-02-09)

When importing a symbol, types are resolved on-demand via `LazyTypeCheckCallback`:

```rust
pub trait LazyTypeCheckCallback: Send + Sync {
    fn type_check_module(&self, module_id: ModuleId) -> Result<(), ModuleError>;
}

// In resolve_import_type():
fn resolve_import_type<'arena>(
    &self,
    module_path: &str,
    symbol_name: &str,
    is_type_only_import: bool,
    lazy_callback: Option<&dyn LazyTypeCheckCallback>,
) -> Result<Type<'arena>, ModuleError> {
    let module_id = self.registry.get_module_id(module_path)?;

    // Lazy type-check the dependency if needed
    if let Some(callback) = lazy_callback {
        if !self.registry.is_module_checked(module_id) {
            callback.type_check_module(module_id)?;
        }
    }

    // Get the exported type
    let exports = self.registry.get_exports(module_id)?;
    let symbol = exports.get(symbol_name)
        .ok_or(ModuleError::ExportNotFound)?;

    // Validate compatibility
    self.validate_import_export_compatibility(
        symbol,
        is_type_only_import
    )?;

    Ok(symbol.typ.clone())
}
```

**Circuit Breaker**: Prevents infinite loops in circular type resolution:
- `MAX_LAZY_DEPTH = 10` prevents unbounded recursion
- `ModuleRegistry::type_check_depth` tracks depth per module
- Returns `Unknown` type gracefully when depth exceeded

**Type Validation**: `validate_import_export_compatibility()` enforces:
- Runtime imports (value imports) cannot reference type-only exports (TypeAlias, Interface)
- Type-only imports can reference any export
- Clear error messages with module and export information

##### 2. Circular Dependency Handling (Phase 3 - 2026-02-09)

LuaNext distinguishes between **type-only** and **value** dependencies:

```rust
pub enum EdgeKind {
    TypeOnly,  // import type { Foo } - allowed to be circular
    Value,     // import { foo } - NOT allowed to be circular
}

pub struct DependencyGraph {
    edges: FxHashMap<ModuleId, Vec<(ModuleId, EdgeKind)>>,
}

impl DependencyGraph {
    pub fn build_order(&self) -> Result<Vec<ModuleId>, CycleError> {
        // Topological sort considering only Value edges
        // Type-only cycles are allowed and don't affect build order
        self.topological_sort_with_filter(|edge| {
            matches!(edge, EdgeKind::Value)
        })
    }

    pub fn detect_value_cycles(&self) -> Option<Vec<ModuleId>> {
        // Only report cycles that involve Value edges
        // Type-only cycles are perfectly valid
    }
}
```

**Why This Works**:
- Type-only imports are erased at runtime (no Lua code generated)
- Circular type references resolve during type-checking, not runtime
- Value imports require actual module execution, so cycles would deadlock

**Error Messages**: When a value cycle is detected:
```
Error: Circular value dependency detected:
  module_a.luax → module_b.luax → module_a.luax

This creates a runtime deadlock. Consider using type-only imports:

Before:
  import { Foo } from './module_b'  // Value import

After:
  import type { Foo } from './module_b'  // Type-only import
```

##### 3. Re-Export Resolution (Phase 4 - 2026-02-09)

Re-exports enable modules to re-export symbols from other modules:

```rust
// In module_phase.rs
fn resolve_re_export<'arena>(
    &self,
    module_id: ModuleId,
    symbol_name: &str,
    lazy_callback: Option<&dyn LazyTypeCheckCallback>,
    visited: &mut HashSet<(ModuleId, String)>,
    depth: usize,
) -> Result<ExportedSymbol, ModuleError> {
    const MAX_REEXPORT_DEPTH: usize = 10;

    // Prevent infinite loops
    if depth > MAX_REEXPORT_DEPTH {
        return Err(ModuleError::ReExportChainTooDeep);
    }

    // Prevent circular re-exports
    if !visited.insert((module_id, symbol_name.to_string())) {
        return Err(ModuleError::CircularReExport);
    }

    let exports = self.registry.get_exports(module_id)?;
    let export = exports.get(symbol_name)?;

    match export.kind {
        ExportKind::Local => Ok(export),
        ExportKind::ReExport { source_module, source_name } => {
            // Recursively follow the chain
            self.resolve_re_export(
                source_module,
                &source_name,
                lazy_callback,
                visited,
                depth + 1,
            )
        }
    }
}
```

**Re-Export Syntax Support**:
```typescript
// Named re-exports
export { Foo, Bar } from './other'

// Type-only re-exports
export type { Baz } from './types'

// Export all (wildcard)
export * from './utils'

// Type-only export all
export type * from './interfaces'
```

**Codegen Strategy**:
```lua
-- export { Foo } from './other'
local _other = require('./other')
exports.Foo = _other.Foo

-- export * from './utils'
local _utils = require('./utils')
for k, v in pairs(_utils) do
  exports[k] = v
end

-- export type * from './interfaces'
-- (generates no code - types are erased)
```

##### 4. LSP Integration (Phase 2 - 2026-02-09)

IDE features distinguish and track type-only imports:

**Completion Provider**:
- Shows `(type-only import)` suffix for type-only imported symbols
- Via `get_type_only_imports()` AST scanner

**Hover Provider**:
- Displays `*Imported as type-only*` note in markdown
- Shows original module for re-exported symbols
- Via `is_type_only_import()` AST checker

**Symbol Index**:
```rust
pub struct ExportInfo {
    pub name: String,
    pub range: Range,
    pub is_type_only: bool,      // New field
    pub is_reexport: bool,        // New field
    pub reexport_source: Option<String>,  // New field
    pub original_module: Option<String>,  // New field
}

pub struct ImportInfo {
    pub name: String,
    pub range: Range,
    pub is_type_only: bool,      // New field
    pub source_module: String,
}
```

#### Usage Examples

See the **Cross-File Type Examples** section below for complete working examples of:
- Basic cross-file imports
- Type-only imports
- Circular type dependencies
- Re-exports (single-level and multi-level)
- Export all (`export *`)

#### Error Handling

**ModuleError Types**:
```rust
pub enum ModuleError {
    // Lazy resolution errors
    TypeCheckInProgress { module: String },
    ExportNotFound { module: String, symbol: String },
    ExportTypeMismatch {
        module: String,
        symbol: String,
        expected: String,
        actual: String
    },
    RuntimeImportOfTypeOnly {
        module: String,
        symbol: String
    },

    // Circular dependency errors
    CircularValueDependency { cycle: Vec<String> },

    // Re-export errors
    CircularReExport { chain: Vec<String> },
    ReExportChainTooDeep { depth: usize },
    TypeOnlyReExportAsValue {
        module: String,
        symbol: String
    },
}
```

#### Performance Optimizations (Phase 4.5 - 2026-02-09)

**Re-Export Chain Caching**:
```rust
pub struct SymbolIndex {
    // Cache resolved re-export chains
    reexport_cache: RefCell<HashMap<(ModuleId, String), ExportInfo>>,
}
```

**Tree-Shaking for `export *`**:
```rust
// Only copy reachable exports when reachable_exports is set
if let Some(reachable) = &self.reachable_exports {
    if reachable.is_empty() {
        // Skip entire export * if nothing is reachable
        return Ok(());
    }

    // Generate selective copy
    for export_name in reachable {
        lua.push_str(&format!(
            "exports[{:?}] = _mod[{:?}]\n",
            export_name, export_name
        ));
    }
}
```

### ModuleRegistry

Tracks all modules with enhanced type resolution:

```rust
pub struct ModuleRegistry {
    modules: FxHashMap<ModuleId, ModuleInfo>,
    exports: FxHashMap<ModuleId, ModuleExports>,
    // Circuit breaker for lazy type-checking
    type_check_depth: FxHashMap<ModuleId, usize>,
    next_id: AtomicUsize,
}

struct ModuleExports {
    named: FxHashMap<String, ExportedSymbol>,
    default: Option<ExportedSymbol>,
}

struct ExportedSymbol {
    symbol: Symbol<'static>,
    is_type_only: bool,  // Distinguishes type-only exports
}
```

**Key Methods**:

- `get_exports()`: Retrieves module exports with proper error handling
- `increment_type_check_depth()` / `decrement_type_check_depth()`: Circuit breaker
- `is_ready_for_type_checking()`: Prevents circular type-checking

### DependencyGraph

Computes compilation order based on dependencies:

```rust
impl DependencyGraph {
    pub fn build_order(&self) -> Result<Vec<ModuleId>, CycleError> {
        // Topological sort of modules
        // Ensures dependencies are compiled before dependents
        // (Future: will separate type-only and value edges for Phase 3)
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

### Type-Only Import/Export

**Import Clause Types**:

```rust
pub enum ImportClause<'arena> {
    Named(&'arena [ImportSpecifier]),           // Regular imports
    Default(Ident),                               // Default imports
    Namespace(Ident),                             // Namespace imports
    TypeOnly(&'arena [ImportSpecifier]),         // Type-only imports
    Mixed { default: Ident, named: &'arena [...] }, // Mixed imports
}
```

**Export Handling**:

- TypeAlias and Interface exports marked as `is_type_only: true`
- Codegen skips generating runtime code for type-only imports
- Type checker validates import compatibility via `is_type_only_import` parameter

## Cross-File Type Examples

This section provides complete working examples of cross-file type resolution features.

### Basic Cross-File Imports

**types.luax**:
```typescript
export interface User {
  id: number
  name: string
  email: string
}

export interface Post {
  id: number
  title: string
  author: User
}

export type UserRole = "admin" | "user" | "guest"
```

**user-service.luax**:
```typescript
import { User, UserRole } from './types'

export function createUser(name: string, email: string, role: UserRole): User {
  return {
    id: generateId(),
    name: name,
    email: email
  }
}

function generateId(): number {
  return math.random(1, 10000)
}
```

**main.luax**:
```typescript
import { createUser } from './user-service'
import type { User } from './types'

const admin: User = createUser("Alice", "alice@example.com", "admin")
print(admin.name)  // "Alice"
```

### Type-Only Imports

Type-only imports use the `import type` syntax and generate **no runtime code**:

**models.luax**:
```typescript
export interface Product {
  id: number
  name: string
  price: number
}

export interface Cart {
  items: Product[]
  total: number
}

export function createCart(): Cart {
  return { items: [], total: 0 }
}
```

**checkout.luax**:
```typescript
// Type-only import - no runtime code generated
import type { Cart, Product } from './models'

// Value import - generates require() call
import { createCart } from './models'

export function checkout(cart: Cart): void {
  print("Checking out cart with " .. #cart.items .. " items")
}

const myCart: Cart = createCart()
checkout(myCart)
```

**Generated Lua for checkout.luax**:
```lua
-- Notice: Cart and Product are NOT imported
local _models = require('./models')
local createCart = _models.createCart

local function checkout(cart)
  print("Checking out cart with " .. #cart.items .. " items")
end

local myCart = createCart()
checkout(myCart)
```

### Circular Type Dependencies (Allowed)

Circular type-only imports are **allowed** because types are erased at runtime:

**user.luax**:
```typescript
import type { Post } from './post'

export interface User {
  id: number
  name: string
  posts: Post[]  // References Post from another file
}
```

**post.luax**:
```typescript
import type { User } from './user'

export interface Post {
  id: number
  title: string
  author: User  // References User from another file - circular!
}
```

**app.luax**:
```typescript
import type { User } from './user'
import type { Post } from './post'

const user: User = {
  id: 1,
  name: "Alice",
  posts: []
}

const post: Post = {
  id: 1,
  title: "Hello World",
  author: user
}

user.posts.push(post)  // Circular data structure is fine
```

This works because:
1. `user.luax` and `post.luax` use `import type` (type-only)
2. No runtime dependency between the modules
3. Only `app.luax` creates actual values

### Circular Value Dependencies (Error)

Circular **value** imports are **not allowed** because they create runtime deadlocks:

**bad-example-a.luax**:
```typescript
import { funcB } from './bad-example-b'  // Value import

export function funcA() {
  return funcB() + 1
}
```

**bad-example-b.luax**:
```typescript
import { funcA } from './bad-example-a'  // Value import - CIRCULAR!

export function funcB() {
  return funcA() + 2
}
```

**Error**:
```
Error: Circular value dependency detected:
  bad-example-a.luax → bad-example-b.luax → bad-example-a.luax

This creates a runtime deadlock because both modules require each
other to execute before they can initialize.

Suggestion: If you only need types, use type-only imports:
  import type { Foo } from './module'
```

**Fix**: Use forward declarations or dependency injection to break the cycle:

**fixed-a.luax**:
```typescript
export interface ICalculator {
  calculate(): number
}

export function funcA(calc: ICalculator): number {
  return calc.calculate() + 1
}
```

**fixed-b.luax**:
```typescript
import type { ICalculator } from './fixed-a'
import { funcA } from './fixed-a'

export class Calculator implements ICalculator {
  calculate(): number {
    return 42
  }
}

export function funcB(): number {
  const calc = new Calculator()
  return funcA(calc) + 2
}
```

### Re-Exports (Single Level)

Re-exports allow modules to re-export symbols from dependencies:

**database/user-model.luax**:
```typescript
export interface User {
  id: number
  name: string
}

export function findUser(id: number): User | null {
  // ... database query
  return null
}
```

**database/post-model.luax**:
```typescript
export interface Post {
  id: number
  title: string
}

export function findPost(id: number): Post | null {
  // ... database query
  return null
}
```

**database/index.luax** (Barrel export):
```typescript
// Re-export everything from sub-modules
export { User, findUser } from './user-model'
export { Post, findPost } from './post-model'

// Now consumers can import from one place
```

**app.luax**:
```typescript
// Import from the barrel instead of individual files
import { User, Post, findUser, findPost } from './database'

const user: User | null = findUser(1)
const post: Post | null = findPost(1)
```

**Generated Lua for database/index.luax**:
```lua
local _user_model = require('./user-model')
local _post_model = require('./post-model')

local exports = {}
exports.User = _user_model.User
exports.findUser = _user_model.findUser
exports.Post = _post_model.Post
exports.findPost = _post_model.findPost

return exports
```

### Re-Exports (Multi-Level Chain)

Re-exports can be chained across multiple modules:

**core/types.luax**:
```typescript
export interface Config {
  apiUrl: string
  timeout: number
}
```

**core/index.luax**:
```typescript
export { Config } from './types'
```

**lib/index.luax**:
```typescript
export { Config } from '../core'
```

**app.luax**:
```typescript
import type { Config } from './lib'

const config: Config = {
  apiUrl: "https://api.example.com",
  timeout: 5000
}
```

The type checker follows the chain:
1. `app.luax` imports `Config` from `./lib`
2. `./lib/index.luax` re-exports `Config` from `../core`
3. `../core/index.luax` re-exports `Config` from `./types`
4. `./core/types.luax` defines `Config`

Protection: Maximum re-export depth is 10 to prevent infinite loops.

### Export All (Wildcard)

Use `export *` to re-export all symbols from a module:

**utils/string-utils.luax**:
```typescript
export function capitalize(s: string): string {
  return s:sub(1, 1):upper() .. s:sub(2)
}

export function trim(s: string): string {
  return s:match("^%s*(.-)%s*$")
}
```

**utils/number-utils.luax**:
```typescript
export function clamp(x: number, min: number, max: number): number {
  return math.max(min, math.min(max, x))
}

export function round(x: number): number {
  return math.floor(x + 0.5)
}
```

**utils/index.luax**:
```typescript
// Export all from both modules
export * from './string-utils'
export * from './number-utils'
```

**app.luax**:
```typescript
import { capitalize, trim, clamp, round } from './utils'

print(capitalize("hello"))  // "Hello"
print(clamp(15, 0, 10))     // 10
```

**Generated Lua for utils/index.luax**:
```lua
local _string_utils = require('./string-utils')
local _number_utils = require('./number-utils')

local exports = {}

-- Copy all from string-utils
for k, v in pairs(_string_utils) do
  exports[k] = v
end

-- Copy all from number-utils
for k, v in pairs(_number_utils) do
  exports[k] = v
end

return exports
```

### Type-Only Export All

Use `export type *` to re-export only types (interfaces, type aliases):

**api/request-types.luax**:
```typescript
export interface GetRequest {
  method: "GET"
  url: string
}

export interface PostRequest {
  method: "POST"
  url: string
  body: string
}

export function createRequest(url: string): GetRequest {
  return { method: "GET", url: url }
}
```

**api/index.luax**:
```typescript
// Re-export only the types, not the function
export type * from './request-types'

// Re-export the function separately
export { createRequest } from './request-types'
```

**app.luax**:
```typescript
import type { GetRequest, PostRequest } from './api'
import { createRequest } from './api'

const req: GetRequest = createRequest("https://example.com")
```

**Generated Lua for api/index.luax**:
```lua
-- export type * generates NO code (types are erased)

local _request_types = require('./request-types')

local exports = {}
exports.createRequest = _request_types.createRequest

return exports
```

### Forward Type Declarations

Forward declarations enable mutual references within a single file:

**models.luax**:
```typescript
// Forward declarations (empty bodies)
interface Node {}
interface Edge {}

// Full definitions can now reference each other
interface Node {
  id: number
  edges: Edge[]
}

interface Edge {
  from: Node
  to: Node
  weight: number
}

export { Node, Edge }
```

This works because:
1. Parser detects empty interface bodies as forward declarations
2. Type checker registers the names before processing bodies
3. Full definitions can then reference each other

**Limitations**: Forward declarations cannot have:
- Type parameters: `interface Foo<T> {}` (not allowed)
- Extends clause: `interface Foo extends Bar {}` (not allowed)
- Class modifiers: `abstract class Foo {}` (not allowed)

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
