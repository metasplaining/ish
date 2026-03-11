# ish — Requirements TODO

A detailed list of requirements that are missing, ambiguous, or need further work.

---

## 1. Syntax and Language Surface

- [ ] **No syntax description or examples.** The README describes what ish can do, but never shows what ish code looks like. Even a small "Hello World" example would ground the reader. Key questions:
  - What is the basic expression and statement syntax?
  - Is ish C-family, ML-family, Lisp-family, or something novel?
  - What are the delimiters (braces, indentation, keywords)?
  - How are comments written?
- [ ] **No description of the type system.** Types are mentioned as a standard feature but never defined:
  - What primitive types exist (integers, floats, strings, booleans, etc.)?
  - Are there user-defined types (structs, classes, enums, unions)?
  - Is the type system structural, nominal, or both?
  - How do types interact with the streamlined ↔ encumbered continuum? (e.g., optional type annotations in streamlined mode, mandatory in encumbered mode?)
- [ ] **No description of control flow.** If/else is mentioned as an AST primitive, but:
  - What looping constructs exist (for, while, loop, iterators)?
  - Is there pattern matching / switch / match?
  - How does early return work?
- [ ] **No description of functions.** Functions are mentioned as a standard feature, but:
  - What is the function declaration syntax?
  - Are functions first-class values? Closures?
  - How are parameters and return values typed (or not, in streamlined mode)?
  - Are there anonymous / lambda functions?
  - How does overloading work, if at all?
- [x] **Modules / imports / namespaces are now described in [MODULES.md](MODULES.md).** Covers file-to-module mapping, visibility directives, `use`/`pub` for imports and re-exports, project configuration, package encodings, and distribution strategy. Remaining open questions are tracked in [MODULES_TODO.md](MODULES_TODO.md).
- [ ] **No description of error handling.** Exceptions? Result types? Panics? How does error handling vary across the encumbrance continuum?
- [ ] **No description of concurrency / parallelism.** Is there async/await? Threads? Channels? Actors? How does this interact with memory management models?

## 2. "Agreement" System

- [x] **"Agreement checks" is now defined in [AGREEMENT.md](AGREEMENT.md).** Agreement is a concept from linguistics: certain parts of a program must be consistent with each other (analogous to subject-verb agreement in natural language). What language features require agreement is configurable via the encumbrance system — these are called "marked" features. The list of configurable marked features is maintained in AGREEMENT.md.

Remaining open questions:
  - [ ] What happens when an agreement is violated at build time vs. runtime?
  - [ ] What is the syntax for marking/unmarking features at the project, file, function, or variable level?
  - [ ] How does agreement interact with boundaries between differently-encumbered code?

## 3. Polymorphism

- [ ] **Developer-facing interface is undefined.** The README explains five implementation strategies, but not what the developer writes. How does a developer define an interface/trait/protocol? How does a developer implement it for a type? What does polymorphic code look like syntactically?
- [ ] **Strategy selection rules are vague.** "The language processor will choose the highest-performing alternative for which all constraints are met" — what exactly are the constraints for each strategy? Can the developer override the automatic selection?
- [ ] **Interaction with type system is undefined.** How do generics / type parameters work? Are there trait bounds?

## 4. Memory Management

- [ ] **Developer-facing controls are undefined.** Can a developer explicitly choose a memory management model for a variable? Or is it always inferred? What annotations or syntax exist to influence the choice?
- [ ] **Ownership and borrowing rules are not specified.** The heap (owned) model is clearly Rust-inspired, but:
  - Are there borrowing / lifetime annotations?
  - Is there a borrow checker in encumbered mode?
  - What are the rules for passing owned values to functions?
- [ ] **Reference cycle handling for reference counting.** Reference counting cannot collect cycles. Is this addressed (e.g., weak references)?

## 5. Reasoning About Code

