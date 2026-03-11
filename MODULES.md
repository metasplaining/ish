# ish Module System

## Overview

ish's module system follows Rust's module system closely. It governs how code is organized into files, how symbols are shared between files, and how projects are packaged for distribution.

---

## Projects

An ish project is contained in a directory and its subdirectories. A configuration file in the top-level directory specifies the project's package dependencies, including the hosting repository and version for each dependency. This enables build tooling to download and cache all package dependencies automatically.

> **Note:** The configuration file format is TBD.

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

## Imports and Re-exports

A `use` directive is required to reference symbols from a different module. Combining `pub` and `use` directives allows symbols to be re-exported.

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
3. **Dedicated registry.** Build a dedicated ish registry once the packaging semantics (encumbrance, module compatibility, execution configuration) are well-defined enough to encode in registry metadata.