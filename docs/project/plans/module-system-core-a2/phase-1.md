---
title: "Plan Phase 1: Documentation"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md]
---

# Phase 1: Documentation

*Part of: [module-system-core-a2/overview.md](overview.md)*

Update all documentation before any code changes. This phase covers the roadmap, specification, architecture doc, user guide, error catalog, and AGENTS.md.

## Context Files

- [context/error-codes.md](context/error-codes.md) — error code table and summary strings

## Requirements

- Roadmap has "Module System Core A-2 (Execution and Tooling)" listed under "In Progress".
- `docs/spec/modules.md` covers all module system semantics as listed in the proposal.
- `docs/architecture/vm.md` has a "Module Loading" section and updated "Code Analyzer" section.
- `docs/user-guide/modules.md` is the complete user-facing reference (not a placeholder).
- `docs/errors/INDEX.md` has all nine error codes E016–E024 in the table.
- `AGENTS.md` has module-system guidance and a row for "Working on modules" in the task playbooks.

## Tasks

- [x] 1. Update `docs/project/roadmap.md` — add `- [ ] Module System Core A-2 (Execution and Tooling)` to the "In Progress" section.

- [x] 2. Rewrite `docs/spec/modules.md` — full rewrite covering:
  - Three visibility levels (`priv`, `pkg`, `pub`) and their scopes; `pkg` as default; entry point pattern
  - The file = module rule; `index.ish`; path conflict rule; `src/` source root
  - Four `use` directive forms; relative vs. external paths; qualified access without `use`
  - `declare { }` blocks; mutual-recursion model; implicit declare wrapping for `use`; cross-module cycle rule
  - Project layout (`src/`, `scripts/`, `tools/`); `project.json` discovery rule; `pkg` access for scripts
  - `bootstrap` directive: three forms, what it grants, deferred aspects (config parsing, URL fetching)
  - Interface files: generation via `ish interface freeze`, enforcement, `.ishi` format, error codes
  - Deferred topics section (conditional compilation, incremental compilation, script distribution, bootstrap config, ISH_PROXY)
  - YAML frontmatter: `status: draft`, `last-verified: 2026-04-06`; update `depends-on` list
  - `## Referenced by` section

- [x] 3. Update `docs/architecture/vm.md` — add a "Module Loading" section after the "Code Analyzer" subsection, covering:
  - How the interpreter resolves a `use` path (`module_loader::resolve_module_path`)
  - `index.ish` special case and conflict detection (E019)
  - The loading stack used for cycle detection (E017)
  - Implicit declare wrapping: what triggers it, what E018 means
  - Project root discovery at interpreter startup (`module_loader::find_project_root`)
  - How `ProjectContext` flows through the interpreter
  - `pkg` access checks: when `access_control::check_access` is called and what it tests
  - Interface file consistency checking: when it runs, which errors it produces (E022–E024)
  - `DeclareBlock` evaluation and the analyzer's yielding propagation rules for mutual recursion (D22)

  Also update the "Code Analyzer" section to remove the "Known limitations" bullet "No call cycles: mutually recursive functions are not supported" (D22 resolves this for `declare { }` blocks).

  Update `last-verified: 2026-04-06`.

- [x] 4. Write `docs/user-guide/modules.md` — replace the placeholder content with the full user guide covering:
  - Getting started: minimal project layout with one module and one importing file
  - Visibility model: `priv`, `pkg` (default), `pub`; when to use each
  - Writing importable modules: `.ish` extension, declarations-only constraint
  - Writing scripts: shebang convention, entry point pattern
  - Project layout: `src/`, `scripts/`, `tools/`; what belongs where
  - The `use` directive: all four forms with examples for each
  - `declare { }` blocks: when to use them, mutual recursion example, REPL usage
  - The `bootstrap` directive: standalone script example, when to use it vs. a project
  - Interface files: running `ish interface freeze`, checking in `.ishi` files, what the errors mean
  - Common mistakes: importing a script file (E018), creating a path conflict (E019), using `bootstrap` inside a project (E021)
  - YAML frontmatter: `status: draft`, `last-verified: 2026-04-06`

- [x] 5. Update `docs/errors/INDEX.md` — add nine rows after E015:

  | E016 | `ModuleNotFound` | `CodedError` | `use` path has no matching `.ish` file | `module_loader::resolve_module_path` |
  | E017 | `ModuleCycle` | `CodedError` | Circular `use` dependency detected | `interpreter.rs` — Use evaluation |
  | E018 | `ModuleScriptNotImportable` | `CodedError` | File imported via `use` contains top-level commands | `interpreter.rs` — Use evaluation |
  | E019 | `ModulePathConflict` | `CodedError` | Both `foo.ish` and `foo/index.ish` exist | `module_loader::resolve_module_path` |
  | E020 | `ModuleDeclareBlockCommand` | `CodedError` | `declare { }` block contains a non-declaration statement | `interpreter.rs` — DeclareBlock evaluation |
  | E021 | `ModuleBootstrapInProject` | `CodedError` | `bootstrap` used inside a project hierarchy | `interpreter.rs` — Bootstrap evaluation |
  | E022 | `InterfaceSymbolNotInImplementation` | `CodedError` | `.ishi` declares a symbol absent from the `.ish` file | `interface_checker.rs` |
  | E023 | `InterfaceSymbolNotInInterface` | `CodedError` | `.ish` has a `pub` symbol not declared in `.ishi` | `interface_checker.rs` |
  | E024 | `InterfaceSymbolMismatch` | `CodedError` | Symbol present in both `.ishi` and `.ish` with mismatched signatures | `interface_checker.rs` |

  Update `last-verified: 2026-04-06`.

- [x] 6. Update `AGENTS.md` — add module-system guidance. Find the "Task Playbooks" table and add:

  `| Working on modules | [docs/spec/modules.md](docs/spec/modules.md), [docs/user-guide/modules.md](docs/user-guide/modules.md) |`

  Also add a new section (after the existing task playbooks or in a "Module System" guidance section) covering:
  - Where to add new module-system features: `module_loader.rs`, `access_control.rs`, `interface_checker.rs`, `interpreter.rs` Use handling
  - Acceptance tests for module features: `proto/ish-tests/modules/`
  - Interface file format and where it is generated (`ish interface freeze`)
  - **Note:** `CLAUDE.md` is a symlink to `AGENTS.md`. Never edit `CLAUDE.md` directly.

## Verification

Run: `grep -r "E016\|E017\|E018\|E019\|E020\|E021\|E022\|E023\|E024" docs/errors/INDEX.md`
Check: All nine error codes appear in the output.

Run: `grep "Module System Core A-2" docs/project/roadmap.md`
Check: Line found.

Run: `grep "module_loader\|access_control\|interface_checker" docs/architecture/vm.md`
Check: All three module names appear.

Invoke: `/verify module-system-core-a2/phase-1.md`