- [x] **Now described in [REASONING.md](REASONING.md).** Covers the shared reasoning tool, atomic and compound propositions, and the plugin system. Remaining open questions are tracked in [REASONING_TODO.md](REASONING_TODO.md).

## 6. Flexible Configuration

- [ ] **Configuration mechanism is undefined.** How is the encumbrance level configured?
  - Is there a project-level config file? What format?
  - What is the syntax for configuring encumbrance at the function or variable level? Annotations? Attributes? Pragmas?
  - What is the full list of independently configurable features?
- [ ] **Defaults are unspecified.** What is the default encumbrance level for a new project? Can defaults be inherited?
- [ ] **Interaction between differently-encumbered code is undefined.** What happens when streamlined code calls encumbered code and vice versa? Are there safety guarantees at the boundary?

## 7. Standard Library

- [ ] **No mention of a standard library.** What built-in functionality ships with ish?
  - Collections (arrays, lists, maps, sets)?
  - String handling?
  - I/O (file, network, stdio)?
  - Math?
  - Date/time?

## 8. Build System and Tooling

- [ ] **Build process is vague.** The README mentions a build step for encumbered ish and a Rust code generator, but:
  - What does the build command look like?
  - What is the output? A native binary? Rust source that is then compiled?
  - How are mixed-encumbrance projects built?
- [x] **Package management and distribution are now described in [MODULES.md](MODULES.md).** Covers project configuration, package encodings, and a phased distribution strategy (git deps → OCI/ORAS → dedicated registry). Remaining open questions are tracked in [MODULES_TODO.md](MODULES_TODO.md).
- [ ] **No mention of debugging support** (source maps, debugger integration, REPL beyond the shell).
- [ ] **No mention of testing support** (built-in test framework? test runner?).
- [ ] **No mention of IDE/editor support** (LSP, syntax highlighting).

## 9. Internals

- [ ] **AST primitives are only partially listed.** The README lists if/else, variable declaration/assignment, function declaration/call. What about: loops, match/switch, return, struct/type definitions, imports, error handling constructs, annotations?
- [ ] **Linker role is ambiguous.** "Orchestrates the linking of code into the virtual machine" — does this mean loading modules at runtime? Resolving imports? This is distinct from what "linker" traditionally means (combining object files).
- [ ] **Code analyzer scope is undefined.** "Analyzes ASTs and marks them up with metadata" — what metadata? Type information? Lifetime analysis? Optimization hints? Agreement checks? All of the above?
- [ ] **Rust generator output is unspecified.** Does it generate idiomatic Rust? Is the generated code meant to be human-readable or purely for compilation? What Rust features does it target?
- [ ] **Bootstrapping strategy needs detail.** "Just enough of ish is written in Rust to make streamlined ish work" — what specifically is written in Rust vs. ish? At what point in the project does self-hosting begin?

## 10. Semantics and Edge Cases

- [ ] **Mutability model is not described.** Are variables mutable by default? Immutable by default? Configurable?
- [ ] **Null/nil/undefined handling is not described.** Does ish have null? Option types? How are missing values represented?
- [ ] **Operator semantics are undefined.** Arithmetic, comparison, logical, bitwise — what operators exist and how do they behave across types?
- [ ] **String model is undefined.** UTF-8? UTF-16? Byte strings? How are strings represented internally, and does this vary with encumbrance?
- [ ] **Numeric model is undefined.** Fixed-width integers? Arbitrary precision? Floating point? Does this vary with encumbrance?

## 11. Interoperability

- [ ] **FFI (Foreign Function Interface) is not mentioned.** Can ish call C/Rust/other libraries? Can other languages call ish?
- [ ] **Compiled ish ↔ interpreted ish interop is undefined.** Can compiled and interpreted ish code coexist in the same program? How?

## 12. Documentation and Onboarding

- [ ] **No getting-started guide or installation instructions.**
- [ ] **No language reference or specification.**
- [ ] **No examples directory or tutorial.**
- [ ] **No contributing guide.**
