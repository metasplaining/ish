---
title: ish Module System
category: spec
audience: [all]
status: draft
last-verified: 2026-04-06
depends-on: [docs/spec/assurance-ledger.md, docs/spec/execution.md, docs/spec/syntax.md, docs/spec/errors.md]
---

# ish Module System

## Overview

ish's module system governs how code is organized into files, how symbols are shared between files, and how projects are structured. It uses a file-based module identity, three-level visibility model, and a `declare { }` block construct for mutual recursion.

---

## Visibility

ish has three visibility levels. The default is `pkg`.

| Keyword | Scope | Description |
|---------|-------|-------------|
| `priv` | Current module (file) only | Not visible to other modules or scripts in the same project. |
| `pkg` | Entire project | **Default.** Visible to all project members: modules under `src/` and scripts anywhere under the project root. |
| `pub` | External dependents | Required for items exported from the project. Visible to all code everywhere. |

```ish
fn internal_helper() { ... }       // pkg (default) — visible within the project
priv fn secret() { ... }           // priv — visible only in this file
pub fn exported() { ... }          // pub — visible to external dependents
```

### `pkg` Scope — Project Containment

`pkg` visibility is granted to any file physically contained anywhere under the project root (the directory that holds `project.json`). This includes:

- Module files under `src/`
- Scripts in `scripts/` (public scripts)
- Scripts in `tools/` (internal scripts)
- Scripts in any other location under the project root

It **excludes**:

- Inline scripts (e.g., `ish -e "..."`)
- An interactive shell session that has `cd`'d into the project directory
- Any file not physically located under the project root

The determining criterion is the source file path. If the file path is under the project root, the file is a project member and receives `pkg` access. Files with no physical path (inline input) or paths outside the project root do not qualify.

### Entry Point Pattern

A common project shape is to define modules with no `pub` declarations — their contents are entirely `pkg` — and then provide a short script physically located inside the project (typically in `scripts/` or at the project root) that calls into those modules and serves as the executable entry point. The script receives `pkg` access to all project internals. External consumers cannot `use` the modules (no `pub` API), but they can run the script.

### Interface Files

Each module may have an associated `.ishi` interface file that declares the module's public contract — specifically its `pub` items visible to external dependents.

Interface files are never generated automatically. To generate one:

```
ish interface freeze              -- generates .ishi for all modules under src/
ish interface freeze module_name  -- generates .ishi for a single module
```

The generated file contains only the `pub` declarations from the corresponding `.ish` file: function signatures (no bodies) and type aliases, one per line.

Once a `.ishi` file is committed to the project, the compiler enforces it:

- Every symbol declared in `.ishi` must exist in the implementation with a matching signature.
- Every `pub` symbol in the implementation must be declared in `.ishi`.
- Signatures must match exactly.

The `.ishi` file may be regenerated at any time by running `ish interface freeze` again, which overwrites the existing file.

There is no separate low-assurance / high-assurance generation mode. The distinction is whether a `.ishi` file exists: if it does, it is enforced; if it does not, there is no enforcement.

**Interface file error codes:**

| Condition | Error Code |
|-----------|-----------|
| Symbol declared in `.ishi` but absent from implementation | E022 (`InterfaceSymbolNotInImplementation`) |
| `pub` symbol in implementation but absent from `.ishi` | E023 (`InterfaceSymbolNotInInterface`) |
| Symbol present in both but signatures do not match | E024 (`InterfaceSymbolMismatch`) |

---

## Module Identity

### File = Module Rule

Every `.ish` file under `src/` defines exactly one module. The module path is derived from the file's path relative to `src/`. The `src/` prefix never appears in a `use` statement.

| File | Module Path |
|------|-------------|
| `src/net/http.ish` | `net/http` |
| `src/util.ish` | `util` |
| `src/foo/bar.ish` | `foo/bar` |

### `index.ish` Rule

A file named `index.ish` in a directory maps to the module path of that directory, not to `<dir>/index`. This applies at any level of the hierarchy:

| File | Module Path |
|------|-------------|
| `src/net/index.ish` | `net` |
| `src/net/http/index.ish` | `net/http` |

`index` is a reserved filename within any directory. There is no escape mechanism: a file named `index.ish` always defines the parent-directory module.

### Path Conflict Rule

If both `src/foo.ish` and `src/foo/index.ish` exist, the build fails for both files with error E019 (`ModulePathConflict`). The diagnostic names both files and explains that they resolve to the same module path.

### Source Root

Projects must have a `src/` directory. All importable module files are nested under `src/`. The `src/` prefix is not part of the module path used in `use` statements. Files without the `.ish` extension cannot be imported.

---

## The `use` Directive

