---
title: ish Type System
category: spec
audience: [all]
status: draft
last-verified: 2026-03-19
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

The ish type system describes the *set of possible values* a variable might hold. This set-of-values perspective is the foundation for the code analyzer's ability to reason about code. It also enables the low-assurance ↔ high-assurance continuum: in low-assurance mode, types are inferred and permissive; in high-assurance mode, types are explicit and strict.

### Value-Entry Model

Values in ish carry **entries** — facts recorded about them by the assurance ledger. Type information is expressed through three kinds of value entries:

- **Actual-value entry**: The exact value at a given point in execution (e.g., `actual_value(5)`). This replaces the concept of literal types — instead of a value "having" a literal type, the ledger records what value it actually holds.
- **Possible-values entry**: The set of values a variable might hold at a given point (e.g., `possible_values(1 | 2 | 3)`). Updated by control flow analysis and narrowing.
- **Allowed-values entry**: The set of values a variable is *permitted* to hold, as declared by a type annotation (e.g., `allowed_values(i32)`). Used for validation.

The ledger maintains these entries and checks them for consistency during audit. See [docs/spec/assurance-ledger.md](assurance-ledger.md) for the full entry system.

### Types, Entries, and Type Declarations

Terminology matters in ish because type declarations serve a dual purpose:

- A **type** is a set of possible values (e.g., `i32` is the set of all 32-bit signed integers).
- A **type declaration** (e.g., `let x: i32 = 5`) is *not* just a structural annotation — it also serves as a hook for behavioral entries. When you write `: i32`, the ledger records an `allowed_values(i32)` entry *and* may add behavioral entries like overflow handling based on the active standard.
- An **entry** is a fact about a value recorded in the ledger. Type entries (`actual_value`, `possible_values`, `allowed_values`) are a subset of all possible entries.

This distinction matters because features like mutability, null safety, and overflow behavior are expressed as entries managed by the assurance ledger, not as part of the type system itself. The type system specifies *what values are valid*; the ledger specifies *what checks are performed and when*. See [docs/spec/assurance-ledger.md](assurance-ledger.md) for details on entries and the features they govern.

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

