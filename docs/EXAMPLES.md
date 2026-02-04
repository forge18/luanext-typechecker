# Examples

This document provides practical examples of using the TypedLua type checker.

## Basic Usage

### Simple Type Checking

```rust
use typedlua_typechecker::{TypeChecker, TypeCheckError};
use typedlua_parser::{parse, string_interner::StringInterner};
use std::sync::Arc;

fn type_check_source(source: &str) -> Result<(), TypeCheckError> {
    let interner = StringInterner::new();
    let common = interner.common();

    // Create diagnostic handler that prints errors
    let handler: Arc<dyn typedlua_typechecker::cli::diagnostics::DiagnosticHandler> =
        Arc::new(typedlua_typechecker::cli::diagnostics::DefaultDiagnosticHandler::new());

    // Create and configure type checker
    let mut checker = TypeChecker::new(handler, &interner, &common)
        .with_stdlib()?;

    // Parse source code
    let (mut program, _) = parse(source, &interner)?;

    // Type check
    checker.check_program(&mut program)
}

// Usage
fn main() {
    let source = r#"
        local x: number = 42
        local y: string = "hello"
        local z = x + 10
    "#;

    match type_check_source(source) {
        Ok(()) => println!("Type checking passed!"),
        Err(e) => println!("Error: {}", e),
    }
}
```

### With Custom Diagnostic Handler

```rust
use typedlua_typechecker::cli::diagnostics::{Diagnostic, DiagnosticHandler, DiagnosticSeverity};
use typedlua_parser::span::Span;

struct CollectingDiagnosticHandler {
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
}

impl CollectingDiagnosticHandler {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl DiagnosticHandler for CollectingDiagnosticHandler {
    fn error(&self, span: Span, message: &str) {
        self.errors.push(Diagnostic::new(
            DiagnosticSeverity::Error,
            span,
            message.to_string(),
        ));
    }

    fn warning(&self, span: Span, message: &str) {
        self.warnings.push(Diagnostic::new(
            DiagnosticSeverity::Warning,
            span,
            message.to_string(),
        ));
    }

    fn info(&self, _span: Span, _message: &str) {}

    fn error_count(&self) -> usize {
        self.errors.len()
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.errors
    }
}

// Usage
let handler = Arc::new(CollectingDiagnosticHandler::new());
let mut checker = TypeChecker::new(handler.clone(), &interner, &common);

// After type checking
if handler.has_errors() {
    for diag in handler.get_diagnostics() {
        eprintln!("Error at {}:{}: {}", diag.span.line, diag.span.column, diag.message);
    }
}
```

## Generics

### Generic Function

```lua
-- generics.lua
function identity<T>(value: T): T
    return value
end

local num = identity(42)            -- inferred as number
local str = identity("hello")        -- inferred as string
local bool = identity<boolean>(true) -- explicit type argument
```

### Generic Class

```lua
-- container.lua
class Container<T> {
    private items: T[]

    constructor()
        self.items = {}
    end

    add(item: T): void
        table.insert(self.items, item)
    end

    get(index: number): T | nil
        return self.items[index]
    end

    size(): number
        return #self.items
    end
}

-- Usage
local numbers = Container<number>()
numbers.add(1)
numbers.add(2)
numbers.add(3)

for i = 1, numbers.size() do
    print(numbers.get(i))  -- number | nil
end
```

### Type Constraints

```lua
-- constrained.lua
interface Loggable {
    __tostring(): string
}

function log<T extends Loggable>(value: T): void
    print(tostring(value))
end

class User {
    name: string

    constructor(name: string)
        self.name = name
    end

    __tostring(): string
        return "User(" .. self.name .. ")"
    end
}

local user = User("Alice")
log(user)  -- OK: User implements Loggable
```

## Utility Types

### Pick and Omit

```lua
-- utility_types.lua
interface User {
    id: number
    username: string
    email: string
    password_hash: string
    created_at: string
}

-- PublicUser omits sensitive fields
type PublicUser = Omit<User, "password_hash">

-- Get only id and username
type UserSummary = Pick<User, "id" | "username">

local user: User = {
    id = 1,
    username = "alice",
    email = "alice@example.com",
    password_hash = "xxx",
    created_at = "2024-01-01"
}

local public: PublicUser = user  -- OK, password_hash excluded
```

