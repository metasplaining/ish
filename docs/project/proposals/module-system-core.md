---
title: "Proposal A: Module Core"
category: proposal
audience: [ai-dev, human-dev]
status: split
last-verified: 2026-04-05
depends-on: [docs/project/proposals/module-system.md, docs/project/rfp/module-system.md, docs/spec/modules.md, GLOSSARY.md]
---

# Proposal A: Module Core

*Derived from [module-system.md](module-system.md) v3 on 2026-04-05. Revised on 2026-04-05 (×7).*

> **Status: Split.** This proposal has been split into two child proposals:
> - [Proposal A-1: Module Core — Language Representation](module-system-core-a1.md) — ish-ast, ish-parser, GLOSSARY, spec/syntax.md, architecture/ast.md
> - [Proposal A-2: Module Core — Execution and Tooling](module-system-core-a2.md) — ish-vm, ish-shell, error catalog, user guide, acceptance tests, AGENTS.md
>
> This document is the language design reference. The decision register (D1–D19) and all feature sections remain here as the authoritative source. Implementation work proceeds via the child proposals.

Module Core covers the in-process module system: visibility, mutual recursion, file mapping, import syntax, project membership, and standalone script configuration. It can be implemented without any network dependencies or package management infrastructure.

---

## Decision Register

| # | Topic | Decision |
|---|-------|---------|
| 1 | Implementation order | Module Core (this proposal) first |
| 4 | Visibility | Three levels: `priv`, `pkg` (default), `pub`. `pkg` visibility extends to all project members: modules under `src/` and any script file physically contained anywhere under the project root. Inline scripts and shells that merely change into the project directory do not qualify. Interface files use `.ishi` extension. Generated on demand by `ish interface freeze [module_name]`; contain only `pub` declarations. If a `.ishi` file exists, build-time check: all declared symbols must match the implementation exactly |
| 5 | Module identity | Default: one file = one module, path derived from filesystem position under `src/`. `declare { }` blocks are anonymous, declarations-only groupings that allow mutual recursion within a scope. When a file is pulled into compilation via `use`, it is implicitly wrapped in a declare block; any top-level commands in the file cause a compile error |
| 8 | Project membership | A file is a member of a project if and only if it is physically located under the project root (discovered by directory hierarchy search for `project.json`). Modules must additionally be under `src/` to be importable. Scripts anywhere under the project root qualify for `pkg` access |
| 10 | Import syntax | Hierarchical with aliases: `use foo/bar`, `use foo/bar as b`, `use foo/bar { Type }`, `use foo/bar { Type as T }`. The `src/` source root is implicit — not part of the module path |
| 12 | Module-to-file mapping | Strict `.ish`-extension file = module. `index.ish` in directory `foo/` maps to module path `foo`. If both `foo.ish` and `foo/index.ish` exist, it is a build error. Files without `.ish` extension cannot be imported |
| 13 | Interface file error codes | Compile-time checks. Granular error codes: symbol-not-in-interface, symbol-not-in-implementation, symbol-mismatch, and others as discovered |
| 14 | Assurance and dependency levels | Assurance levels do not restrict which modules a module may depend on |
| 15 | Top-level package exports | `index.ish` in any directory maps to the module path of that directory. Works at every level of the directory hierarchy |
| 16 | Standalone script configuration | The `bootstrap` directive provides dependency configuration to files not under any `project.json`. It performs the same role as `project.json` for standalone scripts: setting ish version, defining dependencies, and setting standards and assurance levels. It grants access to the `pub` APIs of listed packages only; it does not confer project membership or `pkg` visibility |
| 17 | Script vs. module distinction | A `.ish` extension is required for importable modules; `use` does not locate files without it. The interpreter accepts any input as a script. Convention (unenforced): executable scripts use a shebang line and omit the `.ish` extension |
| 18 | Project script directories | Scripts for public consumption are placed in `scripts/`. Scripts for internal use (build tools, test runners) are placed in `tools/`. These are build conventions; script distribution is deferred to Package Management |
| 19 | Source root | Projects must have a `src/` directory. All importable module files are nested under `src/`. The `src/` prefix is not part of the module path used in `use` statements |
| 20 | Bootstrap deferral | `Statement::Bootstrap` evaluation checks only that the caller is not under a `project.json` hierarchy (step 1). Config parsing, application, and URL fetching are deferred. `ISH_PROXY` specification is deferred |
| 21 | VM module decomposition | `interpreter.rs` remains thin. Module-loading logic goes in `module_loader.rs`, access control in `access_control.rs`, interface file checking in `interface_checker.rs`. Significant logic must not accumulate in `interpreter.rs` |
| 22 | Analyzer update for declare blocks | The code analyzer must propagate yielding through mutual recursion in `declare { }` blocks. If any function in a cycle is yielding, all become yielding. If no function in a cycle has other yielding criteria, all are unyielding |
| 23 | Defer ish-codegen | Changes to `ish-codegen` are deferred pending validation of the module loading design in the interpreter |
| 24 | Error code assignment | Module/interface errors are assigned E016–E024 |
| 25 | AGENTS.md vs CLAUDE.md | `CLAUDE.md` and any copilot instructions files are symlinks to `AGENTS.md`. Never update them directly. All agent-facing documentation goes to `AGENTS.md` |

