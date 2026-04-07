---
title: "Proposal A-2: Module Core — Execution and Tooling"
category: proposal
audience: [ai-dev, human-dev]
status: accepted
last-verified: 2026-04-06
depends-on: [docs/project/proposals/module-system-core-a1.md, docs/project/proposals/module-system-core.md, docs/project/proposals/module-system.md, docs/spec/modules.md, GLOSSARY.md]
---

# Proposal A-2: Module Core — Execution and Tooling

*Split from [module-system-core.md](module-system-core.md) v10 on 2026-04-05.*

Proposal A-2 covers execution and tooling: the VM, the shell subcommand, error codes, architecture documentation, the user guide, and the acceptance test suite. It depends on [Proposal A-1](module-system-core-a1.md), which must be merged first.

For the language design, see [module-system-core.md](module-system-core.md). For the AST and parser changes that this proposal targets, see [module-system-core-a1.md](module-system-core-a1.md).

---

## Decision Register

See [module-system-core.md](module-system-core.md) for the full decision register (D1–D19). This proposal adds:

| # | Topic | Decision |
|---|-------|---------|
| 20 | Bootstrap deferral | `Statement::Bootstrap` evaluation checks only that the caller is not under a `project.json` hierarchy (step 1). Config parsing, application, and URL fetching are deferred. `ISH_PROXY` specification is deferred |
| 21 | VM module decomposition | `interpreter.rs` remains thin. Module-loading logic goes in `module_loader.rs`, access control in `access_control.rs`, interface file checking in `interface_checker.rs`. Significant logic must not accumulate in `interpreter.rs` |
| 22 | Analyzer update for declare blocks | The code analyzer must propagate yielding through mutual recursion in `declare { }` blocks. If any function in a cycle is yielding, all become yielding. If no function in a cycle has other yielding criteria, all are unyielding |
| 23 | Defer ish-codegen | Changes to `ish-codegen` are deferred pending validation of the module loading design in the interpreter |
| 24 | Error code assignment | Module/interface errors are assigned E016–E024 |
| 25 | AGENTS.md vs CLAUDE.md | `CLAUDE.md` and any copilot instructions files are symlinks to `AGENTS.md`. Never update them directly. All agent-facing documentation goes to `AGENTS.md` |

---

## Prototype Changes

### `ish-vm`

The VM currently stubs `Statement::Use` and `Statement::ModDecl` as no-ops. This section describes the real implementations. Per D21, interpreter.rs remains thin: each major concern lives in its own module.

**New module: `proto/ish-vm/src/module_loader.rs`**

This module handles all filesystem and project-structure concerns. It has no global state; callers pass in a `ProjectContext`.

```
find_project_root(start_dir: &Path) -> Option<PathBuf>
    Walk up from start_dir, return the first directory containing project.json.
    Return None if the filesystem root is reached without finding one.

derive_module_path(file_path: &Path, src_root: &Path) -> Result<Vec<String>, ModuleError>
    Strip the src_root prefix and .ish extension from file_path.
    Apply the index.ish rule: if the filename is "index", use the parent directory name instead.
    Return the path segments as a Vec<String>.

resolve_module_path(module_path: &[String], src_root: &Path) -> Result<PathBuf, ModuleError>
    Given a module path (from a use statement), find the corresponding .ish file.
    Candidates: src_root/a/b/c.ish and src_root/a/b/c/index.ish.
    If both exist: return Err(ModuleError::PathConflict { ... }).
    If neither exists: return Err(ModuleError::NotFound { ... }).
    Files without .ish extension are never considered.

check_cycle(loading_stack: &[Vec<String>], candidate: &[String]) -> bool
    Return true if candidate already appears in the loading_stack.
```

**New module: `proto/ish-vm/src/access_control.rs`**

