# ish Module System — Outstanding Issues

Remaining open questions, missing details, and an evaluation of the module system described in [MODULES.md](MODULES.md).

---

## 1. Project Configuration

- [ ] **Configuration file format.** The spec states the format is TBD. Key questions:
  - What format will it use (TOML like Rust, JSON, YAML, ish-native)?
  - What fields are required (name, version, dependencies, entry point)?
  - Can configuration files inherit from or extend other configurations?
  - How are dependency version constraints expressed (semver ranges, exact pins, git refs)?

## 2. Module Mapping

- [ ] **Root module.** Is there a distinguished root module (e.g., `main.ish`, `lib.ish`, or `mod.ish`)? How does the build tool identify the entry point?
- [ ] **Directory modules.** How are directories themselves treated? In Rust, `foo/mod.rs` or `foo.rs` defines the module `foo`. What is the ish equivalent?
- [ ] **`mod` directive semantics.** The spec mentions a `mod` directive for declaring modules outside the directory tree, but does not specify:
  - The syntax of the `mod` directive.
  - Where `mod` directives can appear (only in the root module? any module?).
  - Whether `mod` can alias a module to a different path.
  - Whether `mod` can declare inline modules (module bodies within a file, as in Rust).

## 3. Visibility System

- [ ] **Visibility interaction with re-exports.** When a symbol is re-exported via `pub use`, does the re-export's visibility override the original, or must it be no broader than the original?
- [ ] **`pub(in path)` semantics.** What constitutes a valid path? Must it be an ancestor of the current module, or can it be any module in the project?
- [ ] **Default visibility for different declarations.** Are all declaration types (functions, types, constants, modules) `pub(self)` by default, or do some default to broader visibility?
- [ ] **Visibility of module-level items vs. nested items.** Do items declared inside a function or block have the same visibility options as module-level items?

## 4. Import System

- [ ] **`use` directive syntax.** The spec does not show the syntax for `use`. Questions:
  - Is it `use a::b::c;` (Rust-style), `import a.b.c` (Java/Python-style), or something else?
  - Are glob imports supported (e.g., `use a::b::*`)?
  - Are selective imports supported (e.g., `use a::b::{c, d}`)?
  - Can imports be renamed (e.g., `use a::b::c as d`)?
- [ ] **Relative vs. absolute paths.** Can `use` directives reference modules with relative paths (e.g., `use super::sibling`), or only absolute paths from the project root?
- [ ] **Conditional imports.** Can imports be conditional on encumbrance level or platform?

## 5. Circular Dependency Enforcement

- [ ] **Granularity.** Is the circular dependency prohibition at the module level, the package level, or both?
- [ ] **Detection mechanism.** Is this enforced at parse time, build time, or runtime (for interpreted mode)?
- [ ] **Error reporting.** When a cycle is detected, how is the cycle reported to the developer? Is the full cycle path shown?

## 6. Package Encodings