---

## Problems Not in the Original RFP

- **Cyclic dependencies:** Cross-module cycles are a compile error. Within-module mutual recursion is fully supported (D5).
- **Script vs. module distinction:** A `.ish` extension is required for importable modules (D17). Executable scripts use a shebang and no extension by convention.
- **Top-level package exports:** Addressed by D15 (`index.ish`).
- **Standalone script configuration:** The `bootstrap` directive serves standalone scripts in the role that `project.json` plays for project members (D16).
- **Project directory layout:** Projects have a required `src/` directory for modules, a conventional `scripts/` for public scripts, and a conventional `tools/` for internal scripts (D18, D19).
- **Script entry points:** Projects may consist entirely of non-public modules plus a project-contained script that serves as the executable entry point. This pattern is a first-class use case, not a workaround.
- **Conditional compilation:** Deferred; intersects with execution configurations.
- **Incremental compilation:** Module boundaries are natural incremental compilation units; the design supports this without prescribing it.
- **Script distribution:** How projects distribute executable scripts to consumers is deferred to Package Management.

---

## Feature: Visibility / Encapsulation

### Issues to Watch Out For

- Too restrictive: makes code hard to test.
- Too permissive: makes code hard to reason about.
- Testing often needs access to internals without `pub` exposure.
- Scripts as entry points need access to project internals without requiring those internals to be `pub`.

### Design

Three visibility levels:

| Keyword | Scope | Notes |
|---------|-------|-------|
| `priv` | Current module only | Not visible to other modules or scripts in the same project |
| `pkg` | Entire project | **Default.** Visible to all project members: modules and contained scripts |
| `pub` | External dependents | Required for items exported from the project |

The default visibility is `pkg`. Items are project-visible without annotation. This follows the principle that code within a project is trusted.

**`pkg` scope — project containment:** `pkg` visibility is granted to any file physically contained anywhere under the project root (the directory that holds `project.json`). This includes:

- Module files under `src/`
- Scripts in `scripts/` (public scripts)
- Scripts in `tools/` (internal scripts)
- Scripts in any other location under the project root

It excludes:

- Inline scripts (e.g., `ish -e "..."`)
- An interactive shell session that has changed directory into the project root
- Any file not physically located under the project root

The determining criterion is the source file path at the time of compilation or execution. If the file path is under the project root, the file is a project member and receives `pkg` access. If the file has no physical path (inline input) or its path is outside the project root, it does not.

**Entry point pattern:** A common project shape is to define many modules with no `pub` declarations — their contents are entirely `pkg` — and then provide a short script physically located inside the project (typically in `scripts/` or at the project root) that calls into those modules and serves as the executable entry point. The script receives `pkg` access to all project internals. External consumers cannot `use` the modules (no `pub` API), but they can run the script.

**Interface files:** Each module may have an associated `.ishi` interface file. The interface file declares the public contract of the module — specifically the `pub` items visible to external dependents.

Interface files are never generated automatically. To generate one, the developer runs:

```
ish interface freeze              -- generates .ishi for all modules in the project
ish interface freeze module_name  -- generates .ishi for a single module
```

The generated file contains only the `pub` declarations from the corresponding `.ish` file. Once a `.ishi` file is committed to the project, the compiler enforces it: every symbol declared in the `.ishi` file must exist in the implementation with a matching signature. The `.ishi` file may be regenerated at any time by running `ish interface freeze` again, which overwrites the existing file.

There is no separate low-assurance / high-assurance generation mode. The distinction is whether a `.ishi` file exists: if it does, it is enforced; if it does not, there is no enforcement.

**Interface file error codes (D13):** Compile-time. Error codes are granular:

| Condition | Error code family |
|-----------|------------------|
| Symbol declared in `.ishi` but absent from implementation | `interface/symbol-not-in-implementation` |
| Symbol present in implementation but absent from `.ishi` | `interface/symbol-not-in-interface` |
| Symbol present in both but signatures do not match | `interface/symbol-mismatch` |

Additional error codes may be introduced as edge cases are discovered.

**Assurance and dependency levels (D14):** Assurance levels do not restrict which modules a module may depend on. A high-assurance module may depend on a low-assurance module. The assurance level is a property of the module's own declarations, not a filter on its dependencies.

### Acceptance Tests

- A `priv` item in module A is inaccessible from module B in the same project (compile error).
- A `pkg` item in module A is accessible from module B in the same project.
- A script physically located under the project root can access `pkg` items from project modules.
- A script in `scripts/` and a script in `tools/` both receive `pkg` access; a script outside the project root does not.
- An inline script (`ish -e "..."`) cannot access `pkg` items from a project even if invoked from within the project directory.
- A `pub` item in module A is accessible from an external dependent project.
- A `pkg` item in module A is inaccessible from an external dependent project (access error).
- Running `ish interface freeze` on a module generates a `.ishi` file containing only `pub` declarations.
- A `.ishi` file that declares a symbol absent from the implementation produces a compile error with code `interface/symbol-not-in-implementation`.
- A `.ishi` file where a declared symbol's signature does not match the implementation produces `interface/symbol-mismatch`.
- Running `ish interface freeze` again overwrites the existing `.ishi` file with the current `pub` declarations.

