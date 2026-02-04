# Type System Reference

This document describes the TypedLua type system in detail.

## Overview

TypedLua uses a **structural type system** with optional **nominal subtyping** for classes. Types are represented as AST-like structures with span information for error reporting.

## Primitive Types

### Basic Primitives

```lua
local n: number = 42
local s: string = "hello"
local b: boolean = true
local nil_value: nil = nil
```

| Type      | Lua Equivalent | Description                    |
|-----------|----------------|--------------------------------|
| `nil`     | `nil`          | The nil value                  |
| `boolean` | `boolean`      | true or false                  |
| `number`  | `number`       | All Lua numbers                |
| `string`  | `string`       | All Lua strings                |
| `any`     | -              | Dynamic type, accepts anything |
| `unknown` | -              | Top type, type-safe any        |

### Special Primitives

```lua
-- Void: no return value
function log(msg: string): void
    print(msg)
end

-- Never: unreachable code
function error(message: string): never
    error(message)
end
```

| Type    | Description                                |
|---------|--------------------------------------------|
| `void`  | No return value (like TypeScript's void)   |
| `never` | Unreachable type (like TypeScript's never) |

## Object Types

### Interface

```lua
interface User {
    name: string
    age: number
    email?: string  -- Optional property
}

interface Repository {
    getById(id: number): User | nil
    save(user: User): boolean
}
```

**Syntax:**

```typescript
interface Name {
    property1: Type1
    property2?: Type2  // Optional
    method(param: Type): ReturnType
    (param: Type): ReturnType  // Callable
    [key: Type]: ValueType  // Index signature
}
```

### Class

```lua
class Animal {
    name: string

    constructor(name: string)
        self.name = name
    end

    speak(): string
        return "..."
    end
end

class Dog extends Animal {
    breed: string

    override speak(): string
        return "woof"
    end
}
```

**Members:**

- Properties: `name: Type`
- Methods: `name(params): ReturnType`
- Constructors: `constructor(params)`
- Getters: `get name(): Type`
- Setters: `set name(value: Type)`

### Access Modifiers

```lua
class Counter {
    private count: number = 0    -- Only accessible within class
    protected items: number[]    -- Accessible in class and subclasses
    public total: number         -- Accessible anywhere (default)
}
```

| Modifier    | Access Scope                   |
|-------------|--------------------------------|
| `private`   | Declaring class only           |
| `protected` | Declaring class and subclasses |
| `public`    | Anywhere (default)             |

## Generic Types

### Type Parameters

```lua
-- Generic function
function identity<T>(value: T): T
    return value
end

-- Usage
local num = identity<number>(42)    -- T = number
local str = identity<string>("hi")  -- T = string
```

### Type Constraints

```lua
function log<T extends { __tostring: () => string }>(value: T): void
    print(tostring(value))
end

-- T must have __tostring method
```

### Generic Classes

```lua
class Container<T> {
    value: T

    constructor(value: T)
        self.value = value
    end

    get(): T
        return self.value
    end
}

local number_container = Container<number>(42)
local string_container = Container<string>("hello")
```

### Multiple Type Parameters

```lua
class Pair<K, V> {
    key: K
    value: V

    constructor(key: K, value: V)
        self.key = key
        self.value = value
    end
}
```

## Utility Types

### Pick<T, K>

Select a subset of properties from a type.

```lua
type Point3D = {
    x: number
    y: number
    z: number
    label: string
}

type Point2D = Pick<Point3D, "x" | "y">
-- Result: { x: number, y: number }
```

### Omit<T, K>

Exclude properties from a type.

```lua
type User = {
    id: number
    name: string
    password: string
}

type PublicUser = Omit<User, "password">
-- Result: { id: number, name: string }
```

### Partial<T>

Make all properties optional.

```lua
type User = {
    name: string
    email: string
}

type PartialUser = Partial<User>
-- Result: { name?: string, email?: string }
```

### Required<T>

Make all properties required (remove optional).

```lua
type User = {
    name?: string
    age?: number
}

type StrictUser = Required<User>
-- Result: { name: string, age: number }
```

### Readonly<T>

Make all properties read-only.

```lua
type Point = {
    x: number
    y: number
}

type FrozenPoint = Readonly<Point>
-- Cannot modify: p.x = 10  -- Error!
```

### Record<K, V>

Construct an object type with specific key and value types.

```lua
type StringToNumber = Record<string, number>
-- Result: { [key: string]: number }

-- Usage
local scores: Record<string, number> = {
    alice = 100,
    bob = 95,
}
```

### Keyof<T>

Get all property keys as a union type.

```lua
type Point = {
    x: number
    y: number
}

type PointKeys = keyof Point
-- Result: "x" | "y"
```

### Exclude<T, U>

Exclude types from a union.

```lua
type T0 = Exclude<"a" | "b" | "c", "a">
-- Result: "b" | "c"

type T1 = Exclude<number, string>  -- number (types, not values)
```

### Extract<T, U>

Extract types that are assignable to a union.

```lua
type T0 = Extract<"a" | "b" | "c", "a" | "f">
-- Result: "a"
```

### NonNullable<T>

Remove nil/null from a type.

```lua
type T0 = NonNullable<string | nil>
-- Result: string

type T1 = NonNullable<string[] | null | undefined>
-- Result: string[]
```

### ReturnType<F>

Get the return type of a function type.

```lua
function greet(): string
    return "hello"
end

type GreetReturn = ReturnType<typeof(greet)>
-- Result: string
```

### InstanceType<C>

Get the instance type of a class constructor.

```lua
class Point {
    x: number
    y: number
}

type PointInstance = InstanceType<typeof(Point)>
-- Result: Point
```

### ThisType<T>

Specify the type of `this` in methods.

```lua
type ObjectWithThis = {
    value: number
    increment(this: this): void
}
```

### Conditional Types

```lua
-- Basic conditional
type IsString<T> = T extends string ? "yes" : "no"

type R1 = IsString<string>   -- "yes"
type R2 = IsString<number>   -- "no"

-- Distributive conditional
type ToArray<T> = T extends any ? T[] : never

type R3 = ToArray<string | number>
-- Result: string[] | number[]
```

### Mapped Types

```lua
-- Property modifiers
type Partial<T> = { [P in keyof T]?: T[P] }
type Required<T> = { [P in keyof T]: T[P] }
type Readonly<T> = { readonly [P in keyof T]: T[P] }

-- Key remapping
type Getters<T> = { [P in keyof T as `get${Capitalize<string & P>}`]: () => T[P] }

type User = {
    name: string
    age: number
}

type UserGetters = Getters<User>
-- Result: { getName: () => string, getAge: () => number }
```

## Union and Intersection Types

### Union Types

```lua
local id: number | string = "abc"
local value: string | nil = nil
```

### Intersection Types

```lua
type PartA = { a: number }
type PartB = { b: string }

type Combined = PartA & PartB
-- Result: { a: number, b: string }
```

## Function Types

### Function Type Syntax

```lua
-- Named function
function add(a: number, b: number): number
    return a + b
end

-- Function type annotation
local fn: (number, number) => number = add

-- Callable signatures
interface Callable {
    (x: number): string
    (x: string): number
}
```

### Optional and Rest Parameters

```lua
function variadic(first: number, ...rest: number[]): number
    local sum = first
    for _, v in ipairs(rest) do
        sum = sum + v
    end
    return sum
end

-- Optional parameter
function greet(name: string, greeting?: string): string
    return (greeting or "Hello") .. " " .. name
end
```

### `this` Parameter

```lua
function modify(this: Table, key: string, value: any): any
    self[key] = value
end
```

## Type Compatibility

### Structural Compatibility

TypedLua uses **structural typing** - types are compatible if they have compatible shapes.

```lua
interface Point {
    x: number
    y: number
}

class ThreeDPoint {
    x: number
    y: number
    z: number
}

-- Point is assignable to ThreeDPoint
local p: Point = ThreeDPoint(1, 2, 3)  -- OK

-- ThreeDPoint is NOT assignable to Point
-- local p2: ThreeDPoint = Point(1, 2)  -- Error!
```

### Function Compatibility

Functions are compatible if:

1. Return types are compatible
2. Parameter types are contravariantly compatible

```lua
-- Handler accepts any (string | number) -> void
local handler: (string | number) => void = function(x: string) end  -- OK
local handler2: (string | number) => void = function(x: number) end  -- OK

-- Not OK - handler requires specific type
local specific: (string) => void = function(x: string | number) end  -- Error!
```

### Array Compatibility

Arrays are covariant in TypedLua:

```lua
local numbers: number[] = {1, 2, 3}
local values: (number | string)[] = numbers  -- OK (covariant)
```

### Nullable Types

```lua
local value: number | nil = nil
local value2: number? = nil  -- Shorthand
```

## Type Narrowing

### Nil Check Narrowing

```lua
local maybe: string | nil

if maybe ~= nil then
    -- maybe is narrowed to string
    print(maybe:upper())  -- OK
end
```

### Type Predicates

```lua
function isString(x: any): x is string
    return type(x) == "string"
end

local value: any = "hello"

if isString(value) then
    -- value is narrowed to string
    print(value:upper())
end
```

### Equality Narrowing

```lua
local result: number | string

if typeof(result) == "number" then
    -- result is narrowed to number
    print(result + 1)
else
    -- result is narrowed to string
    print(result .. " is a string")
end
```

### In-Operator Narrowing

```lua
type Color = "red" | "green" | "blue"

local color: Color = "red"

if color == "red" then
    -- color is narrowed to "red"
end
```

## Advanced Patterns

### Type Aliases

```lua
-- Type alias
type ID = string | number

type User = {
    id: ID
    name: string
}

-- Generic type alias
type Array<T> = T[]

type NumberArray = Array<number>
```

### Enum Types

```lua
enum Status {
    Pending = 0
    Active = 1
    Completed = 2
}

local current: Status = Status.Active
```

### Declaration Files

```lua
-- globals.d.ts
declare function print(message: string): void
declare module "io" {
    export function write(data: string): void
}
```

### Overload Signatures

```lua
-- lib.lua
function format(value: number): string
function format(value: string): string
function format(value: any): string
    return tostring(value)
end

-- TypeScript-style declaration
function format(value: number): string
function format(value: string): string
function format(value: any): string
    -- Implementation
end
```

## Type Inference

### Variable Type Inference

```lua
-- Type is inferred from initializer
local x = 42           -- number
local s = "hello"      -- string
local t = true         -- boolean
local n = nil          -- nil
```

### Return Type Inference

```lua
-- Return type inferred
function add(a: number, b: number)
    return a + b
end
```

### Generic Type Inference

```lua
-- Type arguments inferred from parameters
function first<T>(arr: T[]): T | nil
    return arr[1]
end

local nums = first({1, 2, 3})  -- T = number, returns number | nil
local strs = first({"a", "b"})  -- T = string, returns string | nil
```

### Contextual Typing

```lua
-- Parameter type inferred from callback
local values: number[] = {1, 2, 3}
table.sort(values, function(a, b) return a < b end)
-- a and b are inferred as number
```