- [ ] **Annotated AST format.** What serialization format is used for the annotated AST (binary, JSON, a custom format)? Is the format versioned?
- [ ] **Object code ABI stability.** Is there a stable ABI for compiled packages, or must all packages in a dependency tree be compiled together?
- [ ] **Cross-compilation details.** How does a developer request cross-compilation? Is there a target triple system (like Rust's)?
- [ ] **Mixed-encoding dependencies.** Can a project depend on a package available only as an annotated AST and another available only as object code? How does the build tool resolve this?

## 7. Dynamic Linking Interface

- [ ] **Index function contract.** What is the exact data structure returned by the index function? How are function signatures represented?
- [ ] **Value object format.** What is the layout of the value object passed through the shim? Is it the same `IshValue` type used in the prototype's `ish-runtime` crate?
- [ ] **Error handling across the shim boundary.** How are errors (panics, exceptions, result types) propagated through the shim?
- [ ] **Parent shim semantics.** The parent shim allows callbacks into the ish shell — what symbols are accessible through it? Is it the full runtime environment, or a restricted subset?
- [ ] **Versioning the dynamic interface.** How is backward compatibility maintained when the shim protocol evolves?

## 8. Package Distribution

- [ ] **Git-based dependency resolution.** How are transitive dependencies resolved? Is there a lock file mechanism?
- [ ] **OCI/ORAS registry details.** What metadata is stored alongside the compiled module (target architecture, ish version, encumbrance level, dependency manifest)?
- [ ] **Dependency conflict resolution.** How are diamond dependencies handled (A depends on B and C, both of which depend on different versions of D)?
- [ ] **Private/authenticated registries.** Is there a mechanism for hosting private packages?
- [ ] **Security and verification.** How are packages authenticated? Is there signature verification or checksum validation?

## 9. Interaction with Encumbrance

- [ ] **Encumbrance boundaries at module edges.** MODULES.md does not address how encumbrance interacts with the module system:
  - Can a streamlined module import an encumbered module and vice versa?
  - What checks are performed at the boundary between differently-encumbered modules?
  - Is encumbrance metadata recorded in the package encoding?
- [ ] **Per-module encumbrance configuration.** Can encumbrance level be set per module, or only per project?

## 10. Interaction with Execution Configurations

- [ ] **Thin shell module loading.** Can the thin shell (interpreted mode) load compiled modules? [EXECUTION_CONFIGURATIONS.md](EXECUTION_CONFIGURATIONS.md) says yes (fat shell), but the interop semantics are not fully specified in MODULES.md.
- [ ] **Module loading at runtime vs. build time.** In compiled mode, are all modules resolved at build time, or can modules be loaded dynamically at runtime?

## 11. Standard Library Packaging

- [ ] **Is the standard library a module?** If so, how is it distributed — bundled with the ish installation, or fetched as a dependency?
- [ ] **Prelude / auto-imports.** Are there modules or symbols that are automatically available without a `use` directive?

---

## 12. Evaluation

### 12.1 Pros

1. **Familiar model.** Following Rust's module system closely gives ish a well-understood and battle-tested design. Developers coming from Rust will find it immediately familiar, and the approach has proven scalable to large codebases.
2. **Fine-grained visibility.** The five-level visibility system (`pub(self)`, `pub(super)`, `pub(in path)`, `pub(project)`, `pub(global)`) is more expressive than most languages. It allows precise encapsulation — libraries can expose a public API while keeping internal helpers private, without needing an additional convention layer.
3. **Multiple package encodings.** Offering annotated AST, static object code, and dynamic object code gives consumers flexibility. The AST encoding is particularly valuable — it allows consumer-side optimization and cross-compilation without requiring the original source.
4. **Phased distribution strategy.** Starting with git-based deps, moving to OCI/ORAS, and eventually building a dedicated registry is pragmatic. It avoids premature infrastructure investment while keeping a clear upgrade path.
5. **Re-export support.** Combining `pub` and `use` for re-exports allows library authors to present a clean public API while internally organizing code however they prefer. This is a proven pattern from Rust.
6. **Dynamic linking interface.** The index/shim design for dynamic linking is practical — it enables compiled modules to be loaded into the interpreter at runtime, bridging the gap between the thin shell and compiled performance.

### 12.2 Cons

1. **Rust familiarity assumed.** Closely following Rust's module system is an advantage for Rust developers but may be confusing for developers from other backgrounds. Rust's module system is widely considered one of the language's steeper learning curves. For a language that aims to be "approachable by anyone who knows at least one other programming language," this is a tension.
2. **No circular dependencies is strict.** While circular dependency prohibition simplifies the build model, it can be painful in practice — especially for large projects where two modules naturally reference each other. Some languages allow circular dependencies within a package while prohibiting them across packages, which may be a more practical middle ground.
3. **Dynamic linking complexity.** The shim-based dynamic linking interface introduces marshaling overhead and a complex protocol (index function, value objects, parent shim callbacks). This adds a significant surface area for bugs and performance issues at the FFI boundary.
4. **Distribution strategy is vague.** The phased plan (git → OCI/ORAS → registry) outlines the direction but leaves most details unspecified — dependency resolution, versioning, conflict handling, security, and lock files are all open. These are essential for a usable package ecosystem.
5. **No specification of `mod` or `use` syntax.** The two most important directives in the module system — how you declare modules and how you import symbols — have no concrete syntax. This makes it hard to evaluate whether the system will feel ergonomic in practice.
6. **Encumbrance interaction is unaddressed.** The module system spec does not discuss how differently-encumbered modules interoperate, even though this is a central challenge for a language with configurable encumbrance. Cross-boundary safety guarantees are critical and currently undefined.
7. **No versioning or compatibility story.** The spec mentions dependency versions in the config file but does not describe how version compatibility is determined, how breaking changes are detected, or whether there is a concept of API stability.

### 12.3 Overall Assessment

The module system provides a solid structural foundation by building on Rust's proven design. The multiple package encodings and phased distribution strategy are pragmatic choices that give the project room to evolve. However, the specification is currently a high-level sketch — the most important developer-facing details (syntax, dependency resolution, encumbrance interaction, versioning) are undefined. Fleshing these out is necessary before the module system can be implemented or meaningfully evaluated for ergonomics.