---

## Feature: Module Identity and Mutual Recursion

### Issues to Watch Out For

- Cyclic type definitions across modules require careful design.
- Cyclic function definitions across modules are rare but arise in callback patterns.
- The shell REPL needs a way to define mutually recursive functions interactively.
- The distinction between runnable scripts and importable modules must be clear to users.

### Design

#### Default model: file = module

Each `.ish` source file under `src/` defines exactly one module. The module path is derived from the file's path relative to `src/`. Within a file, all definitions are mutually visible — forward references and mutual recursion are supported without restriction. The programmer does not need to order declarations within a file.

Files without the `.ish` extension cannot be imported via `use`. They may be run directly by the interpreter.

Cross-module dependency cycles are a **compile error**. The dependency graph across modules must be a DAG. Error messages show the full cycle path and suggest extracting shared types into a common base module.

#### `declare { }` blocks

A `declare { }` block is an anonymous, declarations-only grouping construct. It allows mutually recursive definitions to be stated together:

```ish
declare {
  fn is_even(n: Int) -> Bool { if n == 0 { true } else { is_odd(n - 1) } }
  fn is_odd(n: Int) -> Bool  { if n == 0 { false } else { is_even(n - 1) } }
}
```

Rules for `declare { }` blocks:

- **Anonymous.** A `declare { }` block does not introduce a module path or namespace.
- **Declarations only.** Only function definitions, type definitions, and other declaration forms are permitted inside a `declare { }` block. Top-level command invocations and function calls are not permitted.
- **Internal calls allowed.** Functions being declared inside a `declare { }` block may call each other freely, including cyclically.
- **No cross-block cycles.** Two separate `declare { }` blocks in the same file may not call each other. Each block is internally self-contained.

`declare { }` blocks are the recommended mechanism for writing mutually recursive definitions in the REPL, where the REPL processes input in units and a mutual recursion must be submitted as one unit.

#### Script vs. module: implicit declare wrapping

When a file is imported via `use`, the compiler implicitly wraps its contents in a `declare { }` block before processing. This enforces the declarations-only rule at import time. Any top-level command in the file causes a `module/script-not-importable` error.

A file that contains only declarations is both runnable and importable. A file that contains top-level commands is runnable only.

#### Script identification (D17)

The ish interpreter accepts any input as a script — a file path, inline input, or REPL session. There is no mechanism that declares a file to be a script.

For importable modules, the `.ish` extension is required. `use` does not locate files without it. This is the only enforced distinction: if it has `.ish`, it can be imported (subject to content rules); if it does not, it cannot.

**Convention (unenforced):** Executable scripts intended to be run directly should use a shebang line (`#!/usr/bin/env ish`) and omit the `.ish` extension. This is a convention for human readability and OS integration, not a compiler rule.

### Acceptance Tests

- Two mutually recursive functions in the same file (no explicit `declare` block) compile and run correctly.
- Two modules (files) that import each other produce a cycle compile error; the diagnostic names both files.
- A `declare { }` block containing two mutually recursive functions compiles and runs correctly.
- A top-level command invocation inside a `declare { }` block is a compile error.
- A file containing a top-level command can be run directly with `ish`.
- The same file, when referenced by `use`, produces a `module/script-not-importable` error.
- A file containing only function definitions can be both run directly and imported via `use`.
- Two separate `declare { }` blocks in the same file that reference each other produce a cycle compile error.
- `use my-tool` where `my-tool` has no `.ish` extension produces `module/not-found`.

---

## Feature: Module-to-File Mapping

### Issues to Watch Out For

- The mapping must be predictable: given a module path, a developer should know which file to open.
- `src/` is the implicit source root and is not part of any module path.
- The `index.ish` convention must not create ambiguity when both `foo.ish` and `foo/index.ish` exist.

### Design

Each `.ish` file under `src/` defines exactly one module. The module path is derived from the file's path relative to `src/`. The `src/` prefix never appears in a `use` statement.

**Standard rule:** `src/net/http.ish` defines module `net/http`.

**`index.ish` exception (D15):** A file named `index.ish` in a directory maps to the module path of that directory, not to `<dir>/index`. This applies at any level of the hierarchy:

| File | Module path |
|------|-------------|
| `src/net/index.ish` | `net` |
| `src/net/http/index.ish` | `net/http` |

`index` is a reserved filename within any directory. There is no escape mechanism: a file named `index.ish` always defines the parent-directory module, never a module named `<dir>/index`.

**Conflict rule:** If both `src/foo.ish` and `src/foo/index.ish` exist, the build fails for both files. The error code is `module/path-conflict`. The diagnostic names both files and explains that they resolve to the same module path.

### Acceptance Tests

- `use net/http` resolves to `src/net/http.ish`.
- `use net` resolves to `src/net/index.ish` when that file exists.
- `src/net/index.ish` defines module `net`, not `net/index`.
- `src/net/http/index.ish` defines module `net/http`, not `net/http/index`.
- A `use` path with no corresponding file produces a `module/not-found` error naming the expected file path(s).
- A file at `src/foo/bar.ish` defines module `foo/bar`, not `src/foo/bar`.
- Both `src/foo.ish` and `src/foo/index.ish` existing produces a `module/path-conflict` build error naming both files.

