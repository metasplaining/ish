---
title: ish Open Questions
category: project
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/types.md, docs/spec/modules.md, docs/spec/reasoning.md, docs/spec/assurance-ledger.md, docs/spec/execution.md, docs/spec/memory.md, docs/spec/polymorphism.md, docs/spec/syntax.md, docs/spec/concurrency.md]
---

# ish Open Questions

Consolidated index of all open questions, organized by topic. Each question also appears in the `## Open Questions` section of the relevant specification file. Cross-links go both ways.

---

## Syntax and Language Surface

See also [docs/spec/syntax.md](../spec/syntax.md).

- [x] **~~No syntax description or examples.~~** Resolved — ish is a C-family language with braces, `fn` for functions, `let` for variables, newline-terminated statements, `and`/`or`/`not` for logical operators, and no parentheses around conditions. See [docs/spec/syntax.md](../spec/syntax.md) and [language syntax proposal](proposals/language-syntax.md).
- [x] **~~No description of control flow.~~** Resolved — `if`/`else`, `while`, `for x in iter`, `break`/`continue`. No C-style `for` loop. `match` keyword reserved for future pattern matching. See [docs/spec/syntax.md](../spec/syntax.md).
- [x] **~~No description of functions.~~** Resolved — `fn` for declaration, TypeScript-style arrow functions `(x) => expr` for lambdas, closures supported, default parameters supported. Function types use `fn(Args) -> Ret`. See [docs/spec/syntax.md](../spec/syntax.md).
- [x] **~~No description of error handling.~~** Resolved — ish uses thrown exceptions with try/catch/finally, with blocks, defer, and the `?` operator. See [docs/user-guide/error-handling.md](../user-guide/error-handling.md) and [proposal](proposals/error-handling.md).
- [x] **~~Error handling open questions from proposal.~~** Resolved — `with` identifies close method (TBD convention vs. annotation). Defer is function-scoped per [defer-scoping proposal](proposals/defer-scoping.md). The `?` operator is syntactic sugar for throw-on-error. See [docs/spec/syntax.md](../spec/syntax.md).
- [x] **~~No description of concurrency / parallelism.~~** Resolved — cooperative multitasking via async/await on Tokio, parallelism via parallel shims, guaranteed yield mechanism, two-thread shell/VM architecture. See [docs/spec/concurrency.md](../spec/concurrency.md) and [concurrency proposal](proposals/concurrency.md).

---

## Type System