The `use` directive imports symbols from other modules. There are four forms:

### Four Import Forms

```ish
use foo/bar                // qualified import — access as bar.Name
use foo/bar as b           // aliased import — access as b.Name
use foo/bar { Type, fn }   // selective import — brings names into local scope
use foo/bar { Type as T }  // selective import with rename
```

### Internal vs. External Paths

**Within-project imports** use a module path relative to the `src/` root:

```ish
use net/http             // imports src/net/http.ish
use net                  // imports src/net/index.ish
```

**External package imports** use a full package path (resolved by the package system):

```ish
use example.com/foo/bar  // imports from an external package
```

External paths are distinguished by containing a `.` in the first segment.

### Qualified Access Without `use`

A module may be accessed by its full path without importing it:

```ish
net/http.Get(...)        // resolves the module at point of use
```

This resolves the module by path without bringing any names into scope.

### `use` Evaluation Steps

When the interpreter evaluates a `use` directive for an internal path:

1. Resolve the module path to a `.ish` file via `module_loader::resolve_module_path`.
2. Check for cycles against the current loading stack. Cycle detected → E017 (`ModuleCycle`).
3. Load and parse the file.
4. Wrap the file's contents in an implicit `declare { }` block. If any statement is not a declaration, return E018 (`ModuleScriptNotImportable`).
5. Run `interface_checker::check_interface` on the file. Surface any interface errors (E022–E024).
6. Evaluate the `declare { }` block in a fresh child environment.
7. Bind the module namespace into the caller's environment according to the import form.
8. On selective imports, call `access_control::check_access` for each imported symbol.

### Circular Dependencies

Circular `use` dependencies between modules are not allowed. If module A imports module B ("use") and module B imports module A, the compiler reports E017 (`ModuleCycle`) listing the full cycle path.

Within a single file, mutual recursion is supported via `declare { }` blocks. Across separate `declare { }` blocks in the same file, forward references are also restricted — two separate `declare { }` blocks may not call each other.

---

## `declare { }` Blocks

A `declare { }` block is an anonymous, declarations-only grouping construct. It allows mutually recursive definitions to be stated together:

```ish
declare {
  fn is_even(n: Int) -> Bool { if n == 0 { true } else { is_odd(n - 1) } }
  fn is_odd(n: Int) -> Bool  { if n == 0 { false } else { is_even(n - 1) } }
}
```

### Rules

- **Anonymous.** A `declare { }` block does not introduce a module path or namespace.
- **Declarations only.** Only function definitions, type definitions, and other declaration forms are permitted. Top-level command invocations and function calls are not permitted. Violation → E020 (`ModuleDeclareBlockCommand`).
- **Internal calls allowed.** Functions declared inside a `declare { }` block may call each other freely, including cyclically.
- **No cross-block cycles.** Two separate `declare { }` blocks in the same file may not call each other. Each block is internally self-contained.

### Evaluation

1. Pre-register all function and type names in the block (enabling mutual forward references).
2. Validate that all statements are declarations (E020 on violation).
3. Evaluate declarations in order.
4. Merge the resulting bindings into the parent environment.

### Implicit Declare Wrapping

When a file is imported via `use`, the interpreter implicitly wraps its contents in a `declare { }` block. This enforces the declarations-only rule at import time. Any top-level command in the file causes E018 (`ModuleScriptNotImportable`).

A file that contains only declarations is both runnable and importable. A file that contains top-level commands is runnable only.

### Analyzer: Yielding Propagation (D22)

The code analyzer propagates yielding classification through mutual recursion in `declare { }` blocks:

1. Register all function names in the block as a mutual-recursion group before analyzing any bodies.
2. Determine yielding for each function based on its own operations first.
3. If any function in the group calls another function in the group, propagate yielding transitively.
4. If a cycle is detected within the group and at least one function is yielding, all functions in the cycle become yielding.
5. If a cycle is detected and no function has any yielding criterion other than the cyclic call, all functions in the cycle are unyielding.

### REPL Usage

`declare { }` blocks are the recommended mechanism for writing mutually recursive definitions in the REPL, where the REPL processes input in units and a mutual recursion must be submitted as one unit.

---

## Project Layout

### Directory Structure

```
myproject/
  project.json     // project manifest
  src/             // REQUIRED — module source files (.ish)
  scripts/         // OPTIONAL — scripts for public consumption
  tools/           // OPTIONAL — scripts for internal use
```

### `src/` — Source Root

The `src/` directory is the source root. All importable `.ish` module files are nested under it. A project without a `src/` directory is malformed; tools report an error when operating on it.

### `scripts/` Convention

Scripts placed in `scripts/` are considered public: they are part of the project's distributed interface. Build tooling looks here when packaging the project for distribution.

