---
title: Module System A-2 — Execution and Tooling — Design History
category: project
audience: [all]
status: current
last-verified: 2026-04-06
depends-on: [docs/project/proposals/module-system-core-a2.md, docs/project/history/2026-04-05-module-system-a1/summary.md]
---

# Module System A-2 — Execution and Tooling — Design History

*April 6, 2026*

This directory captures the history of Proposal A-2: Module Core — Execution and Tooling, which covers the VM module loader, access control, interface file checking, shell subcommands, error catalog, architecture documentation, user guide, and acceptance tests. The language representation work is in Proposal A-1.

---

## Version 1 — Initial and Accepted

*Split from [module-system-core.md](../../proposals/module-system-core.md) v10 on 2026-04-05. Accepted on 2026-04-06.*

Proposal A-2 was created alongside A-1 when the module-system-core proposal was split at v10. While A-1 covered the AST and grammar layer that has no runtime dependencies, A-2 covers everything that requires a running interpreter: the module loader, access control, interface consistency checking, the `ish interface freeze` subcommand, project root discovery at startup, and the full acceptance test suite.

The proposal arrived fully resolved. Decisions D20–D25 were established during or immediately after the split and recorded directly in the A-2 decision register. D20 sets clear boundaries around the `bootstrap` directive: only the project-containment check is implemented; config parsing, URL fetching, and `ISH_PROXY` are deferred. D21 mandates that `interpreter.rs` remain thin by delegating to three new modules — `module_loader.rs`, `access_control.rs`, and `interface_checker.rs` — preventing a single file from becoming a logic sink. D22 specifies how the existing code analyzer must handle yielding propagation through mutual recursion inside `declare { }` blocks. D23 defers `ish-codegen` changes until the interpreter path is validated. D24 assigns error codes E016–E024. D25 clarifies that `CLAUDE.md` is a symlink to `AGENTS.md` and must never be edited directly.

No `-->` markers or open questions were present at the time of acceptance. The design was complete as written.

### What A-2 covers

The accepted proposal specifies:

**`ish-vm` changes:** Three new modules are added. `module_loader.rs` handles all filesystem concerns: `find_project_root` walks directory ancestors for `project.json`, `derive_module_path` maps a `.ish` file path to its module path (with `index.ish` special-casing), `resolve_module_path` finds the `.ish` file for a given `use` path (detecting conflicts and not-found), and `check_cycle` detects circular `use` chains against the loading stack. `access_control.rs` defines `ProjectContext` and implements `check_access` for all nine combinations of visibility level × caller location. `interface_checker.rs` locates sibling `.ishi` files and compares their declarations against the module's `pub` items, emitting granular errors for each mismatch type.

`interpreter.rs` receives real implementations for `Statement::Use`, `Statement::DeclareBlock`, and `Statement::Bootstrap`, replacing the existing no-ops, and an internal error for `Statement::ModDecl` (which the new parser will never produce). `Statement::Use` delegates to the three new modules in sequence: resolve path → check cycle → load and parse → implicit declare-wrap → interface check → evaluate → bind namespace → access-check selective imports.

The code analyzer is extended to handle `DeclareBlock`'s mutual-recursion group: all names in the block are pre-registered, yielding is propagated transitively through the cycle, and the cycle-with-no-external-yielding-criterion rule produces unyielding.

**`ish-shell` changes:** A new `interface` subcommand is added to the shell binary, implemented in `interface_cmd.rs`. `ish interface freeze [module]` walks `src/` (or resolves a single module) and writes `.ishi` files containing only `pub` function signatures and type aliases. Project root discovery is added at startup: the `ProjectContext` is determined from the executing file's directory (file mode) or the current working directory (REPL mode) and passed to the interpreter.

**Error catalog:** Nine new error codes are added — E016 through E024 — covering module not found, cycle, script-not-importable, path conflict, declare-block command, bootstrap-in-project, and the three interface mismatch types. All are `CodedError` structural type, with production sites in the new VM modules.

**Documentation:** `docs/spec/modules.md` is fully rewritten to cover all module system semantics. `docs/architecture/vm.md` gains a Module Loading section and updates to the Builtins and Environment sections. `docs/user-guide/modules.md` is written as the primary user-facing reference. `docs/errors/INDEX.md` gains the nine new codes. `AGENTS.md` gains module-system guidance and a task playbooks row.

**Tests:** Unit tests cover `module_loader`, `access_control`, and `interface_checker` at the function level. A new acceptance test file `proto/ish-tests/modules.sh` covers visibility rules, module identity, module-to-file mapping, all four import syntax forms, project layout, project membership, and bootstrap behavior.

### Relationship to A-1 and the broader module system

A-2 depends on A-1 being implemented first. The AST node types, grammar rules, and `Visibility` enum changes in A-1 are the foundation that A-2's VM and shell code builds on. Once A-1 is merged and the prototype compiles cleanly, A-2 can be implemented in full. The language design reference for both proposals remains [module-system-core.md](../../proposals/module-system-core.md).

Full text: [v1.md](v1.md), accepted proposal at [module-system-core-a2.md](../../proposals/module-system-core-a2.md)

## Implementation

Implementation was completed on 2026-04-06 across nine phases following the plan at [module-system-core-a2/overview.md](../../plans/module-system-core-a2/overview.md).

The implementation added three new modules to `ish-vm`: `module_loader` (project root discovery, module path resolution, cycle detection), `access_control` (visibility enforcement with `priv`/`pkg`/`pub` semantics), and `interface_checker` (`.ishi` file consistency checking). The interpreter received full implementations of `use`, `declare { }`, and `bootstrap` statements, including forward reference pre-registration for declare blocks, nine-step module loading with access control checks, and the implicit declarations-only rule for importable modules. The shell gained the `ish interface freeze` subcommand for generating `.ishi` files from `pub` declarations, plus project root discovery at startup that propagates to the VM's `ProjectContext`.

During implementation, three design clarifications emerged: (1) `.ishi` files use empty-body function syntax (`pub fn name() {}`) so the existing parser can handle them without grammar changes; (2) `is_declaration` was broadened to include `Use`, `DeclareBlock`, and `Bootstrap` in addition to `FunctionDecl` and `TypeAlias`, enabling modules that import other modules to remain importable themselves; (3) the `bootstrap` grammar rule only accepts single-quoted string literals, matching the existing `bootstrap_stmt` pest rule.

Final state: `cargo build --workspace` clean (warnings only), `cargo test --workspace` passes all 395 tests (including 26 new module-system unit tests), and `bash ish-tests/run_all.sh` passes all 283 acceptance tests across 9 groups (including 28 new module-system acceptance tests).

---

## Referenced by

- [docs/project/proposals/module-system-core-a2.md](../../proposals/module-system-core-a2.md)
- [docs/project/history/INDEX.md](../INDEX.md)