See also [docs/spec/types.md — Open Questions](../spec/types.md#open-questions).

### Naming Convention

- [ ] **Capitalization of special types.** Should `void`, `null`, `undefined`, `never` match lowercase primitives or capitalized complex types?

### Object Types — Syntax Gaps

- [ ] **Open vs. closed object type syntax.** No syntax for declaring open or closed.
- [ ] **Property mutability syntax.** Annotation syntax not defined.
- [ ] **Method syntax on object types.** Syntax not defined.

### Union Types

- [ ] **Discriminated unions — full specification.** What constitutes a discriminant property? Exhaustive switching?
- [ ] **Union type flattening.** Is `(A | B) | C` the same as `A | B | C`?

### Type Widening

- [ ] **Widening rules not specified.** What triggers widening? What types are widened to?

### Generic Types

- [ ] **Variance.** Covariant, contravariant, or invariant?

### Function Types

- [ ] **Function type syntax.** Not decided.
- [ ] **Generic function types.** Type parameters in function type signatures?
- [ ] **Overloaded function types.** Multiple type signatures?

### Runtime Type Operations

- [ ] **Performance implications of `validate`.** Guidelines for deeply nested validation?
- [ ] **Custom type guard syntax.** Not defined.

### Rust Mapping

- [ ] **Union type representation details.** Variant name generation? Pattern matching interaction?
- [ ] **Object representation selection.** Cross-reference with polymorphism spec.
- [ ] **`undefined` Rust mapping.** Not specified.

### Error Types

- [ ] **`Error` type status.** First-class or standard library type?
- [ ] **Exception model details.** Typed exceptions? Function signature declarations?

### Assurance Level Configuration

- [ ] **Per-variable assurance level syntax.** Not designed.

### The `Type` Metatype

- [ ] **First-class vs. restricted.** Full first-class type or restricted?
- [ ] **Runtime type construction.** Can new types be constructed at runtime?
- [ ] **Type reflection.** Can code inspect `Type` values at runtime?
- [ ] **Rust mapping for `Type`.** Trait object? Enum? Type ID with registry?

### Type Compatibility and Assignability

- [ ] **Subtype / assignability rules not formalized.** When is type A assignable to type B?
- [ ] **Coercion rules.** Any implicit coercions beyond numeric conversions?

---

## Module System

See also [docs/spec/modules.md — Open Questions](../spec/modules.md#open-questions).

### Project Configuration

- [ ] **Configuration file format.** Format, required fields, inheritance, version constraints.

### Module Mapping

- [ ] **Root module.** Distinguished root module? Entry point identification?
- [ ] **Directory modules.** How are directories treated?
- [ ] **`mod` directive semantics.** Syntax, location, aliasing, inline modules.

### Visibility System

- [ ] **Visibility interaction with re-exports.** Override or restrict?
- [ ] **`pub(in path)` semantics.** Valid paths?
- [ ] **Default visibility for different declarations.** All `pub(self)`?
- [ ] **Visibility of nested items.** Same options as module-level?

### Interface Files

- [ ] **Interface freeze must capture fully analyzed signatures.** When `ish interface freeze` generates a `.ishi` file, the signatures must reflect the fully analyzed form of each declaration — for example, a function that is implicitly yielding must be written as explicitly yielding in the `.ishi` file. The analysis requirements and the format for encoding analyzed attributes in `.ishi` are not yet specified.

### Import System

- [ ] **`use` directive syntax.** Style, glob imports, selective imports, renaming.
- [ ] **Relative vs. absolute paths.** Relative path support?
- [ ] **Conditional imports.** Conditional on assurance level or platform?

### Circular Dependency Enforcement

- [ ] **Granularity.** Module level, package level, or both?
- [ ] **Detection mechanism.** Parse time, build time, or runtime?
- [ ] **Error reporting.** Full cycle path?

### Package Encodings

- [ ] **Annotated AST format.** Serialization format? Versioned?
- [ ] **Object code ABI stability.** Stable ABI?
- [ ] **Cross-compilation details.** Target triple system?
- [ ] **Mixed-encoding dependencies.** Different encodings in one dependency tree?

### Dynamic Linking Interface

- [ ] **Index function contract.** Data structure returned?
- [ ] **Value object format.** Layout?
- [ ] **Error handling across the shim boundary.** Propagation?
- [ ] **Parent shim semantics.** Accessible symbols?
- [ ] **Versioning the dynamic interface.** Backward compatibility?

### Package Distribution

- [ ] **Git-based dependency resolution.** Transitive deps? Lock file?
- [ ] **OCI/ORAS registry details.** Metadata stored?
- [ ] **Dependency conflict resolution.** Diamond dependencies?
- [ ] **Private/authenticated registries.** Private package hosting?
- [ ] **Security and verification.** Authentication? Signatures? Checksums?

### Assurance Level Interaction

- [ ] **Assurance level boundaries at module edges.** Cross-boundary checks? Metadata in package encoding?
- [ ] **Per-module assurance level configuration.** Per module or per project?

### Execution Configuration Interaction

- [ ] **Thin shell module loading.** Interop semantics.
- [ ] **Module loading at runtime vs. build time.** Dynamic loading in compiled mode?

### Standard Library Packaging

- [ ] **Is the standard library a module?** Distribution method?
- [ ] **Prelude / auto-imports.** Automatically available symbols?

---

## Reasoning System

See also [docs/spec/reasoning.md — Open Questions](../spec/reasoning.md#open-questions).

- [ ] **Plugin interface.** AST node type? State parameter? Cross-proposition access? Statefulness?
- [ ] **Annotation syntax.** Attribute/decorator/inline? Valid locations? Assertion vs. query distinction?
- [ ] **Assurance level interaction.** Behavior in low-assurance mode? Independent configurability?
- [ ] **Assurance ledger relationship.** Standards as propositions? Pre/post-conditions?
- [ ] **Compound proposition semantics.** Implication? Quantification? Variable references?
- [ ] **Error reporting.** Message format? Custom messages? Failure explanations?
- [ ] **Plugin registration.** Declarative or imperative? Scope? Naming conflicts? Third-party plugins?
- [ ] **Evaluation model.** Fixed order or fixpoint? Circular dependencies? Lazy or eager?
- [ ] **Scope of reasoning.** Inter-procedural? Separate compilation interaction?
- [ ] **Performance.** Bounding analysis time.
- [ ] **Soundness.** Undecidable property policy. Over/under-approximation.
- [ ] **Plugin sandboxing.** Termination. Side effects.
- [ ] **Bootstrapping.** Built-in vs. plugin boundary.
- [ ] **LSP integration.** Incremental analysis. Partial results.

---

## Assurance Ledger

See also [docs/spec/assurance-ledger.md — Open Questions](../spec/assurance-ledger.md#open-questions).

- [x] **~~What happens when an agreement is violated at build time vs. runtime?~~** Resolved — audit states (optional/live/pre) determine when and how discrepancies are reported. See [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md).
- [x] **~~Syntax for marking/unmarking features at project, file, function, or variable level.~~** Resolved — `@standard[name]` applies standards to scopes; `@[entry(params)]` annotates individual items. See [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md).
- [x] **~~Agreement interaction with boundaries between differently-encumbered code.~~** Resolved — cross-scope standard interactions define boundary rules. See [docs/spec/assurance-ledger.md § Cross-Scope Standard Interactions](../spec/assurance-ledger.md#cross-scope-standard-interactions).

---

## Execution Configurations

See also [docs/spec/execution.md — Open Questions](../spec/execution.md#open-questions).

### Thin Shell

- [ ] **Footprint metrics.** Actual disk and memory footprint.
- [ ] **Relationship to low-assurance ish.** Can high-assurance code be used from the thin shell?

### Fat Shell

- [ ] **Module generation process.** What does optimization involve?
- [ ] **Module compatibility.** Compatibility across ish versions.

### Compiler

- [ ] **Which modules constitute the compiler?** Minimal module set per output type.
- [ ] **Compilation target.** Native code, Rust source, LLVM IR?
- [ ] **Local code loading.** Compiled/interpreted interop.
- [ ] **Incremental compilation.** Supported?

### Executable

- [ ] **Executable contents.** What's included beyond user code?
- [ ] **Runtime requirements.** Dependencies?
- [ ] **Cross-compilation.** Cross-platform executables?

### Assurance Level Relationship

- [ ] **Configuration selection vs. assurance level.** Minimum assurance level for executables? Mixed assurance levels?

### Transitions

- [ ] **Fat shell → compiler transition.** Seamless or behavioral change?
- [ ] **Interactive compilation.** Supported?

---

## Memory Management

See also [docs/spec/memory.md — Open Questions](../spec/memory.md#open-questions).

- [ ] **Developer-facing controls.** Explicit model selection? Annotations?
- [ ] **Ownership and borrowing rules.** Lifetime annotations? Borrow checker?
- [ ] **Reference cycle handling.** Weak references?

---

## Polymorphism

See also [docs/spec/polymorphism.md — Open Questions](../spec/polymorphism.md#open-questions).

- [ ] **Developer-facing interface.** How to define interfaces/traits? How to implement them?
- [ ] **Strategy selection rules.** Constraints per strategy? Developer override?
- [ ] **Interaction with type system.** Generics? Trait bounds?

---

## Standard Library

- [ ] **No mention of a standard library.** Collections, string handling, I/O, math, date/time?

---

## Build System and Tooling

- [ ] **Build process.** Build command? Output format? Mixed-assurance builds?
- [ ] **Debugging support.** Source maps? Debugger integration? REPL beyond the shell?
- [ ] **Testing support.** Built-in test framework? Test runner?
- [ ] **IDE/editor support.** LSP? Syntax highlighting?

---

## Internals

- [ ] **AST primitives.** Loops, match/switch, return, struct/type definitions, imports, error handling, annotations — are these all AST nodes?
- [ ] **Linker role.** Loading modules at runtime? Resolving imports?
- [x] **~~Code analyzer scope.~~** Partially resolved — the stub analyzer produces yielding classification (`Some(true)` or `Some(false)`) for every function at declaration time. Further analysis passes (type inference, purity, etc.) are future work. See [docs/architecture/vm.md](../architecture/vm.md) § Code Analyzer.
- [ ] **Rust generator output.** Idiomatic Rust? Human-readable?
- [ ] **Bootstrapping strategy.** What is written in Rust vs. ish? When does self-hosting begin?

---

## Semantics and Edge Cases

- [ ] **Mutability model.** Mutable by default? Immutable by default? Configurable?
- [ ] **Null/undefined handling.** Option types interaction?
- [ ] **Operator semantics.** Arithmetic, comparison, logical, bitwise — behavior across types?
- [ ] **String model.** UTF-8? UTF-16? Internal representation?
- [ ] **Numeric model.** Fixed-width? Arbitrary precision? Variation with assurance level?

---

## Interoperability

- [ ] **FFI.** Can ish call C/Rust libraries? Can other languages call ish?
- [ ] **Compiled ↔ interpreted interop.** Coexistence in the same program?

---

## Concurrency

See also [docs/spec/concurrency.md — Open Questions](../spec/concurrency.md#open-questions).

- [ ] **Program exit with running futures.** When the main program finishes but spawned futures are still running, should the runtime wait for them, cancel them, or apply a timeout?
- [ ] **FFI and blocking.** If ish supports arbitrary C/Rust FFI calls, blocking may need reintroduction — a blocking FFI call on the LocalSet thread would stall all ish tasks.
- [x] **~~Function yielding categorization at declaration.~~** Resolved — the stub code analyzer classifies all functions as yielding or unyielding at declaration time by walking the function body for yielding nodes (await, spawn, yield, shell commands, command substitution) and checking callee yielding status. `await` and `spawn` now enforce classification via E012/E013. See [docs/architecture/vm.md](../architecture/vm.md) § Code Analyzer and the [stubbed-analyzer plan](plans/stubbed-analyzer.md).
- [ ] **Builtin replacement by standard library.** All builtins are temporary scaffolding. They should be replaced by standard library functions when the standard library is available. Parallel builtins should become parallel stdlib functions. See concurrency-correctness proposal Decision 7.

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
- [docs/spec/types.md](../spec/types.md)
- [docs/spec/modules.md](../spec/modules.md)
- [docs/spec/reasoning.md](../spec/reasoning.md)
- [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md)
- [docs/spec/execution.md](../spec/execution.md)
- [docs/spec/memory.md](../spec/memory.md)
- [docs/spec/polymorphism.md](../spec/polymorphism.md)
- [docs/spec/concurrency.md](../spec/concurrency.md)