```
pub struct ProjectContext {
    pub project_root: Option<PathBuf>,  // None = installation default
    pub src_root: Option<PathBuf>,      // project_root/src/
}

check_access(
    item_visibility: Visibility,
    item_project_root: Option<&Path>,
    caller_file_path: Option<&Path>,    // None = inline/REPL input
    caller_project_root: Option<&Path>,
) -> Result<(), AccessError>
    Priv:  caller must be in the same module (same file path).
    Pkg:   caller must be under item_project_root (containment check).
    Pub:   always allowed.
    Returns AccessError::Private, AccessError::PackageOnly as appropriate.

is_project_member(file_path: &Path, project_root: &Path) -> bool
    Returns true if file_path starts with project_root.
```

**New module: `proto/ish-vm/src/interface_checker.rs`**

This module handles interface file (`.ishi`) consistency checking:

```
check_interface(
    module_file: &Path,
    pub_declarations: &[Declaration],
) -> Vec<InterfaceError>
    Locate the sibling .ishi file at the same path with .ishi extension.
    If no .ishi file exists: return empty (no enforcement).
    Parse the .ishi file for its pub declarations (function signatures and type definitions).
    Compare against pub_declarations.
    Emit:
      InterfaceError::SymbolNotInImplementation for each symbol in .ishi absent from the .ish file
      InterfaceError::SymbolNotInInterface for each pub symbol in .ish absent from the .ishi file
      InterfaceError::SymbolMismatch for each symbol present in both with different signatures
```

**`interpreter.rs` changes**

Replace the `Statement::Use { .. } => Ok(ControlFlow::None)` stub with a real implementation that delegates to the new modules:

1. Determine if the module path is external (contains a `.` in the first segment) or internal.
2. For internal paths: call `module_loader::resolve_module_path` against the caller's `src_root`.
3. Check for cycles against the current loading stack. If a cycle is found, return `RuntimeError` with code E017 (`module/cycle`), listing the full cycle path.
4. Load and parse the file.
5. Wrap its contents in an implicit `DeclareBlock`. If any statement in the file is not a declaration, return `RuntimeError` with code E018 (`module/script-not-importable`), naming the file.
6. Call `interface_checker::check_interface` on the file. Surface any interface errors before proceeding.
7. Evaluate the `DeclareBlock` in a fresh child environment.
8. Bind the module namespace into the caller's environment according to the import form (qualified, aliased, or selective).
9. On selective imports, call `access_control::check_access` for each imported symbol.

Replace the `Statement::ModDecl { .. }` stub with an internal error: `ModDecl` is no longer a valid statement; the parser should never produce it. If encountered, return an internal error.

Add `Statement::DeclareBlock` evaluation:

1. Collect all declarations in the block into a temporary scope.
2. Evaluate them with mutual forward-reference resolution (all function and type names are pre-registered before any body is evaluated).
3. Merge the resulting bindings into the parent environment.
4. If any statement in the block is not a declaration, return a compile error with code E020 (`module/declare-block-command`).

Add `Statement::Bootstrap` evaluation (D20 — partially deferred):

1. Check that the caller file is not under any `project.json` in its hierarchy (using `module_loader::find_project_root`). If it is, return E021 (`module/bootstrap-in-project`).
2. Config parsing, application, and URL fetching are deferred to a future revision. `ISH_PROXY` specification is deferred.

**Analyzer update for declare blocks (D22)**

Update the existing code analyzer (in `ish-stdlib` or wherever yielding analysis runs) to handle mutually recursive functions declared together in a `declare { }` block:

1. When analyzing a `DeclareBlock`, first register all function names in the block as a mutual-recursion group before analyzing any bodies.
2. For each function in the group, determine yielding based on its own operations first.
3. If any function in the group calls another function in the group, propagate yielding transitively.
4. If a cycle is detected within the group and at least one function is yielding (by any criterion), mark all functions in the cycle as yielding.
5. If a cycle is detected and no function has any yielding criterion other than the cyclic call, mark all functions in the cycle as unyielding.

**`lib.rs`**

Export the new `module_loader`, `access_control`, and `interface_checker` modules.

### `ish-shell`

**New subcommand: `ish interface freeze`**