### `tools/` Convention

Scripts placed in `tools/` are internal. They are not distributed with the package and are not part of the project's public interface. Typical contents: build scripts, code generators, test runner wrappers, developer utilities.

### `pkg` Access for Scripts

Scripts anywhere under the project root — including in `scripts/`, `tools/`, or at the project root itself — are project members and receive `pkg` access to all project modules. The directory a script lives in does not affect its access rights; any script file under the project root qualifies.

Scripts follow the executable convention (shebang + no `.ish` extension). They are not importable via `use`.

### Project Discovery

ish walks up the directory tree from the directory containing the source file, looking for a `project.json` file. The first one found establishes the project root.

If no `project.json` is found, ish uses the default project file bundled with the installation. This default project includes only the standard library packages distributed with ish. Files using the installation default are standalone and receive no `pkg` access.

---

## The `bootstrap` Directive

Standalone scripts — files not under any `project.json` — may need external packages, specific ish version requirements, or non-default assurance settings. The `bootstrap` directive provides this configuration.

### Three Forms

```ish
bootstrap "path/to/config.json"           // filesystem path
bootstrap "https://example.com/cfg.json"  // URL (resolved via ISH_PROXY)
bootstrap { "ish": ">=1.0", "dependencies": { "example.com/http": "v1.2.3" } }
                                           // inline JSON object
```

| Form | Purpose |
|------|---------|
| Filesystem path | Load JSON config from the local filesystem |
| URL | Load JSON config from an HTTPS URL (resolved via ISH_PROXY) |
| Inline JSON object | Configuration embedded directly in source |

### What `bootstrap` Grants

Access to the `pub` APIs of listed packages.

### What `bootstrap` Does Not Grant

`pkg` visibility into any project. The `bootstrap` directive is not a claim of membership in any existing project.

### Interaction with Project Discovery

If a file uses `bootstrap` and is also under a `project.json` in its directory hierarchy, the `bootstrap` directive is an error — E021 (`ModuleBootstrapInProject`). Project members receive their configuration from the project manifest; they do not use `bootstrap`.

### Deferred Aspects (D20)

The current prototype checks only that the caller is not under a `project.json` hierarchy. Config parsing, application, and URL fetching are deferred to a future revision. `ISH_PROXY` specification is deferred.

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| E016 | `ModuleNotFound` | `use` path has no matching `.ish` file |
| E017 | `ModuleCycle` | Circular `use` dependency detected |
| E018 | `ModuleScriptNotImportable` | File imported via `use` contains top-level commands |
| E019 | `ModulePathConflict` | Both `foo.ish` and `foo/index.ish` exist |
| E020 | `ModuleDeclareBlockCommand` | `declare { }` block contains a non-declaration statement |
| E021 | `ModuleBootstrapInProject` | `bootstrap` used inside a project hierarchy |
| E022 | `InterfaceSymbolNotInImplementation` | `.ishi` declares a symbol absent from the `.ish` file |
| E023 | `InterfaceSymbolNotInInterface` | `.ish` has a `pub` symbol not declared in `.ishi` |
| E024 | `InterfaceSymbolMismatch` | Symbol present in both with mismatched signatures |

---

## Open Questions

- [ ] **Interface freeze must capture fully analyzed signatures.** When `ish interface freeze` generates a `.ishi` file, the signatures must reflect the fully analyzed form of each declaration — for example, a function that is implicitly yielding must be written as explicitly yielding in the `.ishi` file. The analysis requirements and the format for encoding analyzed attributes in `.ishi` are not yet specified. See also [docs/project/open-questions.md — Interface Files](../project/open-questions.md#interface-files).

---

## Deferred Topics

The following aspects of the module system are deferred to future proposals:

- **Conditional compilation.** Intersects with execution configurations.
- **Incremental compilation.** Module boundaries are natural incremental compilation units; the design supports this without prescribing it.
- **Script distribution.** How projects distribute executable scripts to consumers is deferred to Package Management.
- **Bootstrap config parsing.** The `bootstrap` directive's config parsing, application, and URL fetching are deferred (D20).
- **`ISH_PROXY` specification.** The proxy mechanism for URL-based bootstrap configs is deferred.
- **`ish-codegen` changes.** Codegen support for modules is deferred pending validation of the interpreter-based module loading (D23).
- **Glob imports.** `use foo/bar { * }` imports all exported names. Available but discouraged; may be restricted at higher assurance levels.
- **Re-exports.** Combining `pub` and `use` to re-export symbols is specified but implementation is deferred.

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/concurrency.md](concurrency.md)
- [docs/architecture/overview.md](../architecture/overview.md)
- [docs/user-guide/modules.md](../user-guide/modules.md)