### Record and Mapped Types

```lua
-- mapping.lua
type StringMap<T> = Record<string, T>

local scores: StringMap<number> = {
    alice = 100,
    bob = 95,
    charlie = 88
}

-- Using Partial
type OptionalUser = Partial<User>
local partial: OptionalUser = { name = "Alice" }  -- All fields optional
```

### Keyof and Conditional Types

```lua
-- keyof_example.lua
interface Config {
    host: string
    port: number
    ssl: boolean
    timeout: number
}

-- Get all keys as union type
type ConfigKey = keyof Config  -- "host" | "port" | "ssl" | "timeout"

-- Function that gets a config value by key
function getConfig<K extends keyof Config>(key: K): Config[K]
    -- Implementation
end

local host = getConfig("host")  -- string
local port = getConfig("port")  -- number
```

### Conditional Types

```lua
-- conditional.lua
type ToString<T> = T extends number ? string
                : T extends boolean ? "true" | "false"
                : string

local a: ToString<number> = "42"    -- number -> string
local b: ToString<boolean> = "true" -- boolean -> "true" | "false"
local c: ToString<table> = "table"  -- other -> string

-- Distributive conditional
type Flatten<T> = T extends any[] ? T[number] : T

type R = Flatten<number | string[]>
-- = Flatten<number> | Flatten<string[]>
-- = number | string
```

## Classes and Inheritance

### Basic Class

```lua
-- class_example.lua
class Animal {
    name: string

    constructor(name: string)
        self.name = name
    end

    speak(): string
        return "..."
    end
}

class Dog extends Animal {
    breed: string

    constructor(name: string, breed: string)
        super(name)
        self.breed = breed
    end

    override speak(): string
        return self.name .. " barks!"
    end
}

local dog = Dog("Rex", "German Shepherd")
print(dog:speak())  -- "Rex barks!"
```

### Access Control

```lua
-- access_control.lua
class Counter {
    private _count: number = 0
    protected _history: number[] = {}

    public increment(): void
        self._count = self._count + 1
        table.insert(self._history, self._count)
    end

    public getCount(): number
        return self._count
    end

    public getHistory(): number[]
        return self._history
    end
}

class LimitedCounter extends Counter {
    private _limit: number

    constructor(limit: number)
        super()
        self._limit = limit
    end

    override increment(): void
        if self.getCount() < self._limit then
            super.increment()
        end
    end
}

local counter = Counter()
counter.increment()
counter.increment()
print(counter:getCount())  -- 2
-- counter._count  -- Error: private
```

## Interfaces

### Interface with Methods

```lua
-- interface_example.lua
interface Repository<T> {
    findById(id: number): T | nil
    findAll(): T[]
    save(entity: T): boolean
    delete(id: number): boolean
}

class UserRepository implements Repository<User> {
    private users: { [number]: User } = {}

    findById(id: number): User | nil
        return self.users[id]
    end

    findAll(): User[]
        local all = {}
        for _, user in pairs(self.users) do
            table.insert(all, user)
        end
        return all
    end

    save(user: User): boolean
        self.users[user.id] = user
        return true
    end

    delete(id: number): boolean
        if self.users[id] then
            self.users[id] = nil
            return true
        end
        return false
    end
}
```

## Module System

### Import and Export

```lua
-- models/user.lua
export interface User {
    id: number
    name: string
    email: string
end

export class UserService {
    private users: User[] = {}

    getUser(id: number): User | nil
        for _, user in ipairs(self.users) do
            if user.id == id then
                return user
            end
        end
        return nil
    end
}
```

```lua
-- main.lua
import { User, UserService } from "./models/user"

local service = UserService()
local user: User = {
    id = 1,
    name = "Alice",
    email = "alice@example.com"
}
service:save(user)

local found = service:getUser(1)
if found then
    print(found.name)
end
```

## Type Narrowing

### Nil Narrowing

```lua
-- narrowing.lua
function process(value: string | nil): string
    if value ~= nil then
        -- value is string here
        return value:upper()
    end
    return "EMPTY"
end

print(process(nil))     -- "EMPTY"
print(process("hi"))    -- "HI"
```

### Type Predicates