Add an `interface` subcommand to the shell binary (`ish-shell/src/main.rs`). Implement in a new file `ish-shell/src/interface_cmd.rs`:

```
interface_freeze(target: Option<String>, project_root: &Path)
    If target is None: walk src/ and process all .ish files.
    If target is Some(module_name): resolve module_name to a .ish file path.
    For each .ish file:
        Parse the file.
        Collect all FunctionDecl and TypeAlias nodes with Visibility::Pub.
        Format them as a .ishi declaration file (signatures only, no bodies).
        Write to the sibling .ishi path, overwriting any existing file.
        Print: "Wrote <module_path>.ishi"
```

The `.ishi` file format is a subset of ish source: function signatures (`fn name(params) -> RetType`) and type aliases (`type Name = Definition`), each with `pub` keyword, one per line. No function bodies.

**Project root discovery at startup**

In `ish-shell/src/main.rs`, before launching the REPL or executing a file, determine the `ProjectContext`:

1. If executing a file: call `module_loader::find_project_root` from the file's directory.
2. If in REPL mode: call `find_project_root` from the current working directory.
3. Store the `ProjectContext` and pass it to the interpreter.

### Error Codes

All new error codes are production sites in `ish-vm`. Add to `docs/errors/INDEX.md` and add variants to `ErrorCode` in `ish-runtime/src/error.rs`:

| Code | ErrorCode Variant | Structural Type | Summary | Production site |
|------|------------------|-----------------|---------|----------------|
| E016 | `ModuleNotFound` | `CodedError` | `use` path has no matching `.ish` file | `module_loader::resolve_module_path` |
| E017 | `ModuleCycle` | `CodedError` | Circular `use` dependency detected | `interpreter.rs` — Use evaluation |
| E018 | `ModuleScriptNotImportable` | `CodedError` | File imported via `use` contains top-level commands | `interpreter.rs` — Use evaluation |
| E019 | `ModulePathConflict` | `CodedError` | Both `foo.ish` and `foo/index.ish` exist | `module_loader::resolve_module_path` |
| E020 | `ModuleDeclareBlockCommand` | `CodedError` | `declare { }` block contains a non-declaration statement | `interpreter.rs` — DeclareBlock evaluation |
| E021 | `ModuleBootstrapInProject` | `CodedError` | `bootstrap` used inside a project hierarchy | `interpreter.rs` — Bootstrap evaluation |
| E022 | `InterfaceSymbolNotInImplementation` | `CodedError` | `.ishi` declares a symbol absent from the `.ish` file | `interface_checker.rs` |
| E023 | `InterfaceSymbolNotInInterface` | `CodedError` | `.ish` has a `pub` symbol not declared in `.ishi` | `interface_checker.rs` |
| E024 | `InterfaceSymbolMismatch` | `CodedError` | Symbol present in both `.ishi` and `.ish` with mismatched signatures | `interface_checker.rs` |

### `ish-codegen` (D23 — Deferred)

Changes to `ish-codegen` are deferred. Once the interpreter-based module loading is validated, a follow-up proposal will specify the codegen path. For context, the intended changes are:

- When generating Rust code for `Statement::Use` targeting an internal module, resolve the module path to a `.ish` file and include that file's generated Rust in the temporary project.
- `Statement::DeclareBlock` generates a Rust `mod` block with `pub(crate)` for `pkg`-visible declarations and `pub` for `pub`-visible declarations.
- Remove `Statement::ModDecl` handling.

---

## Documentation Updates

### `docs/spec/modules.md`

Full rewrite. The current content does not reflect any of the decisions in this proposal. The rewritten document must cover:

- The three visibility levels and their scopes; the `pkg` default; the entry point pattern
- The file = module rule; `index.ish`; the conflict rule; the `src/` source root
- The four `use` directive forms; relative vs. external paths; qualified access without `use`
- `declare { }` blocks; the mutual-recursion model; implicit declare wrapping; cross-module cycle rule
- Project layout (`src/`, `scripts/`, `tools/`); the `project.json` discovery rule; `pkg` access for scripts
- The `bootstrap` directive and its three forms; what it grants and does not grant; deferred aspects (config parsing, URL fetching)
- Interface files: generation, enforcement, error codes
- Deferred topics (conditional compilation, incremental compilation, script distribution, bootstrap config application, ISH_PROXY)

