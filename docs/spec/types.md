---
title: ish Type System
category: spec
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/assurance-ledger.md, docs/spec/polymorphism.md, docs/spec/memory.md, docs/spec/syntax.md]
---

# ish Type System

## Goals

The ish type system has four primary goals:

1. **Reasoning about values.** Enable the code analyzer to determine the possible values a variable might hold, so it can reason about code paths, optimize, and catch errors.
2. **Rust representation.** Every ish value must have a well-defined mapping to a Rust type, since all ish code — whether interpreted or compiled — is ultimately translated into Rust.
3. **First-class complex types.** Common, important data structures are treated as first-class types with built-in syntax and semantics.
4. **Runtime validation.** The type system provides mechanisms for checking and enforcing types at runtime.

## Design Philosophy

The ish type system draws heavily from TypeScript's approach to types. Like TypeScript, ish types describe the *set of possible values* a variable might hold. Unlike most type systems that classify values into broad categories (e.g., "this is an integer"), ish types can be as narrow as a single literal value.

This set-of-values perspective is the foundation for the code analyzer's ability to reason about code. It also enables the low-assurance ↔ high-assurance continuum: in low-assurance mode, types are inferred and permissive; in high-assurance mode, types are explicit and strict.

---

## Type Categories

### Primitive Types

ish uses the same primitive types as Rust, since all ish code maps to Rust at some point.

| Type     | Description                                  | Rust equivalent |
|----------|----------------------------------------------|-----------------|
| `bool`   | Boolean value (`true` or `false`)            | `bool`          |
| `char`   | A single Unicode scalar value                | `char`          |
| `i8`     | Signed 8-bit integer                         | `i8`            |
| `i16`    | Signed 16-bit integer                        | `i16`           |
| `i32`    | Signed 32-bit integer                        | `i32`           |
| `i64`    | Signed 64-bit integer                        | `i64`           |
| `i128`   | Signed 128-bit integer                       | `i128`          |
| `u8`     | Unsigned 8-bit integer                       | `u8`            |
| `u16`    | Unsigned 16-bit integer                      | `u16`           |
| `u32`    | Unsigned 32-bit integer                      | `u32`           |
| `u64`    | Unsigned 64-bit integer                      | `u64`           |
| `u128`   | Unsigned 128-bit integer                     | `u128`          |
| `usize`  | Pointer-sized unsigned integer (for indexing) | `usize`        |
| `f32`    | 32-bit floating point                        | `f32`           |
| `f64`    | 64-bit floating point                        | `f64`           |

`isize` is intentionally excluded — ish does not expose user-facing pointer operations, so only `usize` (for collection indexing) is needed.

