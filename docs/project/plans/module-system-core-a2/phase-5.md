---
title: "Plan Phase 5: Interpreter — Use, DeclareBlock, Bootstrap"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md, docs/project/plans/module-system-core-a2/phase-2.md, docs/project/plans/module-system-core-a2/phase-3.md, docs/project/plans/module-system-core-a2/phase-4.md]
---

# Phase 5: Interpreter — Use, DeclareBlock, Bootstrap

*Part of: [module-system-core-a2/overview.md](overview.md)*

Replace the three no-op stubs in `interpreter.rs` with real implementations. Depends on Phases 2, 3, and 4 (error codes, module_loader, access_control, interface_checker must all exist).

## Context Files

- [context/interpreter-changes.md](context/interpreter-changes.md) — full step-by-step spec for each statement kind, stub locations

## Requirements

- `Statement::Use` evaluation in the yielding path (`exec_statement_yielding`) does all 9 steps from the proposal.
- External `use` paths (first segment contains `.`) are silently skipped (not yet implemented).
- `Statement::DeclareBlock` evaluation pre-registers all function/type names before analyzing any body; enforces declarations-only (E020); merges into parent environment.
- `Statement::Bootstrap` evaluation checks for project membership (E021); otherwise is a no-op (config parsing deferred).
- Both exec paths (yielding and unyielding) are updated — `DeclareBlock` and `Bootstrap` need implementations in both. `Use` only needs the yielding path (it triggers async file I/O).
- The unyielding path's `Use`, `DeclareBlock`, and `Bootstrap` arms: `Use` remains `Ok(ControlFlow::None)` (modules are only loaded via the yielding path); `DeclareBlock` and `Bootstrap` delegate to shared synchronous helpers.
- `cd proto && cargo build --workspace` passes.

## Tasks

- [x] 1. Edit `proto/ish-vm/src/interpreter.rs` — add `use` imports at the top for the new modules:

  ```rust
  use crate::{module_loader, access_control, interface_checker};
  use std::path::{Path, PathBuf};
  ```

- [x] 2. Edit `proto/ish-vm/src/interpreter.rs` — add `loading_stack` and `current_file` fields to `TaskContext` (or thread them as parameters). The loading stack is needed for cycle detection across nested `use` evaluations.

  Design: add `loading_stack: Vec<Vec<String>>` and `current_file: Option<PathBuf>` to `TaskContext`. Initialize as empty/None in `TaskContext::new()`.

- [x] 3. Edit `proto/ish-vm/src/interpreter.rs` — add `project_context: access_control::ProjectContext` to `IshVm`. Initialize in `IshVm::new()` with `project_root: None, src_root: None`. Shell startup will set this before running programs (Phase 6).

- [x] 4. Edit `proto/ish-vm/src/interpreter.rs` — replace the `Statement::Use { .. }` yielding stub (around line 711) with:

  ```
  Statement::Use { module_path, alias, selective } =>
      Self::eval_use(vm, task, yc, env, module_path, alias, selective).await
  ```

  Implement `async fn eval_use(...)` as a new associated function on `IshVm`:

  1. If `module_path[0]` contains `.`: return `Ok(ControlFlow::None)` (external path, deferred).
  2. Get `src_root` from `vm.borrow().project_context.src_root`. If `None`, return error: "use statement requires a project context (src_root not set)".
  3. Call `module_loader::resolve_module_path(module_path, &src_root)` → `file_path`. On error, convert to RuntimeError and return.
  4. Call `module_loader::check_cycle(&task.loading_stack, module_path)`. If true, return E017 with message listing the cycle.
  5. Read the file at `file_path` using `tokio::fs::read_to_string` (or sync `std::fs::read_to_string` since file I/O inside `exec_statement_yielding` can use blocking reads in the prototype).
  6. Parse the file content using `ish_parser::parse`.
  7. Check every top-level statement: if any is not `FunctionDecl` or `TypeAlias`, return E018 naming the file.
  8. Collect all `pub` (`Visibility::Pub`) declarations from the parsed program.
  9. Call `interface_checker::check_interface(&file_path, &pub_declarations)`. If errors returned, format and return first error as RuntimeError.
  10. Push `module_path.to_vec()` onto `task.loading_stack` and set `task.current_file = Some(file_path.clone())`.
  11. Create a child environment for the module. Call `exec_statement_yielding` for each declaration in the module.
  12. Pop `task.loading_stack` and restore `task.current_file`.
  13. Bind namespace into caller's environment:
      - No alias, no selective: bind `module_name` → the child env's top-level bindings as an object.
      - With alias: bind `alias` → same.
      - Selective: for each selective import, call `access_control::check_access` on the item's visibility, then bind the item name (or alias) into the caller env.

- [x] 5. Edit `proto/ish-vm/src/interpreter.rs` — replace the `Statement::DeclareBlock { .. }` yielding stub (around line 716) with a real implementation:

  ```
  Statement::DeclareBlock { body } =>
      Self::eval_declare_block(vm, task, yc, env, body).await
  ```

  Implement `async fn eval_declare_block(...)`:

  1. Validate: for each statement in `body`, if not `FunctionDecl` or `TypeAlias`, return E020.
  2. Pre-registration pass: for each `FunctionDecl` in `body`, `env.define(name, Value::Null)` to seed forward references; for each `TypeAlias`, same.
  3. Yielding analysis pass (D22): collect all `FunctionDecl` names. Seed an analysis environment with all names. Call `crate::analyzer::classify_function` for each. Apply the mutual-recursion propagation rule (if any function in the block calls another function in the block and is yielding, all in that call cycle become yielding). See [context/analyzer-update.md](context/analyzer-update.md).
  4. Evaluation pass: for each statement in `body` (in order), call `exec_statement_yielding`. This evaluates each `FunctionDecl` normally — the pre-registration ensures cross-calls are resolvable.

- [x] 6. Edit `proto/ish-vm/src/interpreter.rs` — replace the `Statement::Bootstrap { .. }` yielding stub (around line 721) with:

  1. Get the caller's file path from `task.current_file`.
  2. If `current_file` is set, call `module_loader::find_project_root(file.parent())`.
  3. If a project root is found, return E021.
  4. Otherwise, return `Ok(ControlFlow::None)` (config parsing deferred).

- [x] 7. Edit `proto/ish-vm/src/interpreter.rs` — update the unyielding path (`exec_statement_unyielding`) for the same three arms (around lines 2212–2214):
  - `Statement::Use { .. }`: keep `Ok(ControlFlow::None)` — modules are loaded via the yielding path only.
  - `Statement::DeclareBlock { body }`: call a synchronous helper `eval_declare_block_sync` that performs steps 1–4 of the yielding version but using `exec_statement_unyielding` for the evaluation pass.
  - `Statement::Bootstrap { .. }`: same check as yielding (synchronous — `find_project_root` is synchronous).

## Verification

Run: `cd proto && cargo build -p ish-vm`
Check: No errors.

Run: `cd proto && cargo run -p ish-shell -- -c 'declare { fn even(n: Int) -> Bool { if n == 0 { true } else { odd(n - 1) } }; fn odd(n: Int) -> Bool { if n == 0 { false } else { even(n - 1) } } }; println(even(4))'`
Check: Output is `true`.

Run: `cd proto && cargo run -p ish-shell -- -c 'declare { let x = 1 }'`
Check: Error output contains `E020` or "declare-block-command".

Invoke: `/verify module-system-core-a2/phase-5.md`