```lua
-- type_predicates.lua
function isString(x: any): x is string
    return type(x) == "string"
end

function isNumber(x: any): x is number
    return type(x) == "number"
end

function process(value: any): string | number
    if isString(value) then
        -- value is string
        return value .. " is a string"
    elseif isNumber(value) then
        -- value is number
        return value * 2
    end
    return "unknown"
end
```

### Pattern Matching Exhaustiveness

```lua
-- exhaustive.lua
type Status = "pending" | "active" | "completed" | "cancelled"

function processStatus(status: Status): string
    if status == "pending" then
        return "Waiting..."
    elseif status == "active" then
        return "In progress"
    elseif status == "completed" then
        return "Done"
    elseif status == "cancelled" then
        return "Cancelled"
    end
    -- Exhaustiveness check: if all cases handled, this is unreachable
    return "Unknown status"
end

-- TypeScript-style exhaustive switch
function processStatusSwitch(status: Status): string
    switch status do
        case "pending" then return "Waiting..."
        case "active" then return "In progress"
        case "completed" then return "Done"
        case "cancelled" then return "Cancelled"
    end
    -- If all cases handled, no error
end
```

## Dependency Injection

### Basic DI Setup

```rust
use typedlua_typechecker::di::{DiContainer, ServiceLifetime};
use typedlua_typechecker::cli::diagnostics::DefaultDiagnosticHandler;
use std::sync::Arc;

struct MyService {
    name: String,
}

impl MyService {
    fn new() -> Self {
        Self { name: "MyService".to_string() }
    }
}

trait ServiceTrait: Send + Sync {
    fn do_something(&self);
}

impl ServiceTrait for MyService {
    fn do_something(&self) {
        println!("{} doing something!", self.name);
    }
}

// Set up DI container
let mut container = DiContainer::new();

// Register services
container.register(
    |_| Arc::new(MyService::new()) as Arc<dyn ServiceTrait>,
    ServiceLifetime::Singleton,
);

container.register(
    |_| {
        let handler = DefaultDiagnosticHandler::new();
        Arc::new(handler) as Arc<dyn typedlua_typechecker::cli::diagnostics::DiagnosticHandler>
    },
    ServiceLifetime::Singleton,
);

// Resolve services
let service: Arc<dyn ServiceTrait> = container.resolve().unwrap();
service.do_something();
```

### Multi-Module Compilation

```rust
use typedlua_typechecker::module_resolver::{ModuleResolver, ModuleRegistry, ModuleId};
use typedlua_typechecker::{TypeChecker, TypeCheckError};
use typedlua_parser::string_interner::StringInterner;
use std::sync::Arc;
use std::path::Path;

fn type_check_project(root: &Path) -> Result<(), TypeCheckError> {
    let interner = StringInterner::new();
    let common = interner.common();

    let handler: Arc<dyn typedlua_typechecker::cli::diagnostics::DiagnosticHandler> =
        Arc::new(typedlua_typechecker::cli::diagnostics::DefaultDiagnosticHandler::new());

    // Create module registry and resolver
    let registry = Arc::new(ModuleRegistry::new());
    let resolver = Arc::new(ModuleResolver::new());

    // Add root as base path
    resolver.add_base_path(root.to_path_buf());

    // Discover all modules
    let modules = discover_modules(root, &resolver)?;

    // Build dependency graph
    let dependency_graph = registry.build_dependency_graph(&modules)?;

    // Get compilation order (dependencies first)
    let order = dependency_graph.build_order()?;

    // Type check in order
    for module_path in order {
        let module_id = registry.get_module_id(&module_path).unwrap();

        let mut checker = TypeChecker::new_with_module_support(
            handler.clone(),
            &interner,
            &common,
            registry.clone(),
            module_id,
            resolver.clone(),
        );

        let source = std::fs::read_to_string(&module_path)?;
        let (mut program, _) = parse(&source, &interner)?;

        checker.check_program(&mut program)?;
    }

    Ok(())
}

fn discover_modules(root: &Path, resolver: &ModuleResolver) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut modules = Vec::new();

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "lua").unwrap_or(false) {
            modules.push(path);
        }
    }

    Ok(modules)
}
```

## Error Handling

### Custom Error Types

