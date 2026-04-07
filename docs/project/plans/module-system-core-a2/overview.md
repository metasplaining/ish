---
title: "Plan: Module System Core A-2 — Execution and Tooling"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/proposals/module-system-core-a2.md, docs/project/proposals/module-system-core.md, docs/spec/modules.md, docs/architecture/vm.md, GLOSSARY.md]
---

# Plan: Module System Core A-2 — Execution and Tooling

*Derived from [module-system-core-a2.md](../../proposals/module-system-core-a2.md) on 2026-04-06.*

## Overview

This plan implements all changes in Proposal A-2: the execution and tooling layer of the module system. It adds three new VM modules (`module_loader`, `access_control`, `interface_checker`), replaces stub implementations in the interpreter for `use`, `declare { }`, and `bootstrap`, adds the `ish interface freeze` shell subcommand, adds project root discovery at startup, adds nine new error codes (E016–E024), rewrites the module spec, adds the user guide, extends the architecture doc, and creates a complete acceptance test suite.

**Prerequisite:** Proposal A-1 (Language Representation) must be implemented and merged before this plan begins. The A-1 plan status is `completed` — the AST types (`Visibility::Priv/Pkg/Pub`, `Statement::Use`, `Statement::DeclareBlock`, `Statement::Bootstrap`) are in place and the prototype builds cleanly.

## Requirements

1. `module_loader::find_project_root` walks directory ancestors to find `project.json`.
2. `module_loader::resolve_module_path` maps a module path to a `.ish` file, returning E016 (not found) or E019 (conflict).
3. `module_loader::derive_module_path` maps a `.ish` file path to a module path, applying the `index.ish` rule.
4. `module_loader::check_cycle` detects circular `use` dependencies.
5. `access_control::check_access` enforces `priv` (same-file), `pkg` (containment), `pub` (always) rules. Inline callers (`None` file path) are denied `priv` and `pkg`.
6. `interface_checker::check_interface` produces E022/E023/E024 errors when a `.ishi` file disagrees with the module's `pub` declarations. Returns empty vec when no `.ishi` exists.
7. `Statement::Use` evaluation (yielding path) performs all 9 steps: path resolution, cycle check, file load, implicit-declare validation, interface check, evaluation, namespace binding, access control on selective imports.
8. `Statement::DeclareBlock` evaluation pre-registers all names (mutual forward-reference), validates declarations-only (E020), applies D22 yielding propagation, and evaluates in order.
9. `Statement::Bootstrap` evaluation checks project membership (E021); config parsing is deferred.
10. `ish interface freeze [module]` generates `.ishi` files for `pub` declarations in all (or one) module(s) under `src/`.
11. Project root is discovered at interpreter startup for file execution and REPL mode.
12. `ErrorCode` enum in `ish-runtime` has variants E016–E024 with correct `as_str` values.
13. `docs/spec/modules.md` is fully rewritten to cover all module system semantics.
14. `docs/architecture/vm.md` has a "Module Loading" section.
15. `docs/user-guide/modules.md` is complete (not a placeholder).
16. `docs/errors/INDEX.md` lists E016–E024.
17. `AGENTS.md` has module system guidance and an updated task playbooks row.
18. All acceptance tests in `proto/ish-tests/modules/` pass.
19. `cargo build --workspace` and `cargo test --workspace` pass with no new errors.

## Phase Dependency Graph

```
Phase 1 (Documentation)
    ↓
Phase 2 (Error codes — ish-runtime)
    ↓
Phase 3 (module_loader + access_control)    Phase 4 (interface_checker)
         ↘                                 ↙
              Phase 5 (Interpreter: Use, DeclareBlock, Bootstrap)
                   ↓
              Phase 6 (Shell: interface freeze + project root)
                   ↓
Phase 7 (Unit tests)          Phase 8 (Acceptance tests)
         ↘                   ↙
              Phase 9 (Finalize)
```

Phases 3 and 4 are independent and can run in parallel once Phase 2 is done.
Phases 7 and 8 are independent and can run in parallel once Phase 6 is done.

## Authority Order

1. Roadmap (in progress) — Phase 1
2. `docs/spec/modules.md` — Phase 1
3. `docs/architecture/vm.md` — Phase 1
4. `docs/user-guide/modules.md` — Phase 1
5. `docs/errors/INDEX.md` — Phase 1
6. `AGENTS.md` — Phase 1
7. `proto/ish-runtime/src/error.rs` — Phase 2
8. `proto/ish-vm/src/module_loader.rs` (new) — Phase 3
9. `proto/ish-vm/src/access_control.rs` (new) — Phase 3
10. `proto/ish-vm/src/interface_checker.rs` (new) — Phase 4
11. `proto/ish-vm/src/lib.rs` — Phases 3, 4
12. `proto/ish-vm/src/interpreter.rs` — Phase 5
13. `proto/ish-vm/src/analyzer.rs` — Phase 5 (D22 propagation is done inline in interpreter)
14. `proto/ish-shell/src/interface_cmd.rs` (new) — Phase 6
15. `proto/ish-shell/src/main.rs` — Phase 6
16. `proto/ish-shell/src/repl.rs` — Phase 6
17. Unit tests in ish-vm — Phase 7
18. Acceptance tests — Phase 8
19. Roadmap (completed), plan index, history — Phase 9

## Context Files

- [context/vm-new-modules.md](context/vm-new-modules.md) — verbatim specs for module_loader + access_control
- [context/interface-checker.md](context/interface-checker.md) — verbatim spec for interface_checker
- [context/interpreter-changes.md](context/interpreter-changes.md) — verbatim interpreter change spec + stub locations
- [context/analyzer-update.md](context/analyzer-update.md) — verbatim D22 analyzer spec
- [context/shell-changes.md](context/shell-changes.md) — verbatim shell change spec + current main.rs structure
- [context/error-codes.md](context/error-codes.md) — verbatim error code table + how to extend error.rs

## Phases

- [phase-1.md](phase-1.md) — Documentation (roadmap, spec/modules.md, vm.md, user-guide, errors, AGENTS.md)
- [phase-2.md](phase-2.md) — Error codes (ish-runtime/src/error.rs, E016–E024)
- [phase-3.md](phase-3.md) — module_loader.rs + access_control.rs + lib.rs exports
- [phase-4.md](phase-4.md) — interface_checker.rs + lib.rs export
- [phase-5.md](phase-5.md) — interpreter.rs: Use, DeclareBlock, Bootstrap real implementations
- [phase-6.md](phase-6.md) — ish-shell: interface_cmd.rs + main.rs subcommand + project root discovery
- [phase-7.md](phase-7.md) — Unit tests (module_loader, access_control, interface_checker)
- [phase-8.md](phase-8.md) — Acceptance tests (proto/ish-tests/modules/)
- [phase-9.md](phase-9.md) — Finalize (roadmap completed, plan index, history)

## Referenced by

- [docs/project/proposals/module-system-core-a2.md](../../proposals/module-system-core-a2.md)
- [docs/project/plans/INDEX.md](../INDEX.md)