### `docs/architecture/vm.md`

Add a new section "Module Loading" covering:

- How the interpreter resolves a `use` path to a filesystem path (`module_loader::resolve_module_path`)
- The `index.ish` special case and conflict detection
- The loading stack used for cycle detection
- Implicit declare wrapping: what triggers it, what the error means
- Project root discovery at interpreter startup (`module_loader::find_project_root`)
- How `ProjectContext` flows through the interpreter
- `pkg` access checks: when `access_control::check_access` is called and what it tests
- Interface file consistency checking: when it runs, which errors it produces (`interface_checker`)
- `DeclareBlock` evaluation and the analyzer's yielding propagation rules for mutual recursion (D22)

Update the existing "Builtins" and "Environment" sections to note that `pkg`-visible bindings from imported modules are stored in a module namespace within the environment, not the global scope.

### `docs/user-guide/modules.md`

This file needs to be written (it may be a placeholder or absent). It is the primary user-facing reference for the module system. It must cover:

- Getting started: a minimal project layout with one module and one importing file
- The visibility model: `priv`, `pkg` (default), `pub`; when to use each
- Writing importable modules: the `.ish` extension, declarations-only constraint
- Writing scripts: shebang convention, entry point pattern
- Project layout: `src/`, `scripts/`, `tools/`; what belongs where
- The `use` directive: all four forms, with examples for each
- `declare { }` blocks: when to use them, mutual recursion example, REPL usage
- The `bootstrap` directive: standalone script example, when to use it vs. a project
- Interface files: running `ish interface freeze`, checking in `.ishi` files, what the errors mean
- Common mistakes: importing a script file (error), creating a path conflict (error), using `bootstrap` inside a project (error)

### `docs/errors/INDEX.md`

Add all nine new error codes E016–E024 from the Error Codes table above, using the format established in the existing file.

Also add the `module/` and `interface/` string codes to `docs/spec/errors.md` if that document enumerates error code strings.

### `AGENTS.md`

Add module-related guidance covering:

- Where to look when adding a new module-system feature: `module_loader.rs`, `access_control.rs`, `interface_checker.rs`, `interpreter.rs` Use handling
- Acceptance test location for module tests: `proto/ish-tests/modules.sh`
- The interface file format and where it is generated
- **Note:** `CLAUDE.md` and any copilot instructions files are symlinks to `AGENTS.md`. Never update them directly. Any agent-facing documentation changes — including task playbook rows — belong in `AGENTS.md`.

Also add a row to the task playbooks table in `AGENTS.md`:

| Working on modules | [docs/spec/modules.md](docs/spec/modules.md) |

---

## Unit Tests

Add unit tests to `ish-vm` for:

- `module_loader::derive_module_path`: standard path, `index.ish` path, path-conflict detection.
- `module_loader::resolve_module_path`: found, not-found, conflict cases.
- `access_control::check_access`: all nine combinations of `(Priv/Pkg/Pub) × (same-module/same-project/external)`.
- `module_loader::find_project_root`: found at current dir, found via walk, not found.
- `interface_checker::check_interface`: symbol-not-in-implementation, symbol-not-in-interface, symbol-mismatch, no .ishi file present (no errors).

---

## Acceptance Tests

New tests belong in a dedicated file `proto/ish-tests/modules.sh`. The file follows the conventions in `proto/ish-tests/lib/test_lib.sh`.