---

## Feature: Import Syntax / Use Directive

### Issues to Watch Out For

- Import syntax is used throughout every ish source file — ergonomics matter.
- Ambiguous imports cause bugs.
- Selective imports reduce namespace pollution.

### Design

The `use` directive imports a module. Four forms are supported (D10):

```ish
use foo/bar              -- import module; access as bar.Name
use foo/bar as b         -- import with alias; access as b.Name
use foo/bar { Type, fn } -- selective import into local scope
use foo/bar { Type as T }-- selective import with rename
```

**Within-project imports** use a module path relative to the `src/` root:

```ish
use net/http             -- imports src/net/http.ish
use net                  -- imports src/net/index.ish
use ./util               -- relative import from the same directory subtree
```

**External package imports** use a full package path (resolved by the package system):

```ish
use example.com/foo/bar  -- imports from an external package
use example.com/foo      -- imports the top-level module of the foo package
```

The `use` directive may appear at the top of a file or at the top of a `declare { }` block.

**Implicit declare wrapping:** When `use` resolves a file, that file's contents are implicitly wrapped in a `declare { }` block before processing. Files with top-level commands fail at this point with `module/script-not-importable`. Files without the `.ish` extension are never found by `use`.

**Qualified access without `use`:** A module may be accessed by its full path without importing it: `net/http.Get(...)`. This resolves the module by path at the point of use without bringing any names into scope.

**Glob imports:** `use foo/bar { * }` imports all exported names from the module. Glob imports are available but discouraged. They may be restricted or flagged at higher assurance levels.

### Concurrency

The `use` directive is evaluated at compile time. It has no runtime concurrency implications.

### Acceptance Tests

- `use foo/bar` makes `bar.Func` available where `Func` is defined in `src/foo/bar.ish`.
- `use foo/bar as b` makes `b.Func` available.
- `use foo/bar { Func }` brings `Func` into local scope.
- `use foo/bar { Func as F }` brings `F` into local scope with the name `F`.
- `use net` resolves to `src/net/index.ish` and makes `net.Func` available.
- Accessing a `priv` item from another module via `use` produces an access error.
- Accessing a `pkg` item from an external project via `use` produces an access error.
- `net/http.Get(...)` resolves without a prior `use net/http`.
- `use` of a file containing top-level commands produces `module/script-not-importable`.
- `use my-tool` where `my-tool` has no `.ish` extension produces `module/not-found`.

---

## Feature: Project Layout

### Issues to Watch Out For

- The layout must be predictable so that tooling (editor integrations, `ish interface freeze`, script discovery) can locate files without configuration.
- The project root boundary determines which scripts receive `pkg` access.

### Design

A project has the following conventional directory layout:

```
myproject/
  project.json     -- project manifest
  src/             -- REQUIRED. Module source files (.ish)
  scripts/         -- OPTIONAL. Scripts for public consumption (distributed with the package)
  tools/           -- OPTIONAL. Scripts for internal use (build tools, test runners, dev utilities)
```

**`src/` is required.** The `src/` directory is the source root. All importable `.ish` module files are nested under it. The `src/` prefix is never part of a module path in a `use` statement. A project without a `src/` directory is malformed; tools report an error when operating on it.

**`scripts/` convention.** Scripts placed in `scripts/` are considered public: they are part of the project's distributed interface. Build tooling looks here when packaging the project for distribution. The contents, naming, and distribution format for scripts are addressed in Package Management.

**`tools/` convention.** Scripts placed in `tools/` are internal. They are not distributed with the package and are not part of the project's public interface. Typical contents: build scripts, code generators, test runner wrappers, developer utilities.

**`pkg` access for scripts.** Scripts anywhere under the project root — including in `scripts/`, `tools/`, or at the project root itself — are project members and receive `pkg` access to all project modules. The convention about which directory a script lives in does not affect its access rights; any script file under the project root qualifies.

Scripts in all directories follow the executable convention (shebang + no `.ish` extension). They are not importable via `use`.

### Acceptance Tests

- A project missing a `src/` directory produces an error when `ish` operates on it.
- Files under `src/` are resolved by `use` statements using paths relative to `src/`.
- Files under `scripts/` and `tools/` are not resolved by `use` statements.
- A script in `scripts/` has `pkg` access to modules in `src/`.
- A script in `tools/` has `pkg` access to modules in `src/`.
- A script at the project root (sibling of `project.json`) has `pkg` access to modules in `src/`.
- `ish interface freeze` generates `.ishi` files for modules under `src/`, not for files in `scripts/` or `tools/`.

---

## Feature: Project Membership and Standalone Script Configuration

### Issues to Watch Out For

- Shell scripts are often standalone; loading a full project manifest for a one-liner must not require extra setup.
- The module system must not make the shell experience worse.
- Every ish file must resolve to some configuration for dependency resolution.
- Standalone scripts must not be able to claim `pkg` visibility into projects they are not part of.

### Design

#### Project membership (D8, D19)

A file is a member of a project if and only if it is physically located under the project root — the directory that contains `project.json`, discovered by walking up the directory tree. Project membership grants:

