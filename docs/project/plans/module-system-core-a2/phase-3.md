---
title: "Plan Phase 3: VM Module Loader and Access Control"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md, docs/project/plans/module-system-core-a2/phase-2.md]
---

# Phase 3: VM Module Loader and Access Control

*Part of: [module-system-core-a2/overview.md](overview.md)*

Create `module_loader.rs` and `access_control.rs` in `ish-vm`. These modules have no dependency on each other and can be written together. Both must exist before the interpreter can use them (Phase 5).

## Context Files

- [context/vm-new-modules.md](context/vm-new-modules.md) — full function signatures and behavior specs
- [context/error-codes.md](context/error-codes.md) — error codes used here: E016, E019

## Requirements

- `proto/ish-vm/src/module_loader.rs` exists with four public functions: `find_project_root`, `derive_module_path`, `resolve_module_path`, `check_cycle`.
- `proto/ish-vm/src/access_control.rs` exists with `ProjectContext` struct and two public functions: `check_access`, `is_project_member`.
- Both modules are exported from `proto/ish-vm/src/lib.rs`.
- `find_project_root` returns `None` at filesystem root (terminates correctly).
- `resolve_module_path` returns `E016` (ModuleNotFound) when no file exists, `E019` (ModulePathConflict) when both candidates exist.
- `derive_module_path` applies the `index.ish` → parent directory substitution.
- `check_access` returns `Ok(())` for `Pub`, enforces containment for `Pkg`, enforces same-file for `Priv`.
- `cd proto && cargo check -p ish-vm` passes.

## Tasks

- [x] 1. Create `proto/ish-vm/src/module_loader.rs`:

  ```rust
  use std::path::{Path, PathBuf};
  use crate::error::{ErrorCode, RuntimeError};

  pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
      // Walk ancestors of start_dir.
      // For each dir, check if dir.join("project.json").exists().
      // Return Some(dir) on first match, None if filesystem root reached.
  }

  pub fn derive_module_path(file_path: &Path, src_root: &Path) -> Result<Vec<String>, RuntimeError> {
      // Strip src_root prefix from file_path.
      // Strip .ish extension from the final segment.
      // If final segment after stripping is "index", replace with parent dir name.
      // Return segments as Vec<String>.
      // Error if file_path is not under src_root or has no .ish extension.
  }

  pub fn resolve_module_path(module_path: &[String], src_root: &Path) -> Result<PathBuf, RuntimeError> {
      // Build two candidates:
      //   candidate_file: src_root / a / b / c.ish
      //   candidate_index: src_root / a / b / c / index.ish
      // If both exist: return Err with E019 (ModulePathConflict) naming both paths.
      // If neither exists: return Err with E016 (ModuleNotFound) naming the expected paths.
      // If exactly one exists: return Ok(that path).
  }

  pub fn check_cycle(loading_stack: &[Vec<String>], candidate: &[String]) -> bool {
      // Return true if candidate == any element in loading_stack.
  }
  ```

- [x] 2. Create `proto/ish-vm/src/access_control.rs`:

  ```rust
  use std::path::{Path, PathBuf};
  use ish_ast::Visibility;

  pub struct ProjectContext {
      pub project_root: Option<PathBuf>,
      pub src_root: Option<PathBuf>,
  }

  pub enum AccessError {
      Private { symbol: String },
      PackageOnly { symbol: String },
  }

  pub fn check_access(
      item_visibility: &Visibility,
      item_file_path: Option<&Path>,
      caller_file_path: Option<&Path>,
  ) -> Result<(), AccessError> {
      // Visibility::Priv: caller_file_path must equal item_file_path.
      //   None caller (inline) always fails for Priv.
      // Visibility::Pkg: checked at call site using is_project_member + project_root.
      //   Caller must be in same project as the item. None caller fails for Pkg.
      // Visibility::Pub: always Ok(()).
  }

  pub fn is_project_member(file_path: &Path, project_root: &Path) -> bool {
      file_path.starts_with(project_root)
  }
  ```

  Note: The `check_access` signature in the proposal passes `item_project_root` and `caller_project_root` separately. Implement as specified — the function needs both to do the Pkg containment check. The simplified sketch above is for orientation only; follow the proposal's specification exactly.

- [x] 3. Edit `proto/ish-vm/src/lib.rs` — add:

  ```rust
  pub mod module_loader;
  pub mod access_control;
  ```

## Verification

Run: `cd proto && cargo check -p ish-vm`
Check: Compiles with no errors.

Run: `grep -c "pub fn" proto/ish-vm/src/module_loader.rs`
Check: 4 (find_project_root, derive_module_path, resolve_module_path, check_cycle).

Run: `grep "pub struct ProjectContext" proto/ish-vm/src/access_control.rs`
Check: Line found.

Invoke: `/verify module-system-core-a2/phase-3.md`