```rust
use typedlua_typechecker::{TypeCheckError, TypeCheckError};

#[derive(Debug)]
enum MyError {
    TypeCheck(TypeCheckError),
    Parse(String),
    Io(std::io::Error),
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::TypeCheck(e) => write!(f, "Type error: {}", e),
            MyError::Parse(msg) => write!(f, "Parse error: {}", msg),
            MyError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for MyError {}

fn type_check_with_context(source: &str) -> Result<(), MyError> {
    let interner = StringInterner::new();
    let common = interner.common();

    let handler: Arc<dyn typedlua_typechecker::cli::diagnostics::DiagnosticHandler> =
        Arc::new(typedlua_typechecker::cli::diagnostics::DefaultDiagnosticHandler::new());

    let mut checker = TypeChecker::new(handler, &interner, &common)
        .with_stdlib()
        .map_err(|e| MyError::TypeCheck(TypeCheckError::new(e, Default::default())))?;

    let (mut program, _) = parse(source, &interner)
        .map_err(|e| MyError::Parse(e.to_string()))?;

    checker.check_program(&mut program)
        .map_err(|e| MyError::TypeCheck(e))?;

    Ok(())
}
```

## Integration with Build Tools

### Custom Lint Rule

```rust
use typedlua_typechecker::cli::diagnostics::{Diagnostic, DiagnosticHandler, DiagnosticSeverity};
use typedlua_typechecker::visitors::TypeCheckVisitor;
use typedlua_parser::ast::*;
use typedlua_parser::span::Span;

struct NoPrintDiagnosticHandler {
    inner: Box<dyn DiagnosticHandler>,
    print_count: usize,
}

impl NoPrintDiagnosticHandler {
    fn new(inner: Box<dyn DiagnosticHandler>) -> Self {
        Self { inner, print_count: 0 }
    }
}

impl DiagnosticHandler for NoPrintDiagnosticHandler {
    fn error(&self, span: Span, message: &str) {
        if message.contains("print") {
            self.print_count += 1;
        }
        self.inner.error(span, message);
    }

    fn warning(&self, span: Span, message: &str) {
        self.inner.warning(span, message);
    }

    fn info(&self, span: Span, message: &str) {
        self.inner.info(span, message);
    }

    fn error_count(&self) -> usize {
        self.inner.error_count()
    }

    fn has_errors(&self) -> bool {
        self.inner.has_errors()
    }

    fn get_diagnostics(&self) -> &[Diagnostic] {
        self.inner.get_diagnostics()
    }

    fn print_count(&self) -> usize {
        self.print_count
    }
}
```

## Complete Example

```rust
use typedlua_typechecker::{TypeChecker, TypeCheckError};
use typedlua_parser::{parse, string_interner::StringInterner};
use typedlua_typechecker::cli::diagnostics::DefaultDiagnosticHandler;
use std::sync::Arc;
use std::path::Path;

fn main() {
    let source = r#"
        -- Define types
        type User = {
            id: number,
            name: string,
            email?: string
        }

        -- Generic function
        function first<T>(arr: T[]): T | nil
            return arr[1]
        end

        -- Class
        class Counter {
            private count: number = 0

            public increment(): void
                self.count = self.count + 1
            end

            public getCount(): number
                return self.count
            end
        }

        -- Usage
        local user: User = {
            id = 1,
            name = "Alice",
            email = "alice@example.com"
        }

        local counter = Counter()
        counter:increment()
        print(counter:getCount())
    "#;

    match type_check_source(source) {
        Ok(()) => {
            println!("✓ Type checking passed!");
        }
        Err(e) => {
            eprintln!("✗ Type error: {}", e);
            std::process::exit(1);
        }
    }
}

fn type_check_source(source: &str) -> Result<(), TypeCheckError> {
    let interner = StringInterner::new();
    let common = interner.common();

    let handler: Arc<dyn typedlua_typechecker::cli::diagnostics::DiagnosticHandler> =
        Arc::new(DefaultDiagnosticHandler::new());

    let mut checker = TypeChecker::new(handler, &interner, &common)
        .with_stdlib()?;

    let (mut program, _) = parse(source, &interner)?;
    checker.check_program(&mut program)?;

    Ok(())
}
```