- `pkg` access to all modules within the project
- The project's dependency configuration, ish version requirements, and standards settings

**Directory hierarchy search:** ish walks up the directory tree from the directory containing the source file, looking for a `project.json` file. The first one found establishes the project root.

**Installation default:** If no `project.json` is found in the hierarchy, ish uses the default project file bundled with the installation. This project includes only the standard library packages distributed with ish. Files using the installation default are standalone and receive no `pkg` access.

#### The `bootstrap` directive (D16)

Standalone scripts — files not under any `project.json` — may need external packages, specific ish version requirements, or non-default assurance settings. The `bootstrap` directive provides everything that a `project.json` would provide for a project member, scoped to the single file:

```ish
bootstrap "path/to/config.json"           -- filesystem path to a config file
bootstrap "https://example.com/cfg.json"  -- URL (resolved via ISH_PROXY)
bootstrap { "ish": ">=1.0", "dependencies": { "example.com/http": "v1.2.3" } }
                                          -- inline JSON object
```

The `bootstrap` directive may configure:

- **`ish`**: minimum ish version required
- **`dependencies`**: external packages to resolve and make available
- **`standards`**: assurance levels and standards settings
- Any other field that `project.json` supports, with the exception of publishing metadata

**What `bootstrap` grants:** Access to the `pub` APIs of listed packages.

**What `bootstrap` does not grant:** `pkg` visibility into any project. The `bootstrap` directive is not a claim of membership in any existing project.

**Interaction with directory search:** If a file uses `bootstrap` and is also under a `project.json` in its directory hierarchy, the `bootstrap` directive is an error. Project members receive their configuration from the project manifest; they do not use `bootstrap`.

### Concurrency

Project membership resolution and `bootstrap` parsing occur at startup before any concurrent execution begins.

### Acceptance Tests

- A file with no `bootstrap` directive in an isolated directory uses the installation default project.
- A file inside a directory containing `project.json` is a member of that project.
- A member file has `pkg` access to other modules in the same project.
- A file using `bootstrap` in an isolated directory can access `pub` APIs from its listed dependencies.
- A file using `bootstrap` cannot access `pkg` items from any project.
- A file using `bootstrap` while also under a `project.json` in its hierarchy produces an error.
- A `bootstrap` directive may configure ish version, dependencies, and assurance standards.
- Changing `ISH_PROXY` changes how URL-based `bootstrap` configurations are resolved.

---

## Prototype Changes

The prototype currently has stub implementations for `use` and `mod` statements — they are parsed into AST nodes but silently ignored at runtime. The `Visibility` enum has incorrect variant names (`Private`, `Public`, `PubScope`) and is missing the `Pkg` variant. The grammar uses `::` path separators instead of `/`. This section details all changes needed across each crate.

### `ish-core`

No changes required. `TypeAnnotation` is unchanged.

### `ish-ast`

**`Visibility` enum** — Replace the current `Private / Public / PubScope(String)` variants with the three variants matching the language design:

```rust
pub enum Visibility {
    Priv,   // priv — current module only
    Pkg,    // pkg — all project members (default when omitted)
    Pub,    // pub — external dependents
}
```

All existing uses of `Visibility` in `Statement` variants (`VariableDecl`, `FunctionDecl`, `TypeAlias`, `ModDecl`) use `Option<Visibility>`. The semantics of `None` change: `None` now means "default visibility" (i.e., `Pkg`). The `Priv` and `Pub` variants are only present when explicitly written in source.

**`Statement::Use`** — Replace the current `Use { path: Vec<String> }` with a richer node that captures all four import forms:

```rust
Statement::Use {
    module_path: Vec<String>,       // segments split on / (or . for external host)
    alias: Option<String>,          // as b
    selective: Option<Vec<SelectiveImport>>,  // { Type, fn as F }
},

pub struct SelectiveImport {
    pub name: String,
    pub alias: Option<String>,
}
```

**`Statement::ModDecl`** — Remove this variant. It represented the old `mod { }` block design that has been superseded by `declare { }` blocks. All references in the AST, parser, and VM must be updated.

**`Statement::DeclareBlock`** — Add this new variant:

```rust
Statement::DeclareBlock {
    body: Vec<Statement>,   // declarations only; commands are rejected at eval time
},
```

**`Statement::Bootstrap`** — Add this new variant:

```rust
pub enum BootstrapSource {
    Path(String),    // filesystem path to a JSON config file
    Url(String),     // https:// URL resolved via ISH_PROXY
    Inline(String),  // inline JSON object serialized as a string
}

Statement::Bootstrap {
    source: BootstrapSource,
},
```

**`display.rs`** — Update the display formatter for all changed/added node types.

**`builder.rs`** — Add builder helpers for `UseDirective`, `DeclareBlock`, and `Bootstrap`.

### `ish-parser`

**Grammar (`ish.pest`)**

*Visibility rule* — Replace the current `pub_modifier` rule (which only handles `pub` and `pub(scope)`) with a full three-way `visibility` rule:

```pest
visibility = { "priv" | "pkg" | "pub" }
```

Update all declaration rules that use `pub_modifier?` to use `visibility?` instead: `fn_decl`, `let_stmt`, `type_alias`.