**Visibility:**
- A `priv` item in module A is inaccessible from module B in the same project (E code access error).
- A `pkg` item in module A is accessible from module B in the same project.
- A script physically located under the project root can access `pkg` items from project modules.
- A script in `scripts/` and a script in `tools/` both receive `pkg` access; a script outside the project root does not.
- An inline script (`ish -e "..."`) cannot access `pkg` items from a project even if invoked from within the project directory.
- A `pub` item in module A is accessible from an external dependent project.
- A `pkg` item in module A is inaccessible from an external dependent project (access error).
- Running `ish interface freeze` on a module generates a `.ishi` file containing only `pub` declarations.
- A `.ishi` file that declares a symbol absent from the implementation produces E022 (`interface/symbol-not-in-implementation`).
- A `.ishi` file where a declared symbol's signature does not match the implementation produces E024 (`interface/symbol-mismatch`).
- Running `ish interface freeze` again overwrites the existing `.ishi` file with the current `pub` declarations.

**Module Identity:**
- Two mutually recursive functions in the same file (no explicit `declare` block) compile and run correctly.
- Two modules (files) that import each other produce E017 (`module/cycle`); the diagnostic names both files.
- A `declare { }` block containing two mutually recursive functions compiles and runs correctly.
- A top-level command invocation inside a `declare { }` block produces E020 (`module/declare-block-command`).
- A file containing a top-level command can be run directly with `ish`.
- The same file, when referenced by `use`, produces E018 (`module/script-not-importable`).
- A file containing only function definitions can be both run directly and imported via `use`.
- Two separate `declare { }` blocks in the same file that reference each other produce E017 (`module/cycle`).
- `use my-tool` where `my-tool` has no `.ish` extension produces E016 (`module/not-found`).

**Module-to-File Mapping:**
- `use net/http` resolves to `src/net/http.ish`.
- `use net` resolves to `src/net/index.ish` when that file exists.
- `src/net/index.ish` defines module `net`, not `net/index`.
- `src/net/http/index.ish` defines module `net/http`, not `net/http/index`.
- A `use` path with no corresponding file produces E016 (`module/not-found`) naming the expected file path(s).
- A file at `src/foo/bar.ish` defines module `foo/bar`, not `src/foo/bar`.
- Both `src/foo.ish` and `src/foo/index.ish` existing produces E019 (`module/path-conflict`) naming both files.

**Import Syntax:**
- `use foo/bar` makes `bar.Func` available where `Func` is defined in `src/foo/bar.ish`.
- `use foo/bar as b` makes `b.Func` available.
- `use foo/bar { Func }` brings `Func` into local scope.
- `use foo/bar { Func as F }` brings `F` into local scope with the name `F`.
- `use net` resolves to `src/net/index.ish` and makes `net.Func` available.
- Accessing a `priv` item from another module via `use` produces an access error.
- Accessing a `pkg` item from an external project via `use` produces an access error.
- `net/http.Get(...)` resolves without a prior `use net/http`.
- `use` of a file containing top-level commands produces E018 (`module/script-not-importable`).
- `use my-tool` where `my-tool` has no `.ish` extension produces E016 (`module/not-found`).

**Project Layout:**
- A project missing a `src/` directory produces an error when `ish` operates on it.
- Files under `src/` are resolved by `use` statements using paths relative to `src/`.
- Files under `scripts/` and `tools/` are not resolved by `use` statements.
- A script in `scripts/` has `pkg` access to modules in `src/`.
- A script in `tools/` has `pkg` access to modules in `src/`.
- A script at the project root (sibling of `project.json`) has `pkg` access to modules in `src/`.
- `ish interface freeze` generates `.ishi` files for modules under `src/`, not for files in `scripts/` or `tools/`.

**Project Membership and Bootstrap:**
- A file with no `bootstrap` directive in an isolated directory uses the installation default project.
- A file inside a directory containing `project.json` is a member of that project.
- A member file has `pkg` access to other modules in the same project.
- A file using `bootstrap` in an isolated directory does not produce a bootstrap-in-project error (step 1 only implemented; config parsing deferred).
- A file using `bootstrap` while also under a `project.json` in its hierarchy produces E021 (`module/bootstrap-in-project`).

---

## Referenced by

- [docs/project/proposals/module-system-core.md](module-system-core.md)
- [docs/project/proposals/module-system-core-a1.md](module-system-core-a1.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