In low-assurance mode, numeric literals without annotations default to `f64` (matching JavaScript's behavior, which is familiar to most developers). In high-assurance mode, the developer must specify the exact numeric type, either via an explicit annotation or through type inference within the same statement (e.g., passing a literal to a function with a typed parameter).

**Integer overflow** behavior is configurable via the active standard. Options include wrapping, panicking, or saturating.

**Implicit numeric conversions** (e.g., `i32` → `i64`) are configurable via the active standard. In low-assurance mode, safe widening conversions may be implicit. In high-assurance mode, all conversions must be explicit.

### Literal Types

Any concrete value can serve as a type. A literal type contains exactly one value. Literal types are supported for all value kinds — numeric, string, boolean, object, and list.

```ish
let x = 5            // x has type: 5
let y = "hello"      // y has type: "hello"
let z = true         // z has type: true
let p = { a: 1 }     // p has type: { a: 1 }
let ns = [1, 2, 3]   // ns has type: [1, 2, 3]
```

Literal types are the most specific types possible. They enable the code analyzer to reason precisely about code paths:

```ish
let x = 5
if x < 3 {
    // The code analyzer determines this branch is unreachable,
    // since the only possible value of x is 5, and 5 < 3 is false.
}
```

The code analyzer tracks arithmetic and other operations through literal types. When an expression can be fully evaluated at compile time, the analyzer does so, preserving literal type precision:

```ish
let x = 5
let y = x + 1   // y has type: 6 (computed at compile time)
let z = x * 2   // z has type: 10
```

**String literal types** are supported:

```ish
type Direction = "north" | "south" | "east" | "west"
let d: Direction = "north"
```

**Char literal types** use the `c'...'` syntax:

```ish
let ch: c'A' = c'A'
let letter = c'Z'   // letter has type: c'Z'
```

### Special Types

ish has four built-in special types representing different kinds of "nothing":

| Type        | Description                                                                                  |
|-------------|----------------------------------------------------------------------------------------------|
| `void`      | The type returned by a function that does not have a return value. Has one value: `void`.     |
| `null`      | The value of a nullable variable when no value is present. Has one value: `null`.             |
| `undefined` | The value of a property on an open object type when that property does not exist. Has one value: `undefined`. |
| `never`     | The bottom type — has no values. Represents unreachable code or impossible types.             |

### First-Class Complex Types

| Type     | Description                                                                                   |
|----------|-----------------------------------------------------------------------------------------------|
| `String` | A sequence of characters. Distinct from the primitive integer types used for raw byte data.    |
| `List`   | An ordered, indexable sequence of elements.                                                    |
| `Tuple`  | A fixed-length, heterogeneous ordered sequence of elements.                                    |
| `Set`    | An unordered collection of unique elements.                                                    |
| `Map`    | A collection of key-value pairs with unique keys.                                              |
| `Object` | A record with named properties, each of which has its own type.                                |

### Tuple Types

Tuples are fixed-length sequences where each element has its own type:

```ish
let point: (f64, f64) = (3.0, 4.0)
let record: (String, i32, bool) = ("Alice", 30, true)
```

Tuples are distinct from `List` (homogeneous, variable-length) and `Object` (named properties).

### The `Object` Type

Objects are the primary structured data type in ish. An object type is defined by its set of named, typed properties.

```ish
let person = {
    name: "Alice",
    age: 30,
}
// person has type: { name: "Alice", age: 30 }
```

#### Structural and Nominal Typing

The ish type system supports both **structural** and **nominal** typing.

By default, types are **structural**: two object types are compatible if they have the same shape (property names and compatible property types), regardless of how they were declared.

Types can be explicitly declared as **nominal**, in which case compatibility requires that the types be declared as related, not merely that they have the same shape:

Nominal typing is handled through entries/annotations in the assurance ledger rather than a `nominal type` keyword. See [docs/spec/assurance-ledger.md](assurance-ledger.md) for details on how to annotate types as nominal.

The structural/nominal choice does not vary with assurance level. A type is structural unless explicitly annotated as nominal.

#### Open and Closed Object Types

An object type can be either **open** or **closed**:

- **Closed** (default): The object has exactly the declared properties. Passing an object with extra properties is an error.
- **Open**: The object has at least the declared properties but may have additional ones. Accessing an undeclared property returns `undefined`.

Open object types must be implemented as associative arrays at the polymorphism level. The code analyzer can detect when an object is declared open but the open capability is never used, and replace it with a closed type for better performance.

Open objects arise naturally when deserializing data (e.g., from JSON). Once validated against a type, the resulting object is open or closed based on the target type.

#### Optional Properties

Object properties can be declared optional. An optional property may be absent from the object entirely:

```ish
type Person = {
    name: String,
    age?: i32,      // optional — may be present or absent
}
```

An optional property is typed as `T | null` — when absent, accessing it yields `null`.

#### Mutable and Immutable Properties

Individual object properties can be marked as mutable or immutable. Depending on the active standard, specifying mutability may be required.

#### Index Signatures

An object type can declare a catch-all type for arbitrary string keys:

```ish
type StringMap = { [key: String]: i32 }
```

#### Methods and `self`

Object types can include function-typed properties (methods). Methods have access to `self`, a reference to the object on which the method was called.

#### Recursive Types

Object types can reference themselves, enabling recursive data structures:

```ish
type TreeNode = {
    value: i32,
    children: List<TreeNode>,
}
```

---

## Type Composition

### Union Types

A union type represents a value that could be one of several types. Written with `|`:

```ish
let value: i32 | String = getValue()
```

Union types arise naturally through control flow:

```ish
let x
if condition {
    x = 5
} else {
    x = "hello"
}
// x has type: 5 | "hello"
```

**Discriminated unions** (tagged unions) are supported, where each variant has a common property that identifies it. This enables exhaustive pattern matching.

### Intersection Types

An intersection type represents a value that satisfies all of several types simultaneously. Written with `&`:

```ish
type Named = { name: String }
type Aged = { age: i32 }
type Person = Named & Aged
// Person is: { name: String, age: i32 }
```

The intersection of incompatible types produces `never`:

```ish
type Impossible = i32 & String  // Impossible is: never
```

### Optional Types

A type suffixed with `?` is shorthand for a union with `null`:

```ish
let x: i32? = maybeGetNumber()
// Equivalent to: let x: i32 | null = maybeGetNumber()
```

---

## Type Inference and Narrowing

### Inference

In low-assurance mode, types are inferred from usage. Developers are not required to write type annotations. The code analyzer tracks the set of possible values for each variable at each point in the program.

```ish
let x = 5         // inferred type: 5
let y = x + 1     // inferred type: 6 (evaluated at compile time)
```

In high-assurance mode, the code analyzer may require explicit type annotations where inference is ambiguous.

### Narrowing

Control flow narrows the type of a variable within a branch:

```ish
let x: i32 | String = getValue()

if isType(String, x) {
    // Here, x has type: String
} else {
    // Here, x has type: i32
}
```

Type narrowing applies to any conditional check that provides type information, including `isType` calls, comparison operators, truthiness checks, and custom type guards.

---

## Mutability

Variables are declared as either mutable or immutable:

```ish
let x = 5         // immutable
let mut y = 5     // mutable
```

Mutability affects type widening: mutable variables are more likely to have their literal types widened to broader types.

---

## Type Widening

Literal types are precise but often impractical for mutable variables. **Type widening** is the process by which the code analyzer generalizes a literal type to a broader type.

```ish
let x = 5       // literal type: 5 (immutable — no widening needed)
let mut y = 5   // widened type (exact rules TBD)
```

The exact widening rules interact with the active standard's configuration. In low-assurance mode, widening is aggressive (for convenience). In high-assurance mode, widening is conservative (for precision).

---

## Type Aliases

Developers can name types for reuse and readability:

```ish
type Age = i32
type Name = String
type Person = { name: Name, age: Age }
type Result = Person | Error
```

Structural type aliases introduce a name but do not create a distinct type. To create a distinct type, annotate it as nominal via the assurance ledger (see [Structural and Nominal Typing](#structural-and-nominal-typing)).

---

## Generic Types

Types can be parameterized:

```ish
type Pair<A, B> = { first: A, second: B }
let p: Pair<i32, String> = { first: 1, second: "hello" }
```

The first-class complex types are generic:

```ish
let names: List<String> = ["Alice", "Bob"]
let scores: Map<String, i32> = { "Alice": 95, "Bob": 87 }
```

### Type Parameter Constraints

```ish
type Sortable<T: Comparable> = List<T>
```

### Type Parameter Defaults

```ish
type Result<T, E = Error> = { value: T } | { error: E }
```

### Type Parameter Inference

```ish
fn first<T>(list: List<T>) -> T { ... }
let x = first([1, 2, 3])   // T is inferred as i32
```

### Higher-Kinded Types

```ish
type Functor<F<_>> = {
    map: fn<A, B>(F<A>, fn(A) -> B) -> F<B>,
}
```

---

## Function Types

Functions are first-class values in ish. Closures are supported and capture variables by reference.

---

## The `Type` Metatype

Types themselves are representable as values via the `Type` metatype:

```ish
let t: Type = i32
let u: Type = List<String>
```

`Type` is what makes `isType(t, i)` and `validate(t, i)` possible — their first argument is a value of type `Type`. It also enables generic types to retain full type information at runtime rather than being erased.

---

## Runtime Type Operations

### `isType(t, i)`

Returns `true` if instance `i` is of type `t`.

```ish
let x: i32 | String = getValue()
if isType(i32, x) {
    // x is narrowed to i32
}
```

### `validate(t, i)`

Returns `i` cast as type `t` if valid. Throws an exception otherwise.

```ish
let raw = parseJson(input)
let person = validate(Person, raw)
```

### Custom Type Guards

Developers can define custom type guard functions that provide additional validation beyond structural type checks.

---

## Error Handling

ish uses thrown exceptions for error handling. There is a built-in `Error` type — an object with a `message` property and error metadata. Errors are created with `new_error(message)`, thrown with `throw`, and caught with `try`/`catch`/`finally`.

Functions that can throw errors may declare their error types using union types:

```ish
fn read_file(path: String) -> String | FileError { ... }
```

In high-assurance mode, the compiler infers error union types automatically and can optionally require explicit declarations. In low-assurance mode, error types are not tracked.

Error declaration requirements are configurable via the `undeclared_errors` feature in the active standard. See [docs/spec/assurance-ledger.md](assurance-ledger.md) and [docs/user-guide/error-handling.md](../user-guide/error-handling.md) for details.

---

## Interaction with Assurance Levels

| Aspect                   | Low-assurance                                | High-assurance                                       |
|--------------------------|----------------------------------------------|------------------------------------------------------|
| Type annotations         | Optional; inferred from usage                | Required where inference is ambiguous                 |
| Numeric types            | Default to `f64`                             | Exact type required (e.g., `i32`, `u64`)             |
| Type widening            | Aggressive                                   | Conservative                                         |
| Unreachable code         | Warning (or silent)                          | Build error                                          |
| Type errors              | Runtime exceptions                           | Build-time errors                                    |
| `isType` / `validate`   | Primary type-checking mechanism              | Supplement to build-time checks                      |
| Integer overflow         | Wrapping (or runtime error)                  | Build-time configurable (wrap, panic, saturate)       |
| Implicit conversions     | Safe widening conversions are implicit        | All conversions must be explicit                      |
| Property mutability      | Optional to declare                          | Required to declare                                  |
| Generic bounds           | Optional                                     | Required where applicable                            |
| Exhaustiveness checking  | Warning (or silent)                          | Build error on non-exhaustive union matches           |
| Null safety              | Relaxed                                      | Strict — nullable types must be explicitly handled    |

---

## Rust Mapping

Every ish type has a defined mapping to Rust:

| ish type          | Rust representation                                          |
|-------------------|--------------------------------------------------------------|
| Primitives        | Corresponding Rust primitive (`i32`, `f64`, `bool`, `char`, `usize`, etc.) |
| `String`          | `String` (or `&str` where the compiler determines it is safe) |
| `List<T>`         | `Vec<T>`                                                     |
| `Tuple`           | Rust tuple (e.g., `(i32, String, bool)`)                     |
| `Set<T>`          | `HashSet<T>`                                                 |
| `Map<K, V>`       | `HashMap<K, V>`                                              |
| `Object`          | Depends on polymorphism strategy (struct, enum, `HashMap<String, Value>`, etc.) |
| Union types       | Rust `enum` with variants for each member type               |
| Optional (`T?`)   | `Option<T>`                                                  |
| `null`            | `None` (within `Option<T>`)                                  |
| `void`            | `()`                                                         |
| `never`           | `!` (the never type)                                         |

Generic types are not erased at runtime. Depending on the polymorphism strategy, generics may be monomorphized or stored as type-tagged values.

---

## Open Questions

Open questions for the type system. See also [docs/project/open-questions.md](../project/open-questions.md#type-system) for a consolidated view.

### Naming Convention

- [ ] **Capitalization of special types.** The spec uses lowercase (`void`, `null`, `undefined`, `never`) but this convention has not been finalized. Should these match the lowercase primitives (`bool`, `i32`) or the capitalized complex types (`String`, `List`)?

### Object Types — Syntax Gaps

- [ ] **Open vs. closed object type syntax.** The concept is defined but there is no syntax for declaring an object type as open or closed.
- [ ] **Property mutability syntax.** Individual properties can be mutable or immutable, but the annotation syntax has not been defined.
- [ ] **Method syntax on object types.** Object types can include function-typed properties (methods), but the syntax has not been defined.

### Union Types

- [ ] **Discriminated unions — full specification.** What constitutes a discriminant property? How does pattern matching / exhaustive switching work?
- [ ] **Union type flattening.** Is `(A | B) | C` the same as `A | B | C`? Are nested unions automatically flattened?

### Type Widening

- [ ] **Widening rules are not specified.** Does `let mut x = 5` widen to `i32`? `f64`? Does `let mut s = "hello"` widen to `String`? Does `let mut b = true` widen to `bool`? What triggers widening?

### Generic Types

- [ ] **Variance.** Are generic types covariant, contravariant, or invariant? Is `List<Dog>` assignable to `List<Animal>`?

### Function Types

- [x] **~~Function type syntax.~~** Resolved — function types use `fn(Args) -> Ret` syntax. See [docs/spec/syntax.md](syntax.md).
- [ ] **Generic function types.** How do type parameters work in function type signatures?
- [ ] **Overloaded function types.** Can a function have multiple type signatures?

### Runtime Type Operations

- [ ] **Performance implications of `validate`.** Validating deeply nested object types at runtime could be expensive. Are there guidelines or lazy validation strategies?
- [ ] **Custom type guard syntax.** Custom type guards are confirmed as supported, but the syntax has not been defined.

### Rust Mapping

- [ ] **Union type representation details.** How are variant names generated? How does this interact with pattern matching on the Rust side?
- [ ] **Object representation selection.** Rules for choosing between struct, enum, and `HashMap<String, Value>` need cross-referencing with the polymorphism spec.
- [ ] **`undefined` Rust mapping.** The Rust representation of `undefined` (for open object property access) needs to be specified.

### Error Types

- [x] **`Error` type status.** Error objects are structural — created with `new_error()` which produces an object with `message` and `__is_error__` metadata. Not a nominal type. Open question whether this should become a nominal type.
- [x] **Exception model details.** ish uses thrown exceptions. Functions can declare thrown error types as union types in high-assurance mode. The compiler infers error union types automatically. Three mode presets: low-assurance, high-assurance, no-throw.

### Assurance Level Configuration

- [ ] **Per-variable assurance level syntax.** The syntax for configuring assurance levels per-variable has not been designed.

### The `Type` Metatype

- [ ] **First-class vs. restricted.** Should `Type` be a full first-class type or restricted to specific patterns?
- [ ] **Runtime type construction.** Can new types be constructed at runtime?
- [ ] **Type reflection.** Can code inspect the structure of a `Type` value at runtime?
- [ ] **Rust mapping for `Type`.** What is the Rust representation of a `Type` value?

### Type Compatibility and Assignability

- [ ] **Subtype / assignability rules are not formalized.** When is type A assignable to type B?
- [ ] **Coercion rules.** Are there any implicit coercions beyond configurable numeric conversions?

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/architecture/vm.md](../architecture/vm.md)
- [docs/user-guide/types.md](../user-guide/types.md)
- [docs/ai-guide/orientation.md](../ai-guide/orientation.md)