*Module path rule* — Replace the current `module_path` rule (which uses `::` separators) with one that uses `/`:

```pest
module_path = {
    domain_segment ~ ("/" ~ path_segment)*  -- external: example.com/foo/bar
  | path_segment ~ ("/" ~ path_segment)*    -- internal: net/http or ./util
}
domain_segment = { identifier ~ "." ~ identifier ~ ("." ~ identifier)* }
path_segment   = { "." ~ "." | "." | identifier }
```

*`use` rule* — Extend `use_stmt` to cover all four import forms:

```pest
use_stmt = {
    "use" ~ module_path ~ ("as" ~ identifier)?
  | "use" ~ module_path ~ "{" ~ selective_import_list ~ "}"
}
selective_import_list = { selective_import ~ ("," ~ selective_import)* }
selective_import      = { identifier ~ ("as" ~ identifier)? }
```

*`declare` rule* — Replace the `mod_stmt` rule with a `declare_block` rule. Remove `mod_stmt` entirely:

```pest
declare_block = { "declare" ~ "{" ~ statement* ~ "}" }
```

*`bootstrap` rule* — Add a new top-level directive rule:

```pest
bootstrap_stmt = {
    "bootstrap" ~ (string_literal | inline_json_object)
}
inline_json_object = { "{" ~ json_content ~ "}" }
json_content = { (!("{" | "}") ~ ANY | inline_json_object)* }
```

*`program` and `statement` rules* — Add `declare_block` and `bootstrap_stmt` to the `statement` rule. Remove `mod_stmt`. Add `IncompleteKind::DeclareBlock` for unterminated `declare {`.

**`ast_builder.rs`**

- Update `build_visibility` to map `"priv"` → `Visibility::Priv`, `"pkg"` → `Visibility::Pkg`, `"pub"` → `Visibility::Pub`.
- Update `build_use_stmt` to produce the new `Statement::Use` with alias and selective import fields.
- Remove `build_mod_stmt`. Replace with `build_declare_block`.
- Add `build_bootstrap_stmt`.

### `ish-vm`

The VM currently stubs `Statement::Use` and `Statement::ModDecl` as no-ops (lines 711 and 716 of `interpreter.rs`, and the second pass at lines 2207–2208). This section describes the real implementations.

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

**`interpreter.rs` changes**

Replace the `Statement::Use { .. } => Ok(ControlFlow::None)` stub with a real implementation:

1. Determine if the module path is external (contains a `.` in the first segment) or internal.
2. For internal paths: call `module_loader::resolve_module_path` against the caller's `src_root`.
3. Check for cycles against the current loading stack. If a cycle is found, return `RuntimeError` with code `module/cycle`, listing the full cycle path.
4. Load and parse the file.
5. Wrap its contents in an implicit `DeclareBlock`. If any statement in the file is not a declaration (i.e., is a command, assignment, loop, etc.), return `RuntimeError` with code `module/script-not-importable`, naming the file.
6. Evaluate the `DeclareBlock` in a fresh child environment.
7. Bind the module namespace into the caller's environment according to the import form (qualified, aliased, or selective).
8. On selective imports, call `access_control::check_access` for each imported symbol.

Replace the `Statement::ModDecl { .. } => Ok(ControlFlow::None)` stub with an error: `ModDecl` is no longer a valid statement; the parser should never produce it. If encountered, return an internal error.

Add `Statement::DeclareBlock` evaluation:

1. Collect all declarations in the block into a temporary scope.
2. Evaluate them with mutual forward-reference resolution (all function and type names are pre-registered before any body is evaluated).
3. Merge the resulting bindings into the parent environment.
4. If any statement in the block is not a declaration, return a compile error with code `module/declare-block-command`.

Add `Statement::Bootstrap` evaluation (D20 — partially deferred):

1. Check that the caller file is not under any `project.json` in its hierarchy (using `module_loader::find_project_root`). If it is, return E021 (`module/bootstrap-in-project`).
2. Config parsing, application, and URL fetching are deferred. `ISH_PROXY` specification is deferred.

**Interface file consistency checking**

Interface file checking is handled by a new `interface_checker.rs` module (D21). When loading a module file (during `Statement::Use` processing), the checker looks for a sibling `.ishi` file. If one is found:

1. Parse the `.ishi` file for its `pub` declarations (function signatures and type definitions).
2. Compare against the `pub` declarations in the `.ish` implementation.
3. Emit granular errors:
   - E022 (`interface/symbol-not-in-implementation`) — symbol in `.ishi` not found in `.ish`
   - E023 (`interface/symbol-not-in-interface`) — `pub` symbol in `.ish` not declared in `.ishi`
   - E024 (`interface/symbol-mismatch`) — symbol present in both but with different signatures

**Analyzer update for declare blocks (D22)**

The code analyzer must be updated to handle yielding propagation for mutually recursive functions in `declare { }` blocks. See [Proposal A-2](module-system-core-a2.md) for the full specification.

**`lib.rs`**

Export the new `module_loader`, `access_control`, and `interface_checker` modules.

### `ish-codegen`

The codegen crate (`ish-codegen/src/lib.rs`) generates a temporary Cargo project from ish source and compiles it to a `.so`. Module loading in the compiled path must mirror the interpreter's behavior.

