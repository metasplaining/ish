# ish Type System — Outstanding Issues

Remaining open questions and items that need further work in TYPES.md.

---

## 1. Naming Convention

- [ ] **Capitalization of special types.** The spec uses lowercase (`void`, `null`, `undefined`, `never`) but this convention has not been finalized. Should these match the lowercase primitives (`bool`, `i32`) or the capitalized complex types (`String`, `List`)?

## 2. Object Types — Syntax Gaps

- [ ] **Open vs. closed object type syntax.** The concept is defined (open objects allow extra properties, closed do not) but there is no syntax for declaring an object type as open or closed.
- [ ] **Property mutability syntax.** Individual properties can be mutable or immutable, but the annotation syntax has not been defined.
- [ ] **Method syntax on object types.** Object types can include function-typed properties (methods), but the syntax has not been defined.

## 3. Union Types

- [ ] **Discriminated unions — full specification.** Discriminated unions are confirmed as supported, but the details need further work: What constitutes a discriminant property? How does pattern matching / exhaustive switching work?
- [ ] **Union type flattening.** Is `(A | B) | C` the same as `A | B | C`? Are nested unions automatically flattened? This needs to be decided.

## 4. Type Widening

- [ ] **Widening rules are not specified.** The spec describes what widening is and that it varies with encumbrance, but the concrete rules are absent:
  - Does `let mut x = 5` widen to `i32`? `f64`? The widened numeric type in streamlined mode?
  - Does `let mut s = "hello"` widen to `String`?
  - Does `let mut b = true` widen to `bool`?
  - What triggers widening — mutability alone? Re-assignment? Something else?

## 5. Generic Types

- [ ] **Variance.** Are generic types covariant, contravariant, or invariant in their type parameters? e.g., is `List<Dog>` assignable to `List<Animal>`? This needs to be thought through.

## 6. Function Types

- [ ] **Function type syntax.** Functions are first-class values, closures capture by reference, but the syntax for writing function types (e.g., `(i32, String) -> bool`) has not been decided.
- [ ] **Generic function types.** How do type parameters work in function type signatures?
- [ ] **Overloaded function types.** Can a function have multiple type signatures?

## 7. Runtime Type Operations

- [ ] **Performance implications of `validate`.** Validating deeply nested object types at runtime could be expensive. Are there guidelines, limits, or lazy validation strategies?
- [ ] **Custom type guard syntax.** Custom type guards are confirmed as supported, but the syntax has not been defined.

## 8. Rust Mapping

- [ ] **Union type representation details.** Unions map to Rust enums, but: how are variant names generated? How does this interact with pattern matching on the Rust side?
- [ ] **Object representation selection.** The rules for choosing between struct, enum, and `HashMap<String, Value>` need to be cross-referenced with the polymorphism spec.
- [ ] **`undefined` Rust mapping.** The Rust representation of `undefined` (for open object property access) needs to be specified.

## 9. Error Types

- [ ] **`Error` type status.** It is TBD whether `Error` is a first-class type or a standard library type.
- [ ] **Exception model details.** Thrown exceptions are confirmed, but: how are exceptions typed? Can a function signature declare what exceptions it may throw? How do exceptions interact with the type system in encumbered mode?

## 10. Encumbrance Configuration

- [ ] **Per-variable encumbrance syntax.** The README says encumbrance can be configured per-variable. The syntax for this has not been designed.

## 11. The `Type` Metatype

- [ ] **First-class vs. restricted.** Should `Type` be a full first-class type (can be stored in collections, returned from functions, used as a generic parameter) or should its usage be restricted to specific patterns like `isType` / `validate`?
- [ ] **Runtime type construction.** Can new types be constructed at runtime (e.g., building an object type from a schema)? Or are `Type` values limited to types that exist in the source?
- [ ] **Type reflection.** Can code inspect the structure of a `Type` value at runtime (e.g., enumerate the properties of an object type, check if a type is a union)?
- [ ] **Rust mapping for `Type`.** What is the Rust representation of a `Type` value? A trait object? An enum? A type ID with a metadata registry?

## 12. Type Compatibility and Assignability

- [ ] **Subtype / assignability rules are not formalized.** When is type A assignable to type B? Key cases:
  - Literal type → widened type (e.g., `5` → `i32`)
  - Closed object with extra properties → open object with fewer properties
  - Union type assignability
  - Generic type assignability
  - Nominal type assignability
- [ ] **Coercion rules.** Implicit numeric conversions are configurable, but are there any other implicit coercions? This should be explicitly stated.
