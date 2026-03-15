---
title: ish Module System
category: spec
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/assurance-ledger.md, docs/spec/execution.md, docs/spec/syntax.md]
---

# ish Module System

## Overview

ish's module system follows Rust's module system closely. It governs how code is organized into files, how symbols are shared between files, and how projects are packaged for distribution.

---

## Projects

An ish project is contained in a directory and its subdirectories. A configuration file in the top-level directory specifies the project's package dependencies, including the hosting repository and version for each dependency. This enables build tooling to download and cache all package dependencies automatically.

---

## Modules

Every `.ish` file in the project directory tree defines a module. The module path mirrors the file path — for example, `a/b/c.ish` defines the module `a::b::c`.

A `mod` directive allows modules to be declared that do not follow the directory tree.

---

## Visibility

By default, all symbols are visible only within their own module. Visibility directives control how symbols are exposed:

| Directive      | Meaning                                                        |
|----------------|----------------------------------------------------------------|
| `pub(self)`    | Visible only within the current module (default).              |
| `pub(super)`   | Visible to the parent module.                                  |
| `pub(in path)` | Visible to all modules at or below the specified path.         |
| `pub(project)` | Visible to all modules within the same project.                |
| `pub(global)`  | Visible to all code, including external consumers.             |

Bare `pub` means `pub(global)`. The default visibility is configurable via a standard.

```ish
fn internal_helper() { ... }         // pub(self) — default
pub fn exported() { ... }            // pub(global)
pub(super) fn parent_only() { ... }  // visible to parent module
pub(project) fn project_wide() { ... } // visible within the project
```

## Imports and Re-exports

A `use` directive is required to reference symbols from a different module:

```ish
use std::io
use mylib::utils
```

Combining `pub` and `use` directives allows symbols to be re-exported.

---

## Circular Dependencies

Circular dependencies between modules are not allowed.

---

## Packages

Each project builds into a package. Packages have the following possible encodings:

| Encoding                             | Description                                                                          |
|--------------------------------------|--------------------------------------------------------------------------------------|
| Annotated AST                        | The parsed and analyzed syntax tree, preserving all metadata.                        |
| Object code (static linking)         | Native code compiled for linking into an executable at build time.                   |
| Object code (dynamic linking)        | Native code compiled as a shared library, loaded at runtime.                         |

All encodings are intended to be available, including cross-compiled versions for each supported architecture. In practice, as long as the Annotated AST encoding is available, a consumer can use it to produce any other encoding it needs.

### Dynamic Linking Interface

A dynamically linked package exposes two entry points:

1. **Index function.** The module loader calls this to obtain metadata about all public symbols in the library, including function signatures.
2. **Shim function.** Accepts a symbol name, a value object, and a parent shim, and returns a value object. The shim resolves the symbol to a function in the library, marshals the value object into the expected parameters, calls the function, and marshals the return value back into a value object. The parent shim allows library functions to call back into the ish shell.

---

## Package Distribution

The package distribution strategy is expected to evolve as the language grows. The tentative plan:

1. **Git-based source packages.** Distribute ish source packages via git-based dependencies initially — minimal infrastructure, full control.
2. **OCI/ORAS compiled modules.** Distribute compiled ish modules via OCI/ORAS once the compiled module format stabilizes.
3. **Dedicated registry.** Build a dedicated ish registry once the packaging semantics (assurance levels, module compatibility, execution configuration) are well-defined enough to encode in registry metadata.

---

## Open Questions

Open questions for the module system. See also [docs/project/open-questions.md](../project/open-questions.md#module-system) for a consolidated view.

### Project Configuration

- [ ] **Configuration file format.** What format (TOML, JSON, YAML, ish-native)? What fields are required (name, version, dependencies, entry point)? Can configuration files inherit from or extend other configurations? How are dependency version constraints expressed?

### Module Mapping

- [ ] **Root module.** Is there a distinguished root module (e.g., `main.ish`, `lib.ish`, `mod.ish`)? How does the build tool identify the entry point?
- [ ] **Directory modules.** How are directories themselves treated? What is the ish equivalent of Rust's `foo/mod.rs`?
- [ ] **`mod` directive semantics.** Syntax, where it can appear, whether it can alias or declare inline modules.

### Visibility System

- [ ] **Visibility interaction with re-exports.** Does a re-export's visibility override the original, or must it be no broader?
- [ ] **`pub(in path)` semantics.** Must the path be an ancestor of the current module, or any module in the project?
- [ ] **Default visibility for different declarations.** Are all declaration types `pub(self)` by default?
- [ ] **Visibility of nested items.** Do items inside functions or blocks have the same visibility options as module-level items?

### Import System

- [ ] **`use` directive syntax.** Rust-style `use a::b::c;`, Java-style `import a.b.c`, or something else? Glob imports? Selective imports? Renaming?
- [ ] **Relative vs. absolute paths.** Can `use` directives reference modules with relative paths (e.g., `use super::sibling`)?
- [ ] **Conditional imports.** Can imports be conditional on assurance level or platform?

### Circular Dependency Enforcement

- [ ] **Granularity.** Is the prohibition at the module level, the package level, or both?
- [ ] **Detection mechanism.** Enforced at parse time, build time, or runtime?
- [ ] **Error reporting.** Is the full cycle path shown?

### Package Encodings

- [ ] **Annotated AST format.** What serialization format? Is the format versioned?
- [ ] **Object code ABI stability.** Stable ABI, or must all packages be compiled together?
- [ ] **Cross-compilation details.** Target triple system?
- [ ] **Mixed-encoding dependencies.** Can a project depend on packages with different encodings?

### Dynamic Linking Interface

- [ ] **Index function contract.** Exact data structure returned? How are function signatures represented?
- [ ] **Value object format.** Layout of the value object passed through the shim?
- [ ] **Error handling across the shim boundary.** How are errors propagated?
- [ ] **Parent shim semantics.** What symbols are accessible through the parent shim?
- [ ] **Versioning the dynamic interface.** How is backward compatibility maintained?

### Package Distribution

- [ ] **Git-based dependency resolution.** Transitive dependencies? Lock file mechanism?
- [ ] **OCI/ORAS registry details.** What metadata is stored alongside compiled modules?
- [ ] **Dependency conflict resolution.** How are diamond dependencies handled?
- [ ] **Private/authenticated registries.** Mechanism for hosting private packages?
- [ ] **Security and verification.** Package authentication? Signature verification? Checksum validation?

### Interaction with Assurance Levels

- [ ] **Assurance level boundaries at module edges.** Can a low-assurance module import a high-assurance module and vice versa? What checks are performed at the boundary? Is assurance level metadata recorded in the package encoding?
- [ ] **Per-module assurance level configuration.** Can assurance level be set per module, or only per project?

### Interaction with Execution Configurations

- [ ] **Thin shell module loading.** Interop semantics between interpreted shell and compiled modules.
- [ ] **Module loading at runtime vs. build time.** In compiled mode, can modules be loaded dynamically at runtime?

### Standard Library Packaging

- [ ] **Is the standard library a module?** How is it distributed?
- [ ] **Prelude / auto-imports.** Are there symbols that are automatically available without a `use` directive?

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](../architecture/overview.md)
- [docs/user-guide/modules.md](../user-guide/modules.md)