**Changes:**

- When generating Rust code for a `Statement::Use` targeting an internal module, the codegen driver must resolve the module path to a `.ish` file and include that file's generated Rust code in the temporary project.
- Apply the same `index.ish` resolution logic as the interpreter (share `module_loader` or duplicate the path resolution logic).
- The `Statement::DeclareBlock` generates a Rust `mod` block with `pub(crate)` items for `pkg`-visible declarations and `pub` for `pub`-visible declarations.
- `Statement::ModDecl` handling should be removed.

**Deferred (D23):** These changes are deferred pending validation of the module loading design in the interpreter.

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

All new error codes are production sites in `ish-vm`. Add to `docs/errors/INDEX.md`:

| Code | ErrorCode Variant | Summary | Production site |
|------|------------------|---------|----------------|
| E016 | `ModuleNotFound` | `use` path has no matching `.ish` file | `module_loader::resolve_module_path` |
| E017 | `ModuleCycle` | Circular `use` dependency detected | `interpreter.rs` — Use evaluation |
| E018 | `ModuleScriptNotImportable` | File imported via `use` contains top-level commands | `interpreter.rs` — Use evaluation |
| E019 | `ModulePathConflict` | Both `foo.ish` and `foo/index.ish` exist | `module_loader::resolve_module_path` |
| E020 | `ModuleDeclareBlockCommand` | `declare { }` block contains a non-declaration statement | `interpreter.rs` — DeclareBlock evaluation |
| E021 | `ModuleBootstrapInProject` | `bootstrap` used inside a project hierarchy | `interpreter.rs` — Bootstrap evaluation |
| E022 | `InterfaceSymbolNotInImplementation` | `.ishi` declares a symbol absent from the `.ish` file | `interface_checker.rs` |
| E023 | `InterfaceSymbolNotInInterface` | `.ish` has a `pub` symbol not declared in `.ishi` | `interface_checker.rs` |
| E024 | `InterfaceSymbolMismatch` | Symbol present in both `.ishi` and `.ish` with mismatched signatures | `interface_checker.rs` |

Add variants to `ErrorCode` in `ish-runtime/src/error.rs` and add rows to `docs/errors/INDEX.md` for all nine codes (D24).

### Unit Tests

Add unit tests to `ish-vm` for:

- `module_loader::derive_module_path`: standard path, `index.ish` path, path-conflict detection.
- `module_loader::resolve_module_path`: found, not-found, conflict cases.
- `access_control::check_access`: all nine combinations of `(Priv/Pkg/Pub) × (same-module/same-project/external)`.
- `module_loader::find_project_root`: found at current dir, found via walk, not found.

Add unit tests to `ish-ast` for:

- `Visibility` serialization round-trip.
- `Statement::Use` with alias, with selective imports, plain.
- `Statement::DeclareBlock` construction.

---

## Documentation Updates

The following files require changes. Each entry describes specifically what must be added, changed, or removed — not just that the file is affected.

### `GLOSSARY.md`

Add or update these entries:

| Term | Action | Definition |
|------|--------|-----------|
| module | Update | A `.ish` source file under `src/`. The module path is derived from the file path relative to `src/`, with `index.ish` mapping to the parent directory path |
| module path | Add | The `/`-separated identifier sequence used in `use` statements to name a module. Does not include the `src/` prefix |
| source root | Add | The `src/` directory under a project root. All importable modules are nested under it |
| project root | Update | The directory containing `project.json`. Determined by walking up the directory tree from the source file. Defines the boundary for `pkg` visibility |
| project member | Add | Any file physically located under the project root. Project members receive `pkg` access and the project's dependency configuration |
| standalone script | Add | A file not under any `project.json`. Uses the installation default project, or a `bootstrap` directive for custom configuration |
| `use` directive | Add | The import statement for modules. Supports four forms: plain, aliased, selective, and selective-with-rename |
| `declare` block | Add | An anonymous, declarations-only grouping. Does not introduce a namespace. Allows mutually recursive declarations. Implicit declare wrapping is applied to files loaded via `use` |
| `bootstrap` directive | Add | A standalone script directive that provides the same configuration as `project.json` for a single file |
| interface file | Add | A `.ishi` file that declares the `pub` contract of a module. Generated by `ish interface freeze`. Enforced at compile time when present |
| index module | Add | A module defined by `index.ish` in a directory. Its module path is the directory path, not `<dir>/index` |
| installation default | Add | The default project used when no `project.json` is found in the directory hierarchy. Includes only the standard library |
| visibility | Update | One of `priv` (current module only), `pkg` (all project members, the default), or `pub` (external dependents) |

### `docs/spec/modules.md`

Full rewrite. The current content does not reflect any of the decisions in this proposal. The rewritten document must cover:

- The three visibility levels and their scopes; the `pkg` default; the entry point pattern
- The file = module rule; `index.ish`; the conflict rule; the `src/` source root
- The four `use` directive forms; relative vs. external paths; qualified access without `use`
- `declare { }` blocks; the mutual-recursion model; implicit declare wrapping; cross-module cycle rule
- Project layout (`src/`, `scripts/`, `tools/`); the `project.json` discovery rule; `pkg` access for scripts
- The `bootstrap` directive and its three forms; what it grants and does not grant
- Interface files: generation, enforcement, error codes
- Deferred topics (conditional compilation, incremental compilation, script distribution)