In low-assurance mode, numeric literals without annotations default to `f64` (matching JavaScript's behavior). In high-assurance mode, the developer must specify the exact numeric type.

**Integer overflow** behavior and **implicit numeric conversions** are configured via the active standard as assurance ledger features, not as type system rules. See [docs/spec/assurance-ledger.md](assurance-ledger.md) § Feature State Table for the `overflow` and `implicit_conversions` features.

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
```

#### Open and Closed Object Types

An object type can be either **open** or **closed**:

- **Closed**: The object has exactly the declared properties. Passing an object with extra properties is a discrepancy.
- **Open**: The object has at least the declared properties but may have additional ones. Accessing an undeclared property returns `undefined`.

**Object literals are closed by default.** When you write `{ name: "Alice", age: 30 }`, the resulting object has a `Closed` entry. This means it has exactly those two properties — no extras allowed.

**Type declarations are indeterminate.** When you write `type Person = { name: String, age: i32 }`, the type declaration itself does not specify open or closed. Whether objects of type `Person` are open or closed depends on the context — either an explicit `@[Open]` or `@[Closed]` annotation, or the active standard's `open_closed_objects` feature.

Open and closed are expressed as entries in the assurance ledger:

```ish
@[Open]
let config = load_config()    // config is open — extra properties allowed

@[Closed]
let point = { x: 0, y: 0 }   // point is closed — only x and y
```

When no explicit annotation is present and no standard requires one, objects are **closed by default** (matching the common case of structured data). Open objects must be implemented as associative arrays at the polymorphism level. The code analyzer can detect when an object declared open never uses the open capability and optimize it to a closed representation.

#### Structural and Nominal Typing

The ish type system supports both **structural** and **nominal** typing.

By default, types are **structural**: two object types are compatible if they have the same shape (property names and compatible property types), regardless of how they were declared.

Types can be explicitly declared as **nominal**, in which case compatibility requires that the types be declared as related, not merely that they have the same shape. Nominal typing is handled through entries in the assurance ledger rather than a `nominal type` keyword. See [docs/spec/assurance-ledger.md](assurance-ledger.md) for details.

The structural/nominal choice does not vary with assurance level. A type is structural unless explicitly annotated as nominal.

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

Individual object properties can be marked as mutable or immutable. Whether specifying mutability is required depends on the active standard's `immutability` feature.  See [docs/spec/assurance-ledger.md](assurance-ledger.md) § Feature State Table.

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
let value: i32 | String = get_value()
```

Union types arise naturally through control flow:

```ish
let x
if condition {
    x = 5
} else {
    x = "hello"
}
// x has possible_values: i32 | String
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

For object types, the intersection merges all properties from all constituent types. If two types define the same property with compatible types, the more specific type is used. If they define the same property with incompatible types, the result is `never` for that property, which typically makes the entire intersection `never`.

The intersection of incompatible primitive types produces `never`:

```ish
type Impossible = i32 & String  // Impossible is: never
```

Intersection types can be used for mixins and capability composition:

```ish
type Serializable = { serialize: fn() -> String }
type Loggable = { log: fn(String) -> void }
type SerializableAndLoggable = Serializable & Loggable
```

### Optional Types

A type suffixed with `?` is shorthand for a union with `null`:

```ish
let x: i32? = maybe_get_number()
// Equivalent to: let x: i32 | null = maybe_get_number()
```

---

## Type Inference and Narrowing

### Inference

In low-assurance mode, types are inferred from usage. Developers are not required to write type annotations. The code analyzer tracks the set of possible values for each variable at each point in the program.

```ish
let x = 5         // actual_value entry: 5, possible_values: i32
let y = x + 1     // actual_value entry: 6 (evaluated at analysis time)
```

**Deferred type inference**: When a binding has no type annotation, the ledger does not immediately assign a type entry. Instead, the string literal of the initializer is retained and the type is resolved on demand — when the value is used in a context that requires type information. This avoids premature widening and allows the analyzer to make the most informed decision.

In high-assurance mode, the active standard may require explicit type annotations where inference is ambiguous (`type_annotations` set to `required`).

### Narrowing

Control flow narrows the set of possible values for a variable within a branch. Narrowing is implemented as **entry set maintenance by the assurance ledger** — after every statement, the ledger produces revised entry sets that reflect the information gained from that statement.

```ish
let x: i32 | String = get_value()

if is_type(x, String) {
    // Here, the ledger has narrowed x's possible_values to: String
} else {
    // Here, the ledger has narrowed x's possible_values to: i32
}
// After the branch, possible_values are restored or merged
```

Narrowing rules:

- **`is_type()` narrowing**: In the true branch of `if is_type(x, T)`, `x`'s possible-values entry is narrowed to `T`. In the false branch, `T` is excluded from the possible values.
- **Null comparison narrowing**: In the true branch of `if x != null`, `null` is excluded from `x`'s possible-values entry. In the false branch, `x`'s possible-values entry is narrowed to `null`.
- **Branch merge**: When branches converge (after if/else, at the end of a while loop, etc.), the ledger unions the entry sets from all branches.
- **Entry restoration**: On branch exit, entries are restored to their pre-branch state, then updated with the merged result. This prevents narrowing from one branch from leaking into code after the branch.
- **Nested narrowing**: Narrowing composes — narrowing inside a nested branch further refines the outer branch's narrowing.

See [docs/spec/assurance-ledger.md](assurance-ledger.md) for the full specification of entry maintenance and type narrowing as ledger behavior.

---

## Type Widening

Value entries are precise but often impractical for mutable variables. **Type widening** is the process by which the code analyzer generalizes a value's possible-values entry to a broader type.

```ish
let x = 5       // immutable — actual_value: 5, no widening needed
let mut y = 5   // mutable — widened possible_values (exact rules TBD)
```

The exact widening rules interact with the active standard's configuration. In low-assurance mode, widening is aggressive (for convenience). In high-assurance mode, widening is conservative (for precision).

---

## Type Aliases

Developers can name types for reuse and readability:

```ish
type Age = i32
type Name = String
type Person = { name: Name, age: Age }
```

Structural type aliases introduce a name but do not create a distinct type. Type declarations are **indeterminate** with respect to open/closed — see [Open and Closed Object Types](#open-and-closed-object-types). To create a distinct type, annotate it as nominal via the assurance ledger (see [Structural and Nominal Typing](#structural-and-nominal-typing)).

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

### Variance

Generic types are **invariant by default**. Variance is structurally determined: the type checker examines how a type parameter is used (in covariant, contravariant, or invariant positions) and determines the variance accordingly. In a structural type system, variance is largely moot — structural subtyping handles most cases where variance would matter in a nominal system.

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

Function types are written as `fn(ArgTypes) -> ReturnType`:

```ish
let callback: fn(i32, i32) -> i32 = add
let predicate: fn(String) -> bool = is_valid
```

---

## The `Type` Metatype

Types themselves are representable as values via the `Type` metatype:

```ish
let t: Type = i32
let u: Type = List<String>
```

`Type` is what makes `is_type(value, type)` and `validate(type, value)` possible — their first argument is a value of type `Type`. It also enables generic types to retain full type information at runtime rather than being erased.

---

## Runtime Type Operations

### `is_type(value, type)`

Returns `true` if `value` is of type `type`. In addition to checking the type, this function triggers narrowing — the ledger updates the possible-values entry for the value in the true and false branches.

```ish
let x: i32 | String = get_value()
if is_type(x, i32) {
    // x's possible_values narrowed to i32
}
```

### `validate(type, value)`

Returns `value` cast as `type` if valid. Throws an exception otherwise. The ledger records an `allowed_values` entry for the result.

```ish
let raw = parse_json(input)
let person = validate(Person, raw)
```

---

## Error Handling

Error handling is specified in [docs/spec/errors.md](errors.md). Errors in ish are ordinary objects with `Error` entries managed by the assurance ledger. The error hierarchy (`Error`, `CodedError`, `SystemError`) and throw/catch semantics are documented there.

---

## Interaction with the Assurance Ledger

Many behaviors traditionally considered part of the "type system" are actually assurance ledger features in ish. The type system specifies *what values are valid*; the ledger specifies *what checks are performed, when, and how strictly*.

The following features are configured via the active standard and documented in [docs/spec/assurance-ledger.md](assurance-ledger.md):

| Feature | Type system role | Ledger role |
|---------|-----------------|-------------|
| Type annotations | Defines allowed values | `type_annotations`: whether annotations are required |
| Type checking | Defines compatibility rules | `type_audit`: when checking occurs (runtime vs. build) |
| Null safety | `null` is a type in unions | `null_safety`: whether nullable types must be explicit |
| Mutability | — | `immutability`: whether mut/immut must be declared |
| Integer overflow | — | `overflow`: wrapping/panicking/saturating behavior |
| Numeric precision | Default numeric type | `numeric_precision`: whether exact types are required |
| Implicit conversions | Safe widening rules | `implicit_conversions`: allow or deny |
| Open/closed objects | Structural compatibility | `open_closed_objects`: whether annotation is required |

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
| Intersection types | Merged struct (or `never` if incompatible)                  |
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

- [ ] **Open vs. closed object type syntax.** Semantic model is defined (`@[Open]`/`@[Closed]` entries), but inline type-declaration syntax for indicating open or closed has not been finalized.
- [ ] **Property mutability syntax.** Individual properties can be mutable or immutable, but the annotation syntax has not been defined.
- [ ] **Method syntax on object types.** Object types can include function-typed properties (methods), but the syntax has not been defined.

### Union Types

- [ ] **Discriminated unions — full specification.** What constitutes a discriminant property? How does pattern matching / exhaustive switching work?
- [ ] **Union type flattening.** Is `(A | B) | C` the same as `A | B | C`? Are nested unions automatically flattened?

### Type Widening

- [ ] **Widening rules are not specified.** Does `let mut x = 5` widen to `i32`? `f64`? Does `let mut s = "hello"` widen to `String`? Does `let mut b = true` widen to `bool`? What triggers widening?

### Generic Types

- [ ] **Generic function types.** How do type parameters work in function type signatures?
- [ ] **Overloaded function types.** Can a function have multiple type signatures?

### Runtime Type Operations

- [ ] **Performance implications of `validate`.** Validating deeply nested object types at runtime could be expensive. Are there guidelines or lazy validation strategies?
- [ ] **Custom type guard syntax.** Custom type guards are confirmed as deferred. Built-in `is_type()` covers the immediate need.

### Rust Mapping

- [ ] **Union type representation details.** How are variant names generated? How does this interact with pattern matching on the Rust side?
- [ ] **Object representation selection.** Rules for choosing between struct, enum, and `HashMap<String, Value>` need cross-referencing with the polymorphism spec.
- [ ] **`undefined` Rust mapping.** The Rust representation of `undefined` (for open object property access) needs to be specified.

### The `Type` Metatype

- [ ] **First-class vs. restricted.** Should `Type` be a full first-class type or restricted to specific patterns?
- [ ] **Runtime type construction.** Can new types be constructed at runtime?
- [ ] **Type reflection.** Can code inspect the structure of a `Type` value at runtime?
- [ ] **Rust mapping for `Type`.** What is the Rust representation of a `Type` value?

### Type Compatibility and Assignability

- [ ] **Subtype / assignability rules are not formalized.** When is type A assignable to type B? (Implementation in progress — see type compatibility checking in the ledger engine.)
- [ ] **Coercion rules.** Are there any implicit coercions beyond configurable numeric conversions?

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/errors.md](errors.md)
- [docs/spec/assurance-ledger.md](assurance-ledger.md)
- [docs/architecture/vm.md](../architecture/vm.md)
- [docs/user-guide/types.md](../user-guide/types.md)
- [docs/ai-guide/orientation.md](../ai-guide/orientation.md)