### `docs/spec/syntax.md`

Add a new section "Module Directives" with formal syntax for:

- `use` directive: all four forms with examples
- `declare { }` block syntax
- `bootstrap` directive: all three forms with inline JSON example
- Visibility keywords: `priv`, `pkg`, `pub` — where they may appear (before `fn`, `let`, `type`)
- Note on `index.ish` as a naming convention (cross-reference to modules.md)

### `docs/architecture/vm.md`

Add a new section "Module Loading" covering:

- How the interpreter resolves a `use` path to a filesystem path (`module_loader::resolve_module_path`)
- The `index.ish` special case and conflict detection
- The loading stack used for cycle detection
- Implicit declare wrapping: what triggers it, what the error means
- Project root discovery at interpreter startup (`module_loader::find_project_root`)
- How `ProjectContext` flows through the interpreter
- `pkg` access checks: when `access_control::check_access` is called and what it tests
- Interface file consistency checking: when it runs, which errors it produces

Update the existing "Builtins" and "Environment" sections to note that `pkg`-visible bindings from imported modules are stored in a module namespace within the environment, not the global scope.

### `docs/architecture/codegen.md`

Add a section "Module Resolution in Compiled Execution" covering:

- How `Statement::Use` is handled during code generation
- How the codegen driver resolves module paths and includes generated code for dependencies
- How `DeclareBlock` maps to a Rust `mod` block
- Visibility mapping: `priv` → `(private)`, `pkg` → `pub(crate)`, `pub` → `pub`

### `docs/architecture/ast.md`

Add entries for the new/changed node types:

- `Visibility` enum: updated variant names and their semantics
- `Statement::Use`: fields, the `SelectiveImport` struct
- `Statement::DeclareBlock`: purpose, constraints on contents
- `Statement::Bootstrap`: the `BootstrapSource` enum and its three variants
- Removal of `Statement::ModDecl`

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

Add all nine new error codes listed in the Prototype Changes section, using the format established in the existing file: code, domain, description, production site.

### `AGENTS.md`

Add module-related guidance covering:

- Where to look when adding a new module-system feature: `module_loader.rs`, `access_control.rs`, `interface_checker.rs`, `interpreter.rs` Use handling
- Acceptance test location for module tests: `proto/ish-tests/modules.sh`
- The interface file format and where it is generated
- **Note (D25):** `CLAUDE.md` and any copilot instructions files are symlinks to `AGENTS.md`. Never update them directly. All agent-facing documentation — including task playbook rows — goes in `AGENTS.md`.

Add a row to the task playbooks table in `AGENTS.md`:

| Working on modules | [docs/spec/modules.md](docs/spec/modules.md) |

---

## Acceptance Tests

New tests belong in a dedicated file `proto/ish-tests/modules.sh`. The file follows the conventions in `proto/ish-tests/lib/test_lib.sh`. Each test listed in the feature sections above corresponds to one `check` call. The complete test list, grouped by feature:

**Visibility:**
`priv` inaccessible cross-module · `pkg` accessible within project · script under project root gets `pkg` · script outside does not · inline script does not · `pub` accessible from external · `pkg` inaccessible from external · `ish interface freeze` generates `.ishi` · `.ishi` symbol-not-in-implementation error · `.ishi` symbol-mismatch error · freeze overwrites existing `.ishi`

**Module Identity:**
Mutual recursion in same file · cross-module cycle error · `declare { }` mutual recursion · command in `declare { }` is error · script runnable directly · same script via `use` is `script-not-importable` · declarations-only file runnable and importable · cross-block `declare { }` cycle error · `use` without `.ish` extension is `not-found`

**Module-to-File Mapping:**
`use net/http` → `src/net/http.ish` · `use net` → `src/net/index.ish` · `index.ish` maps to parent path · nested `index.ish` maps correctly · `use` with no file produces `not-found` · both `foo.ish` and `foo/index.ish` produces `path-conflict`

**Import Syntax:**
Plain `use` · aliased `use as` · selective `use { }` · selective with rename · `use net` via index · `priv` access error · `pkg` external access error · qualified access without `use` · `use` of script-file produces `script-not-importable` · `use` without extension produces `not-found`

**Project Layout:**
Missing `src/` produces error · `use` resolves relative to `src/` · scripts not resolvable via `use` · `scripts/` has `pkg` access · `tools/` has `pkg` access · project-root script has `pkg` access · `ish interface freeze` scoped to `src/`

**Project Membership and Bootstrap:**
Isolated file uses installation default · file under `project.json` is member · member has `pkg` access · `bootstrap` in isolation grants `pub` access · `bootstrap` cannot access `pkg` · `bootstrap` inside project is error · `bootstrap` configures ish version and dependencies · `ISH_PROXY` used for URL bootstrap

---

## Referenced by

- [docs/project/proposals/module-system.md](module-system.md)
- [docs/project/proposals/module-system-core-a1.md](module-system-core-a1.md)
- [docs/project/proposals/module-system-core-a2.md](module-system-core-a2.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
